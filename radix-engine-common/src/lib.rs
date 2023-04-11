#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"] // Enables certain tests of deep typed SBOR to function

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// RE bech32 address library.
pub mod address;
/// RE crypto library
pub mod crypto;
/// RE scrypto data model.
pub mod data;
/// RE math library.
pub mod math;
/// RE network identifier model.
pub mod network;
/// RE time library.
pub mod time;
/// RE types
pub mod types;

mod macros;
pub use macros::*;

// Re-export SBOR derive.
extern crate sbor;
pub use sbor::{Categorize, Decode, Encode, Sbor};

// Re-export radix engine derive.
extern crate radix_engine_derive;
pub use radix_engine_derive::{
    ManifestCategorize, ManifestDecode, ManifestEncode, ManifestSbor, ScryptoCategorize,
    ScryptoDecode, ScryptoEncode, ScryptoEvent, ScryptoSbor,
};

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
pub extern crate self as radix_engine_common;
