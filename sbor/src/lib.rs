#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// SBOR basic, no custom types
pub mod basic;
/// SBOR Categorize trait
pub mod categorize;
/// SBOR codec for core Rust types.
pub mod codec;
/// SBOR constants
pub mod constants;
/// SBOR decode trait.
pub mod decode;
/// SBOR decoding.
pub mod decoder;
/// SBOR encode trait.
pub mod encode;
/// SBOR encoding.
pub mod encoder;
/// SBOR paths.
pub mod path;
/// A facade of Rust types.
pub mod rust;
/// SBOR structured payload traversal.
pub mod traversal;

/// SBOR Schema
pub mod schema;
/// SBOR value model and any decoding/encoding.
pub mod value;
/// SBOR value kinds - ie the types of value that are supported.
pub mod value_kind;

#[cfg(feature = "serde")]
pub mod serde_serialization;

pub use basic::*;
pub use categorize::Categorize;
pub(crate) use categorize::{categorize_generic, categorize_simple};
pub use constants::*;
pub use decode::Decode;
pub use decoder::{BorrowingDecoder, DecodeError, Decoder, VecDecoder};
pub use encode::Encode;
pub use encoder::{EncodeError, Encoder, VecEncoder};
pub use path::{SborPath, SborPathBuf};

pub use schema::*;
pub use value::*;
pub use value_kind::*;

// Re-export derives
extern crate sbor_derive;
pub use sbor_derive::{Categorize, Decode, Encode, Sbor};

pub use sbor_derive::Describe;

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
extern crate self as sbor;
