use core::ffi::{c_char, c_int, c_void};
use std::alloc::{alloc, alloc_zeroed, dealloc, Layout};

#[no_mangle]
pub extern "C" fn rust_lzma_wasm_shim_malloc(size: usize) -> *mut c_void {
    wasm_shim_alloc::<false>(size)
}

#[no_mangle]
pub extern "C" fn rust_lzma_wasm_shim_calloc(nmemb: usize, size: usize) -> *mut c_void {
    // note: calloc expects the allocation to be zeroed
    wasm_shim_alloc::<true>(nmemb * size)
}

#[no_mangle]
pub unsafe extern "C" fn rust_lzma_wasm_shim_free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    wasm_shim_free(ptr)
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

const USIZE_ALIGN: usize = core::mem::align_of::<usize>();
const USIZE_SIZE: usize = core::mem::size_of::<usize>();

#[inline]
fn wasm_shim_alloc<const ZEROED: bool>(size: usize) -> *mut c_void {
    // in order to recover the size upon free, we store the size below the allocation
    // special alignment is never requested via the malloc API,
    // so it's not stored, and usize-alignment is used
    // memory layout: [size] [allocation]

    let full_alloc_size = size + USIZE_SIZE;

    unsafe {
        let layout = Layout::from_size_align_unchecked(full_alloc_size, USIZE_ALIGN);

        let ptr = if ZEROED {
            alloc_zeroed(layout)
        } else {
            alloc(layout)
        };

        // SAFETY: ptr is usize-aligned and we've allocated sufficient memory
        ptr.cast::<usize>().write(full_alloc_size);

        ptr.add(USIZE_SIZE).cast()
    }
}

unsafe fn wasm_shim_free(ptr: *mut c_void) {
    // the layout for the allocation needs to be recovered for dealloc
    // - the size must be recovered from directly below the allocation
    // - the alignment will always by USIZE_ALIGN

    let alloc_ptr = ptr.sub(USIZE_SIZE);
    // SAFETY: the allocation routines must uphold having a valid usize below the provided pointer
    let full_alloc_size = alloc_ptr.cast::<usize>().read();

    let layout = Layout::from_size_align_unchecked(full_alloc_size, USIZE_ALIGN);
    dealloc(alloc_ptr.cast(), layout);
}
