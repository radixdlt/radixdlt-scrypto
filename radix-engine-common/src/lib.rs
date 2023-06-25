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
/// Fixed native addresses
pub mod native_addresses;
/// RE network identifier model.
pub mod network;
/// RE time library.
pub mod time;
/// RE types.
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

/// Each module should have its own prelude, which:
/// * Adds preludes of upstream crates
/// * Exports types with specific-enough names which mean they can safely be used downstream.
///
/// The idea is that we can just include the current crate's prelude and avoid messing around with tons of includes.
/// This makes refactors easier, and makes integration into the node less painful.
pub mod prelude {
    // Exports from upstream libaries
    pub use radix_engine_constants::*;
    pub use radix_engine_derive::{
        ManifestCategorize, ManifestDecode, ManifestEncode, ManifestSbor, ScryptoCategorize,
        ScryptoDecode, ScryptoEncode, ScryptoEvent, ScryptoSbor,
    };
    pub use sbor::prelude::*;
    pub use sbor::{Categorize, Decode, Encode, Sbor};

    // Exports from this crate
    pub use super::address::*;
    pub use super::crypto::*;
    pub use super::data::manifest::prelude::*;
    pub use super::data::scrypto::prelude::*;
    pub use super::macros::*;
    pub use super::math::*;
    pub use super::native_addresses::*;
    pub use super::network::*;
    pub use super::time::*;
    pub use super::types::*;
    pub use crate::{
        dec, define_wrapped_hash, manifest_args, pdec, scrypto_args, to_manifest_value_and_unwrap,
    };
}

pub(crate) mod internal_prelude {
    pub use super::prelude::*;
    pub use sbor::representations::*;
    pub use sbor::traversal::*;
    pub use sbor::*;
}
