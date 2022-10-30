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
mod utils;

pub use any::{decode_any, encode_any, encode_any_with_buffer, Value};
pub use decode::{Decode, DecodeError, Decoder};
pub use describe::{Describe, Type};
pub use encode::{Encode, Encoder};
pub use type_id::TypeId;
pub use utils::*;

// Re-export derives
extern crate sbor_derive;
pub use sbor_derive::{Decode, Describe, Encode, TypeId};

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
extern crate self as sbor;
