#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"] // Enables certain tests of deep typed SBOR to function

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// RE bech32 address library.
pub mod address;
/// RE protocol constants
pub mod constants;
/// RE crypto library
pub mod crypto;
/// RE scrypto data model.
pub mod data;
/// RE math library.
pub mod math;
/// RE network identifier model.
pub mod network;
/// Common models for state changes in RE
pub mod state;
/// RE time library.
pub mod time;
/// RE types.
pub mod types;

pub mod traits;

mod macros;

// Re-export SBOR derive.
extern crate sbor;
pub use sbor::{Categorize, Decode, Encode, Sbor};

// Re-export radix engine derive.
extern crate radix_sbor_derive;
pub use radix_sbor_derive::{
    ManifestCategorize, ManifestDecode, ManifestEncode, ManifestSbor, ScryptoCategorize,
    ScryptoDecode, ScryptoEncode, ScryptoEvent, ScryptoSbor, ScryptoSborAssertion,
};

// extern crate self as X; in lib.rs allows ::X and X to resolve to this crate inside this crate.
// This enables procedural macros which output code involving paths to this crate, to work inside
// this crate. See this link for details:
// https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
//
// IMPORTANT:
// This should never be pub, else `X::X::X::X::...` becomes a valid path in downstream crates,
// which we've discovered can cause really bad autocomplete times (when combined with other
// specific imports, generic traits, resolution paths which likely trigger edge cases in
// Rust Analyzer which get stuck on these infinite possible paths)
extern crate self as radix_common;

/// Each module should have its own prelude, which:
/// * Adds preludes of upstream crates
/// * Exports types with specific-enough names which mean they can safely be used downstream.
///
/// The idea is that we can just include the current crate's prelude and avoid messing around with tons of includes.
/// This makes refactors easier, and makes integration into the node less painful.
pub mod prelude {
    // Exports from upstream libraries
    pub use radix_sbor_derive::{
        ManifestCategorize, ManifestDecode, ManifestEncode, ManifestSbor, ScryptoCategorize,
        ScryptoDecode, ScryptoDescribe, ScryptoEncode, ScryptoEvent, ScryptoSbor,
        ScryptoSborAssertion,
    };
    pub use sbor::prelude::*;
    pub use sbor::*;

    // Exports from this crate
    pub use super::address::*;
    pub use super::constants::*;
    pub use super::crypto::*;
    pub use super::data::manifest::prelude::*;
    pub use super::data::scrypto::prelude::*;
    pub use super::math::*;
    pub use super::network::*;
    pub use super::state::*;
    pub use super::time::*;
    pub use super::traits::*;
    pub use super::types::*;
    pub use crate::{
        define_wrapped_hash, i, manifest_args, scrypto_args, to_manifest_value_and_unwrap,
    };
}

pub(crate) mod internal_prelude {
    pub use super::prelude::*;
    pub use sbor::representations::*;
    pub use sbor::traversal::*;
}
