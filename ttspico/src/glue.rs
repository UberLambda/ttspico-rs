//! Utilities to glue pico's C code with Rust.

use crate::PicoError;
use std::{ffi, mem};

/// A C string of fixed size that can be used to hold strings returned by Pico.
#[derive(Debug)]
pub struct PicoString {
    mem_layout: std::alloc::Layout,
    buf: *mut u8,
}

impl PicoString {
    /// Allocates memory for a [`PicoString`] of a certain size and returns it.
    pub fn new(size: usize) -> PicoString {
        let mem_size: usize = mem::size_of::<std::os::raw::c_char>() * size;
        let mem_layout = std::alloc::Layout::from_size_align(mem_size, 16).unwrap();
        let buf = unsafe { std::alloc::alloc(mem_layout) };
        PicoString { mem_layout, buf }
    }
}

impl PicoString {
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

    /// Tries to convert the underlying C string to a Rust [`str`].
    pub fn to_str(&self) -> Result<&str, std::str::Utf8Error> {
        let c_str = unsafe { ffi::CStr::from_ptr(self.buf as *const std::os::raw::c_char) };
        c_str.to_str()
    }
}

impl Drop for PicoString {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.buf, self.mem_layout);
        }
    }
}

/// Convert `string` to a [`ffi::CString`]; on failure, returns a `PicoError` with the given description (and code -1).
pub fn make_cstring(string: impl AsRef<str>, err_descr: &str) -> Result<ffi::CString, PicoError> {
    ffi::CString::new(string.as_ref()).map_err(|err| PicoError {
        code: -1,
        descr: format!("{}: {}", err_descr, err),
    })
}
