/// Encodes arguments according to Scrypto ABI.
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
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// error!("Input number: {}", 100);
/// ```
#[macro_export]
macro_rules! error {
    ($($args: expr),+) => {{
        ::scrypto::core::Logger::log(scrypto::core::Level::Error, ::scrypto::rust::format!($($args),+));
    }};
}

/// Logs a `WARN` message.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// warn!("Input number: {}", 100);
/// ```
#[macro_export]
macro_rules! warn {
    ($($args: expr),+) => {{
        ::scrypto::core::Logger::log(scrypto::core::Level::Warn, ::scrypto::rust::format!($($args),+));
    }};
}

/// Logs an `INFO` message.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// info!("Input number: {}", 100);
/// ```
#[macro_export]
macro_rules! info {
    ($($args: expr),+) => {{
        ::scrypto::core::Logger::log(scrypto::core::Level::Info, ::scrypto::rust::format!($($args),+));
    }};
}

/// Logs a `DEBUG` message.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// debug!("Input number: {}", 100);
/// ```
#[macro_export]
macro_rules! debug {
    ($($args: expr),+) => {{
        ::scrypto::core::Logger::log(scrypto::core::Level::Debug, ::scrypto::rust::format!($($args),+));
    }};
}

/// Logs a `TRACE` message.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// trace!("Input number: {}", 100);
/// ```
#[macro_export]
macro_rules! trace {
    ($($args: expr),+) => {{
        ::scrypto::core::Logger::log(scrypto::core::Level::Trace, ::scrypto::rust::format!($($args),+));
    }};
}

/// Includes package code as a byte array.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// let code = include_code!("lib_name");
/// let code2 = include_code!("/path/to/package", "lib_name");
/// ```
#[macro_export]
macro_rules! include_code {
    ($lib_name: expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/wasm32-unknown-unknown/release/",
            $lib_name,
            ".wasm"
        ))
    };
    ($package_dir: expr, $lib_name: expr) => {
        include_bytes!(concat!(
            $package_dir,
            "/target/wasm32-unknown-unknown/release/",
            $lib_name,
            ".wasm"
        ))
    };
}
