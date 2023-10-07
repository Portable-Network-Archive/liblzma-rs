use std::alloc::{alloc, dealloc, Layout};
use std::os::raw::c_void;

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
