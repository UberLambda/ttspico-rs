//! Rust bindings to the Pico TTS engine.  
//! Wraps [`ttspico-sys`](../ttspico_sys/index.html).

mod glue;
use glue::RetString;
use std::fmt;
use ttspico_sys as native;

#[derive(Debug, PartialEq, Eq)]
pub struct PicoError {
    code: native::pico_Status,
    descr: String,
}

impl fmt::Display for PicoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (error {})", self.descr, self.code)
    }
}

#[derive(Debug)]
pub struct System {
    sys: native::pico_System,
    mem: *mut u8,
    mem_layout: std::alloc::Layout,
}

impl PartialEq for System {
    fn eq(&self, other: &Self) -> bool {
        self.sys == other.sys
    }
}

impl Eq for System {}

impl System {
    unsafe fn get_error(&self, code: native::pico_Status) -> PicoError {
        let mut c_str = RetString::new();
        native::pico_getSystemStatusMessage(self.sys, code, c_str.as_mut_ptr());
        PicoError {
            code: code,
            descr: c_str.to_string(),
        }
    }

    pub fn new(memsz: usize) -> Result<System, PicoError> {
        unsafe {
            let mem_layout = std::alloc::Layout::from_size_align(memsz, 16).unwrap();

            let mut ret = System {
                sys: std::ptr::null_mut(),
                mem: std::alloc::alloc(mem_layout),
                mem_layout,
            };

            let init_code = native::pico_initialize(
                ret.mem as *mut std::os::raw::c_void,
                ret.mem_layout.size() as native::pico_Uint32,
                &mut ret.sys,
            );

            match init_code {
                native::PICO_OK => Ok(ret),
                err_code => Err(ret.get_error(err_code)),
            }
        }
    }
}

impl Drop for System {
    fn drop(&mut self) {
        unsafe {
            if !self.sys.is_null() {
                native::pico_terminate(&mut self.sys);
                self.sys = std::ptr::null_mut();
            }
            if !self.mem.is_null() {
                std::alloc::dealloc(self.mem, self.mem_layout);
                self.mem = std::ptr::null_mut();
            }
        }
    }
}
