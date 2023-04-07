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

/// Scrypto component abstraction.
pub mod component;
/// Scrypto engine abstraction.
pub mod engine;
/// Scrypto module abstraction.
pub mod modules;
/// Scrypto preludes.
pub mod prelude;
/// Scrypto resource abstraction.
pub mod resource;
/// Scrypto runtime abstraction.
pub mod runtime;

// Export macros
mod macros;
pub use macros::*;

// Re-export Scrypto derive.
extern crate scrypto_derive;
pub use scrypto_derive::{blueprint, NonFungibleData};

// Re-export Radix Engine Interface modules.
extern crate radix_engine_interface;
pub use radix_engine_interface::{
    address, api, blueprints, constants, crypto, data, math, network, schema, time, types,
};

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
pub extern crate self as scrypto;

/// Sets up panic hook.
pub fn set_up_panic_hook() {
    #[cfg(not(feature = "alloc"))]
    std::panic::set_hook(Box::new(|info| {
        // parse message
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(ToString::to_string)
            .or(info
                .payload()
                .downcast_ref::<String>()
                .map(ToString::to_string))
            .unwrap_or(String::new());

        // parse location
        let location = if let Some(l) = info.location() {
            format!("{}:{}:{}", l.file(), l.line(), l.column())
        } else {
            "<unknown>".to_owned()
        };

        crate::runtime::Logger::error(sbor::rust::format!(
            "Panicked at '{}', {}",
            payload,
            location
        ));
    }));
}
