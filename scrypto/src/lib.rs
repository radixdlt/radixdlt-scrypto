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
/// Scrypto crypto utilities abstraction.
pub mod crypto_utils;
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

// Re-export Scrypto derive.
extern crate scrypto_derive;

pub use scrypto_derive::{blueprint, NonFungibleData};

// Re-export Radix Engine Interface modules.
extern crate radix_engine_interface;
pub use radix_engine_interface::{api, blueprints, object_modules};

// Re-export Radix Engine Common modules.
extern crate radix_common;
pub use radix_common::{address, constants, crypto, data, math, network, time};

pub mod types {
    // for backward compatibility
    pub use radix_common::types::*;
    pub use radix_engine_interface::types::*;
}

// Re-export blueprint schema init, for use in `#[blueprint]` macro
pub use radix_blueprint_schema_init;

// extern crate self as X; in lib.rs allows ::X and X to resolve to this crate inside this crate.
// This enables procedural macros which output code involving paths to this crate, to work inside
// this crate. See this link for details:
// https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
//
// IMPORTANT:
// This should never be pub, else `X::X::X::X::...` becomes a valid path in downstream crates,
// which we've discovered can cause really bad autocomplete times (when combined with other
// specific imports, generic traits, resolution paths which likely trigger edge cases in
// Rust Analyzer which get stuck on these infinite possible paths)
extern crate self as scrypto;

/// Sets up panic hook.
pub fn set_up_panic_hook() {
    #[cfg(not(feature = "alloc"))]
    std::panic::set_hook(Box::new(|info| {
        let mut message = String::new();

        // parse payload
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            message.push_str(s);
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            message.push_str(s);
        } else {
            message.push_str("Panic")
        }

        message.push_str(" @ ");

        // parse location
        if let Some(l) = info.location() {
            message.push_str(l.file());
            message.push_str(":");
            message.push_str(&l.line().to_string());
            message.push_str(":");
            message.push_str(&l.column().to_string());
        } else {
            message.push_str("<unknown>");
        };

        crate::runtime::Runtime::panic(message);
    }));
}

#[cfg(all(feature = "coverage"))]
#[no_mangle]
pub unsafe extern "C" fn dump_coverage() -> types::Slice {
    let mut coverage = vec![];
    minicov::capture_coverage(&mut coverage).unwrap();
    engine::wasm_api::forget_vec(coverage)
}
