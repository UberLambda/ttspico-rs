//! Rust bindings to the Pico TTS engine.  
//! Wraps [`ttspico-sys`](../ttspico_sys/index.html).

mod glue;
use glue::{make_cstring, PicoString};
use std::{ffi, fmt};
use ttspico_sys as native;

#[derive(Debug, PartialEq, Eq)]
pub struct PicoError {
    pub code: native::pico_Status,
    pub descr: String,
}

impl fmt::Display for PicoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (error {})", self.descr, self.code)
    }
}

#[derive(Debug)]
pub struct System {
    c_sys: native::pico_System,
    mem: *mut u8,
    mem_layout: std::alloc::Layout,
}

impl System {
    unsafe fn get_error(&self, code: native::pico_Status) -> Result<(), PicoError> {
        if code == native::PICO_OK {
            Ok(())
        } else {
            let mut c_str = PicoString::new(native::PICO_RETSTRINGSIZE);
            native::pico_getSystemStatusMessage(self.c_sys, code, c_str.as_mut_ptr());
            Err(PicoError {
                code,
                descr: match c_str.to_str() {
                    Ok(pico_msg) => pico_msg.to_string(),
                    Err(utf8_err) => format!("[invalid Pico message: {}]", utf8_err),
                },
            })
        }
    }

    pub fn new(memsz: usize) -> Result<System, PicoError> {
        unsafe {
            let mem_layout = std::alloc::Layout::from_size_align(memsz, 16).unwrap();
            let mut ret = System {
                c_sys: std::ptr::null_mut(),
                mem: std::alloc::alloc(mem_layout),
                mem_layout,
            };
            let c_code = native::pico_initialize(
                ret.mem as *mut std::os::raw::c_void,
                ret.mem_layout.size() as native::pico_Uint32,
                &mut ret.c_sys,
            );
            match ret.get_error(c_code) {
                Ok(_) => Ok(ret),
                Err(err) => Err(err),
            }
        }
    }

    pub fn load_resource<'a>(&'a self, path: impl AsRef<str>) -> Result<Resource<'a>, PicoError> {
        let c_path = make_cstring(path, "Invalid resource name")?;
        unsafe {
            let mut c_res = std::ptr::null_mut::<native::pico_resource>();
            self.get_error(native::pico_loadResource(
                self.c_sys,
                c_path.as_ptr() as *const native::pico_Char,
                &mut c_res,
            ))?;

            let mut c_name = PicoString::new(native::PICO_MAX_RESOURCE_NAME_SIZE);
            self.get_error(native::pico_getResourceName(
                self.c_sys,
                c_res,
                c_name.as_mut_ptr(),
            ))?;

            Ok(Resource::<'a> {
                sys: &self,
                c_res,
                c_name,
            })
        }
    }

    pub fn create_voice<'a>(&'a self, name: impl AsRef<str>) -> Result<Voice<'a>, PicoError> {
        let c_name = make_cstring(name, "Invalid voice name")?;
        unsafe {
            self.get_error(native::pico_createVoiceDefinition(
                self.c_sys,
                c_name.as_ptr() as *const native::pico_Char,
            ))?;
        }
        Ok(Voice { sys: &self, c_name })
    }
}

impl Drop for System {
    fn drop(&mut self) {
        unsafe {
            if !self.c_sys.is_null() {
                native::pico_terminate(&mut self.c_sys);
            }
            if !self.mem.is_null() {
                std::alloc::dealloc(self.mem, self.mem_layout);
                self.mem = std::ptr::null_mut();
            }
        }
    }
}

impl PartialEq for System {
    fn eq(&self, other: &Self) -> bool {
        self.c_sys == other.c_sys
    }
}

impl Eq for System {}

unsafe impl Send for System {}

#[derive(Debug)]
pub struct Resource<'a> {
    sys: &'a System,
    c_res: native::pico_Resource,
    c_name: PicoString,
}

impl<'a> Resource<'a> {
    pub fn sys(&self) -> &System {
        &self.sys
    }

    pub fn name(&self) -> Result<&str, std::str::Utf8Error> {
        self.c_name.to_str()
    }
}

impl<'a> Drop for Resource<'a> {
    fn drop(&mut self) {
        unsafe {
            if !self.c_res.is_null() {
                let _ = native::pico_unloadResource(self.sys.c_sys, &mut self.c_res);
            }
        }
    }
}

impl<'a> PartialEq for Resource<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.sys == other.sys && self.c_res == other.c_res
    }
}

impl<'a> Eq for Resource<'a> {}

unsafe impl<'a> Send for Resource<'a> {}

#[derive(Debug)]
pub struct Voice<'a> {
    sys: &'a System,
    c_name: ffi::CString,
}

impl<'a> Voice<'a> {
    pub fn sys(&self) -> &System {
        &self.sys
    }

    pub fn name(&self) -> Result<&str, std::str::Utf8Error> {
        self.c_name.to_str()
    }

    pub fn add_resource(&mut self, resource: &'a Resource) -> Result<(), PicoError> {
        unsafe {
            let c_code = native::pico_addResourceToVoiceDefinition(
                self.sys.c_sys,
                self.c_name.as_ptr() as *const native::pico_Char,
                resource.c_name.as_ptr() as *const native::pico_Char,
            );
            self.sys.get_error(c_code)
        }
    }

    /// SEGMENTATION FAULT
    /// Not adding all resources to the Voice (TA and SG) WILL RESULT IN A SEGFAULT!
    pub unsafe fn create_engine<'b>(&'b self) -> Result<Engine<'b>, PicoError> {
        let mut c_engine = std::ptr::null_mut::<native::pico_engine>();
        unsafe {
            self.sys.get_error(native::pico_newEngine(
                self.sys.c_sys,
                self.c_name.as_ptr() as *const native::pico_Char,
                &mut c_engine,
            ))?;
        }
        Ok(Engine {
            voice: &self,
            c_engine,
        })
    }
}

impl<'a> Drop for Voice<'a> {
    fn drop(&mut self) {
        unsafe {
            let _ = native::pico_releaseVoiceDefinition(
                self.sys.c_sys,
                self.c_name.as_ptr() as *const native::pico_Char,
            );
        }
    }
}

impl<'a> PartialEq for Voice<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.sys == other.sys && self.c_name == other.c_name
    }
}

impl<'a> Eq for Voice<'a> {}

unsafe impl<'a> Send for Voice<'a> {}

#[derive(Debug)]
pub struct Engine<'a> {
    voice: &'a Voice<'a>,
    c_engine: native::pico_Engine,
}

#[derive(Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EngineStatus {
    Idle = native::PICO_STEP_IDLE,
    Busy = native::PICO_STEP_BUSY,
}

#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum EngineResetMode {
    Full = native::PICO_RESET_FULL,
    Soft = native::PICO_RESET_SOFT,
}

impl<'a> Engine<'a> {
    unsafe fn get_error(&self, code: native::pico_Status) -> Result<(), PicoError> {
        if code == native::PICO_OK {
            Ok(())
        } else {
            let mut c_str = PicoString::new(native::PICO_RETSTRINGSIZE);
            native::pico_getEngineStatusMessage(self.c_engine, code, c_str.as_mut_ptr());
            Err(PicoError {
                code,
                descr: match c_str.to_str() {
                    Ok(pico_msg) => pico_msg.to_string(),
                    Err(utf8_err) => format!("[invalid Pico message: {}]", utf8_err),
                },
            })
        }
    }

    pub fn put_text(&mut self, utf8_text: impl AsRef<[u8]>) -> Result<usize, PicoError> {
        let buf_size = std::cmp::min(utf8_text.as_ref().len(), native::PICO_INT16_MAX as usize);
        let mut bytes_put: i16 = 0;
        unsafe {
            self.get_error(native::pico_putTextUtf8(
                self.c_engine,
                utf8_text.as_ref().as_ptr() as *const native::pico_Char,
                buf_size as i16,
                &mut bytes_put,
            ))?;
        }
        Ok(bytes_put as usize)
    }

    pub fn flush(&mut self) -> Result<usize, PicoError> {
        self.put_text(b"\0")
    }

    pub fn reset(&mut self, mode: EngineResetMode) -> Result<(), PicoError> {
        unsafe {
            self.get_error(native::pico_resetEngine(
                self.c_engine,
                mode as native::pico_Int32,
            ))
        }
    }

    pub fn get_data(
        &mut self,
        mut buf: impl AsMut<[i16]>,
    ) -> Result<(usize, EngineStatus), PicoError> {
        let c_buf = buf.as_mut().as_mut_ptr() as *mut std::os::raw::c_void;
        let max_size = buf.as_mut().len() * std::mem::size_of::<i16>();
        let max_size_i16 =
            std::cmp::min(max_size, native::PICO_INT16_MAX as usize) as native::pico_Int16;

        let mut written_size: native::pico_Int16 = 0;
        let mut written_dtype: native::pico_Int16 = 0;
        unsafe {
            let c_code = native::pico_getData(
                self.c_engine,
                c_buf,
                max_size_i16,
                &mut written_size,
                &mut written_dtype,
            );

            // (Should never fail, only 16-bit PCM seems to be supported)
            assert_eq!(written_dtype, native::PICO_DATA_PCM_16BIT);

            let n_written = (written_size as usize) / std::mem::size_of::<i16>();
            match c_code {
                native::PICO_STEP_BUSY => Ok((n_written, EngineStatus::Busy)),
                native::PICO_STEP_IDLE => Ok((n_written, EngineStatus::Idle)),
                err_code => Err(self.voice.sys.get_error(err_code).unwrap_err()),
            }
        }
    }
}

impl<'a> PartialEq for Engine<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.voice == other.voice && self.c_engine == other.c_engine
    }
}

impl<'a> Eq for Engine<'a> {}

unsafe impl<'a> Send for Engine<'a> {}
