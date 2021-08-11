#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// Scrypto component ABI.
pub mod abi {
    pub use scrypto_abi::*;
}
/// Scrypto data encoding/decoding and memory allocation scheme.
pub mod buffer;
/// Scrypto high level abstraction.
pub mod constructs;
/// Kernel APIs and helper functions.
pub mod kernel;
/// Scrypto resource containers and references.
pub mod resource;
/// Scrypto primitive types.
pub mod types {
    pub use scrypto_types::*;
}
/// Utility functions, such as hashing and signature validation.
pub mod utils;

// Re-export Scrypto derive.
extern crate scrypto_derive;
pub use scrypto_derive::*;

/// Call a method of a blueprint.
#[macro_export]
macro_rules! call_blueprint {
    ($rtn_type: ty, $blueprint: expr, $component: expr, $method: expr $(,)?) => {
        {
            let blueprint = scrypto::constructs::Blueprint::from($blueprint);
            let rtn = blueprint.call($component, $method, scrypto::types::rust::vec::Vec::new());
            scrypto::buffer::scrypto_decode::<$rtn_type>(&rtn).unwrap()
        }
    };

    ($rtn_type: ty, $blueprint: expr, $component: expr, $method: expr, $($args: expr),+ $(,)?) => {
        {
            let blueprint = scrypto::constructs::Blueprint::from($blueprint);
            let mut args = scrypto::types::rust::vec::Vec::new();
            $(args.push(scrypto::buffer::scrypto_encode(&$args));)+
            let rtn = blueprint.call($component, $method, args);
            scrypto::buffer::scrypto_decode::<$rtn_type>(&rtn).unwrap()
        }
    };
}

/// Call a method of a component.
#[macro_export]
macro_rules! call_component {
    ($rtn_type: ty, $component: expr, $method: expr $(,)?) => {
        {
            let component = scrypto::constructs::Component::from($component);
            let rtn = component.call($method, scrypto::types::rust::vec::Vec::new());
            scrypto::buffer::scrypto_decode::<$rtn_type>(&rtn).unwrap()
        }
    };

    ($rtn_type: ty, $component: expr, $method: expr, $($args: expr),+ $(,)?) => {
        {
            let component = scrypto::constructs::Component::from($component);
            let mut args = scrypto::types::rust::vec::Vec::new();
            $(args.push(scrypto::buffer::scrypto_encode(&$args));)+
            let rtn = component.call($method, args);
            scrypto::buffer::scrypto_decode::<$rtn_type>(&rtn).unwrap()
        }
    };
}

/// Log an `ERROR` message.
#[macro_export]
macro_rules! error {
    ($($args: expr),+) => {{
        scrypto::constructs::Logger::log(scrypto::kernel::Level::Error, scrypto::types::rust::format!($($args),+));
    }};
}

/// Log a `WARN` message.
#[macro_export]
macro_rules! warn {
    ($($args: expr),+) => {{
        scrypto::constructs::Logger::log(scrypto::kernel::Level::Warn, scrypto::types::rust::format!($($args),+));
    }};
}

/// Log an `INFO` message.
#[macro_export]
macro_rules! info {
    ($($args: expr),+) => {{
        scrypto::constructs::Logger::log(scrypto::kernel::Level::Info, scrypto::types::rust::format!($($args),+));
    }};
}

/// Log a `DEBUG` message.
#[macro_export]
macro_rules! debug {
    ($($args: expr),+) => {{
        scrypto::constructs::Logger::log(scrypto::kernel::Level::Debug, scrypto::types::rust::format!($($args),+));
    }};
}

/// Log a `TRACE` message.
#[macro_export]
macro_rules! trace {
    ($($args: expr),+) => {{
        scrypto::constructs::Logger::log(scrypto::kernel::Level::Trace, scrypto::types::rust::format!($($args),+));
    }};
}
