#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

pub mod native_client_api;
pub mod account;
pub mod component;
pub mod consensus_manager;
pub mod modules;
pub mod resource;
pub mod runtime;

pub mod prelude {
    pub use crate::native_client_api::*;
}