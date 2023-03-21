#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

mod contextual_display;
#[cfg(feature = "serde")]
mod contextual_serialize;
pub mod rust;
mod slice;

pub use contextual_display::*;
#[cfg(feature = "serde")]
pub use contextual_serialize::*;
pub use slice::*;
