//! Misc. common utility functions

use crate::webp::types::*;
use std::mem;

/// Maximum memory amount that libwebp will ever try to allocate
#[cfg(target_pointer_width = "64")]
pub const WEBP_MAX_ALLOCABLE_MEMORY: u64 = 1u64 << 34;
#[cfg(target_pointer_width = "32")]
pub const WEBP_MAX_ALLOCABLE_MEMORY: u64 = (1u64 << 31) - (1 << 16);

/// Check if size can be safely allocated without overflow
#[inline]
pub fn check_size_overflow(size: u64) -> bool {
    size == size as usize as u64
}

/// Safe malloc: verify that the requested size is not too large, or return NULL.
///
/// # Safety
/// This function is unsafe because it returns a raw pointer.
#[no_mangle]
pub unsafe fn WebPSafeMalloc(nmemb: u64, size: usize) -> *mut std::ffi::c_void {
    if !check_size_arguments_overflow(nmemb, size) {
        return std::ptr::null_mut();
    }
    assert!(nmemb * size as u64 > 0);
    let layout =
        std::alloc::Layout::from_size_align((nmemb * size as u64) as usize, mem::align_of::<u8>())
            .unwrap_or(std::alloc::Layout::new::<u8>());

    let ptr = std::alloc::alloc(layout);
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    ptr as *mut std::ffi::c_void
}

/// Safe calloc: verify that the requested size is not too large, or return NULL.
///
/// # Safety
/// This function is unsafe because it returns a raw pointer.
#[no_mangle]
pub unsafe fn WebPSafeCalloc(nmemb: u64, size: usize) -> *mut std::ffi::c_void {
    if !check_size_arguments_overflow(nmemb, size) {
        return std::ptr::null_mut();
    }
    assert!(nmemb * size as u64 > 0);
    let layout =
        std::alloc::Layout::from_size_align((nmemb * size as u64) as usize, mem::align_of::<u8>())
            .unwrap_or(std::alloc::Layout::new::<u8>());

    let ptr = std::alloc::alloc_zeroed(layout);
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    ptr as *mut std::ffi::c_void
}

/// Safe free: companion deallocation function to the above allocations.
///
/// # Safety
/// This function is unsafe because:
/// - The pointer must have been allocated by WebPSafeMalloc or WebPSafeCalloc
/// - The pointer must not be null
/// - The pointer must not have been freed before
#[no_mangle]
pub unsafe fn WebPSafeFree(ptr: *mut std::ffi::c_void) {
    if !ptr.is_null() {
        let ptr = ptr as *mut u8;
        let layout = std::alloc::Layout::from_size_align(1, mem::align_of::<u8>())
            .unwrap_or(std::alloc::Layout::new::<u8>());
        std::alloc::dealloc(ptr, layout);
    }
}

/// Check if size arguments would cause overflow
fn check_size_arguments_overflow(nmemb: u64, size: usize) -> bool {
    if nmemb == 0 {
        return true;
    }
    if size as u64 > WEBP_MAX_ALLOCABLE_MEMORY / nmemb {
        return false;
    }
    check_size_overflow(nmemb * size as u64)
}

/// Read 16 bits stored in little-endian order
#[inline]
pub fn get_le16(data: &[u8]) -> i32 {
    (data[0] as i32) | ((data[1] as i32) << 8)
}

/// Read 24 bits stored in little-endian order
#[inline]
pub fn get_le24(data: &[u8]) -> i32 {
    get_le16(&data[..2]) | ((data[2] as i32) << 16)
}

/// Read 32 bits stored in little-endian order
#[inline]
pub fn get_le32(data: &[u8]) -> uint32_t {
    get_le16(&data[..2]) as uint32_t | ((get_le16(&data[2..4]) as uint32_t) << 16)
}

/// Store 16 bits in little-endian order
#[inline]
pub fn put_le16(data: &mut [u8], val: i32) {
    debug_assert!(val < (1 << 16));
    data[0] = (val >> 0) as u8;
    data[1] = (val >> 8) as u8;
}

/// Store 24 bits in little-endian order
#[inline]
pub fn put_le24(data: &mut [u8], val: i32) {
    debug_assert!(val < (1 << 24));
    put_le16(&mut data[..2], val & 0xffff);
    data[2] = (val >> 16) as u8;
}

/// Store 32 bits in little-endian order
#[inline]
pub fn put_le32(data: &mut [u8], val: uint32_t) {
    put_le16(&mut data[..2], (val & 0xffff) as i32);
    put_le16(&mut data[2..4], (val >> 16) as i32);
}

/// Returns (int)floor(log2(n)). n must be > 0.
#[inline]
pub fn bits_log2_floor(n: uint32_t) -> i32 {
    31 - n.leading_zeros() as i32
}

/// Counts the number of trailing zeros
#[inline]
pub fn bits_ctz(n: uint32_t) -> i32 {
    n.trailing_zeros() as i32
}

/// Copy width x height pixels from src to dst honoring the strides
pub fn copy_plane(
    src: &[u8],
    src_stride: i32,
    dst: &mut [u8],
    dst_stride: i32,
    width: i32,
    height: i32,
) {
    debug_assert!(src_stride.abs() >= width && dst_stride.abs() >= width);
    let mut src_ptr = 0;
    let mut dst_ptr = 0;

    for _ in 0..height {
        dst[dst_ptr..dst_ptr + width as usize]
            .copy_from_slice(&src[src_ptr..src_ptr + width as usize]);
        src_ptr += src_stride as usize;
        dst_ptr += dst_stride as usize;
    }
}
