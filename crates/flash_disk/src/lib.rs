//! A small FAT drive stored in a region of NOR flash, for sharing files with
//! a PC over USB mass storage.
//!
//! - [`FlashDisk`] turns a flash region (4 KiB erase blocks) into a block
//!   device with 512-byte sectors, caching one erase block so a run of sector
//!   writes costs one erase.
//! - [`fat`] formats such a device as FAT12 and lists and reads files in its
//!   root directory. Everything else (creating, editing and deleting files)
//!   is left to the PC, which writes sectors through [`scsi`].
//! - [`scsi`] parses USB Bulk-Only Transport command blocks and answers the
//!   SCSI commands PCs send to a USB drive. The USB transfers themselves are
//!   up to the platform.
//!
//! None of this depends on the hardware, so it is tested on the host against
//! an in-memory flash.
#![no_std]

extern crate alloc;

pub mod fat;
pub mod scsi;

use alloc::boxed::Box;

/// Bytes per sector, as seen by the FAT filesystem and the USB host.
pub const SECTOR_SIZE: usize = 512;
/// Bytes per flash erase block.
pub const ERASE_SIZE: usize = 4096;
/// Bytes per flash program page. Unchanged pages are not reprogrammed.
pub const PAGE_SIZE: usize = 256;

const SECTORS_PER_BLOCK: u32 = (ERASE_SIZE / SECTOR_SIZE) as u32;

pub type Sector = [u8; SECTOR_SIZE];

/// A flash operation failed, or a sector was out of range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiskError;

/// A NOR flash region. Offsets are relative to the start of the region.
///
/// Erasing sets a whole block to `0xFF`; programming can only clear bits.
pub trait Flash {
    /// Region size in bytes, a multiple of [`ERASE_SIZE`].
    fn size(&self) -> u32;
    fn read(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), DiskError>;
    /// Erases the [`ERASE_SIZE`] block starting at `offset`.
    fn erase_block(&mut self, offset: u32) -> Result<(), DiskError>;
    /// Programs whole pages: `offset` and `data.len()` are multiples of
    /// [`PAGE_SIZE`].
    fn program(&mut self, offset: u32, data: &[u8]) -> Result<(), DiskError>;
}

/// Storage addressed in [`SECTOR_SIZE`] sectors.
pub trait BlockDevice {
    fn sector_count(&self) -> u32;
    fn read_sector(&mut self, lba: u32, buf: &mut Sector) -> Result<(), DiskError>;
    fn write_sector(&mut self, lba: u32, buf: &Sector) -> Result<(), DiskError>;
}

struct CachedBlock {
    block: u32,
    dirty: bool,
    data: [u8; ERASE_SIZE],
}

/// A block device on a flash region.
///
/// Writes go to a cached copy of one erase block, which is written back by
/// [`flush`](Self::flush) or when a sector in another block is written. Call
/// `flush` before anything else reads the flash directly and before power may
/// be lost. The cache is allocated on the first write and freed by
/// [`release_cache`](Self::release_cache).
pub struct FlashDisk<F> {
    flash: F,
    cache: Option<Box<CachedBlock>>,
}

impl<F: Flash> FlashDisk<F> {
    pub const fn new(flash: F) -> Self {
        Self { flash, cache: None }
    }

    pub fn flash_mut(&mut self) -> &mut F {
        &mut self.flash
    }

    /// Whether written sectors are still waiting in the cache.
    pub fn is_dirty(&self) -> bool {
        self.cache.as_ref().is_some_and(|cache| cache.dirty)
    }

    /// Writes the cached block back to flash if it changed. Only pages that
    /// differ are programmed, and the block is erased only when some bit has
    /// to go from 0 to 1.
    pub fn flush(&mut self) -> Result<(), DiskError> {
        let Some(cache) = self.cache.as_mut() else {
            return Ok(());
        };
        if !cache.dirty {
            return Ok(());
        }
        let base = cache.block * ERASE_SIZE as u32;
        let mut changed_pages = 0u16;
        let mut needs_erase = false;
        let mut old = [0u8; PAGE_SIZE];
        for (page, new) in cache.data.chunks_exact(PAGE_SIZE).enumerate() {
            self.flash
                .read(base + (page * PAGE_SIZE) as u32, &mut old)?;
            if old[..] != new[..] {
                changed_pages |= 1 << page;
                needs_erase |= old.iter().zip(new).any(|(o, n)| n & !o != 0);
            }
        }
        if needs_erase {
            self.flash.erase_block(base)?;
        }
        for (page, new) in cache.data.chunks_exact(PAGE_SIZE).enumerate() {
            let program = if needs_erase {
                // Everything is 0xFF now, so only pages with data need writing.
                new.iter().any(|&b| b != 0xFF)
            } else {
                changed_pages & (1 << page) != 0
            };
            if program {
                self.flash.program(base + (page * PAGE_SIZE) as u32, new)?;
            }
        }
        cache.dirty = false;
        Ok(())
    }

    /// Flushes, then frees the cache's memory.
    pub fn release_cache(&mut self) -> Result<(), DiskError> {
        self.flush()?;
        self.cache = None;
        Ok(())
    }

    fn check_lba(&self, lba: u32) -> Result<(), DiskError> {
        if lba < self.sector_count() {
            Ok(())
        } else {
            Err(DiskError)
        }
    }
}

impl<F: Flash> BlockDevice for FlashDisk<F> {
    fn sector_count(&self) -> u32 {
        self.flash.size() / SECTOR_SIZE as u32
    }

    fn read_sector(&mut self, lba: u32, buf: &mut Sector) -> Result<(), DiskError> {
        self.check_lba(lba)?;
        let block = lba / SECTORS_PER_BLOCK;
        let within = (lba % SECTORS_PER_BLOCK) as usize * SECTOR_SIZE;
        match &self.cache {
            Some(cache) if cache.block == block => {
                buf.copy_from_slice(&cache.data[within..within + SECTOR_SIZE]);
                Ok(())
            }
            _ => self.flash.read(lba * SECTOR_SIZE as u32, buf),
        }
    }

    fn write_sector(&mut self, lba: u32, buf: &Sector) -> Result<(), DiskError> {
        self.check_lba(lba)?;
        let block = lba / SECTORS_PER_BLOCK;
        let within = (lba % SECTORS_PER_BLOCK) as usize * SECTOR_SIZE;
        if self
            .cache
            .as_ref()
            .is_some_and(|cache| cache.block != block)
        {
            self.flush()?;
        }
        let cache = match &mut self.cache {
            Some(cache) => cache,
            empty @ None => empty.insert(Box::new(CachedBlock {
                block,
                dirty: false,
                data: [0xFF; ERASE_SIZE],
            })),
        };
        if cache.block != block || !cache.dirty {
            // Reload: the cache may hold another (clean) block, or be new.
            self.flash
                .read(block * ERASE_SIZE as u32, &mut cache.data)?;
            cache.block = block;
        }
        if cache.data[within..within + SECTOR_SIZE] != buf[..] {
            cache.data[within..within + SECTOR_SIZE].copy_from_slice(buf);
            cache.dirty = true;
        }
        Ok(())
    }
}

/// Flash kept in memory, with real erase and program rules. For tests.
#[cfg(test)]
pub mod mem {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    pub struct MemFlash {
        pub data: Vec<u8>,
        pub erases: usize,
        pub programs: usize,
    }

    impl MemFlash {
        /// Starts erased, like a new chip.
        pub fn new(size: usize) -> Self {
            assert_eq!(size % ERASE_SIZE, 0);
            Self {
                data: vec![0xFF; size],
                erases: 0,
                programs: 0,
            }
        }
    }

    impl Flash for MemFlash {
        fn size(&self) -> u32 {
            self.data.len() as u32
        }

        fn read(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), DiskError> {
            let start = offset as usize;
            let src = self.data.get(start..start + buf.len()).ok_or(DiskError)?;
            buf.copy_from_slice(src);
            Ok(())
        }

        fn erase_block(&mut self, offset: u32) -> Result<(), DiskError> {
            let start = offset as usize;
            assert_eq!(start % ERASE_SIZE, 0, "unaligned erase");
            self.data
                .get_mut(start..start + ERASE_SIZE)
                .ok_or(DiskError)?
                .fill(0xFF);
            self.erases += 1;
            Ok(())
        }

        fn program(&mut self, offset: u32, data: &[u8]) -> Result<(), DiskError> {
            let start = offset as usize;
            assert_eq!(start % PAGE_SIZE, 0, "unaligned program");
            assert_eq!(data.len() % PAGE_SIZE, 0, "partial page program");
            let dst = self
                .data
                .get_mut(start..start + data.len())
                .ok_or(DiskError)?;
            for (d, s) in dst.iter_mut().zip(data) {
                *d &= s;
            }
            self.programs += 1;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mem::MemFlash;
    use super::*;

    fn disk() -> FlashDisk<MemFlash> {
        FlashDisk::new(MemFlash::new(4 * ERASE_SIZE))
    }

    fn sector(byte: u8) -> Sector {
        [byte; SECTOR_SIZE]
    }

    fn read(disk: &mut FlashDisk<MemFlash>, lba: u32) -> Sector {
        let mut buf = [0; SECTOR_SIZE];
        disk.read_sector(lba, &mut buf).unwrap();
        buf
    }

    #[test]
    fn writes_are_visible_before_and_after_flush() {
        let mut disk = disk();
        disk.write_sector(3, &sector(0x12)).unwrap();
        assert_eq!(read(&mut disk, 3), sector(0x12));
        assert_eq!(read(&mut disk, 2), sector(0xFF));
        disk.flush().unwrap();
        assert_eq!(read(&mut disk, 3), sector(0x12));
        assert_eq!(disk.flash_mut().data[3 * SECTOR_SIZE], 0x12);
    }

    #[test]
    fn writes_within_one_block_cost_one_flush() {
        let mut disk = disk();
        for lba in 0..8 {
            disk.write_sector(lba, &sector(lba as u8)).unwrap();
        }
        assert_eq!(disk.flash_mut().programs, 0);
        disk.flush().unwrap();
        // Fresh flash needs no erase.
        assert_eq!(disk.flash_mut().erases, 0);
        for lba in 0..8 {
            assert_eq!(read(&mut disk, lba), sector(lba as u8));
        }
    }

    #[test]
    fn writing_another_block_flushes_the_cached_one() {
        let mut disk = disk();
        disk.write_sector(0, &sector(1)).unwrap();
        disk.write_sector(8, &sector(2)).unwrap();
        assert_eq!(disk.flash_mut().data[0], 1);
        assert_eq!(disk.flash_mut().data[8 * SECTOR_SIZE], 0xFF);
        disk.release_cache().unwrap();
        assert_eq!(disk.flash_mut().data[8 * SECTOR_SIZE], 2);
        assert!(!disk.is_dirty());
    }

    #[test]
    fn rewriting_erases_and_keeps_the_rest_of_the_block() {
        let mut disk = disk();
        disk.write_sector(0, &sector(0x0F)).unwrap();
        disk.write_sector(1, &sector(0x33)).unwrap();
        disk.flush().unwrap();
        disk.write_sector(0, &sector(0xF0)).unwrap();
        disk.flush().unwrap();
        assert_eq!(disk.flash_mut().erases, 1);
        assert_eq!(read(&mut disk, 0), sector(0xF0));
        assert_eq!(read(&mut disk, 1), sector(0x33));
    }

    #[test]
    fn unchanged_writes_touch_nothing() {
        let mut disk = disk();
        disk.write_sector(0, &sector(0x55)).unwrap();
        disk.flush().unwrap();
        let programs = disk.flash_mut().programs;
        disk.write_sector(0, &sector(0x55)).unwrap();
        assert!(!disk.is_dirty());
        disk.flush().unwrap();
        assert_eq!(disk.flash_mut().programs, programs);
        assert_eq!(disk.flash_mut().erases, 0);
    }

    #[test]
    fn clearing_bits_needs_no_erase() {
        let mut disk = disk();
        disk.write_sector(0, &sector(0xF0)).unwrap();
        disk.flush().unwrap();
        disk.write_sector(0, &sector(0x30)).unwrap();
        disk.flush().unwrap();
        assert_eq!(disk.flash_mut().erases, 0);
        assert_eq!(read(&mut disk, 0), sector(0x30));
    }

    #[test]
    fn out_of_range_sectors_fail() {
        let mut disk = disk();
        let mut buf = sector(0);
        assert_eq!(disk.read_sector(32, &mut buf), Err(DiskError));
        assert_eq!(disk.write_sector(32, &buf), Err(DiskError));
    }
}
