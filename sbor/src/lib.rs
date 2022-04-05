#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// SBOR any data encoding and decoding.
pub mod any;
/// SBOR decoding.
pub mod decode;
/// SBOR describing.
pub mod describe;
/// SBOR encoding.
pub mod encode;
/// SBOR paths.
pub mod path;
/// A facade of Rust types.
pub mod rust;
/// SBOR type ids.
pub mod type_id;

pub use any::{decode_any, encode_any, Value};
pub use decode::{Decode, DecodeError, Decoder};
pub use describe::{Describe, Type};
pub use encode::{Encode, Encoder};
pub use type_id::TypeId;

use crate::rust::vec::Vec;

/// Encode a `T` into byte array, with type info included.
pub fn encode_with_type<T: Encode + ?Sized>(v: &T) -> Vec<u8> {
    let mut buf = Vec::with_capacity(512);
    let mut enc = Encoder::with_type(&mut buf);
    v.encode(&mut enc);
    buf
}

/// Encode a `T` into byte array, with no type info.
pub fn encode_no_type<T: Encode + ?Sized>(v: &T) -> Vec<u8> {
    let mut buf = Vec::with_capacity(512);
    let mut enc = Encoder::no_type(&mut buf);
    v.encode(&mut enc);
    buf
}

/// Decode an instance of `T` from a slice, with type info included.
pub fn decode_with_type<T: Decode>(buf: &[u8]) -> Result<T, DecodeError> {
    let mut dec = Decoder::with_type(buf);
    let v = T::decode(&mut dec)?;
    dec.check_end()?;
    Ok(v)
}

/// Decode an instance of `T` from a slice, with no type info.
pub fn decode_no_type<T: Decode>(buf: &[u8]) -> Result<T, DecodeError> {
    let mut dec = Decoder::no_type(buf);
    let v = T::decode(&mut dec)?;
    dec.check_end()?;
    Ok(v)
}

// Re-export derives
extern crate sbor_derive;
pub use sbor_derive::{Decode, Describe, Encode, TypeId};

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
extern crate self as sbor;
