#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// SBOR basic, no custom types
pub mod basic;
/// SBOR codec for core Rust types.
pub mod codec;
/// SBOR constants
pub mod constants;
/// SBOR decode trait.
pub mod decode;
/// SBOR decoding.
pub mod decoder;
/// SBOR encoding.
pub mod encode;
/// SBOR paths.
pub mod path;
/// A facade of Rust types.
pub mod rust;
/// SBOR type ids.
pub mod type_id;
/// SBOR value model and any decoding/encoding.
pub mod value;

pub use basic::*;
pub use constants::*;
pub use decode::Decode;
pub use decoder::{DecodeError, Decoder, DefaultVecDecoder, VecDecoder};
pub use encode::{Encode, Encoder};
pub use path::{SborPath, SborPathBuf};
pub use type_id::*;
pub use value::*;

/// Encode a `T` into byte array.
pub fn encode<X: CustomTypeId, T: Encode<X> + ?Sized>(v: &T) -> crate::rust::vec::Vec<u8> {
    let mut buf = crate::rust::vec::Vec::with_capacity(512);
    let mut enc = Encoder::new(&mut buf);
    v.encode(&mut enc);
    buf
}

/// Decode an instance of `T` from a slice.
pub fn decode<X: CustomTypeId, T: for<'de> Decode<X, DefaultVecDecoder<'de, X>>>(
    buf: &[u8],
) -> Result<T, DecodeError> {
    let mut decoder = DefaultVecDecoder::new(buf);
    let value = decoder.decode()?;
    decoder.check_end()?;
    Ok(value)
}

// Re-export derives
extern crate sbor_derive;
pub use sbor_derive::{Decode, Encode, TypeId};

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
extern crate self as sbor;
