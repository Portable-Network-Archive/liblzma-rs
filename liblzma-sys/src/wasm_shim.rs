use core::ffi::{c_char, c_int, c_void};
use std::alloc::{alloc, dealloc, Layout};

#[no_mangle]
pub extern "C" fn rust_lzma_wasm_shim_malloc(size: usize) -> *mut c_void {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, 1);
        alloc(layout).cast()
    }
}

#[no_mangle]
pub extern "C" fn rust_lzma_wasm_shim_calloc(nmemb: usize, size: usize) -> *mut c_void {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size * nmemb, 1);
        alloc(layout).cast()
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_lzma_wasm_shim_free(ptr: *mut c_void) {
    if ptr == std::ptr::null_mut() {
        return;
    }
    // layout is not actually used
    let layout = Layout::from_size_align_unchecked(1, 1);
    dealloc(ptr.cast(), layout);
}

#[no_mangle]
pub extern "C" fn rust_lzma_wasm_shim_memcmp(
    str1: *const c_void,
    str2: *const c_void,
    n: usize,
) -> i32 {
    // Safety: function contracts requires str1 and str2 at least `n`-long.
    unsafe {
        let str1: &[u8] = core::slice::from_raw_parts(str1 as *const u8, n);
        let str2: &[u8] = core::slice::from_raw_parts(str2 as *const u8, n);
        match str1.cmp(str2) {
            core::cmp::Ordering::Less => -1,
            core::cmp::Ordering::Equal => 0,
            core::cmp::Ordering::Greater => 1,
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_lzma_wasm_shim_memcpy(
    dest: *mut c_void,
    src: *const c_void,
    n: usize,
) -> *mut c_void {
    core::ptr::copy_nonoverlapping(src as *const u8, dest as *mut u8, n);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn rust_lzma_wasm_shim_memmove(
    dest: *mut c_void,
    src: *const c_void,
    n: usize,
) -> *mut c_void {
    core::ptr::copy(src as *const u8, dest as *mut u8, n);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn rust_lzma_wasm_shim_memset(
    dest: *mut c_void,
    c: c_int,
    n: usize,
) -> *mut c_void {
    core::ptr::write_bytes(dest as *mut u8, c as u8, n);
    dest
}

#[no_mangle]
pub unsafe extern "C" fn rust_lzma_wasm_shim_strlen(s: *const c_char) -> usize {
    let str = unsafe { std::ffi::CStr::from_ptr(s) };
    str.to_bytes().len()
}

#[no_mangle]
pub unsafe extern "C" fn rust_lzma_wasm_shim_memchr(
    s: *const c_void,
    c: c_int,
    n: usize,
) -> *mut c_void {
    let s_slice = unsafe { core::slice::from_raw_parts(s as *const u8, n) };
    s_slice
        .iter()
        .position(|&r| r == c as u8)
        .map_or(core::ptr::null_mut(), |p| unsafe {
            s.add(p) as *mut c_void
        })
}
