use crate::ffi::{pico_Int16, pico_Status};

pub const PICO_MAX_VOICE_NAME_SIZE: u32 = 32;
pub const PICO_MAX_RESOURCE_NAME_SIZE: u32 = 32;
pub const PICO_MAX_DATAPATH_NAME_SIZE: u32 = 128;
pub const PICO_MAX_FILE_NAME_SIZE: u32 = 64;
pub const PICO_MAX_NUM_RESOURCES: u32 = 64;
pub const PICO_MAX_NUM_VOICE_DEFINITIONS: u32 = 64;
pub const PICO_MAX_NUM_RSRC_PER_VOICE: u32 = 16;
pub const PICO_MAX_FOREIGN_HEADER_LEN: u32 = 64;
pub const PICO_INT16_MAX: u32 = 32767;
pub const PICO_UINT16_MAX: u32 = 65535;
pub const PICO_INT32_MAX: u32 = 2147483647;
pub const PICO_UINT32_MAX: u32 = 4294967295;
pub const PICO_RETSTRINGSIZE: u32 = 200;

pub const PICO_RESET_FULL: u32 = 0;
pub const PICO_RESET_SOFT: u32 = 16;

pub const PICO_DATA_PCM_16BIT: pico_Int16 = 1;

// -- Status codes --

pub const PICO_OK: pico_Status = 0;

pub const PICO_EXC_NUMBER_FORMAT: pico_Status = -10;
pub const PICO_EXC_MAX_NUM_EXCEED: pico_Status = -11;
pub const PICO_EXC_NAME_CONFLICT: pico_Status = -12;
pub const PICO_EXC_NAME_UNDEFINED: pico_Status = -13;
pub const PICO_EXC_NAME_ILLEGAL: pico_Status = -14;
pub const PICO_EXC_BUF_OVERFLOW: pico_Status = -20;
pub const PICO_EXC_BUF_UNDERFLOW: pico_Status = -21;
pub const PICO_EXC_BUF_IGNORE: pico_Status = -22;
pub const PICO_EXC_OUT_OF_MEM: pico_Status = -30;
pub const PICO_EXC_CANT_OPEN_FILE: pico_Status = -40;
pub const PICO_EXC_UNEXPECTED_FILE_TYPE: pico_Status = -41;
pub const PICO_EXC_FILE_CORRUPT: pico_Status = -42;
pub const PICO_EXC_FILE_NOT_FOUND: pico_Status = -43;
pub const PICO_EXC_RESOURCE_BUSY: pico_Status = -50;
pub const PICO_EXC_RESOURCE_MISSING: pico_Status = -51;
pub const PICO_EXC_KB_MISSING: pico_Status = -60;

pub const PICO_ERR_NULLPTR_ACCESS: pico_Status = -100;
pub const PICO_ERR_INVALID_HANDLE: pico_Status = -101;
pub const PICO_ERR_INVALID_ARGUMENT: pico_Status = -102;
pub const PICO_ERR_INDEX_OUT_OF_RANGE: pico_Status = -103;
pub const PICO_ERR_OTHER: pico_Status = -999;

pub const PICO_WARN_INCOMPLETE: pico_Status = 10;
pub const PICO_WARN_FALLBACK: pico_Status = 11;
pub const PICO_WARN_OTHER: pico_Status = 19;
pub const PICO_WARN_KB_OVERWRITE: pico_Status = 50;
pub const PICO_WARN_RESOURCE_DOUBLE_LOAD: pico_Status = 51;
pub const PICO_WARN_INVECTOR: pico_Status = 60;
pub const PICO_WARN_CLASSIFICATION: pico_Status = 61;
pub const PICO_WARN_OUTVECTOR: pico_Status = 62;
pub const PICO_WARN_PU_IRREG_ITEM: pico_Status = 70;
pub const PICO_WARN_PU_DISCARD_BUF: pico_Status = 71;

pub const PICO_STEP_IDLE: pico_Status = 200;
pub const PICO_STEP_BUSY: pico_Status = 201;
pub const PICO_STEP_ERROR: pico_Status = -200;
