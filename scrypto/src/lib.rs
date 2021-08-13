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

/// Invokes a blueprint method.
///
/// The first argument is the expected return type of the invoked method. It can
/// be a unit type `()` or any other type with trait `sbor::Decode`.
///
/// The second argument is the component name.
///
/// The third arguments are the method name.
///
/// The fourth argument is the *component address* if you're calling a method with
/// receiver type `&self` or `&mut self`; otherwise, it should be a *blueprint address*.
///
/// Additional arguments are the arguments, of types with trait `sbor::Encode`.
///
/// # Example
///
/// ```no_run
/// use scrypto::call;
/// use scrypto::types::Address;
///
/// /// Invoke a method with no return.
/// call!((), "Greeting", "say_hello", Address::from("06fc7287c4b2eb144df50e6b596631d4add864937e18aad5ff6e76"));
///
/// /// Invoke a method with argument `5` and expect a return of `i32`.
/// let rtn = call!(i32, "Counter", "add", Address::from("06fc7287c4b2eb144df50e6b596631d4add864937e18aad5ff6e76"), 5);
/// ```
///
#[macro_export]
macro_rules! call {
    ($return_type: ty, $component: expr, $method: expr, $address: expr) => {
        {
            // Convert into `Address`
            let addr: scrypto::types::Address = $address.into();

            // Prepare arguments
            let mut args = scrypto::types::rust::vec::Vec::new();

            // Invoke the method
            let rtn = if addr.is_blueprint() {
                scrypto::constructs::Blueprint::from(addr).call($component, $method, args)
            } else {
                scrypto::constructs::Component::from(addr).call($method, args)
            };

            // Decode the return
            scrypto::buffer::scrypto_decode::<$return_type>(&rtn).unwrap()
        }
    };

    ($return_type: ty, $component: expr, $method: expr, $address: expr, $($args: expr),+) => {
        {
            // Convert into `Address`
            let addr: scrypto::types::Address = $address.into();

            // Prepare arguments
            let mut args = scrypto::types::rust::vec::Vec::new();
            $(args.push(scrypto::buffer::scrypto_encode(&$args));)+

            // Invoke the method
            let rtn = if addr.is_blueprint() {
                scrypto::constructs::Blueprint::from(addr).call($component, $method, args)
            } else {
                scrypto::constructs::Component::from(addr).call($method, args)
            };

            // Decode the return
            scrypto::buffer::scrypto_decode::<$return_type>(&rtn).unwrap()
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
