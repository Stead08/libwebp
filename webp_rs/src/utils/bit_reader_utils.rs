//! Boolean decoder and bit reader utilities

use crate::webp::types::*;
use std::ptr;

// The number of bits prefetched by the bit reader
#[cfg(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "wasm32",
    target_arch = "wasm64"
))]
pub const BITS: i32 = 56;
#[cfg(any(target_arch = "x86", target_arch = "arm", target_arch = "mips"))]
pub const BITS: i32 = 24;
#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "wasm32",
    target_arch = "wasm64",
    target_arch = "x86",
    target_arch = "arm",
    target_arch = "mips"
)))]
pub const BITS: i32 = 24;

// Derived types for bit reader
#[cfg(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "wasm32",
    target_arch = "wasm64"
))]
pub type bit_t = u64;
#[cfg(not(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    target_arch = "wasm32",
    target_arch = "wasm64"
)))]
pub type bit_t = u32;

pub type range_t = u32;

/// Boolean decoder structure
#[repr(C)]
pub struct VP8BitReader {
    pub value_: bit_t,       // current value
    pub range_: range_t,     // current range minus 1. In [127, 254] interval
    pub bits_: i32,          // number of valid bits left
    pub buf_: *const u8,     // next byte to be read
    pub buf_end_: *const u8, // end of read buffer
    pub buf_max_: *const u8, // max packed-read position on buffer
    pub eof_: i32,           // true if input is exhausted
}

impl Default for VP8BitReader {
    fn default() -> Self {
        VP8BitReader {
            value_: 0,
            range_: 0,
            bits_: 0,
            buf_: ptr::null(),
            buf_end_: ptr::null(),
            buf_max_: ptr::null(),
            eof_: 0,
        }
    }
}

/// Initialize the bit reader and the boolean decoder
pub fn vp8_init_bit_reader(br: &mut VP8BitReader, start: *const u8, size: usize) {
    assert!(!start.is_null());
    assert!(size < (1u32 << 31) as usize);
    br.range_ = 255 - 1;
    br.value_ = 0;
    br.bits_ = -8; // to load the very first 8bits
    br.eof_ = 0;
    vp8_bit_reader_set_buffer(br, start, size);
    unsafe {
        vp8_load_new_bytes(br);
    }
}

/// Sets the working read buffer
pub fn vp8_bit_reader_set_buffer(br: &mut VP8BitReader, start: *const u8, size: usize) {
    br.buf_ = start;
    br.buf_end_ = unsafe { start.add(size) };
    br.buf_max_ = if size >= std::mem::size_of::<bit_t>() {
        unsafe { start.add(size - std::mem::size_of::<bit_t>() + 1) }
    } else {
        start
    };
}

/// Update internal pointers to displace the byte buffer
pub fn vp8_remap_bit_reader(br: &mut VP8BitReader, offset: isize) {
    if !br.buf_.is_null() {
        unsafe {
            br.buf_ = br.buf_.offset(offset);
            br.buf_end_ = br.buf_end_.offset(offset);
            br.buf_max_ = br.buf_max_.offset(offset);
        }
    }
}

/// Load new bytes into the boolean decoder
#[inline]
pub unsafe fn vp8_load_new_bytes(br: &mut VP8BitReader) {
    if br.buf_ < br.buf_end_ {
        br.value_ = (br.value_ << 8) | (*br.buf_ as bit_t);
        br.buf_ = br.buf_.add(1);
        br.bits_ += 8;
    } else if br.eof_ == 0 {
        br.value_ <<= 8;
        br.bits_ += 8;
        br.eof_ = 1;
    } else {
        br.bits_ = 0; // This is to avoid undefined behaviour with shifts
    }
}

/// Read bits from the boolean decoder
#[inline]
pub fn vp8_get_value(br: &mut VP8BitReader, bits: i32) -> u32 {
    let mut v: u32 = 0;
    let mut n = bits;
    while n > 0 {
        v |= (vp8_get_bit(br, 0x80) as u32) << (n - 1);
        n -= 1;
    }
    v
}

/// Read signed value from the boolean decoder
#[inline]
pub fn vp8_get_signed_value(br: &mut VP8BitReader, bits: i32) -> i32 {
    let value = vp8_get_value(br, bits);
    if vp8_get_bit(br, 0x80) != 0 {
        -(value as i32)
    } else {
        value as i32
    }
}

/// Read a bit from the boolean decoder
#[inline]
pub fn vp8_get_bit(br: &mut VP8BitReader, prob: i32) -> i32 {
    let split = (((br.range_ as u32) * (prob as u32)) >> 8) as u32;
    let bit: i32;
    #[allow(arithmetic_overflow)]
    if (br.value_ as u32) > (split << (BITS + 8)) {
        br.range_ -= split + 1;
        br.value_ -= (split + 1) as bit_t;
        bit = 1;
    } else {
        br.range_ = split;
        bit = 0;
    }

    if br.range_ < 0x7f {
        let shift = kVP8Log2Range[br.range_ as usize] as i32;
        br.range_ = kVP8NewRange[br.range_ as usize] as u32;
        br.value_ <<= shift;
        br.bits_ -= shift;
        if br.bits_ < 0 {
            unsafe {
                vp8_load_new_bytes(br);
            }
        }
    }
    bit
}

// Lookup tables for boolean decoder
const kVP8Log2Range: [u8; 128] = [
    7, 6, 6, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
];

const kVP8NewRange: [u8; 128] = [
    127, 127, 191, 127, 159, 191, 223, 127, 143, 159, 175, 191, 207, 223, 239, 127, 135, 143, 151,
    159, 167, 175, 183, 191, 199, 207, 215, 223, 231, 239, 247, 127, 131, 135, 139, 143, 147, 151,
    155, 159, 163, 167, 171, 175, 179, 183, 187, 191, 195, 199, 203, 207, 211, 215, 219, 223, 227,
    231, 235, 239, 243, 247, 251, 127, 129, 131, 133, 135, 137, 139, 141, 143, 145, 147, 149, 151,
    153, 155, 157, 159, 161, 163, 165, 167, 169, 171, 173, 175, 177, 179, 181, 183, 185, 187, 189,
    191, 193, 195, 197, 199, 201, 203, 205, 207, 209, 211, 213, 215, 217, 219, 221, 223, 225, 227,
    229, 231, 233, 235, 237, 239, 241, 243, 245, 247, 249, 251, 253, 127,
];
