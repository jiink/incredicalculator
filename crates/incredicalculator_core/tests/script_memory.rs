//! Heap budget check for scripted apps. Runs on the host, where pointers are
//! twice as wide as on the RP2350, so the numbers here are an upper bound for
//! the device.
#![cfg(feature = "scripting")]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use incredicalculator_core::script::{ActiveEngine, EXAMPLE_SCRIPT, ScriptEngine, ScriptHost};

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

/// Peak heap allowed for loading and running the example script on the host.
/// The RP2350's 32-bit pointers make the same script take roughly 60% of
/// this, which has to fit in its heap alongside the compiled apps.
const BUDGET: usize = 64 * 1024;

#[test]
fn example_script_fits_heap_budget() {
    let base = USED.load(Ordering::Relaxed);
    PEAK.store(base, Ordering::Relaxed);

    let (mut engine, _) = ActiveEngine::load(EXAMPLE_SCRIPT.1).unwrap();
    let after_load = USED.load(Ordering::Relaxed) - base;
    let mut host = ScriptHost::new();
    engine.init(&mut host).unwrap();
    for frame in 0..120 {
        host.millis = frame * 16;
        host.commands.clear();
        engine.update(&mut host).unwrap();
    }
    let resident = USED.load(Ordering::Relaxed) - base;
    let peak = PEAK.load(Ordering::Relaxed) - base;
    println!("script heap: after load {after_load} B, resident {resident} B, peak {peak} B");
    assert!(peak < BUDGET, "peak {peak} B exceeds {BUDGET} B");
}
