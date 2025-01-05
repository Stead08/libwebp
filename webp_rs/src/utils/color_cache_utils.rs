//! Color Cache for WebP Lossless

use crate::utils::utils::*;
use std::ffi::c_void;

/// Main color cache struct
#[repr(C)]
pub struct VP8LColorCache {
    pub colors_: *mut u32, // color entries
    pub hash_shift_: i32,  // Hash shift: 32 - hash_bits_
    pub hash_bits_: i32,
}

const HASH_MUL: u32 = 0x1e35a7bd;

impl VP8LColorCache {
    /// Initialize the color cache with 'hash_bits' bits for the keys.
    /// Returns false in case of memory error.
    pub fn init(&mut self, hash_bits: i32) -> bool {
        let hash_size = 1 << hash_bits;
        assert!(hash_bits > 0);

        unsafe {
            let colors = WebPSafeCalloc(hash_size as u64, std::mem::size_of::<u32>());
            if colors.is_null() {
                return false;
            }
            self.colors_ = colors as *mut u32;
            self.hash_shift_ = 32 - hash_bits;
            self.hash_bits_ = hash_bits;
            true
        }
    }

    /// Clear the color cache
    pub fn clear(&mut self) {
        if !self.colors_.is_null() {
            unsafe {
                WebPSafeFree(self.colors_ as *mut c_void);
            }
            self.colors_ = std::ptr::null_mut();
        }
    }

    /// Copy color cache from src to dst
    pub fn copy_from(&mut self, src: &VP8LColorCache) {
        assert!(src.hash_bits_ == self.hash_bits_);
        unsafe {
            std::ptr::copy_nonoverlapping(src.colors_, self.colors_, 1 << self.hash_bits_);
        }
    }

    /// Hash a pixel
    #[inline]
    pub fn hash_pix(argb: u32, shift: i32) -> i32 {
        ((argb.wrapping_mul(HASH_MUL)) >> shift) as i32
    }

    /// Lookup a color in the cache
    #[inline]
    pub fn lookup(&self, key: u32) -> u32 {
        assert!((key >> self.hash_bits_) == 0);
        unsafe { *self.colors_.add(key as usize) }
    }

    /// Set a color in the cache
    #[inline]
    pub fn set(&self, key: u32, argb: u32) {
        assert!((key >> self.hash_bits_) == 0);
        unsafe {
            *self.colors_.add(key as usize) = argb;
        }
    }

    /// Insert a color into the cache
    #[inline]
    pub fn insert(&self, argb: u32) {
        let key = Self::hash_pix(argb, self.hash_shift_);
        unsafe {
            *self.colors_.add(key as usize) = argb;
        }
    }

    /// Get the index for a color
    #[inline]
    pub fn get_index(&self, argb: u32) -> i32 {
        Self::hash_pix(argb, self.hash_shift_)
    }

    /// Check if the cache contains a color
    /// Returns the key if cache contains argb, and -1 otherwise
    #[inline]
    pub fn contains(&self, argb: u32) -> i32 {
        let key = Self::hash_pix(argb, self.hash_shift_);
        unsafe {
            if *self.colors_.add(key as usize) == argb {
                key
            } else {
                -1
            }
        }
    }
}

impl Drop for VP8LColorCache {
    fn drop(&mut self) {
        self.clear();
    }
}

impl Default for VP8LColorCache {
    fn default() -> Self {
        VP8LColorCache {
            colors_: std::ptr::null_mut(),
            hash_shift_: 0,
            hash_bits_: 0,
        }
    }
}
