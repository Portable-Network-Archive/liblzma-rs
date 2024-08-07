pub use crate::bindgen::*;

// lzma_action
pub const LZMA_RUN: lzma_action = lzma_action_LZMA_RUN;
pub const LZMA_SYNC_FLUSH: lzma_action = lzma_action_LZMA_SYNC_FLUSH;
pub const LZMA_FULL_FLUSH: lzma_action = lzma_action_LZMA_FULL_FLUSH;
pub const LZMA_FULL_BARRIER: lzma_action = lzma_action_LZMA_FULL_BARRIER;
pub const LZMA_FINISH: lzma_action = lzma_action_LZMA_FINISH;

// lzma_check
pub const LZMA_CHECK_NONE: lzma_check = lzma_check_LZMA_CHECK_NONE;
pub const LZMA_CHECK_CRC32: lzma_check = lzma_check_LZMA_CHECK_CRC32;
pub const LZMA_CHECK_CRC64: lzma_check = lzma_check_LZMA_CHECK_CRC64;
pub const LZMA_CHECK_SHA256: lzma_check = lzma_check_LZMA_CHECK_SHA256;

// lzma_mode
pub const LZMA_MODE_FAST: lzma_mode = lzma_mode_LZMA_MODE_FAST;
pub const LZMA_MODE_NORMAL: lzma_mode = lzma_mode_LZMA_MODE_NORMAL;

// lzma_match_finder
pub const LZMA_MF_HC3: lzma_match_finder = lzma_match_finder_LZMA_MF_HC3;
pub const LZMA_MF_HC4: lzma_match_finder = lzma_match_finder_LZMA_MF_HC4;
pub const LZMA_MF_BT2: lzma_match_finder = lzma_match_finder_LZMA_MF_BT2;
pub const LZMA_MF_BT3: lzma_match_finder = lzma_match_finder_LZMA_MF_BT3;
pub const LZMA_MF_BT4: lzma_match_finder = lzma_match_finder_LZMA_MF_BT4;

// macros
pub const LZMA_TELL_NO_CHECK: u32 = 0x01;
pub const LZMA_TELL_UNSUPPORTED_CHECK: u32 = 0x02;
pub const LZMA_TELL_ANY_CHECK: u32 = 0x04;
pub const LZMA_IGNORE_CHECK: u32 = 0x10;
pub const LZMA_CONCATENATED: u32 = 0x08;

pub const LZMA_PRESET_DEFAULT: u32 = 6;
pub const LZMA_PRESET_LEVEL_MASK: u32 = 0x1f;
pub const LZMA_PRESET_EXTREME: u32 = 1 << 31;

pub const LZMA_DICT_SIZE_MIN: u32 = 4096;
pub const LZMA_DICT_SIZE_DEFAULT: u32 = 1 << 23;

pub const LZMA_BACKWARD_SIZE_MIN: lzma_vli = 4;
pub const LZMA_BACKWARD_SIZE_MAX: lzma_vli = 1 << 34;

// filters
pub const LZMA_FILTER_X86: lzma_vli = 0x04;
pub const LZMA_FILTER_POWERPC: lzma_vli = 0x05;
pub const LZMA_FILTER_IA64: lzma_vli = 0x06;
pub const LZMA_FILTER_ARM: lzma_vli = 0x07;
pub const LZMA_FILTER_ARMTHUMB: lzma_vli = 0x08;
pub const LZMA_FILTER_SPARC: lzma_vli = 0x09;
pub const LZMA_FILTER_ARM64: lzma_vli = 0x0A;
pub const LZMA_FILTER_DELTA: lzma_vli = 0x03;
pub const LZMA_FILTER_RISCV: lzma_vli = 0x0B;
pub const LZMA_FILTER_LZMA1: lzma_vli = 0x4000000000000001;
pub const LZMA_FILTER_LZMA2: lzma_vli = 0x21;

// status
pub const LZMA_OK: lzma_ret = lzma_ret_LZMA_OK;
pub const LZMA_STREAM_END: lzma_ret = lzma_ret_LZMA_STREAM_END;
pub const LZMA_NO_CHECK: lzma_ret = lzma_ret_LZMA_NO_CHECK;
pub const LZMA_UNSUPPORTED_CHECK: lzma_ret = lzma_ret_LZMA_UNSUPPORTED_CHECK;
pub const LZMA_GET_CHECK: lzma_ret = lzma_ret_LZMA_GET_CHECK;
pub const LZMA_MEM_ERROR: lzma_ret = lzma_ret_LZMA_MEM_ERROR;
pub const LZMA_MEMLIMIT_ERROR: lzma_ret = lzma_ret_LZMA_MEMLIMIT_ERROR;
pub const LZMA_FORMAT_ERROR: lzma_ret = lzma_ret_LZMA_FORMAT_ERROR;
pub const LZMA_OPTIONS_ERROR: lzma_ret = lzma_ret_LZMA_OPTIONS_ERROR;
pub const LZMA_DATA_ERROR: lzma_ret = lzma_ret_LZMA_DATA_ERROR;
pub const LZMA_BUF_ERROR: lzma_ret = lzma_ret_LZMA_BUF_ERROR;
pub const LZMA_PROG_ERROR: lzma_ret = lzma_ret_LZMA_PROG_ERROR;
pub const LZMA_SEEK_NEEDED: lzma_ret = lzma_ret_LZMA_SEEK_NEEDED;
