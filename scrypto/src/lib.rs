//! # The Scrypto Standard Library
//!
//! The Scrypto Standard Library is the foundation of Scrypto blueprints, a
//! set of minimal and shared abstractions for the Radix ecosystem.
//!
//! If you know the name of what you're looking for, the fastest way to find
//! it is to use the <a href="#" onclick="focusSearchBar();">search
//! bar</a> at the top of the page.
//!
//! Otherwise, you may want to start with the following modules:
//! * [`core`]
//! * [`resource`]
//! * [`types`]
//!

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// Scrypto data encoding/decoding and memory allocation.
pub mod buffer;
/// Scrypto core abstractions.
pub mod core;
/// Kernel APIs.
pub mod kernel;
/// The prelude of Scrypto library.
pub mod prelude;
/// Scrypto resource abstractions.
pub mod resource;
/// A facade of Rust standard types.
pub mod rust;
/// Scrypto primitive types.
pub mod types;
/// Utility functions.
pub mod utils;

/// Scrypto blueprint ABI.
pub mod abi {
    pub use scrypto_abi::*;
}

// Re-export Scrypto derive.
extern crate scrypto_derive;
pub use scrypto_derive::{auth, blueprint, import, NftData};

/// Encodes arguments according to Scrypto call abi.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// args!(5, "hello")
/// ```
#[macro_export]
macro_rules! args {
    ($($args: expr),*) => {
        {
            let mut args = ::scrypto::rust::vec::Vec::new();
            $(args.push(scrypto::buffer::scrypto_encode(&$args));)*
            args
        }
    };
}

/// Logs an `ERROR` message.
#[macro_export]
macro_rules! error {
    ($($args: expr),+) => {{
        ::scrypto::core::Logger::log(scrypto::kernel::LogLevel::Error, ::scrypto::rust::format!($($args),+));
    }};
}

/// Logs a `WARN` message.
#[macro_export]
macro_rules! warn {
    ($($args: expr),+) => {{
        ::scrypto::core::Logger::log(scrypto::kernel::LogLevel::Warn, ::scrypto::rust::format!($($args),+));
    }};
}

/// Logs an `INFO` message.
#[macro_export]
macro_rules! info {
    ($($args: expr),+) => {{
        ::scrypto::core::Logger::log(scrypto::kernel::LogLevel::Info, ::scrypto::rust::format!($($args),+));
    }};
}

/// Logs a `DEBUG` message.
#[macro_export]
macro_rules! debug {
    ($($args: expr),+) => {{
        ::scrypto::core::Logger::log(scrypto::kernel::LogLevel::Debug, ::scrypto::rust::format!($($args),+));
    }};
}

/// Logs a `TRACE` message.
#[macro_export]
macro_rules! trace {
    ($($args: expr),+) => {{
        ::scrypto::core::Logger::log(scrypto::kernel::LogLevel::Trace, ::scrypto::rust::format!($($args),+));
    }};
}

/// Includes package code as a byte array.
#[macro_export]
macro_rules! include_code {
    () => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/wasm32-unknown-unknown/release/out.wasm"
        ))
    };
    ($package_dir: expr) => {
        include_bytes!(concat!(
            $package_dir,
            "/target/wasm32-unknown-unknown/release/out.wasm"
        ))
    };
}
