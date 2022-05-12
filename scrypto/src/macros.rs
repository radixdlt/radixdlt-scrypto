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
    () => {
        ::sbor::rust::vec::Vec::new()
    };
    ($($args: expr),+) => {
        {
            let mut args = ::sbor::rust::vec::Vec::new();
            $(args.push(scrypto::buffer::scrypto_encode(&$args));)+
            args
        }
    };
}

#[macro_export]
macro_rules! call_data_any_args {
    ($name:expr, $args: expr) => {{
        let variant = ::sbor::Value::Enum {
            name: $name,
            fields: $args,
        };
        ::sbor::encode_any(&variant)
    }};
}

#[macro_export]
macro_rules! call_data_bytes_args {
    ($name:expr, $args: expr) => {{
        let mut fields = Vec::new();
        for arg in $args {
            fields.push(::sbor::decode_any(&arg).unwrap());
        }
        ::scrypto::call_data_any_args!($name, fields)
    }};
}

#[macro_export]
macro_rules! call_data {
    ($name:expr, $($args: expr),*) => {
        {
            let mut fields = Vec::new();
            $(
                let encoded = ::scrypto::prelude::scrypto_encode(&$args);
                fields.push(::sbor::decode_any(&encoded).unwrap());
            )*
            ::scrypto::call_data_any_args!($name, fields)
        }
    };
    ($name:ident($($args: expr),*)) => {
        {
            let mut fields = Vec::new();
            $(
                let encoded = ::scrypto::prelude::scrypto_encode(&$args);
                fields.push(::sbor::decode_any(&encoded).unwrap());
            )*
            ::scrypto::call_data_any_args!(stringify!($name).to_string(), fields)
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
        ::scrypto::core::Logger::log(scrypto::core::Level::Error, ::sbor::rust::format!($($args),+));
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
        ::scrypto::core::Logger::log(scrypto::core::Level::Warn, ::sbor::rust::format!($($args),+));
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
        ::scrypto::core::Logger::log(scrypto::core::Level::Info, ::sbor::rust::format!($($args),+));
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
        ::scrypto::core::Logger::log(scrypto::core::Level::Debug, ::sbor::rust::format!($($args),+));
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
        ::scrypto::core::Logger::log(scrypto::core::Level::Trace, ::sbor::rust::format!($($args),+));
    }};
}

/// Compiles a Scrypto package and returns the output WASM file as byte array.
///
/// Notes:
/// * This macro only works when `std` is linked;
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// // This package
/// let wasm1 = compile_package!();
///
/// // Another package
/// let wasm2 = compile_package!("/path/to/package");
/// ```
#[macro_export]
macro_rules! compile_package {
    () => {
        ::scrypto::misc::compile_package(env!("CARGO_MANIFEST_DIR"))
    };
    ($package_dir: expr) => {
        ::scrypto::misc::compile_package($package_dir)
    };
}

/// Includes the WASM file of a Scrypto package.
///
/// Notes:
/// * This macro will NOT compile the package;
/// * The WASM file name is normally the package name with `-` replaced with `_`.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// // This package
/// let wasm1 = include_package!("wasm_name");
///
/// // Another package
/// let wasm2 = include_package!("/path/to/package", "wasm_name");
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
