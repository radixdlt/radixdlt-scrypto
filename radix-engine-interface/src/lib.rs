#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"] // Enables certain tests of deep typed SBOR to function

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

pub mod api;
pub mod blueprints;
pub mod constants;
pub mod traits;
pub mod types;

mod macros;
pub use macros::*;

// Re-export scrypto schema
pub mod schema {
    pub use scrypto_schema::*;
}

// Re-export radix engine common.
pub extern crate radix_engine_common;
pub use radix_engine_common::*;

// Re-export SBOR derive.
pub extern crate sbor;
pub use sbor::{Categorize, Decode, Encode, Sbor};

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
pub extern crate self as radix_engine_interface;
