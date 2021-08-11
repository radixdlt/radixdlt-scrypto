#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

mod address;
mod bid;
mod h256;
mod u256;

/// A facade around all Rust types scrypto uses from `std` or `core + alloc`.
pub mod rust;

pub use address::{Address, DecodeAddressError};
pub use bid::BID;
pub use h256::{DecodeH256Error, H256};
pub use u256::U256;
