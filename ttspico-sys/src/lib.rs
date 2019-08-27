//! Low-level C bindings to the Pico TTS engine.  
//! See [`ttspico`](../ttspico/index.html) for a high-level Rust wrapper.

mod ffi;
pub use ffi::*;

mod defs;
pub use defs::*;
