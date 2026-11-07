//! The drive's filesystem, using [rust-fatfs](https://github.com/rafalh/rust-fatfs):
//! formatting it as FAT12, and listing and reading files in its root
//! directory. Everything else (creating, editing and deleting files) is left
//! to the computer, which writes sectors through [`scsi`](crate::scsi).
//!
//! Nothing is kept mounted between calls: the computer may rewrite any
//! sector while the drive is shared, so each call mounts afresh. Mounting
//! only reads the boot sector, and listing and reading never write.

use alloc::string::String;
use alloc::vec::Vec;

use fatfs::{
    FatType, FileSystem, FormatVolumeOptions, FsOptions, IoBase, LossyOemCpConverter,
    NullTimeProvider, Read, Seek, SeekFrom, Write,
};

use crate::{BlockDevice, DiskError, SECTOR_SIZE, Sector};

/// Volume label written by [`format`].
pub const VOLUME_LABEL: &[u8; 11] = b"INCREDICALC";
/// Root directory entries made by [`format`]. Long names take extra entries.
pub const ROOT_ENTRIES: u16 = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FatError {
    /// The flash failed.
    Disk,
    /// The drive holds no FAT filesystem.
    NotFat,
    NotFound,
    /// The file is bigger than the caller allows.
    TooBig {
        size: u32,
    },
    /// The filesystem is damaged.
    Corrupt,
    /// The files given to [`format`] don't fit or can't be named.
    DoesNotFit,
}

impl From<DiskError> for FatError {
    fn from(_: DiskError) -> Self {
        FatError::Disk
    }
}

impl From<fatfs::Error<DiskError>> for FatError {
    fn from(error: fatfs::Error<DiskError>) -> Self {
        match error {
            fatfs::Error::Io(_) => FatError::Disk,
            fatfs::Error::NotFound => FatError::NotFound,
            fatfs::Error::NotEnoughSpace
            | fatfs::Error::InvalidFileNameLength
            | fatfs::Error::UnsupportedFileNameCharacter
            | fatfs::Error::AlreadyExists => FatError::DoesNotFit,
            _ => FatError::Corrupt,
        }
    }
}

impl fatfs::IoError for DiskError {
    fn is_interrupted(&self) -> bool {
        false
    }

    fn new_unexpected_eof_error() -> Self {
        DiskError
    }

    fn new_write_zero_error() -> Self {
        DiskError
    }
}

/// A file in the root directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirEntry {
    pub name: String,
    pub size: u32,
    hidden: bool,
}

impl DirEntry {
    /// Hidden and system files, and dot files like the `._name` files macOS
    /// leaves next to each file it copies.
    pub fn is_hidden(&self) -> bool {
        self.hidden || self.name.starts_with('.')
    }
}

/// Byte-addressed access to a [`BlockDevice`], as rust-fatfs expects.
struct SectorIo<'a, D> {
    dev: &'a mut D,
    pos: u64,
}

impl<D: BlockDevice> SectorIo<'_, D> {
    fn len(&self) -> u64 {
        u64::from(self.dev.sector_count()) * SECTOR_SIZE as u64
    }

    /// The sector holding `pos`, the offset of `pos` in it, and how many of
    /// the `wanted` bytes from there lie in that sector.
    fn locate(&self, wanted: usize) -> (u32, usize, usize) {
        let lba = (self.pos / SECTOR_SIZE as u64) as u32;
        let offset = (self.pos % SECTOR_SIZE as u64) as usize;
        let left = (self.len() - self.pos.min(self.len())) as usize;
        (lba, offset, wanted.min(SECTOR_SIZE - offset).min(left))
    }
}

impl<D> IoBase for SectorIo<'_, D> {
    type Error = DiskError;
}

impl<D: BlockDevice> Read for SectorIo<'_, D> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, DiskError> {
        let (lba, offset, len) = self.locate(buf.len());
        if len == 0 {
            return Ok(0);
        }
        let mut sector: Sector = [0; SECTOR_SIZE];
        self.dev.read_sector(lba, &mut sector)?;
        buf[..len].copy_from_slice(&sector[offset..offset + len]);
        self.pos += len as u64;
        Ok(len)
    }
}

impl<D: BlockDevice> Write for SectorIo<'_, D> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, DiskError> {
        let (lba, offset, len) = self.locate(buf.len());
        if len == 0 {
            return Ok(0);
        }
        let mut sector: Sector = [0; SECTOR_SIZE];
        if len < SECTOR_SIZE {
            self.dev.read_sector(lba, &mut sector)?;
        }
        sector[offset..offset + len].copy_from_slice(&buf[..len]);
        self.dev.write_sector(lba, &sector)?;
        self.pos += len as u64;
        Ok(len)
    }

    fn flush(&mut self) -> Result<(), DiskError> {
        Ok(())
    }
}

impl<D: BlockDevice> Seek for SectorIo<'_, D> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, DiskError> {
        let target = match pos {
            SeekFrom::Start(n) => Some(n),
            SeekFrom::Current(n) => self.pos.checked_add_signed(n),
            SeekFrom::End(n) => self.len().checked_add_signed(n),
        };
        match target {
            Some(target) if target <= self.len() => {
                self.pos = target;
                Ok(target)
            }
            _ => Err(DiskError),
        }
    }
}

type Fs<'a, D> = FileSystem<SectorIo<'a, D>, NullTimeProvider, LossyOemCpConverter>;

fn mount<D: BlockDevice>(dev: &mut D) -> Result<Fs<'_, D>, FatError> {
    let io = SectorIo { dev, pos: 0 };
    let options = FsOptions::new().time_provider(NullTimeProvider::new());
    FileSystem::new(io, options).map_err(|error| match error {
        fatfs::Error::Io(_) => FatError::Disk,
        _ => FatError::NotFat,
    })
}

/// Checks that the drive holds a FAT filesystem.
pub fn check<D: BlockDevice>(dev: &mut D) -> Result<(), FatError> {
    mount(dev).map(drop)
}

/// Formats the whole device as FAT12 and writes `files` (name, contents)
/// into the root directory.
pub fn format<D: BlockDevice>(
    dev: &mut D,
    volume_id: u32,
    files: &[(&str, &[u8])],
) -> Result<(), FatError> {
    let mut io = SectorIo { dev, pos: 0 };
    let options = FormatVolumeOptions::new()
        .fat_type(FatType::Fat12)
        .bytes_per_cluster(SECTOR_SIZE as u32)
        .total_sectors(io.dev.sector_count())
        .max_root_dir_entries(ROOT_ENTRIES)
        .volume_id(volume_id)
        .volume_label(*VOLUME_LABEL);
    fatfs::format_volume(&mut io, options)?;
    io.pos = 0;
    let fs = mount(io.dev)?;
    for &(name, contents) in files {
        let mut file = fs.root_dir().create_file(name)?;
        file.truncate()?;
        file.write_all(contents)?;
        file.flush()?;
    }
    fs.unmount()?;
    Ok(())
}

/// The files in the root directory. Subdirectories are left out.
pub fn list_root<D: BlockDevice>(dev: &mut D) -> Result<Vec<DirEntry>, FatError> {
    let fs = mount(dev)?;
    let mut entries = Vec::new();
    for entry in fs.root_dir().iter() {
        let entry = entry?;
        if !entry.is_file() {
            continue;
        }
        let attributes = entry.attributes();
        entries.push(DirEntry {
            name: entry.file_name(),
            size: entry.len() as u32,
            hidden: attributes
                .intersects(fatfs::FileAttributes::HIDDEN | fatfs::FileAttributes::SYSTEM),
        });
    }
    Ok(entries)
}

/// Contents of the root directory file `name` (matched ignoring case), if it
/// is at most `max_len` bytes.
pub fn read_file<D: BlockDevice>(
    dev: &mut D,
    name: &str,
    max_len: usize,
) -> Result<Vec<u8>, FatError> {
    let fs = mount(dev)?;
    let entry = fs
        .root_dir()
        .iter()
        .filter_map(Result::ok)
        .find(|entry| entry.is_file() && entry.file_name().eq_ignore_ascii_case(name))
        .ok_or(FatError::NotFound)?;
    let size = entry.len() as u32;
    if size as usize > max_len {
        return Err(FatError::TooBig { size });
    }
    let mut data = alloc::vec![0; size as usize];
    entry
        .to_file()
        .read_exact(&mut data)
        .map_err(|error| match error {
            // The cluster chain ends before the size the entry claims.
            fatfs::Error::UnexpectedEof => FatError::Corrupt,
            error => error.into(),
        })?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FlashDisk;
    use crate::mem::MemFlash;
    use alloc::vec;

    const DISK_BYTES: usize = 512 * 1024;

    fn new_disk() -> FlashDisk<MemFlash> {
        FlashDisk::new(MemFlash::new(DISK_BYTES))
    }

    fn formatted(files: &[(&str, &[u8])]) -> FlashDisk<MemFlash> {
        let mut disk = new_disk();
        format(&mut disk, 0x1234_5678, files).unwrap();
        disk.release_cache().unwrap();
        disk
    }

    fn names(disk: &mut FlashDisk<MemFlash>) -> Vec<String> {
        list_root(disk)
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect()
    }

    fn read(disk: &mut FlashDisk<MemFlash>, name: &str) -> Result<Vec<u8>, FatError> {
        read_file(disk, name, 16 * 1024)
    }

    /// Edits the drive the way a computer would, through rust-fatfs.
    fn as_computer(
        disk: &mut FlashDisk<MemFlash>,
        edit: impl FnOnce(&Fs<'_, FlashDisk<MemFlash>>),
    ) {
        let fs = mount(disk).unwrap();
        edit(&fs);
        fs.unmount().unwrap();
        disk.release_cache().unwrap();
    }

    fn write(fs: &Fs<'_, FlashDisk<MemFlash>>, name: &str, contents: &[u8]) {
        let mut file = fs.root_dir().create_file(name).unwrap();
        file.truncate().unwrap();
        file.write_all(contents).unwrap();
        file.flush().unwrap();
    }

    #[test]
    fn formats_a_512k_disk_as_fat12() {
        let mut disk = formatted(&[]);
        let mut boot = [0; SECTOR_SIZE];
        disk.read_sector(0, &mut boot).unwrap();
        assert_eq!(&boot[54..62], b"FAT12   ");
        assert_eq!(&boot[43..54], VOLUME_LABEL);
        assert_eq!(&boot[510..], [0x55, 0xAA]);
        assert!(names(&mut disk).is_empty());
    }

    #[test]
    fn format_writes_files_that_read_back() {
        let big: Vec<u8> = (0..3000u32).map(|i| (i * 7) as u8).collect();
        let mut disk = formatted(&[
            ("bouncer.lua", b"print('hi')"),
            ("BIG.DAT", &big),
            ("empty.lua", b""),
            ("A long name.lua", b"x"),
        ]);
        assert_eq!(
            names(&mut disk),
            ["bouncer.lua", "BIG.DAT", "empty.lua", "A long name.lua"]
        );
        assert_eq!(read(&mut disk, "bouncer.lua").unwrap(), b"print('hi')");
        assert_eq!(read(&mut disk, "big.dat").unwrap(), big);
        assert_eq!(read(&mut disk, "empty.lua").unwrap(), b"");
        assert_eq!(read(&mut disk, "a long NAME.lua").unwrap(), b"x");
        assert_eq!(read(&mut disk, "missing.lua"), Err(FatError::NotFound));
    }

    #[test]
    fn too_big_files_are_refused() {
        let mut disk = formatted(&[("big.lua", &[b'-'; 20_000])]);
        assert_eq!(
            read(&mut disk, "big.lua"),
            Err(FatError::TooBig { size: 20_000 })
        );
        assert_eq!(
            read_file(&mut disk, "big.lua", 20_000).unwrap().len(),
            20_000
        );
    }

    #[test]
    fn files_that_dont_fit_are_refused() {
        let mut disk = new_disk();
        let huge = vec![0u8; DISK_BYTES];
        assert_eq!(
            format(&mut disk, 1, &[("huge.bin", &huge)]),
            Err(FatError::DoesNotFit)
        );
    }

    #[test]
    fn unformatted_and_garbage_disks_do_not_mount() {
        let mut disk = new_disk();
        assert_eq!(check(&mut disk), Err(FatError::NotFat));
        assert_eq!(list_root(&mut disk), Err(FatError::NotFat));

        let mut garbage = new_disk();
        let junk: Sector = core::array::from_fn(|i| (i * 31 + 7) as u8);
        for lba in 0..16 {
            garbage.write_sector(lba, &junk).unwrap();
        }
        assert_eq!(check(&mut garbage), Err(FatError::NotFat));

        assert_eq!(check(&mut formatted(&[])), Ok(()));
    }

    #[test]
    fn sees_what_the_computer_changed() {
        let mut disk = formatted(&[("bouncer.lua", b"old"), ("delete me.lua", b"bye")]);
        as_computer(&mut disk, |fs| {
            fs.root_dir().remove("delete me.lua").unwrap();
            write(fs, "bouncer.lua", b"new and longer");
            write(fs, ".hidden.lua", b"");
            fs.root_dir().create_dir("folder").unwrap();
            // Spans several clusters, reusing the deleted file's.
            write(fs, "Ohm's Law.lua", &[b'x'; 2000]);
        });
        let entries = list_root(&mut disk).unwrap();
        let visible: Vec<&str> = entries
            .iter()
            .filter(|e| !e.is_hidden())
            .map(|e| e.name.as_str())
            .collect();
        assert_eq!(visible, ["bouncer.lua", "Ohm's Law.lua"]);
        assert_eq!(read(&mut disk, "bouncer.lua").unwrap(), b"new and longer");
        assert_eq!(read(&mut disk, "ohm's law.lua").unwrap(), [b'x'; 2000]);
    }

    #[test]
    fn listing_and_reading_never_write() {
        let mut disk = formatted(&[("a.lua", b"hello")]);
        let programs = disk.flash_mut().programs;
        let erases = disk.flash_mut().erases;
        names(&mut disk);
        read(&mut disk, "a.lua").unwrap();
        check(&mut disk).unwrap();
        assert!(!disk.is_dirty());
        assert_eq!(disk.flash_mut().programs, programs);
        assert_eq!(disk.flash_mut().erases, erases);
    }

    #[test]
    fn chains_shorter_than_the_file_size_are_corrupt() {
        let mut disk = formatted(&[("a.lua", &[b'x'; 1500])]);
        // Find the file's size field and claim it is bigger than its chain.
        let mut found = false;
        for lba in 0..64 {
            let mut sector = [0; SECTOR_SIZE];
            disk.read_sector(lba, &mut sector).unwrap();
            for entry in sector.chunks_mut(32) {
                if &entry[..11] == b"A       LUA" {
                    entry[28..32].copy_from_slice(&5000u32.to_le_bytes());
                    found = true;
                }
            }
            disk.write_sector(lba, &sector).unwrap();
        }
        assert!(found);
        assert_eq!(read(&mut disk, "a.lua"), Err(FatError::Corrupt));
    }
}
