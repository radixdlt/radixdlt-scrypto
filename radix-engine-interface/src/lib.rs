#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"] // Enables certain tests of deep typed SBOR to function

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// RE client APIs
pub mod api;
/// RE blueprints interface
pub mod blueprints;
/// RE constants
pub mod constants;
/// RE events.
pub mod events;

mod macros;
pub use macros::*;

// Re-export common
pub mod abi {
    pub use scrypto_abi::*;
}
pub mod address {
    pub use radix_engine_common::address::*;
}
pub mod crypto {
    pub use radix_engine_common::crypto::*;
}
pub mod data {
    pub use radix_engine_common::data::*;
}
pub mod math {
    pub use radix_engine_common::math::*;
}
pub mod network {
    pub use radix_engine_common::network::*;
}
pub mod time {
    pub use radix_engine_common::time::*;
}
pub use radix_engine_common::{construct_address, dec, pdec, vanity_address};

// Re-export SBOR derive.
extern crate sbor;
pub use sbor::{Categorize, Decode, Encode, Sbor};

// Re-export Engine derive.
extern crate radix_engine_common;
pub use radix_engine_common::*;

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
pub extern crate self as radix_engine_interface;
