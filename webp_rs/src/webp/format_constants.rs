//! Internal constants related to WebP file format
//! Contains various constants and definitions used in the WebP format

use crate::webp::types::uint32_t;

/// Create fourcc of the chunk from the chunk tag characters
#[inline]
pub const fn mkfourcc(a: u8, b: u8, c: u8, d: u8) -> uint32_t {
    (a as uint32_t) | ((b as uint32_t) << 8) | ((c as uint32_t) << 16) | ((d as uint32_t) << 24)
}

// VP8 related constants
pub const VP8_SIGNATURE: uint32_t = 0x9d012a;
pub const VP8_MAX_PARTITION0_SIZE: uint32_t = 1 << 19; // max size of mode partition
pub const VP8_MAX_PARTITION_SIZE: uint32_t = 1 << 24; // max size for token partition
pub const VP8_FRAME_HEADER_SIZE: uint32_t = 10; // Size of the frame header within VP8 data

// VP8L related constants
pub const VP8L_SIGNATURE_SIZE: uint32_t = 1; // VP8L signature size
pub const VP8L_MAGIC_BYTE: uint32_t = 0x2f; // VP8L signature byte
pub const VP8L_IMAGE_SIZE_BITS: uint32_t = 14; // Number of bits used to store width and height
pub const VP8L_VERSION_BITS: uint32_t = 3; // 3 bits reserved for version
pub const VP8L_VERSION: uint32_t = 0; // version 0
pub const VP8L_FRAME_HEADER_SIZE: uint32_t = 5; // Size of the VP8L frame header

pub const MAX_PALETTE_SIZE: uint32_t = 256;
pub const MAX_CACHE_BITS: uint32_t = 11;
pub const HUFFMAN_CODES_PER_META_CODE: uint32_t = 5;
pub const ARGB_BLACK: uint32_t = 0xff000000;

pub const DEFAULT_CODE_LENGTH: uint32_t = 8;
pub const MAX_ALLOWED_CODE_LENGTH: uint32_t = 15;

pub const NUM_LITERAL_CODES: uint32_t = 256;
pub const NUM_LENGTH_CODES: uint32_t = 24;
pub const NUM_DISTANCE_CODES: uint32_t = 40;
pub const CODE_LENGTH_CODES: uint32_t = 19;

pub const MIN_HUFFMAN_BITS: uint32_t = 2; // min number of Huffman bits
pub const NUM_HUFFMAN_BITS: uint32_t = 3;

// the maximum number of bits defining a transform is
// MIN_TRANSFORM_BITS + (1 << NUM_TRANSFORM_BITS) - 1
pub const MIN_TRANSFORM_BITS: uint32_t = 2;
pub const NUM_TRANSFORM_BITS: uint32_t = 3;

pub const TRANSFORM_PRESENT: uint32_t = 1; // The bit to be written when next data to be read is a transform
pub const NUM_TRANSFORMS: uint32_t = 4; // Maximum number of allowed transform in a bitstream

/// VP8L image transform types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VP8LImageTransformType {
    PredictorTransform = 0,
    CrossColorTransform = 1,
    SubtractGreenTransform = 2,
    ColorIndexingTransform = 3,
}

// Alpha related constants
pub const ALPHA_HEADER_LEN: uint32_t = 1;
pub const ALPHA_NO_COMPRESSION: uint32_t = 0;
pub const ALPHA_LOSSLESS_COMPRESSION: uint32_t = 1;
pub const ALPHA_PREPROCESSED_LEVELS: uint32_t = 1;

// Mux related constants
pub const TAG_SIZE: uint32_t = 4; // Size of a chunk tag (e.g. "VP8L")
pub const CHUNK_SIZE_BYTES: uint32_t = 4; // Size needed to store chunk's size
pub const CHUNK_HEADER_SIZE: uint32_t = 8; // Size of a chunk header
pub const RIFF_HEADER_SIZE: uint32_t = 12; // Size of the RIFF header ("RIFFnnnnWEBP")
pub const ANMF_CHUNK_SIZE: uint32_t = 16; // Size of an ANMF chunk
pub const ANIM_CHUNK_SIZE: uint32_t = 6; // Size of an ANIM chunk
pub const VP8X_CHUNK_SIZE: uint32_t = 10; // Size of a VP8X chunk

pub const MAX_CANVAS_SIZE: uint32_t = 1 << 24; // 24-bit max for VP8X width/height
pub const MAX_IMAGE_AREA: u64 = 1u64 << 32; // 32-bit max for width x height
pub const MAX_LOOP_COUNT: uint32_t = 1 << 16; // maximum value for loop-count
pub const MAX_DURATION: uint32_t = 1 << 24; // maximum duration
pub const MAX_POSITION_OFFSET: uint32_t = 1 << 24; // maximum frame x/y offset

// Maximum chunk payload is such that adding the header and padding won't overflow a uint32_t
pub const MAX_CHUNK_PAYLOAD: uint32_t = !0u32 - CHUNK_HEADER_SIZE - 1;
