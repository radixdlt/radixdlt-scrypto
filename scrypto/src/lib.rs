#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// Scrypto data encoding/decoding and memory allocation scheme.
pub mod buffer;
/// Scrypto high level abstraction.
pub mod constructs;
/// Kernel APIs and helper functions.
pub mod kernel;
/// The prelude of Scrypto library.
pub mod prelude;
/// Scrypto resource containers and references.
pub mod resource;
/// Scrypto traits.
pub mod traits;
/// Utility functions, such as hashing and signature validation.
pub mod utils;

/// Scrypto blueprint ABI.
pub mod abi {
    pub use scrypto_abi::*;
}
/// Scrypto primitive types.
pub mod types {
    pub use scrypto_types::primitives::*;
}
pub use scrypto_types::rust;

// Re-export Scrypto derive.
extern crate scrypto_derive;
pub use scrypto_derive::*;

/// Encode arguments for invoking a blueprint or component.
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

/// Log an `ERROR` message.
#[macro_export]
macro_rules! error {
    ($($args: expr),+) => {{
        ::scrypto::constructs::Logger::log(scrypto::constructs::Level::Error, ::scrypto::rust::format!($($args),+));
    }};
}

/// Log a `WARN` message.
#[macro_export]
macro_rules! warn {
    ($($args: expr),+) => {{
        ::scrypto::constructs::Logger::log(scrypto::constructs::Level::Warn, ::scrypto::rust::format!($($args),+));
    }};
}

/// Log an `INFO` message.
#[macro_export]
macro_rules! info {
    ($($args: expr),+) => {{
        ::scrypto::constructs::Logger::log(scrypto::constructs::Level::Info, ::scrypto::rust::format!($($args),+));
    }};
}

/// Log a `DEBUG` message.
#[macro_export]
macro_rules! debug {
    ($($args: expr),+) => {{
        ::scrypto::constructs::Logger::log(scrypto::constructs::Level::Debug, ::scrypto::rust::format!($($args),+));
    }};
}

/// Log a `TRACE` message.
#[macro_export]
macro_rules! trace {
    ($($args: expr),+) => {{
        ::scrypto::constructs::Logger::log(scrypto::constructs::Level::Trace, ::scrypto::rust::format!($($args),+));
    }};
}
