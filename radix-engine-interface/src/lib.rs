#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"] // Enables certain tests of deep typed SBOR to function

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// RE Scrypto ABI.
pub mod abi {
    pub use scrypto_abi::*;
}
/// RE addresses.
pub mod address;
/// RE APIs
pub mod api;
/// RE constants
pub mod constants;
/// RE crypto library
pub mod crypto;
/// RE data model.
pub mod data;
/// RE math library.
pub mod math;
/// RE node models.
pub mod model;
/// RE network abstractions.
pub mod node;
/// RE wasm interface
pub mod wasm;

mod macros;
pub use macros::*;

// Re-export SBOR derive.
extern crate sbor;
pub use sbor::{Decode, Encode, TypeId};

// Re-export Engine derive.
extern crate radix_engine_derive;
pub use radix_engine_derive::{scrypto, Describe};

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
extern crate self as radix_engine_interface;
