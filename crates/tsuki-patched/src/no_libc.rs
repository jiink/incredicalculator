//! PATCH (incredicalculator): replacements for the few libc functions and
//! `std`-only float methods tsuki used, so it builds for bare-metal and
//! wasm32-unknown-unknown targets that have no libc.

use core::alloc::Layout;
use core::ffi::{c_char, c_int, c_void};

/// Bytes in front of each block holding its total size, so freeing doesn't
/// depend on callers passing the right old size (libc `free` ignored it).
const HEADER: usize = 8;

fn layout(total: usize) -> Layout {
    Layout::from_size_align(total, HEADER).expect("allocation too large")
}

/// `realloc(block, size)` with libc semantics for a non-zero `size`.
pub(crate) unsafe fn realloc(block: *mut c_void, size: usize) -> *mut c_void {
    let total = size.checked_add(HEADER).expect("allocation too large");
    unsafe {
        let base = if block.is_null() {
            alloc::alloc::alloc(layout(total))
        } else {
            let base = (block as *mut u8).sub(HEADER);
            let old_total = *(base as *const usize);
            alloc::alloc::realloc(base, layout(old_total), total)
        };
        if base.is_null() {
            return core::ptr::null_mut();
        }
        *(base as *mut usize) = total;
        base.add(HEADER) as *mut c_void
    }
}

pub(crate) unsafe fn free(block: *mut c_void) {
    if block.is_null() {
        return;
    }
    unsafe {
        let base = (block as *mut u8).sub(HEADER);
        let total = *(base as *const usize);
        alloc::alloc::dealloc(base, layout(total));
    }
}

pub(crate) unsafe fn strcmp(a: *const c_char, b: *const c_char) -> c_int {
    unsafe {
        let (a, b) = (core::ffi::CStr::from_ptr(a), core::ffi::CStr::from_ptr(b));
        match a.to_bytes().cmp(b.to_bytes()) {
            core::cmp::Ordering::Less => -1,
            core::cmp::Ordering::Equal => 0,
            core::cmp::Ordering::Greater => 1,
        }
    }
}

pub(crate) fn abs(x: c_int) -> c_int {
    x.wrapping_abs()
}

/// `f32` methods that only exist with `std`, backed by libm.
#[allow(dead_code)]
pub(crate) trait F32Ext {
    fn floor(self) -> f32;
    fn ceil(self) -> f32;
    fn fract(self) -> f32;
    fn powf(self, n: f32) -> f32;
    fn sqrt(self) -> f32;
    fn sin(self) -> f32;
    fn cos(self) -> f32;
    fn tan(self) -> f32;
    fn asin(self) -> f32;
    fn acos(self) -> f32;
    fn atan2(self, other: f32) -> f32;
    fn exp(self) -> f32;
    fn log(self, base: f32) -> f32;
    fn ln(self) -> f32;
    fn log2(self) -> f32;
    fn log10(self) -> f32;
}

impl F32Ext for f32 {
    fn floor(self) -> f32 {
        libm::floorf(self)
    }
    fn ceil(self) -> f32 {
        libm::ceilf(self)
    }
    fn fract(self) -> f32 {
        self - libm::truncf(self)
    }
    fn powf(self, n: f32) -> f32 {
        libm::powf(self, n)
    }
    fn sqrt(self) -> f32 {
        libm::sqrtf(self)
    }
    fn sin(self) -> f32 {
        libm::sinf(self)
    }
    fn cos(self) -> f32 {
        libm::cosf(self)
    }
    fn tan(self) -> f32 {
        libm::tanf(self)
    }
    fn asin(self) -> f32 {
        libm::asinf(self)
    }
    fn acos(self) -> f32 {
        libm::acosf(self)
    }
    fn atan2(self, other: f32) -> f32 {
        libm::atan2f(self, other)
    }
    fn exp(self) -> f32 {
        libm::expf(self)
    }
    fn log(self, base: f32) -> f32 {
        libm::logf(self) / libm::logf(base)
    }
    fn ln(self) -> f32 {
        libm::logf(self)
    }
    fn log2(self) -> f32 {
        libm::log2f(self)
    }
    fn log10(self) -> f32 {
        libm::log10f(self)
    }
}
