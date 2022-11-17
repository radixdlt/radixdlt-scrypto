#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

pub mod abi {
    pub use scrypto_abi::*;
}
pub mod address;
pub mod core;
pub mod model;

/// Scrypto values.
pub mod data;
pub mod engine;
pub mod math;

// Export macros
pub mod constants;
pub mod crypto;
mod macros;

pub use macros::*;

// Re-export SBOR derive.
extern crate sbor;
pub use sbor::{Decode, Encode, TypeId};

extern crate radix_engine_derive;
pub use radix_engine_derive::{scrypto, Describe};

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
extern crate self as radix_engine_interface;
