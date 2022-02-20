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

/// Compiles a Scrypto package and returns the output WASM file.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// let wasm1 = compile_package!();
/// let wasm2 = compile_package!("/path/to/package");
/// ```
#[macro_export]
macro_rules! compile_package {
    ($wasm_name: expr) => {
        ::scrypto::misc::compile_package(env!("CARGO_MANIFEST_DIR"), $wasm_name)
    };
    ($package_dir: expr, $wasm_name: expr) => {
        ::scrypto::misc::compile_package($package_dir, $wasm_name)
    };
}

/// Returns a static reference to a package WASM file.
///
/// Notes:
/// * This macro will NOT compile the package, thus a pre-requisite compile step
/// may be required;
/// * Normally, the wasm WASM name is the package name with `-` replaced with `_`.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// let wasm1 = include_package!("foo_bar");
/// let wasm2 = include_package!("/path/to/package", "foo_bar");
/// ```
#[macro_export]
macro_rules! include_package {
    ($wasm_name: expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/wasm32-unknown-unknown/release/",
            $wasm_name,
            ".wasm"
        ))
    };
    ($package_dir: expr, $wasm_name: expr) => {
        include_bytes!(concat!(
            $package_dir,
            "/target/wasm32-unknown-unknown/release/",
            $wasm_name,
            ".wasm"
        ))
    };
}
