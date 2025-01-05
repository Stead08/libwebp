//! Bit writing and boolean coder utilities

use crate::webp::types::*;
use std::ptr;

// The number of bits for the bit writer
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

/// Boolean writer structure
#[repr(C)]
pub struct VP8BitWriter {
    pub range_: i32, // range-1
    pub value_: i32,
    pub run_: i32,     // number of outstanding bits
    pub nb_bits_: i32, // number of pending bits
    pub buf_: *mut u8, // internal buffer. Re-allocated regularly. Not owned.
    pub pos_: usize,
    pub max_pos_: usize,
    pub error_: i32, // true in case of error
}

impl Default for VP8BitWriter {
    fn default() -> Self {
        VP8BitWriter {
            range_: 0,
            value_: 0,
            run_: 0,
            nb_bits_: 0,
            buf_: ptr::null_mut(),
            pos_: 0,
            max_pos_: 0,
            error_: 0,
        }
    }
}

// Lookup tables for boolean decoder
const KVP8LOG2RANGE: [u8; 128] = [
    7, 6, 6, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
];

const KVP8NEWRANGE: [u8; 128] = [
    127, 127, 191, 127, 159, 191, 223, 127, 143, 159, 175, 191, 207, 223, 239, 127, 135, 143, 151,
    159, 167, 175, 183, 191, 199, 207, 215, 223, 231, 239, 247, 127, 131, 135, 139, 143, 147, 151,
    155, 159, 163, 167, 171, 175, 179, 183, 187, 191, 195, 199, 203, 207, 211, 215, 219, 223, 227,
    231, 235, 239, 243, 247, 251, 127, 129, 131, 133, 135, 137, 139, 141, 143, 145, 147, 149, 151,
    153, 155, 157, 159, 161, 163, 165, 167, 169, 171, 173, 175, 177, 179, 181, 183, 185, 187, 189,
    191, 193, 195, 197, 199, 201, 203, 205, 207, 209, 211, 213, 215, 217, 219, 221, 223, 225, 227,
    229, 231, 233, 235, 237, 239, 241, 243, 245, 247, 249, 251, 253, 127,
];

impl VP8BitWriter {
    /// Initialize the bit writer
    pub fn init(&mut self, expected_size: usize) -> bool {
        self.range_ = 255 - 1;
        self.value_ = 0;
        self.run_ = 0;
        self.nb_bits_ = -8;
        self.pos_ = 0;
        self.max_pos_ = 0;
        self.error_ = 0;
        self.buf_ = ptr::null_mut();
        if expected_size > 0 {
            self.resize(expected_size)
        } else {
            true
        }
    }

    /// Resize the internal buffer
    fn resize(&mut self, extra_size: usize) -> bool {
        let needed_size_64b = self.pos_ as u64 + extra_size as u64;
        let needed_size = needed_size_64b as usize;
        if needed_size_64b != needed_size as u64 {
            self.error_ = 1;
            return false;
        }
        if needed_size <= self.max_pos_ {
            return true;
        }

        let mut new_size = 2 * self.max_pos_;
        if new_size < needed_size {
            new_size = needed_size;
        }
        if new_size < 1024 {
            new_size = 1024;
        }

        let new_buf = unsafe { libc::malloc(new_size) as *mut u8 };

        if new_buf.is_null() {
            self.error_ = 1;
            return false;
        }

        if self.pos_ > 0 {
            assert!(!self.buf_.is_null());
            unsafe {
                ptr::copy_nonoverlapping(self.buf_, new_buf, self.pos_);
            }
        }

        unsafe {
            if !self.buf_.is_null() {
                libc::free(self.buf_ as *mut libc::c_void);
            }
        }

        self.buf_ = new_buf;
        self.max_pos_ = new_size;
        true
    }

    /// Flush bits to the buffer
    fn flush(&mut self) {
        let s = 8 + self.nb_bits_;
        let bits = self.value_ >> s;
        assert!(self.nb_bits_ >= 0);
        self.value_ -= bits << s;
        self.nb_bits_ -= 8;

        if (bits & 0xff) != 0xff {
            let pos = self.pos_;
            if !self.resize(self.run_ as usize + 1) {
                return;
            }
            if (bits & 0x100) != 0 {
                // overflow -> propagate carry over pending 0xff's
                if pos > 0 {
                    unsafe {
                        *self.buf_.add(pos - 1) += 1;
                    }
                }
            }
            if self.run_ > 0 {
                let value = if (bits & 0x100) != 0 { 0x00 } else { 0xff };
                for _ in 0..self.run_ {
                    unsafe {
                        *self.buf_.add(pos) = value;
                    }
                }
                self.pos_ += self.run_ as usize;
            }
            unsafe {
                *self.buf_.add(self.pos_) = (bits & 0xff) as u8;
            }
            self.pos_ += 1;
        } else {
            self.run_ += 1; // delay writing of bytes 0xff, pending eventual carry.
        }
    }

    /// Put a bit with probability
    pub fn put_bit(&mut self, bit: i32, prob: i32) -> i32 {
        let split = ((self.range_ as u32 * prob as u32) >> 8) as i32;
        if bit != 0 {
            self.value_ += split + 1;
            self.range_ -= split + 1;
        } else {
            self.range_ = split;
        }
        if self.range_ < 127 {
            // emit 'shift' bits out and renormalize
            let shift = KVP8LOG2RANGE[self.range_ as usize];
            self.range_ = KVP8NEWRANGE[self.range_ as usize] as i32;
            self.value_ <<= shift as i32;
            self.nb_bits_ += shift as i32;
            if self.nb_bits_ > 0 {
                self.flush();
            }
        }
        bit
    }

    /// Put a bit with uniform probability
    pub fn put_bit_uniform(&mut self, bit: i32) -> i32 {
        let split = self.range_ >> 1;
        if bit != 0 {
            self.value_ += split + 1;
            self.range_ -= split + 1;
        } else {
            self.range_ = split;
        }
        if self.range_ < 127 {
            self.range_ = KVP8NEWRANGE[self.range_ as usize] as i32;
            self.value_ <<= 1;
            self.nb_bits_ += 1;
            if self.nb_bits_ > 0 {
                self.flush();
            }
        }
        bit
    }

    /// Put multiple bits
    pub fn put_bits(&mut self, value: u32, nb_bits: i32) {
        assert!(nb_bits > 0 && nb_bits < 32);
        let mut mask = 1u32 << (nb_bits - 1);
        while mask != 0 {
            self.put_bit_uniform(if (value & mask) != 0 { 1 } else { 0 });
            mask >>= 1;
        }
    }

    /// Put signed bits
    pub fn put_signed_bits(&mut self, value: i32, nb_bits: i32) {
        if self.put_bit_uniform(if value != 0 { 1 } else { 0 }) == 0 {
            return;
        }
        if value < 0 {
            self.put_bits(((-value as u32) << 1) | 1, nb_bits + 1);
        } else {
            self.put_bits((value as u32) << 1, nb_bits + 1);
        }
    }

    /// Finish writing and return the buffer
    pub fn finish(&mut self) -> *mut u8 {
        self.put_bits(0, 9 - self.nb_bits_);
        self.nb_bits_ = 0; // pad with zeroes
        self.flush();
        self.buf_
    }

    /// Append data to the buffer
    pub fn append(&mut self, data: &[u8]) -> bool {
        if self.nb_bits_ != -8 {
            return false; // Flush() must have been called
        }
        if !self.resize(data.len()) {
            return false;
        }
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.buf_.add(self.pos_), data.len());
        }
        self.pos_ += data.len();
        true
    }

    /// Wipe out the bit writer
    pub fn wipe_out(&mut self) {
        unsafe {
            if !self.buf_.is_null() {
                libc::free(self.buf_ as *mut libc::c_void);
            }
        }
        *self = VP8BitWriter::default();
    }
}

/// VP8L bit writer structure
#[repr(C)]
pub struct VP8LBitWriter {
    pub bits_: bit_t,  // bit accumulator
    pub used_: i32,    // number of bits used in accumulator
    pub buf_: *mut u8, // start of buffer
    pub cur_: *mut u8, // current write position
    pub end_: *mut u8, // end of buffer
    pub error_: i32,   // error condition (true = error)
}

impl Default for VP8LBitWriter {
    fn default() -> Self {
        VP8LBitWriter {
            bits_: 0,
            used_: 0,
            buf_: ptr::null_mut(),
            cur_: ptr::null_mut(),
            end_: ptr::null_mut(),
            error_: 0,
        }
    }
}

impl VP8LBitWriter {
    /// Initialize the bit writer
    pub fn init(&mut self, expected_size: usize) -> bool {
        *self = VP8LBitWriter::default();
        self.resize(expected_size)
    }

    /// Resize the internal buffer
    fn resize(&mut self, extra_size: usize) -> bool {
        let max_bytes = unsafe { self.end_.offset_from(self.buf_) as usize };
        let current_size = unsafe { self.cur_.offset_from(self.buf_) as usize };
        let size_required_64b = current_size as u64 + extra_size as u64;
        let size_required = size_required_64b as usize;

        if size_required_64b != size_required as u64 {
            self.error_ = 1;
            return false;
        }

        if max_bytes > 0 && size_required <= max_bytes {
            return true;
        }

        let mut allocated_size = (3 * max_bytes) >> 1;
        if allocated_size < size_required {
            allocated_size = size_required;
        }
        // make allocated size multiple of 1k
        allocated_size = (((allocated_size >> 10) + 1) << 10);

        let allocated_buf = unsafe { libc::malloc(allocated_size) as *mut u8 };

        if allocated_buf.is_null() {
            self.error_ = 1;
            return false;
        }

        if current_size > 0 {
            unsafe {
                ptr::copy_nonoverlapping(self.buf_, allocated_buf, current_size);
            }
        }

        unsafe {
            if !self.buf_.is_null() {
                libc::free(self.buf_ as *mut libc::c_void);
            }
        }

        self.buf_ = allocated_buf;
        self.cur_ = unsafe { self.buf_.add(current_size) };
        self.end_ = unsafe { self.buf_.add(allocated_size) };
        true
    }

    /// Clone the bit writer
    pub fn clone_from(&mut self, src: &VP8LBitWriter) -> bool {
        let current_size = unsafe { src.cur_.offset_from(src.buf_) as usize };
        assert!(src.cur_ >= src.buf_ && src.cur_ <= src.end_);
        if !self.resize(current_size) {
            return false;
        }
        unsafe {
            ptr::copy_nonoverlapping(src.buf_, self.buf_, current_size);
        }
        self.bits_ = src.bits_;
        self.used_ = src.used_;
        self.error_ = src.error_;
        true
    }

    /// Reset the bit writer
    pub fn reset(&mut self, bw_init: &VP8LBitWriter) {
        self.bits_ = bw_init.bits_;
        self.used_ = bw_init.used_;
        self.cur_ = unsafe {
            self.buf_
                .add(bw_init.cur_.offset_from(bw_init.buf_) as usize)
        };
        self.error_ = bw_init.error_;
    }

    /// Swap two bit writers
    pub fn swap(&mut self, other: &mut VP8LBitWriter) {
        std::mem::swap(self, other);
    }

    /// Flush bits to the buffer
    pub fn flush_bits(&mut self) {
        while self.used_ >= 8 {
            unsafe {
                if self.cur_ >= self.end_ {
                    self.resize((self.cur_.offset_from(self.buf_) + 1) as usize);
                }
                *self.cur_ = self.bits_ as u8;
                self.cur_ = self.cur_.add(1);
                self.bits_ >>= 8;
                self.used_ -= 8;
            }
        }
    }

    /// Put bits internal implementation
    pub fn put_bits_internal(&mut self, bits: u32, n_bits: i32) {
        assert!(n_bits >= 0);
        if n_bits > 0 {
            if self.used_ + n_bits > 32 {
                self.flush_bits();
            }
            self.bits_ |= (bits as bit_t) << self.used_;
            self.used_ += n_bits;
        }
    }

    /// Get number of bytes written
    pub fn num_bytes(&self) -> usize {
        unsafe { (self.cur_.offset_from(self.buf_) as usize) + ((self.used_ + 7) >> 3) as usize }
    }

    /// Wipe out the bit writer
    pub fn wipe_out(&mut self) {
        unsafe {
            if !self.buf_.is_null() {
                libc::free(self.buf_ as *mut libc::c_void);
            }
        }
        *self = VP8LBitWriter::default();
    }
}

impl Drop for VP8BitWriter {
    fn drop(&mut self) {
        self.wipe_out();
    }
}

impl Drop for VP8LBitWriter {
    fn drop(&mut self) {
        self.wipe_out();
    }
}
