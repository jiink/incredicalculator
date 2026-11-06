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

/// `f64` methods that only exist with `std`, backed by libm.
#[allow(dead_code)]
pub(crate) trait F64Ext {
    fn floor(self) -> f64;
    fn ceil(self) -> f64;
    fn fract(self) -> f64;
    fn powf(self, n: f64) -> f64;
    fn sqrt(self) -> f64;
    fn sin(self) -> f64;
    fn cos(self) -> f64;
    fn tan(self) -> f64;
    fn asin(self) -> f64;
    fn acos(self) -> f64;
    fn atan2(self, other: f64) -> f64;
    fn exp(self) -> f64;
    fn log(self, base: f64) -> f64;
    fn ln(self) -> f64;
    fn log2(self) -> f64;
    fn log10(self) -> f64;
}

impl F64Ext for f64 {
    fn floor(self) -> f64 {
        libm::floor(self)
    }
    fn ceil(self) -> f64 {
        libm::ceil(self)
    }
    fn fract(self) -> f64 {
        self - libm::trunc(self)
    }
    fn powf(self, n: f64) -> f64 {
        libm::pow(self, n)
    }
    fn sqrt(self) -> f64 {
        libm::sqrt(self)
    }
    fn sin(self) -> f64 {
        libm::sin(self)
    }
    fn cos(self) -> f64 {
        libm::cos(self)
    }
    fn tan(self) -> f64 {
        libm::tan(self)
    }
    fn asin(self) -> f64 {
        libm::asin(self)
    }
    fn acos(self) -> f64 {
        libm::acos(self)
    }
    fn atan2(self, other: f64) -> f64 {
        libm::atan2(self, other)
    }
    fn exp(self) -> f64 {
        libm::exp(self)
    }
    fn log(self, base: f64) -> f64 {
        libm::log(self) / libm::log(base)
    }
    fn ln(self) -> f64 {
        libm::log(self)
    }
    fn log2(self) -> f64 {
        libm::log2(self)
    }
    fn log10(self) -> f64 {
        libm::log10(self)
    }
}
