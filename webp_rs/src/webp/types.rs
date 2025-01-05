//! Types module for WebP encoding/decoding
//! Contains common type definitions and structures used across the WebP implementation

// Re-export standard integer types for consistency with C
pub use std::os::raw::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ulong, c_ushort};

// Define WebP specific integer types
pub type int8_t = i8;
pub type uint8_t = u8;
pub type int16_t = i16;
pub type uint16_t = u16;
pub type int32_t = i32;
pub type uint32_t = u32;
pub type int64_t = i64;
pub type uint64_t = u64;

/// Memory allocation function compatible with WebP's allocation scheme
///
/// # Safety
/// This function is unsafe because it returns a raw pointer and the caller must ensure proper deallocation
#[no_mangle]
pub unsafe extern "C" fn WebPMalloc(size: usize) -> *mut std::ffi::c_void {
    let layout = std::alloc::Layout::from_size_align(size, std::mem::align_of::<u8>())
        .unwrap_or(std::alloc::Layout::new::<u8>());

    let ptr = std::alloc::alloc(layout);
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    ptr as *mut std::ffi::c_void
}

/// Frees memory allocated by WebPMalloc
///
/// # Safety
/// This function is unsafe because:
/// - The pointer must have been allocated by WebPMalloc
/// - The pointer must not be null
/// - The pointer must not have been freed before
#[no_mangle]
pub unsafe extern "C" fn WebPFree(ptr: *mut std::ffi::c_void) {
    if !ptr.is_null() {
        let ptr = ptr as *mut u8;
        let layout = std::alloc::Layout::from_size_align(1, std::mem::align_of::<u8>())
            .unwrap_or(std::alloc::Layout::new::<u8>());
        std::alloc::dealloc(ptr, layout);
    }
}
