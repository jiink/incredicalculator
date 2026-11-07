//! The flash chip, shared between saved settings and the script drive: a
//! 512 KiB FAT12 filesystem just below the settings sectors, which holds the
//! scripted apps and is shared with a computer by the "USB Drive" app.
//!
//! Flash layout (2 MiB, as `memory.x` assumes):
//!
//! | Range                 | Use                                   |
//! |-----------------------|---------------------------------------|
//! | `0x000000..0x17C000`  | firmware (`FLASH` in memory.x)        |
//! | `0x17C000..0x1FC000`  | script drive ([`DRIVE_FLASH_RANGE`])  |
//! | `0x1FC000..0x200000`  | settings (`settings_storage.rs`)      |
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::{Cell, RefCell};
use core::ops::Range;

use defmt::*;
use embassy_rp::flash::{Blocking, ERASE_SIZE, Flash};
use embassy_rp::peripherals::FLASH;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use flash_disk::fat::{self, FatError};
use flash_disk::{DiskError, FlashDisk};
use incredicalculator_core::platform::{MAX_SCRIPT_BYTES, SCRIPT_EXTENSION};

// Must match the 2 MiB assumed by memory.x, whose FLASH region stops short of
// the drive and settings so firmware never overlaps them.
pub const FLASH_SIZE: usize = 2 * 1024 * 1024;
pub const SETTINGS_SECTORS: usize = 4;
const DRIVE_BYTES: usize = 512 * 1024;
const DRIVE_END: usize = FLASH_SIZE - SETTINGS_SECTORS * ERASE_SIZE;
pub const DRIVE_FLASH_RANGE: Range<u32> = (DRIVE_END - DRIVE_BYTES) as u32..DRIVE_END as u32;

pub type BoardFlash = Flash<'static, FLASH, Blocking, FLASH_SIZE>;

/// The flash driver, used only from core 0's thread-mode tasks. Flash
/// operations stall core 1 and turn interrupts off themselves, so they must
/// not run inside a critical section (core 1 could be spinning on it).
static BOARD_FLASH: Mutex<ThreadModeRawMutex, RefCell<Option<BoardFlash>>> =
    Mutex::new(RefCell::new(None));

pub fn init_flash(flash: BoardFlash) {
    BOARD_FLASH.lock(|cell| *cell.borrow_mut() = Some(flash));
}

fn with_flash<R>(f: impl FnOnce(&mut BoardFlash) -> R) -> R {
    BOARD_FLASH.lock(|cell| f(cell.borrow_mut().as_mut().expect("flash not set up")))
}

/// The whole flash chip, for `sequential-storage`.
pub struct SharedFlash;

impl embedded_storage_async::nor_flash::ErrorType for SharedFlash {
    type Error = embassy_rp::flash::Error;
}

impl embedded_storage_async::nor_flash::ReadNorFlash for SharedFlash {
    const READ_SIZE: usize = 1;

    async fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        with_flash(|flash| flash.blocking_read(offset, bytes))
    }

    fn capacity(&self) -> usize {
        FLASH_SIZE
    }
}

impl embedded_storage_async::nor_flash::NorFlash for SharedFlash {
    const WRITE_SIZE: usize = 1;
    const ERASE_SIZE: usize = ERASE_SIZE;

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        with_flash(|flash| flash.blocking_erase(from, to))
    }

    async fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        with_flash(|flash| flash.blocking_write(offset, bytes))
    }
}

/// The drive's region of the flash chip.
pub struct DriveFlash;

impl flash_disk::Flash for DriveFlash {
    fn size(&self) -> u32 {
        DRIVE_BYTES as u32
    }

    fn read(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), DiskError> {
        with_flash(|flash| flash.blocking_read(DRIVE_FLASH_RANGE.start + offset, buf))
            .map_err(|_| DiskError)
    }

    fn erase_block(&mut self, offset: u32) -> Result<(), DiskError> {
        let start = DRIVE_FLASH_RANGE.start + offset;
        with_flash(|flash| flash.blocking_erase(start, start + ERASE_SIZE as u32))
            .map_err(|_| DiskError)
    }

    fn program(&mut self, offset: u32, data: &[u8]) -> Result<(), DiskError> {
        with_flash(|flash| flash.blocking_write(DRIVE_FLASH_RANGE.start + offset, data))
            .map_err(|_| DiskError)
    }
}

pub type Disk = FlashDisk<DriveFlash>;

static DISK: Mutex<ThreadModeRawMutex, RefCell<Disk>> =
    Mutex::new(RefCell::new(FlashDisk::new(DriveFlash)));

/// Whether the drive holds a filesystem the calculator may read: false while
/// it is shared over USB, or if it can't be read.
static MOUNTED: Mutex<ThreadModeRawMutex, Cell<bool>> = Mutex::new(Cell::new(false));

/// Runs `f` on the drive. Never hold this across an `.await`.
pub fn with_disk<R>(f: impl FnOnce(&mut Disk) -> R) -> R {
    DISK.lock(|disk| f(&mut disk.borrow_mut()))
}

/// Whether the drive holds a FAT filesystem.
pub fn has_filesystem() -> bool {
    with_disk(|disk| fat::check(disk)).is_ok()
}

/// Mounts the drive, first formatting it (with the example script) if it
/// holds no FAT filesystem: the region is only ever used for this drive, so
/// anything else there is left over from older firmware or a fresh chip.
pub fn mount_or_format() {
    let result = with_disk(|disk| match fat::check(disk) {
        Err(FatError::NotFat) => {
            info!("script drive: no filesystem, formatting");
            let (name, source) = incredicalculator_core::script::EXAMPLE_SCRIPT;
            let volume_id = embassy_time::Instant::now().as_ticks() as u32;
            fat::format(disk, volume_id, &[(name, source.as_bytes())])
                .and_then(|()| disk.release_cache().map_err(FatError::from))
                .and_then(|()| fat::check(disk))
        }
        result => result,
    });
    match result {
        Ok(()) => info!("script drive: mounted"),
        Err(e) => warn!("script drive: can't mount: {}", defmt::Debug2Format(&e)),
    }
    MOUNTED.lock(|cell| cell.set(result.is_ok()));
}

/// Stops reading the drive, before a computer gets to change it.
pub fn unmount() {
    MOUNTED.lock(|cell| cell.set(false));
}

fn is_mounted() -> bool {
    MOUNTED.lock(Cell::get)
}

/// Names of the `*.lua` files in the drive's root directory, or `None` if it
/// isn't mounted.
pub fn list_scripts() -> Option<Vec<String>> {
    if !is_mounted() {
        return None;
    }
    match with_disk(|disk| fat::list_root(disk)) {
        Ok(entries) => Some(
            entries
                .into_iter()
                .filter(|entry| !entry.is_hidden() && is_script_name(&entry.name))
                .map(|entry| entry.name)
                .collect(),
        ),
        Err(e) => {
            warn!("script drive: can't list: {}", defmt::Debug2Format(&e));
            None
        }
    }
}

fn is_script_name(name: &str) -> bool {
    name.rsplit_once('.')
        .is_some_and(|(stem, ext)| !stem.is_empty() && ext.eq_ignore_ascii_case(SCRIPT_EXTENSION))
}

/// Source of a script on the drive. Scripts that are too big or not UTF-8
/// come back as a script that fails with an explanation, so they show up as
/// an app with an error instead of disappearing.
pub fn read_script(name: &str) -> Option<String> {
    if !is_mounted() {
        return None;
    }
    let data = with_disk(|disk| fat::read_file(disk, name, MAX_SCRIPT_BYTES));
    let problem = match data {
        Ok(data) => match String::from_utf8(data) {
            Ok(source) => return Some(source),
            Err(_) => alloc::format!("{name} is not UTF-8 text."),
        },
        Err(FatError::TooBig { size }) => alloc::format!(
            "{name} is {} KiB; scripts can be at most {} KiB.",
            size.div_ceil(1024),
            MAX_SCRIPT_BYTES / 1024
        ),
        Err(e) => {
            warn!("script drive: can't read: {}", defmt::Debug2Format(&e));
            alloc::format!("{name} could not be read from the drive.")
        }
    };
    Some(error_script(&problem))
}

/// A script that stops with `message` as soon as it loads.
fn error_script(message: &str) -> String {
    let mut quoted = String::new();
    for c in message.chars() {
        match c {
            '\\' | '"' => {
                quoted.push('\\');
                quoted.push(c);
            }
            c if c.is_control() => quoted.push(' '),
            c => quoted.push(c),
        }
    }
    alloc::format!("error(\"{quoted}\", 0)")
}
