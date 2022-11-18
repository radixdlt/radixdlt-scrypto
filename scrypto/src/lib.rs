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

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// Scrypto blueprint ABI.
pub mod abi {
    pub use scrypto_abi::*;
}
/// Scrypto buffer for wasm-engine data exchange.
pub mod buffer;
/// Scrypto component library.
pub mod component;
/// Scrypto core library, mainly process and transaction context.
pub mod core;
/// Scrypto data model.
pub mod data {
    pub use radix_engine_interface::data::*;
}
/// Scrypto math library.
pub mod math {
    pub use radix_engine_interface::math::*;
}
/// Scrypto RE node model.
pub mod model {
    pub use radix_engine_interface::model::*;
}
/// Radix Engine APIs.
pub mod engine;
/// Miscellaneous functions.
pub mod misc;
/// Scrypto resource library.
pub mod resource;

/// Scrypto preludes.
#[cfg(feature = "prelude")]
pub mod prelude;

// Export macros
mod macros;
pub use macros::*;

// Re-export radix engine derives
pub extern crate radix_engine_derive;
pub extern crate radix_engine_interface;
pub extern crate scrypto_abi;
pub use radix_engine_derive::{scrypto, Describe};

// Re-export Scrypto derive.
extern crate scrypto_derive;
pub use scrypto_derive::{blueprint, import, NonFungibleData};

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
extern crate self as scrypto;
