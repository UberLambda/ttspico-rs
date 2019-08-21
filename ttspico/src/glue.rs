//! Utilities to glue pico's C code with Rust.

use std::{ffi, mem};
use ttspico_sys as native;

/// A C string of size [`ttspico_sys::PICO_RETSTRINGSIZE`], that can be used to
/// hold strings returned by `pico_get*StatusMessage()`.
pub struct RetString {
    mem_layout: std::alloc::Layout,
    buf: *mut u8,
}

impl RetString {
    /// Allocates memory for a [`RetString`] and returns it.
    pub fn new() -> RetString {
        let mem_size: usize =
            mem::size_of::<std::os::raw::c_char>() * native::PICO_RETSTRINGSIZE as usize;
        let mem_layout = std::alloc::Layout::from_size_align(mem_size, 16).unwrap();
        let buf = unsafe { std::alloc::alloc(mem_layout) };
        RetString { mem_layout, buf }
    }
}

impl RetString {
    /// Returns a readonly pointer to the underlying C string.  
    /// **Unsafe**: the pointer becomes dangling when `self` is dropped!
    pub unsafe fn as_ptr(&self) -> *const std::os::raw::c_char {
        self.buf as *const std::os::raw::c_char
    }

    /// Returns a read/write pointer to the underlying C string.  
    /// **Unsafe**: the pointer becomes dangling when `self` is dropped!
    pub unsafe fn as_mut_ptr(&mut self) -> *mut std::os::raw::c_char {
        self.buf as *mut std::os::raw::c_char
    }

    /// Copies the underlying C string to an owned Rust [`String`].
    pub fn to_string(&self) -> String {
        let c_str = unsafe { ffi::CStr::from_ptr(self.buf as *const std::os::raw::c_char) };
        String::from(c_str.to_str().unwrap())
    }
}

impl Drop for RetString {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.buf, self.mem_layout);
        }
    }
}
