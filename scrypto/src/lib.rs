//! # The Scrypto Standard Library
//!
//! The Scrypto Standard Library is the foundation of Scrypto blueprints, a
//! set of minimal and shared abstractions on top of Radix Engine. It enables
//! asset-oriented programming for feature-rich DeFi dApps.
//!
//! If you know the name of what you're looking for, the fastest way to find
//! it is to use the <a href="#" onclick="focusSearchBar();">search
//! bar</a> at the top of the page.
//!

#![cfg_attr(not(feature = "std"), no_std)]

#![feature(type_name_of_val)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// Scrypto blueprint ABI.
pub mod abi {
    pub use scrypto_abi::*;
}
/// Scrypto address.
pub mod address;
/// Scrypto data encoding, decoding and exchange.
pub mod buffer;
/// Scrypto component library.
pub mod component;
/// Scrypto constants.
pub mod constants;
/// Scrypto core library, mainly process and transaction context.
pub mod core;
/// Scrypto cryptography library.
pub mod crypto;
/// Radix engine APIs.
pub mod engine;
/// Scrypto math library.
pub mod math;
/// Miscellaneous functions.
pub mod misc;
/// Scrypto preludes.
pub mod prelude;
/// Scrypto resource library.
pub mod resource;
/// Scrypto values.
pub mod values;

// Export macros
mod macros;
pub use macros::*;

// Re-export SBOR derive.
extern crate sbor;
pub use sbor::{Decode, Describe, Encode, TypeId};

// Re-export Scrypto derive.
extern crate scrypto_derive;
pub use scrypto_derive::{blueprint, import, NonFungibleData};

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
extern crate self as scrypto;
