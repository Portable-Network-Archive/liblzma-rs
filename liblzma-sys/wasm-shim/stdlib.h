#include <stddef.h>

#ifndef	_STDLIB_H
#define	_STDLIB_H	1

void *rust_lzma_wasm_shim_malloc(size_t size);
void *rust_lzma_wasm_shim_calloc(size_t nmemb, size_t size);
void rust_lzma_wasm_shim_free(void *ptr);

#define malloc(size) rust_lzma_wasm_shim_malloc(size)

#define calloc(nmemb, size) rust_lzma_wasm_shim_calloc(nmemb, size);

// Hack: Avoid replacing `allocator->free` to `allocator->rust_lzma_wasm_shim_free` in
// Link: liblzma-sys/xz/src/liblzma/common/common.c:79
//   lzma_free(void *ptr, const lzma_allocator *allocator)
//   {
//      if (allocator != NULL && allocator->free != NULL)
//          allocator->free(allocator->opaque, ptr); //
//      else
//          free(ptr);
//   }
#define free(...) _FREE_DISPATCH(__VA_ARGS__, free2, free1)(__VA_ARGS__)

#define _FREE_DISPATCH(_1, _2, NAME, ...) NAME

// for free
#define free1(ptr) rust_lzma_wasm_shim_free(ptr)
// for allocator->free
#define free2(info, ptr) free(info, ptr)

#endif // _STDLIB_H
