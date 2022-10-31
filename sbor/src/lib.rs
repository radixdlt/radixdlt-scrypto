#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// SBOR decoding.
pub mod decode;
/// SBOR encoding.
pub mod encode;
/// SBOR paths.
pub mod path;
/// A facade of Rust types.
pub mod rust;
/// SBOR type ids.
pub mod type_id;
/// SBOR utility functions.
mod utils;
/// SBOR any value encoding and decoding.
pub mod value;

pub use decode::{Decode, DecodeError, Decoder};
pub use encode::{Encode, Encoder};
pub use path::{SborPath, SborPathBuf};
pub use type_id::{SborTypeId, TypeId};
pub use utils::*;
pub use value::*;

// Re-export derives
extern crate sbor_derive;
pub use sbor_derive::{Decode, Encode, TypeId};

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
extern crate self as sbor;
