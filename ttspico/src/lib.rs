//! Rust bindings to the Pico TTS engine.  
//! Wraps [`ttspico-sys`](../ttspico_sys/index.html).

// Copyright (c) 2019 Paolo Jovon <paolo.jovon@gmail.com>
// Copyright (c) 2019 Sergio Tortosa Benedito
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod glue;
use glue::{make_cstring, PicoString};
use std::{ffi, fmt};
use ttspico_sys as native;
use std::rc::Rc;
use std::cell::RefCell;

/// An error caused by Pico TTS.
#[derive(Debug, PartialEq, Eq)]
pub struct PicoError {
    /// The Pico status code of the error.  
    /// Set to -1 for internal `ttspico-rs` errors.
    pub code: native::pico_Status,

    /// A human-readable description of the error.
    pub descr: String,
}

impl fmt::Display for PicoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (error {})", self.descr, self.code)
    }
}

/// A Pico TTS system, i.e. the context from which to load [`Resource`]s and create [`Voice`]s.
#[derive(Debug)]
pub struct System {
    c_sys: native::pico_System,
    mem: *mut u8,
    mem_layout: std::alloc::Layout,
}

impl System {
    /// Converts a Pico system-level error `code` to a `Err(PicoError)` if code is not
    /// [`PICO_OK`](`ttspico_sys::PICO_OK`), else returns `Ok(())`.
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

    /// Instantiates a Pico [`System`], given the size in bytes of the memory to allocate for it.
    /// # Remarks
    /// Only one [`System`] should be instantiated per thread!
    /// # See
    /// [`ttspico_sys::pico_initialize`].
    pub fn new(memsz: usize) -> Result<Rc<RefCell<System>>, PicoError> {
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
                Ok(_) => Ok(Rc::new(RefCell::new(ret))),
                Err(err) => Err(err),
            }
        }
    }

    /// Creates a Pico [`Resource`] given its filepath.
    /// # See
    /// [`ttspico_sys::pico_loadResource`], [`ttspico_sys::pico_getResourceName`].
    pub fn load_resource(sys: Rc<RefCell<Self>>, path: impl AsRef<str>) -> Result<Rc<RefCell<Resource>>, PicoError> {
        let c_path = make_cstring(path, "Invalid resource name")?;
        unsafe {
            let mut c_res = std::ptr::null_mut::<native::pico_resource>();
            sys.borrow().get_error(native::pico_loadResource(
                sys.borrow().c_sys,
                c_path.as_ptr() as *const native::pico_Char,
                &mut c_res,
            ))?;

            let mut c_name = PicoString::new(native::PICO_MAX_RESOURCE_NAME_SIZE);
            sys.borrow().get_error(native::pico_getResourceName(
                sys.borrow().c_sys,
                c_res,
                c_name.as_mut_ptr(),
            ))?;

            Ok(Rc::new(RefCell::new(Resource {
                sys,
                c_res,
                c_name,
            })))
        }
    }

    /// Creates a Pico [`Voice`] given its name.
    /// # See
    /// [`ttspico_sys::pico_createVoiceDefinition`].
    pub fn create_voice<'a>(sys: Rc<RefCell<Self>>, name: impl AsRef<str>) -> Result<Rc<RefCell<Voice>>, PicoError> {
        let c_name = make_cstring(name, "Invalid voice name")?;
        unsafe {
            sys.borrow().get_error(native::pico_createVoiceDefinition(
                sys.borrow().c_sys,
                c_name.as_ptr() as *const native::pico_Char,
            ))?;
        }
        Ok(Rc::new(RefCell::new(Voice { sys, c_name, resources:Vec::new() })))
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

/// A loaded Pico TTS resource (TA or SG).
#[derive(Debug)]
pub struct Resource {
    //sys: &'a System,
    sys: Rc<RefCell<System>>,
    c_res: native::pico_Resource,
    c_name: PicoString,
}

impl Resource {
    /// Returns a reference to the parent [`System`] that loaded this resource.
    pub fn sys(&self) -> Rc<RefCell<System>> {
        self.sys.clone()
    }

    /// Returns the resource's internal name (if it can be converted to UTF-8).
    pub fn name(&self) -> Result<&str, std::str::Utf8Error> {
        self.c_name.to_str()
    }
}

impl Drop for Resource {
    fn drop(&mut self) {
        unsafe {
            if !self.c_res.is_null() {
                let _ = native::pico_unloadResource(self.sys.borrow().c_sys, &mut self.c_res);
            }
        }
    }
}

impl PartialEq for Resource {
    fn eq(&self, other: &Self) -> bool {
        self.sys == other.sys && self.c_res == other.c_res
    }
}

impl Eq for Resource {}

unsafe impl Send for Resource {}

/// A Pico TTS voice.
#[derive(Debug)]
pub struct Voice {
    sys: Rc<RefCell<System>>,
    c_name: ffi::CString,
    resources: Vec<Rc<RefCell<Resource>>>
}

impl Voice {
    /// Returns a reference to the parent [`System`] that created this voice.
    pub fn sys(&self) -> Rc<RefCell<System>> {
        self.sys.clone()
    }

    /// Returns the voice's name (if it can be converted to UTF-8).
    pub fn name(&self) -> Result<&str, std::str::Utf8Error> {
        self.c_name.to_str()
    }

    /// Adds a loaded [`Resource`] to this voice.  
    /// A [`Voice`] needs both a TA and SG resource to be added to it.
    /// # See
    /// [`ttspico_sys::pico_addResourceToVoiceDefinition`].
    pub fn add_resource(&mut self, resource: Rc<RefCell<Resource>>) -> Result<(), PicoError> {
        let err_code = unsafe {
            let c_code = native::pico_addResourceToVoiceDefinition(
                self.sys.borrow().c_sys,
                self.c_name.as_ptr() as *const native::pico_Char,
                resource.borrow().c_name.as_ptr() as *const native::pico_Char,
            );
            self.sys.borrow().get_error(c_code)
        };

        match err_code {
            Ok(_) => {
                self.resources.push(resource);
                err_code
            }
            _ => {
                err_code
            }
        }
    }

    /// Creates a Pico [`Engine`] for this voice.
    /// # Unsafe
    /// Both a TA and a SG [`Resource`] need to be loaded and [added](`Voice::add_resource`) to a voice before
    /// creating an engine. Failing to do so could result in a segmentation fault!
    /// # See
    /// [`ttspico_sys::pico_newEngine`].
    pub unsafe fn create_engine(voice: Rc<RefCell<Voice>>) -> Result<Engine, PicoError> {
        let mut c_engine = std::ptr::null_mut::<native::pico_engine>();
        voice.borrow().sys.borrow().get_error(native::pico_newEngine(
            voice.borrow().sys.borrow().c_sys,
            voice.borrow().c_name.as_ptr() as *const native::pico_Char,
            &mut c_engine,
        ))?;
        Ok(Engine {
            voice,
            c_engine,
        })
    }
}

impl Drop for Voice {
    fn drop(&mut self) {
        unsafe {
            let _ = native::pico_releaseVoiceDefinition(
                self.sys.borrow().c_sys,
                self.c_name.as_ptr() as *const native::pico_Char,
            );
        }
    }
}

impl PartialEq for Voice {
    fn eq(&self, other: &Self) -> bool {
        self.sys == other.sys && self.c_name == other.c_name
    }
}

impl Eq for Voice {}

unsafe impl Send for Voice {}

/// A Pico TTS engine.
#[derive(Debug)]
pub struct Engine {
    voice: Rc<RefCell<Voice>>,
    c_engine: native::pico_Engine,
}

/// An [`Engine`]'s status after [stepping](`Engine::get_data`) it.
#[derive(Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum EngineStatus {
    /// Idle: no more speech audio to be generated.
    Idle = native::PICO_STEP_IDLE,

    /// Busy: speech audio generation is still ongoing.  
    /// Call `Engine::get_data` again until it returns `Idle` to make sure all speech is generated.
    Busy = native::PICO_STEP_BUSY,
}

/// The ways an [`Engine`]' can be reset.
#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum EngineResetMode {
    /// Full reset: to be used after an [engine error](`PicoError`) is raised.
    Full = native::PICO_RESET_FULL,

    /// Soft reset: flushes internal input and output buffers.
    Soft = native::PICO_RESET_SOFT,
}

impl Engine {
    /// Converts a Pico engine-level error `code` to a `Err(PicoError)` if code is not
    /// [`PICO_OK`](`ttspico_sys::PICO_OK`), else returns `Ok(())`.
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

    /// Puts UTF-8 text to be spoken into the TTS engine.
    /// Returns the number of bytes of `utf8_text` that were in fact put in the engine (or a [`PicoError`] on failure).
    ///
    /// Put null terminators (`\0`) in the text to flush the engine, forcing speech generation.
    /// # See
    /// [`ttspico_sys::pico_putTextUtf8`].
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

    /// Flushes the TTS engine, forcing speech generation.
    /// Equivalent to `self.put_text("\0").
    pub fn flush(&mut self) -> Result<usize, PicoError> {
        self.put_text(b"\0")
    }

    /// Resets the TTS engine according to [`mode`](`EngineResetMode`).
    /// # See
    /// [`ttspico_sys::pico_resetEngine`].
    pub fn reset(&mut self, mode: EngineResetMode) -> Result<(), PicoError> {
        unsafe {
            self.get_error(native::pico_resetEngine(
                self.c_engine,
                mode as native::pico_Int32,
            ))
        }
    }

    /// Generates speech audio from the text input via [`put_text`](`Engine::put_text`), outputting to `buf`.  
    /// Returns either a <number of samples generated, [`EngineStatus`] after stepping> pair (on success) or a
    /// `PicoError` (on failure).
    ///
    /// Output data is encoded as 16-bit signed PCM, at a sample rate of 16kHz.
    /// `buf` should have length <= [`PICO_INT16_MAX`](`ttspico_sys::PICO_INT16_MAX`).
    /// # See
    /// [`ttspico_sys::pico_getData`].
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
                err_code => Err(self.voice.borrow().sys.borrow().get_error(err_code).unwrap_err()),
            }
        }
    }
}

impl PartialEq for Engine {
    fn eq(&self, other: &Self) -> bool {
        self.voice == other.voice && self.c_engine == other.c_engine
    }
}

impl Eq for Engine {}

unsafe impl Send for Engine {}
