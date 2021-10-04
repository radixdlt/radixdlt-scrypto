#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// Radix Engine implementation.
pub mod engine;
/// Radix ledger abstraction.
pub mod ledger;
/// Radix ledger data types.
pub mod model;
/// Radix Engine transaction model.
pub mod transaction;
/// Utility functions.
pub mod utils;
