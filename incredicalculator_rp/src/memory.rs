//! RAM usage tracking and stack overflow protection.
//!
//! - [`TrackingHeap`] wraps the global allocator and remembers peak usage.
//! - Both cores' stacks are filled with [`STACK_PAINT`] at boot. The deepest
//!   point either stack has reached is found later by counting how many
//!   painted words at the bottom are still intact.
//! - Each core's `MSPLIM` register makes the CPU fault as soon as the stack
//!   pointer goes below the bottom of its stack, instead of silently writing
//!   over the heap or other statics. The fault handler records the reason in a
//!   watchdog scratch register (which survives a watchdog reset) and reboots,
//!   so the next boot can show what happened.

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

use embassy_rp::pac;
use embedded_alloc::LlffHeap;
pub use incredicalculator_core::platform::MemoryStats;

const STACK_PAINT: u32 = 0xC0FF_EE5A;

/// Bytes left unpainted below the stack pointer when painting the live
/// core-0 stack, so the painting loop never overwrites its own frame.
const PAINT_MARGIN: usize = 256;

/// embassy-rp places a 256-byte no-access MPU region at the bottom of the
/// core-1 stack. `MSPLIM` is set just above it so overflow is reported as a
/// stack overflow rather than a generic memory fault.
pub const CORE1_GUARD_BYTES: usize = 256;

unsafe extern "C" {
    // Provided by cortex-m-rt's link.x: the core-0 stack spans
    // `_stack_end..._stack_start` (it grows down from `_stack_start`).
    static _stack_start: u32;
    static _stack_end: u32;
}

fn core0_stack_bounds() -> (usize, usize) {
    let bottom = core::ptr::addr_of!(_stack_end) as usize;
    let top = core::ptr::addr_of!(_stack_start) as usize;
    (bottom, top)
}

pub struct TrackingHeap {
    heap: LlffHeap,
    size: AtomicUsize,
    peak: AtomicUsize,
}

impl TrackingHeap {
    pub const fn empty() -> Self {
        Self {
            heap: LlffHeap::empty(),
            size: AtomicUsize::new(0),
            peak: AtomicUsize::new(0),
        }
    }

    /// # Safety
    /// Same requirements as [`LlffHeap::init`]; call once, before allocating.
    pub unsafe fn init(&self, start_addr: usize, size: usize) {
        unsafe { self.heap.init(start_addr, size) };
        self.size.store(size, Ordering::Relaxed);
    }
}

unsafe impl GlobalAlloc for TrackingHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Update the peak with a plain load and store inside a critical
        // section (reentrant, so the heap's own one nests inside it) instead
        // of `fetch_max`. `fetch_max` compiles to a LDREX/STREX retry loop,
        // which was seen livelocked on the RP2350 here, the hottest exclusive
        // loop in the firmware, until key presses on core 1 shook it loose.
        critical_section::with(|_| {
            let ptr = unsafe { self.heap.alloc(layout) };
            if !ptr.is_null() {
                let used = self.heap.used();
                if used > self.peak.load(Ordering::Relaxed) {
                    self.peak.store(used, Ordering::Relaxed);
                }
            }
            ptr
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { self.heap.dealloc(ptr, layout) }
    }
}

/// Number of intact painted words at the start of `bottom..top`.
fn painted_bytes(bottom: usize, top: usize) -> usize {
    let mut addr = bottom;
    while addr < top && unsafe { core::ptr::read_volatile(addr as *const u32) } == STACK_PAINT {
        addr += 4;
    }
    addr - bottom
}

fn paint(bottom: usize, top: usize) {
    let mut addr = bottom;
    while addr < top {
        unsafe { core::ptr::write_volatile(addr as *mut u32, STACK_PAINT) };
        addr += 4;
    }
}

/// Paints the unused part of the core-0 stack and enables its overflow limit.
/// Call first thing in `main`, on core 0.
pub fn init_core0_stack() {
    let (bottom, _) = core0_stack_bounds();
    let sp = cortex_m::register::msp::read() as usize;
    paint(bottom, (sp - PAINT_MARGIN) & !3);
    unsafe { cortex_m::register::msplim::write(((bottom + 7) & !7) as u32) };
}

/// Paints a core-1 stack before core 1 is started.
pub fn paint_core1_stack(stack: &mut [u8]) {
    let bottom = stack.as_mut_ptr() as usize;
    paint((bottom + 3) & !3, (bottom + stack.len()) & !3);
}

/// Enables the overflow limit for the stack core 1 is running on. Call first
/// thing on core 1.
pub fn init_core1_stack(stack_bottom: usize) {
    let limit = (stack_bottom + CORE1_GUARD_BYTES + 7) & !7;
    unsafe { cortex_m::register::msplim::write(limit as u32) };
}

pub fn stats(heap: &TrackingHeap, core1_stack: (usize, usize)) -> MemoryStats {
    let (c0_bottom, c0_top) = core0_stack_bounds();
    let core0_stack_size = c0_top - c0_bottom;
    let (c1_bottom, c1_len) = core1_stack;
    let c1_usable_bottom = c1_bottom + CORE1_GUARD_BYTES;
    let core1_stack_size = c1_bottom + c1_len - c1_usable_bottom;
    MemoryStats {
        heap_used: heap.heap.used(),
        heap_peak: heap.peak.load(Ordering::Relaxed),
        heap_size: heap.size.load(Ordering::Relaxed),
        stack_peaks: [
            core0_stack_size - painted_bytes(c0_bottom, c0_top),
            core1_stack_size - painted_bytes(c1_usable_bottom, c1_bottom + c1_len),
        ],
        stack_sizes: [core0_stack_size, core1_stack_size],
    }
}

// ---- Fault reporting across a reboot ---------------------------------------

/// Watchdog scratch registers 0-3 are free for applications and survive a
/// watchdog reset (the boot ROM uses 4-7).
const FAULT_MAGIC: u32 = 0xFA17_0000;
const FAULT_MAGIC_MASK: u32 = 0xFFFF_0000;
const FAULT_STACK_OVERFLOW: u32 = 0x100;
const FAULT_UNDEFINED_INSTRUCTION: u32 = 0x200;
/// Configurable Fault Status Register; UFSR.UNDEFINSTR is bit 16 and
/// UFSR.STKOF is bit 20.
const SCB_CFSR: *const u32 = 0xE000_ED28 as *const u32;
const CFSR_UNDEFINSTR: u32 = 1 << 16;
const CFSR_STKOF: u32 = 1 << 20;

#[derive(Clone, Copy, defmt::Format)]
pub enum PreviousFault {
    StackOverflow {
        core: u32,
    },
    /// `panic_probe` ends every panic (including allocation failures) with
    /// `udf`, an undefined instruction, so this is almost always a panic.
    Panic {
        core: u32,
    },
    HardFault {
        core: u32,
    },
}

/// Returns (and clears) the fault that caused the previous reboot, if any.
pub fn take_previous_fault() -> Option<PreviousFault> {
    let value = pac::WATCHDOG.scratch0().read();
    if value & FAULT_MAGIC_MASK != FAULT_MAGIC {
        return None;
    }
    pac::WATCHDOG.scratch0().write_value(0);
    let core = value & 0xF;
    Some(if value & FAULT_STACK_OVERFLOW != 0 {
        PreviousFault::StackOverflow { core }
    } else if value & FAULT_UNDEFINED_INSTRUCTION != 0 {
        PreviousFault::Panic { core }
    } else {
        PreviousFault::HardFault { core }
    })
}

// Replaces cortex-m-rt's default `HardFault` (an infinite loop). After a stack
// limit violation the CPU enters the handler with SP at the limit, so the
// limit must be cleared before anything is pushed; the handler resets the
// chip anyway, so running below the old limit is harmless.
core::arch::global_asm!(
    ".section .HardFault.user, \"ax\"",
    ".global HardFault",
    ".type HardFault, %function",
    ".thumb_func",
    "HardFault:",
    "movs r0, #0",
    "msr MSPLIM, r0",
    "b {handler}",
    handler = sym hard_fault_handler,
);

unsafe extern "C" fn hard_fault_handler() -> ! {
    let cfsr = unsafe { core::ptr::read_volatile(SCB_CFSR) };
    let core = pac::SIO.cpuid().read() & 0xF;
    let mut value = FAULT_MAGIC | core;
    if cfsr & CFSR_STKOF != 0 {
        value |= FAULT_STACK_OVERFLOW;
    } else if cfsr & CFSR_UNDEFINSTR != 0 {
        value |= FAULT_UNDEFINED_INSTRUCTION;
    }
    pac::WATCHDOG.scratch0().write_value(value);
    // Reset everything except the oscillators, like the Pico SDK's
    // `watchdog_reboot`. The RP2350's PSM bits differ from the RP2040's
    // (embassy-rp's `0x1ffff & !0b11` mask is the RP2040 layout): on the
    // RP2350 that mask resets the clocks, peripherals and timers but neither
    // processor, leaving both cores running on reset hardware.
    let mut wdsel = pac::psm::regs::Wdsel(0x01FF_FFFF);
    wdsel.set_rosc(false);
    wdsel.set_xosc(false);
    pac::PSM.wdsel().write_value(wdsel);
    // Start from zero rather than the reset value so the pause-on-debug bits
    // are clear and an attached probe can't hold off the reset.
    let mut ctrl = pac::watchdog::regs::Ctrl(0);
    ctrl.set_enable(true);
    ctrl.set_trigger(true);
    pac::WATCHDOG.ctrl().write_value(ctrl);
    loop {
        cortex_m::asm::nop();
    }
}
