//! # Guestio provides utilities to communicate efficiently with the guest.
//!
//! This is similar to serde's "Serialize" and "Deserialize", but has
//! different design targets:
//!
//! * Guest runtime performance is paramount.
//! * It's a significant performance degradation for the guest to have to read a value
//!   that's not aligned on a 32-bit boundary.
//! * Space efficiency is nice, but less important than guest runtime performance.
//! * We want to conveniently be able to take cryptographic hashes of a structure.
//!
//! In response, guestio does these things differently than serde:
//!
//! * There is only one on-wire format; guestio does not try to be as featureful as serde.
//! * Datatypes available are much more limited
//! * We don't want to spend any cycles deserializing or copying, so
//!   we store the data in a format that's easy to access without
//!   copying, similarly to the `rkyv'.
//! * In contrast to rkyv, we don't check the format up front, but only when
//!   accessed.
//! * In contrast to rkyv, we use accessor methods so we don't have to do any work
//!   computing fields that aren't accessed.
//! * We store all data buffers as [u32] (as opposed to the more common [u8]).
//! * There is one canonical format; any serialization of the same
//!   data will construct the same structure.  To take a cryptographic hash,
//!   we simply hash the [u32] slice.
//! * We null-pad all data buffers up to the size of the hash block
//!   (with one word remaining) to avoid copying when computing a
//!   hash; the last word is the index in the block of the root data
//!   structure.
//!
//! Note that for deserializing, while we guarantee sha(a) == sha(b)
//! implies deserialize_from(a) == deserialize_from(b), we do not
//! guarantee the converse, that deserialize_from(a) ==
//! deserialize_from(b) implies a == b.

extern crate alloc;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GuestioError {
    #[error("Buffer is too small; enlarge and try again.")]
    BufferTooSmall,

    #[error("Buffer underrun")]
    BufferUnderrun,
}

pub type Result<T> = core::result::Result<T, GuestioError>;

pub mod serialize_test;

pub const WORD_SIZE: usize = 4;
pub const PAD_WORDS: usize = 8;

mod util;
use util::as_words_padded;

pub mod deserialize;
pub mod serialize;

pub use deserialize::*;
pub use serialize::*;

pub fn align_bytes_to_words(bytes: usize) -> usize {
    (bytes + WORD_SIZE - 1) / WORD_SIZE
}

#[allow(dead_code)]
pub(crate) fn align_words_to_pads(words: usize) -> usize {
    (words + PAD_WORDS - 1) / PAD_WORDS
}

#[cfg(test)]
mod derive_test;
