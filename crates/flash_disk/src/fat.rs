//! Just enough FAT for a small drive the PC edits: formatting (FAT12), and
//! listing and reading files in the root directory (FAT12 or FAT16, with long
//! file names). Subdirectories are ignored.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::{BlockDevice, DiskError, SECTOR_SIZE, Sector};

/// Volume label written by [`format`].
pub const VOLUME_LABEL: &[u8; 11] = b"INCREDICALC";
/// Root directory entries made by [`format`]. Long names take extra entries.
pub const ROOT_ENTRIES: u16 = 128;

const DIR_ENTRY_SIZE: usize = 32;
const ATTR_READ_ONLY: u8 = 0x01;
const ATTR_HIDDEN: u8 = 0x02;
const ATTR_SYSTEM: u8 = 0x04;
const ATTR_VOLUME_ID: u8 = 0x08;
const ATTR_DIRECTORY: u8 = 0x10;
const ATTR_ARCHIVE: u8 = 0x20;
const ATTR_LONG_NAME: u8 = ATTR_READ_ONLY | ATTR_HIDDEN | ATTR_SYSTEM | ATTR_VOLUME_ID;
/// Bits of a short entry's "NT reserved" byte saying the base name or
/// extension is shown in lowercase (used by Windows, Linux and macOS for
/// all-lowercase 8.3 names instead of a long name entry).
const NT_LOWER_BASE: u8 = 0x08;
const NT_LOWER_EXT: u8 = 0x10;
/// Long names hold at most 255 UTF-16 units, 13 per entry.
const MAX_LFN_ENTRIES: usize = 20;
const LFN_CHARS_PER_ENTRY: usize = 13;
/// FAT12 volumes have fewer clusters than this; FAT16 below 65525.
const FAT12_MAX_CLUSTERS: u32 = 4085;
const FAT16_MAX_CLUSTERS: u32 = 65525;
const MEDIA_FIXED: u8 = 0xF8;
/// 2026-01-01, in FAT's date format, for files made by [`format`].
const FORMAT_DATE: u16 = ((2026 - 1980) << 9) | (1 << 5) | 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FatError {
    Disk,
    /// The first sector does not hold a FAT12 or FAT16 boot sector.
    NotFat,
    NotFound,
    /// The file is bigger than the caller allows.
    TooBig {
        size: u32,
    },
    /// A cluster chain or directory entry makes no sense.
    Corrupt,
    /// [`format`] was given more than the volume holds, or a name that does
    /// not fit 8.3.
    DoesNotFit,
}

impl From<DiskError> for FatError {
    fn from(_: DiskError) -> Self {
        FatError::Disk
    }
}

/// A file in the root directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirEntry {
    /// Long name if there is one, otherwise the 8.3 name (`NAME.EXT`).
    pub name: String,
    pub size: u32,
    first_cluster: u32,
    attributes: u8,
}

impl DirEntry {
    /// Hidden or system files, and names starting with a dot like macOS's
    /// `._name` resource forks. PCs make these on their own.
    pub fn is_hidden(&self) -> bool {
        self.attributes & (ATTR_HIDDEN | ATTR_SYSTEM) != 0 || self.name.starts_with('.')
    }
}

/// Whether the device has never been written: its first sector is erased
/// flash. Such a device can be formatted without losing anything.
pub fn is_blank<D: BlockDevice>(dev: &mut D) -> Result<bool, FatError> {
    let mut sector = [0; SECTOR_SIZE];
    dev.read_sector(0, &mut sector)?;
    Ok(sector.iter().all(|&b| b == 0xFF))
}

/// Formats the whole device as FAT12 (no partition table, like a floppy),
/// with `files` (8.3 names) in the root directory.
pub fn format<D: BlockDevice>(
    dev: &mut D,
    volume_id: u32,
    files: &[(&str, &[u8])],
) -> Result<(), FatError> {
    let total_sectors = dev.sector_count();
    let reserved = 1u32;
    let fat_count = 2u32;
    let root_sectors = ROOT_ENTRIES as u32 * DIR_ENTRY_SIZE as u32 / SECTOR_SIZE as u32;
    // Smallest FAT that covers the clusters left over after it.
    let mut fat_sectors = 1;
    let clusters = loop {
        let used = reserved + fat_count * fat_sectors + root_sectors;
        let clusters = total_sectors
            .checked_sub(used)
            .ok_or(FatError::DoesNotFit)?;
        let fat_bytes = (clusters + 2) * 3 / 2 + 1;
        if fat_bytes.div_ceil(SECTOR_SIZE as u32) <= fat_sectors {
            break clusters;
        }
        fat_sectors += 1;
    };
    if clusters >= FAT12_MAX_CLUSTERS {
        return Err(FatError::DoesNotFit);
    }
    let fat_start = reserved;
    let root_start = fat_start + fat_count * fat_sectors;
    let data_start = root_start + root_sectors;

    let mut boot = [0u8; SECTOR_SIZE];
    boot[0..3].copy_from_slice(&[0xEB, 0x3C, 0x90]);
    boot[3..11].copy_from_slice(b"MSWIN4.1");
    boot[11..13].copy_from_slice(&(SECTOR_SIZE as u16).to_le_bytes());
    boot[13] = 1; // sectors per cluster
    boot[14..16].copy_from_slice(&(reserved as u16).to_le_bytes());
    boot[16] = fat_count as u8;
    boot[17..19].copy_from_slice(&ROOT_ENTRIES.to_le_bytes());
    if total_sectors < 0x10000 {
        boot[19..21].copy_from_slice(&(total_sectors as u16).to_le_bytes());
    } else {
        boot[32..36].copy_from_slice(&total_sectors.to_le_bytes());
    }
    boot[21] = MEDIA_FIXED;
    boot[22..24].copy_from_slice(&(fat_sectors as u16).to_le_bytes());
    boot[24..26].copy_from_slice(&32u16.to_le_bytes()); // sectors per track
    boot[26..28].copy_from_slice(&2u16.to_le_bytes()); // heads
    boot[36] = 0x80; // drive number
    boot[38] = 0x29; // extended boot signature
    boot[39..43].copy_from_slice(&volume_id.to_le_bytes());
    boot[43..54].copy_from_slice(VOLUME_LABEL);
    boot[54..62].copy_from_slice(b"FAT12   ");
    boot[510] = 0x55;
    boot[511] = 0xAA;

    let mut fat = vec![0u8; fat_sectors as usize * SECTOR_SIZE];
    set_fat12(&mut fat, 0, 0xF00 | MEDIA_FIXED as u16);
    set_fat12(&mut fat, 1, 0xFFF);
    let mut root = vec![0u8; root_sectors as usize * SECTOR_SIZE];
    let mut label = [0u8; DIR_ENTRY_SIZE];
    label[0..11].copy_from_slice(VOLUME_LABEL);
    label[11] = ATTR_VOLUME_ID;
    root[..DIR_ENTRY_SIZE].copy_from_slice(&label);

    let mut next_cluster = 2u32;
    let mut sector = [0u8; SECTOR_SIZE];
    for (index, (name, data)) in files.iter().enumerate() {
        let (short_name, nt_flags) = short_name(name).ok_or(FatError::DoesNotFit)?;
        let cluster_count = data.len().div_ceil(SECTOR_SIZE) as u32;
        if next_cluster + cluster_count > clusters + 2 || index + 1 >= ROOT_ENTRIES as usize {
            return Err(FatError::DoesNotFit);
        }
        let first_cluster = if cluster_count == 0 { 0 } else { next_cluster };
        for (i, chunk) in data.chunks(SECTOR_SIZE).enumerate() {
            let cluster = next_cluster + i as u32;
            sector.fill(0);
            sector[..chunk.len()].copy_from_slice(chunk);
            dev.write_sector(data_start + cluster - 2, &sector)?;
            let next = if i as u32 + 1 == cluster_count {
                0xFFF
            } else {
                cluster as u16 + 1
            };
            set_fat12(&mut fat, cluster, next);
        }
        next_cluster += cluster_count;

        let entry = &mut root[(index + 1) * DIR_ENTRY_SIZE..(index + 2) * DIR_ENTRY_SIZE];
        entry[0..11].copy_from_slice(&short_name);
        entry[11] = ATTR_ARCHIVE;
        entry[12] = nt_flags;
        for date_offset in [16, 18, 24] {
            entry[date_offset..date_offset + 2].copy_from_slice(&FORMAT_DATE.to_le_bytes());
        }
        entry[26..28].copy_from_slice(&(first_cluster as u16).to_le_bytes());
        entry[28..32].copy_from_slice(&(data.len() as u32).to_le_bytes());
    }

    for copy in 0..fat_count {
        for (i, chunk) in fat.chunks_exact(SECTOR_SIZE).enumerate() {
            sector.copy_from_slice(chunk);
            dev.write_sector(fat_start + copy * fat_sectors + i as u32, &sector)?;
        }
    }
    for (i, chunk) in root.chunks_exact(SECTOR_SIZE).enumerate() {
        sector.copy_from_slice(chunk);
        dev.write_sector(root_start + i as u32, &sector)?;
    }
    // The boot sector goes last, so an interrupted format is not mounted.
    dev.write_sector(0, &boot)?;
    Ok(())
}

fn set_fat12(fat: &mut [u8], cluster: u32, value: u16) {
    let offset = (cluster + cluster / 2) as usize;
    if cluster % 2 == 0 {
        fat[offset] = value as u8;
        fat[offset + 1] = (fat[offset + 1] & 0xF0) | ((value >> 8) as u8 & 0x0F);
    } else {
        fat[offset] = (fat[offset] & 0x0F) | ((value as u8 & 0x0F) << 4);
        fat[offset + 1] = (value >> 4) as u8;
    }
}

/// The 8.3 directory name for `name`, and the lowercase flags to show it as
/// given. `None` if it needs a long name: too long, mixed case within the
/// base or extension, or characters 8.3 doesn't allow.
fn short_name(name: &str) -> Option<([u8; 11], u8)> {
    let (base, ext) = match name.rsplit_once('.') {
        Some((base, ext)) => (base, ext),
        None => (name, ""),
    };
    if base.is_empty() || base.len() > 8 || ext.len() > 3 {
        return None;
    }
    let mut out = [b' '; 11];
    let mut flags = 0;
    for (part, field, lower_flag) in [(base, 0..8, NT_LOWER_BASE), (ext, 8..11, NT_LOWER_EXT)] {
        let has_lower = part.bytes().any(|b| b.is_ascii_lowercase());
        let has_upper = part.bytes().any(|b| b.is_ascii_uppercase());
        if has_lower && has_upper {
            return None;
        }
        if has_lower {
            flags |= lower_flag;
        }
        for (dst, b) in out[field].iter_mut().zip(part.bytes()) {
            if !(b.is_ascii_alphanumeric() || b"!#$%&'()-@^_`{}~".contains(&b)) {
                return None;
            }
            *dst = b.to_ascii_uppercase();
        }
    }
    Some((out, flags))
}

/// A mounted FAT12 or FAT16 volume. It holds only the layout, so it stays
/// valid as long as nobody reformats the device.
#[derive(Clone, Copy, Debug)]
pub struct Volume {
    fat_start: u32,
    root_start: u32,
    root_sectors: u32,
    data_start: u32,
    sectors_per_cluster: u32,
    cluster_count: u32,
    fat16: bool,
}

impl Volume {
    pub fn mount<D: BlockDevice>(dev: &mut D) -> Result<Volume, FatError> {
        let mut boot = [0u8; SECTOR_SIZE];
        dev.read_sector(0, &mut boot)?;
        let u16_at = |i: usize| u16::from_le_bytes([boot[i], boot[i + 1]]) as u32;
        let u32_at =
            |i: usize| u32::from_le_bytes([boot[i], boot[i + 1], boot[i + 2], boot[i + 3]]);
        let bytes_per_sector = u16_at(11);
        let sectors_per_cluster = boot[13] as u32;
        let reserved = u16_at(14);
        let fat_count = boot[16] as u32;
        let root_entries = u16_at(17);
        let total_sectors = match u16_at(19) {
            0 => u32_at(32),
            n => n,
        };
        let fat_sectors = u16_at(22);
        if boot[510..512] != [0x55, 0xAA]
            || bytes_per_sector != SECTOR_SIZE as u32
            || !sectors_per_cluster.is_power_of_two()
            || reserved == 0
            || fat_count == 0
            // FAT32 has no fixed root directory and a 32-bit FAT size.
            || root_entries == 0
            || fat_sectors == 0
            || total_sectors > dev.sector_count()
        {
            return Err(FatError::NotFat);
        }
        let fat_start = reserved;
        let root_start = fat_start + fat_count * fat_sectors;
        let root_sectors = (root_entries * DIR_ENTRY_SIZE as u32).div_ceil(SECTOR_SIZE as u32);
        let data_start = root_start + root_sectors;
        let cluster_count = total_sectors
            .checked_sub(data_start)
            .ok_or(FatError::NotFat)?
            / sectors_per_cluster;
        if cluster_count >= FAT16_MAX_CLUSTERS {
            return Err(FatError::NotFat);
        }
        let fat16 = cluster_count >= FAT12_MAX_CLUSTERS;
        let fat_entry_bytes = if fat16 {
            (cluster_count + 2) * 2
        } else {
            (cluster_count + 2) * 3 / 2
        };
        if fat_sectors * (SECTOR_SIZE as u32) < fat_entry_bytes {
            return Err(FatError::NotFat);
        }
        Ok(Volume {
            fat_start,
            root_start,
            root_sectors,
            data_start,
            sectors_per_cluster,
            cluster_count,
            fat16,
        })
    }

    /// Files (not directories) in the root directory, in directory order.
    pub fn list_root<D: BlockDevice>(&self, dev: &mut D) -> Result<Vec<DirEntry>, FatError> {
        let mut files = Vec::new();
        let mut lfn = LongName::new();
        let mut sector = [0u8; SECTOR_SIZE];
        for s in 0..self.root_sectors {
            dev.read_sector(self.root_start + s, &mut sector)?;
            for raw in sector.chunks_exact(DIR_ENTRY_SIZE) {
                let entry: &[u8; DIR_ENTRY_SIZE] = raw.try_into().unwrap();
                match entry[0] {
                    0x00 => return Ok(files),
                    0xE5 => {
                        lfn.clear();
                        continue;
                    }
                    _ => {}
                }
                let attributes = entry[11];
                if attributes & 0x3F == ATTR_LONG_NAME {
                    lfn.add(entry);
                    continue;
                }
                let long_name = lfn.take(entry);
                if attributes & (ATTR_VOLUME_ID | ATTR_DIRECTORY) != 0 {
                    continue;
                }
                // FAT12/16 ignore the high word at offset 20 (FAT32 only).
                let first_cluster = u16::from_le_bytes([entry[26], entry[27]]) as u32;
                files.push(DirEntry {
                    name: long_name.unwrap_or_else(|| short_entry_name(entry)),
                    size: u32::from_le_bytes([entry[28], entry[29], entry[30], entry[31]]),
                    first_cluster,
                    attributes,
                });
            }
        }
        Ok(files)
    }

    /// Finds a root directory file by name, ignoring ASCII case like FAT does.
    pub fn find<D: BlockDevice>(&self, dev: &mut D, name: &str) -> Result<DirEntry, FatError> {
        self.list_root(dev)?
            .into_iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(name))
            .ok_or(FatError::NotFound)
    }

    /// Reads a whole file, failing with [`FatError::TooBig`] if it is longer
    /// than `max_len` bytes.
    pub fn read_file<D: BlockDevice>(
        &self,
        dev: &mut D,
        entry: &DirEntry,
        max_len: usize,
    ) -> Result<Vec<u8>, FatError> {
        let size = entry.size as usize;
        if size > max_len {
            return Err(FatError::TooBig { size: entry.size });
        }
        let cluster_bytes = self.sectors_per_cluster as usize * SECTOR_SIZE;
        let cluster_count = size.div_ceil(cluster_bytes);
        if cluster_count > self.cluster_count as usize {
            return Err(FatError::Corrupt);
        }
        let mut data = Vec::with_capacity(size);
        let mut cluster = entry.first_cluster;
        let mut sector = [0u8; SECTOR_SIZE];
        for _ in 0..cluster_count {
            if data.len() == size {
                return Ok(data);
            }
            if !(2..self.cluster_count + 2).contains(&cluster) {
                return Err(FatError::Corrupt);
            }
            let first_sector = self.data_start + (cluster - 2) * self.sectors_per_cluster;
            for s in 0..self.sectors_per_cluster {
                dev.read_sector(first_sector + s, &mut sector)?;
                let take = (size - data.len()).min(SECTOR_SIZE);
                data.extend_from_slice(&sector[..take]);
                if data.len() == size {
                    return Ok(data);
                }
            }
            cluster = self.fat_entry(dev, cluster)?;
        }
        // Only reached for an empty file with no chain.
        if data.len() == size {
            Ok(data)
        } else {
            Err(FatError::Corrupt)
        }
    }

    /// The next cluster after `cluster`, from the first FAT. End-of-chain
    /// markers come back as values past the last cluster.
    fn fat_entry<D: BlockDevice>(&self, dev: &mut D, cluster: u32) -> Result<u32, FatError> {
        let offset = if self.fat16 {
            cluster * 2
        } else {
            cluster + cluster / 2
        };
        let mut sector: Sector = [0; SECTOR_SIZE];
        let lba = self.fat_start + offset / SECTOR_SIZE as u32;
        let within = (offset % SECTOR_SIZE as u32) as usize;
        dev.read_sector(lba, &mut sector)?;
        let low = sector[within];
        let high = if within + 1 < SECTOR_SIZE {
            sector[within + 1]
        } else {
            dev.read_sector(lba + 1, &mut sector)?;
            sector[0]
        };
        let raw = u16::from_le_bytes([low, high]) as u32;
        Ok(match (self.fat16, cluster % 2) {
            (true, _) => raw,
            (false, 0) => raw & 0xFFF,
            (false, _) => raw >> 4,
        })
    }
}

/// `NAME.EXT` from a short directory entry, lowercased where its flags say.
fn short_entry_name(entry: &[u8; DIR_ENTRY_SIZE]) -> String {
    let mut name = String::new();
    let mut push_part = |bytes: &[u8], lower: bool| {
        let len = bytes.iter().rposition(|&b| b != b' ').map_or(0, |i| i + 1);
        for (i, &b) in bytes[..len].iter().enumerate() {
            // 0x05 stands in for a leading 0xE5, which marks deleted entries.
            let b = if i == 0 && b == 0x05 { 0xE5 } else { b };
            let c = if b.is_ascii() { b as char } else { '?' };
            name.push(if lower { c.to_ascii_lowercase() } else { c });
        }
    };
    push_part(&entry[0..8], entry[12] & NT_LOWER_BASE != 0);
    if entry[8..11] != *b"   " {
        push_part(&[b'.'], false);
        push_part(&entry[8..11], entry[12] & NT_LOWER_EXT != 0);
    }
    name
}

/// Collects the long name entries that come before a short entry.
struct LongName {
    units: Vec<u16>,
    /// Bit `n` is set once part `n` (counting from 0) has been seen.
    seen: u32,
    parts: usize,
    checksum: u8,
}

impl LongName {
    fn new() -> Self {
        Self {
            units: Vec::new(),
            seen: 0,
            parts: 0,
            checksum: 0,
        }
    }

    fn clear(&mut self) {
        self.units.clear();
        self.seen = 0;
        self.parts = 0;
    }

    fn add(&mut self, entry: &[u8; DIR_ENTRY_SIZE]) {
        let order = entry[0];
        let part = (order & 0x1F) as usize;
        if order & 0x40 != 0 {
            // The last part comes first and says how many there are.
            self.clear();
            if part == 0 || part > MAX_LFN_ENTRIES {
                return;
            }
            self.parts = part;
            self.checksum = entry[13];
            self.units.resize(part * LFN_CHARS_PER_ENTRY, 0xFFFF);
        } else if self.parts == 0 || entry[13] != self.checksum {
            self.clear();
            return;
        }
        if part == 0 || part > self.parts {
            self.clear();
            return;
        }
        let start = (part - 1) * LFN_CHARS_PER_ENTRY;
        let char_offsets = (1..11)
            .step_by(2)
            .chain((14..26).step_by(2))
            .chain((28..32).step_by(2));
        for (unit, offset) in self.units[start..start + LFN_CHARS_PER_ENTRY]
            .iter_mut()
            .zip(char_offsets)
        {
            *unit = u16::from_le_bytes([entry[offset], entry[offset + 1]]);
        }
        self.seen |= 1 << (part - 1);
    }

    /// The long name for this short entry, if a complete one with the right
    /// checksum came before it. Clears the collected parts either way.
    fn take(&mut self, short: &[u8; DIR_ENTRY_SIZE]) -> Option<String> {
        let complete = self.parts > 0
            && self.seen == (1u32 << self.parts) - 1
            && self.checksum == short_name_checksum(short[0..11].try_into().unwrap());
        let name = complete.then(|| {
            let end = self
                .units
                .iter()
                .position(|&u| u == 0x0000 || u == 0xFFFF)
                .unwrap_or(self.units.len());
            char::decode_utf16(self.units[..end].iter().copied())
                .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
                .collect::<String>()
        });
        self.clear();
        name.filter(|name| !name.is_empty())
    }
}

fn short_name_checksum(name: &[u8; 11]) -> u8 {
    name.iter()
        .fold(0u8, |sum, &b| sum.rotate_right(1).wrapping_add(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mem::MemFlash;
    use crate::{FlashDisk, PAGE_SIZE};
    use alloc::string::ToString;

    const DISK_BYTES: usize = 512 * 1024;
    const ENTRIES_PER_SECTOR: usize = SECTOR_SIZE / DIR_ENTRY_SIZE;

    fn new_disk() -> FlashDisk<MemFlash> {
        FlashDisk::new(MemFlash::new(DISK_BYTES))
    }

    fn formatted(files: &[(&str, &[u8])]) -> FlashDisk<MemFlash> {
        let mut disk = new_disk();
        assert!(is_blank(&mut disk).unwrap());
        format(&mut disk, 0x1234_5678, files).unwrap();
        disk.release_cache().unwrap();
        disk
    }

    fn names(disk: &mut FlashDisk<MemFlash>) -> Vec<String> {
        let volume = Volume::mount(disk).unwrap();
        volume
            .list_root(disk)
            .unwrap()
            .into_iter()
            .map(|e| e.name)
            .collect()
    }

    fn read(disk: &mut FlashDisk<MemFlash>, name: &str) -> Result<Vec<u8>, FatError> {
        let volume = Volume::mount(disk)?;
        let entry = volume.find(disk, name)?;
        volume.read_file(disk, &entry, 16 * 1024)
    }

    fn sector(disk: &mut FlashDisk<MemFlash>, lba: u32) -> Sector {
        let mut buf = [0; SECTOR_SIZE];
        disk.read_sector(lba, &mut buf).unwrap();
        buf
    }

    /// Root directory layout made by `format` on a 512 KiB disk.
    const ROOT_START: u32 = 1 + 2 * 3;
    const DATA_START: u32 = ROOT_START + 8;

    #[test]
    fn formats_a_512k_disk_as_fat12() {
        let mut disk = formatted(&[]);
        assert!(!is_blank(&mut disk).unwrap());
        let volume = Volume::mount(&mut disk).unwrap();
        assert!(!volume.fat16);
        assert_eq!(volume.root_start, ROOT_START);
        assert_eq!(volume.data_start, DATA_START);
        assert_eq!(volume.cluster_count, 1024 - DATA_START);
        let boot = sector(&mut disk, 0);
        assert_eq!(&boot[54..62], b"FAT12   ");
        assert_eq!(&boot[43..54], VOLUME_LABEL);
        // The FAT holds every cluster's entry.
        assert!((volume.cluster_count + 2) * 3 / 2 <= 3 * SECTOR_SIZE as u32);
        assert!(names(&mut disk).is_empty());
    }

    #[test]
    fn format_writes_files_that_read_back() {
        let big: Vec<u8> = (0..3000u32).map(|i| (i * 7) as u8).collect();
        let mut disk = formatted(&[
            ("bouncer.lua", b"print('hi')"),
            ("BIG.DAT", &big),
            ("empty.lua", b""),
        ]);
        assert_eq!(names(&mut disk), ["bouncer.lua", "BIG.DAT", "empty.lua"]);
        assert_eq!(read(&mut disk, "bouncer.lua").unwrap(), b"print('hi')");
        assert_eq!(read(&mut disk, "big.dat").unwrap(), big);
        assert_eq!(read(&mut disk, "empty.lua").unwrap(), b"");
        assert_eq!(read(&mut disk, "nope.lua"), Err(FatError::NotFound));
    }

    #[test]
    fn format_rejects_names_needing_a_long_name() {
        let mut disk = new_disk();
        assert_eq!(
            format(&mut disk, 0, &[("long-file-name.lua", b"")]),
            Err(FatError::DoesNotFit)
        );
        assert_eq!(
            format(&mut disk, 0, &[("MixedCase.lua", b"")]),
            Err(FatError::DoesNotFit)
        );
    }

    #[test]
    fn short_names() {
        assert_eq!(
            short_name("my-app.lua"),
            Some((*b"MY-APP  LUA", NT_LOWER_BASE | NT_LOWER_EXT))
        );
        assert_eq!(short_name("README"), Some((*b"README     ", 0)));
        assert_eq!(short_name("a b.lua"), None);
        assert_eq!(short_name("toolongname.lua"), None);
    }

    #[test]
    fn too_big_files_are_refused() {
        let data = [b'x'; 5000];
        let mut disk = formatted(&[("big.lua", &data)]);
        let volume = Volume::mount(&mut disk).unwrap();
        let entry = volume.find(&mut disk, "big.lua").unwrap();
        assert_eq!(
            volume.read_file(&mut disk, &entry, 4096),
            Err(FatError::TooBig { size: 5000 })
        );
    }

    #[test]
    fn unformatted_and_garbage_disks_do_not_mount() {
        let mut disk = new_disk();
        assert_eq!(Volume::mount(&mut disk).err(), Some(FatError::NotFat));
        disk.write_sector(0, &[0x42; SECTOR_SIZE]).unwrap();
        assert_eq!(Volume::mount(&mut disk).err(), Some(FatError::NotFat));
        assert!(!is_blank(&mut disk).unwrap());
    }

    /// Writes long name entries plus a short entry into the root directory at
    /// `slot`, the way a PC does for a name that doesn't fit 8.3.
    fn put_long_name(
        disk: &mut FlashDisk<MemFlash>,
        slot: usize,
        long: &str,
        short: &[u8; 11],
        cluster: u16,
        size: u32,
    ) -> usize {
        let units: Vec<u16> = long.encode_utf16().collect();
        let parts = units.len().div_ceil(LFN_CHARS_PER_ENTRY);
        let checksum = short_name_checksum(short);
        let mut entries = Vec::new();
        for part in (1..=parts).rev() {
            let mut e = [0u8; DIR_ENTRY_SIZE];
            e[0] = part as u8 | if part == parts { 0x40 } else { 0 };
            e[11] = ATTR_LONG_NAME;
            e[13] = checksum;
            let offsets = (1..11)
                .step_by(2)
                .chain((14..26).step_by(2))
                .chain((28..32).step_by(2));
            for (i, offset) in offsets.enumerate() {
                let index = (part - 1) * LFN_CHARS_PER_ENTRY + i;
                let unit = match index.cmp(&units.len()) {
                    core::cmp::Ordering::Less => units[index],
                    core::cmp::Ordering::Equal => 0,
                    core::cmp::Ordering::Greater => 0xFFFF,
                };
                e[offset..offset + 2].copy_from_slice(&unit.to_le_bytes());
            }
            entries.push(e);
        }
        let mut short_entry = [0u8; DIR_ENTRY_SIZE];
        short_entry[0..11].copy_from_slice(short);
        short_entry[11] = ATTR_ARCHIVE;
        short_entry[26..28].copy_from_slice(&cluster.to_le_bytes());
        short_entry[28..32].copy_from_slice(&size.to_le_bytes());
        entries.push(short_entry);
        put_entries(disk, slot, &entries);
        slot + entries.len()
    }

    fn put_entries(disk: &mut FlashDisk<MemFlash>, slot: usize, entries: &[[u8; DIR_ENTRY_SIZE]]) {
        for (i, entry) in entries.iter().enumerate() {
            let index = slot + i;
            let lba = ROOT_START + (index / ENTRIES_PER_SECTOR) as u32;
            let mut buf = sector(disk, lba);
            let at = (index % ENTRIES_PER_SECTOR) * DIR_ENTRY_SIZE;
            buf[at..at + DIR_ENTRY_SIZE].copy_from_slice(entry);
            disk.write_sector(lba, &buf).unwrap();
        }
    }

    fn set_fat_entry(disk: &mut FlashDisk<MemFlash>, cluster: u32, value: u16) {
        for copy in 0..2 {
            let start = 1 + copy * 3;
            let mut fat = Vec::new();
            for s in 0..3 {
                fat.extend_from_slice(&sector(disk, start + s));
            }
            set_fat12(&mut fat, cluster, value);
            for s in 0..3 {
                let chunk: Sector = fat[s * SECTOR_SIZE..(s + 1) * SECTOR_SIZE]
                    .try_into()
                    .unwrap();
                disk.write_sector(start + s as u32, &chunk).unwrap();
            }
        }
    }

    #[test]
    fn reads_long_names_and_scattered_clusters_written_by_a_pc() {
        let mut disk = formatted(&[("bouncer.lua", b"-- bouncer")]);
        // A two-cluster file, stored backwards at clusters 341 then 340, so
        // its FAT entries straddle a FAT sector boundary (cluster 341 starts
        // at byte 511).
        let text: Vec<u8> = (0..700u32).map(|i| b'a' + (i % 26) as u8).collect();
        let mut first = [0u8; SECTOR_SIZE];
        first.copy_from_slice(&text[..SECTOR_SIZE]);
        let mut second = [0u8; SECTOR_SIZE];
        second[..text.len() - SECTOR_SIZE].copy_from_slice(&text[SECTOR_SIZE..]);
        disk.write_sector(DATA_START + 341 - 2, &first).unwrap();
        disk.write_sector(DATA_START + 340 - 2, &second).unwrap();
        set_fat_entry(&mut disk, 341, 340);
        set_fat_entry(&mut disk, 340, 0xFFF);
        put_long_name(
            &mut disk,
            2,
            "My Fancy App ünïcode.lua",
            b"MYFANC~1LUA",
            341,
            text.len() as u32,
        );
        disk.release_cache().unwrap();

        assert_eq!(
            names(&mut disk),
            ["bouncer.lua", "My Fancy App ünïcode.lua"]
        );
        assert_eq!(read(&mut disk, "my fancy app ünïcode.lua").unwrap(), text);
    }

    #[test]
    fn deleted_entries_volume_labels_and_bad_long_names_are_handled() {
        let mut disk = formatted(&[("a.lua", b"a"), ("b.lua", b"b")]);
        // Delete a.lua the way a PC does: mark its entry with 0xE5.
        let mut root = sector(&mut disk, ROOT_START);
        root[DIR_ENTRY_SIZE] = 0xE5;
        disk.write_sector(ROOT_START, &root).unwrap();
        // A long name whose checksum doesn't match falls back to 8.3.
        let next = put_long_name(&mut disk, 3, "orphan.lua", b"ORPHAN  LUA", 0, 0);
        let mut wrong = sector(&mut disk, ROOT_START);
        wrong[3 * DIR_ENTRY_SIZE + 13] ^= 0xFF;
        disk.write_sector(ROOT_START, &wrong).unwrap();
        // A hidden macOS resource fork and a directory.
        let mut fork = [0u8; DIR_ENTRY_SIZE];
        fork[0..11].copy_from_slice(b"_B~1    LUA");
        fork[11] = ATTR_HIDDEN;
        let mut dir = [0u8; DIR_ENTRY_SIZE];
        dir[0..11].copy_from_slice(b"FOLDER     ");
        dir[11] = ATTR_DIRECTORY;
        put_entries(&mut disk, next, &[fork, dir]);

        let volume = Volume::mount(&mut disk).unwrap();
        let entries = volume.list_root(&mut disk).unwrap();
        let listed: Vec<(String, bool)> = entries
            .iter()
            .map(|e| (e.name.to_string(), e.is_hidden()))
            .collect();
        assert_eq!(
            listed,
            [
                ("b.lua".to_string(), false),
                ("ORPHAN.LUA".to_string(), false),
                ("_B~1.LUA".to_string(), true),
            ]
        );
    }

    #[test]
    fn chains_that_dont_match_the_file_size_are_corrupt() {
        let mut disk = formatted(&[("file.lua", &[b'x'; 2000])]);
        let volume = Volume::mount(&mut disk).unwrap();
        let mut entry = volume.find(&mut disk, "file.lua").unwrap();
        // Ends before the size says.
        entry.size = 3000;
        assert_eq!(
            volume.read_file(&mut disk, &entry, usize::MAX),
            Err(FatError::Corrupt)
        );
        // Bigger than the volume, as a looping chain would have to be.
        set_fat_entry(&mut disk, 5, 2);
        entry.size = 600_000;
        assert_eq!(
            volume.read_file(&mut disk, &entry, usize::MAX),
            Err(FatError::Corrupt)
        );
    }

    #[test]
    fn remounting_after_the_pc_deletes_a_file() {
        let mut disk = formatted(&[("a.lua", b"a"), ("b.lua", b"b")]);
        let mut root = sector(&mut disk, ROOT_START);
        root[2 * DIR_ENTRY_SIZE] = 0xE5;
        disk.write_sector(ROOT_START, &root).unwrap();
        disk.release_cache().unwrap();
        // A fresh disk over the same flash, as after a power cycle.
        let flash = core::mem::replace(disk.flash_mut(), MemFlash::new(PAGE_SIZE * 16));
        let mut disk = FlashDisk::new(flash);
        assert_eq!(names(&mut disk), ["a.lua"]);
    }
}
