//! Heap budget check for the script drive: formatting it, and listing and
//! reading the biggest allowed script, as the RP2350 does. Runs on the host,
//! where pointers are twice as wide, so the numbers are an upper bound.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use flash_disk::fat::{self, Volume};
use flash_disk::{DiskError, ERASE_SIZE, Flash, FlashDisk, PAGE_SIZE};

struct CountingAlloc;

static USED: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            let used = USED.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            PEAK.fetch_max(used, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
        USED.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static GLOBAL: CountingAlloc = CountingAlloc;

/// The device keeps the script drive in flash, not RAM; this flash lives in
/// a fixed array so it doesn't count against the heap.
const DRIVE_BYTES: usize = 512 * 1024;
static mut FLASH: [u8; DRIVE_BYTES] = [0xFF; DRIVE_BYTES];

struct StaticFlash;

impl StaticFlash {
    fn bytes(&mut self) -> &mut [u8; DRIVE_BYTES] {
        // Only one test runs in this file, so there is one user.
        unsafe { &mut *core::ptr::addr_of_mut!(FLASH) }
    }
}

impl Flash for StaticFlash {
    fn size(&self) -> u32 {
        DRIVE_BYTES as u32
    }
    fn read(&mut self, offset: u32, buf: &mut [u8]) -> Result<(), DiskError> {
        let start = offset as usize;
        buf.copy_from_slice(&self.bytes()[start..start + buf.len()]);
        Ok(())
    }
    fn erase_block(&mut self, offset: u32) -> Result<(), DiskError> {
        let start = offset as usize;
        self.bytes()[start..start + ERASE_SIZE].fill(0xFF);
        Ok(())
    }
    fn program(&mut self, offset: u32, data: &[u8]) -> Result<(), DiskError> {
        assert_eq!(data.len() % PAGE_SIZE, 0);
        let start = offset as usize;
        for (d, s) in self.bytes()[start..start + data.len()].iter_mut().zip(data) {
            *d &= s;
        }
        Ok(())
    }
}

/// Peak heap for any one step below. Besides the file itself (16 KiB), the
/// write cache is 4 KiB and formatting builds the FAT and root directory.
const BUDGET: usize = 24 * 1024;
const MAX_SCRIPT_BYTES: usize = 16 * 1024;

fn measure<T>(step: &str, f: impl FnOnce() -> T) -> T {
    let before = USED.load(Ordering::Relaxed);
    PEAK.store(before, Ordering::Relaxed);
    let result = f();
    let peak = PEAK.load(Ordering::Relaxed) - before;
    println!("{step}: peak heap {peak} bytes");
    assert!(
        peak <= BUDGET,
        "{step} peaked at {peak} bytes, over {BUDGET}"
    );
    result
}

#[test]
fn drive_operations_fit_the_heap_budget() {
    let script = "-- x\n".repeat(MAX_SCRIPT_BYTES / 5);
    let mut disk = FlashDisk::new(StaticFlash);
    measure("format", || {
        fat::format(&mut disk, 1, &[("big.lua", script.as_bytes())]).unwrap();
        disk.release_cache().unwrap();
    });
    let volume = measure("mount", || Volume::mount(&mut disk).unwrap());
    let entries = measure("list", || volume.list_root(&mut disk).unwrap());
    assert_eq!(entries.len(), 1);
    let data = measure("read", || {
        volume
            .read_file(&mut disk, &entries[0], MAX_SCRIPT_BYTES)
            .unwrap()
    });
    assert_eq!(data, script.as_bytes());
}
