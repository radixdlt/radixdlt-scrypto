/// Logs an `ERROR` message.
///
/// # Example
/// ```no_run
/// use scrypto::prelude::*;
///
/// error!("Input number: {}", 100);
/// ```
#[macro_export]
macro_rules! error {
    ($($args: expr),+) => {{
        ::scrypto::runtime::Logger::log(radix_engine_interface::api::types::Level::Error, ::sbor::rust::format!($($args),+));
    }};
}

/// Logs a `WARN` message.
///
/// # Example
/// ```no_run
/// use scrypto::prelude::*;
///
/// warn!("Input number: {}", 100);
/// ```
#[macro_export]
macro_rules! warn {
    ($($args: expr),+) => {{
        ::scrypto::runtime::Logger::log(radix_engine_interface::api::types::Level::Warn, ::sbor::rust::format!($($args),+));
    }};
}

/// Logs an `INFO` message.
///
/// # Example
/// ```no_run
/// use scrypto::prelude::*;
///
/// info!("Input number: {}", 100);
/// ```
#[macro_export]
macro_rules! info {
    ($($args: expr),+) => {{
        ::scrypto::runtime::Logger::log(radix_engine_interface::api::types::Level::Info, ::sbor::rust::format!($($args),+));
    }};
}

/// Logs a `DEBUG` message.
///
/// # Example
/// ```no_run
/// use scrypto::prelude::*;
///
/// debug!("Input number: {}", 100);
/// ```
#[macro_export]
macro_rules! debug {
    ($($args: expr),+) => {{
        ::scrypto::runtime::Logger::log(radix_engine_interface::api::types::Level::Debug, ::sbor::rust::format!($($args),+));
    }};
}

/// Logs a `TRACE` message.
///
/// # Example
/// ```no_run
/// use scrypto::prelude::*;
///
/// trace!("Input number: {}", 100);
/// ```
#[macro_export]
macro_rules! trace {
    ($($args: expr),+) => {{
        ::scrypto::runtime::Logger::log(radix_engine_interface::api::types::Level::Trace, ::sbor::rust::format!($($args),+));
    }};
}

#[macro_export]
macro_rules! this_package {
    () => {
        env!("CARGO_MANIFEST_DIR")
    };
}

/// Includes the WASM file of a Scrypto package.
///
/// Notes:
/// * This macro will NOT compile the package;
/// * The binary name is normally the package name with `-` replaced with `_`.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// // This package
/// let wasm1 = include_code!("bin_name");
///
/// // Another package
/// let wasm2 = include_code!("/path/to/package", "bin_name");
/// ```
#[macro_export]
macro_rules! include_code {
    ($bin_name: expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/wasm32-unknown-unknown/release/",
            $bin_name,
            ".wasm"
        ))
    };
    ($package_dir: expr, $bin_name: expr) => {
        include_bytes!(concat!(
            $package_dir,
            "/target/wasm32-unknown-unknown/release/",
            $bin_name,
            ".wasm"
        ))
    };
}

/// Includes the ABI file of a Scrypto package.
///
/// Notes:
/// * This macro will NOT compile the package;
/// * The binary name is normally the package name with `-` replaced with `_`.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// // This package
/// let abi1 = include_abi!("bin_name");
///
/// // Another package
/// let abi2 = include_abi!("/path/to/package", "bin_name");
/// ```
#[macro_export]
macro_rules! include_abi {
    ($bin_name: expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/wasm32-unknown-unknown/release/",
            $bin_name,
            ".abi"
        ))
    };
    ($package_dir: expr, $bin_name: expr) => {
        include_bytes!(concat!(
            $package_dir,
            "/target/wasm32-unknown-unknown/release/",
            $bin_name,
            ".abi"
        ))
    };
}

/// Generates a bridge/stub to make package calls to a blueprint.
///
/// If you just wish to instead make calls to an instantiated component, see the `external_component` macro.
///
/// # Examples
/// ```no_run
/// use radix_engine_interface::address::Bech32Decoder;
/// use scrypto::prelude::*;
///
/// external_blueprint! {
///     CustomAccountBlueprint {
///         fn instantiate_global(account_name: &str) -> ComponentAddress;
///     }
/// }
///
/// #[derive(Categorize, Encode, Decode, LegacyDescribe)]
/// enum DepositResult {
///     Success,
///     Failure
/// }
///
/// external_component! {
///     AccountInterface {
///         fn deposit(&mut self, b: Bucket) -> DepositResult;
///         fn deposit_no_return(&mut self, b: Bucket);
///         fn read_balance(&self) -> Decimal;
///     }
/// }
///
/// fn instantiate_custom_account() -> ComponentAddress {
///     let package_address = Bech32Decoder::for_simulator()
///         .validate_and_decode_package_address("package_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsnznk7n")
///         .unwrap();
///     let blueprint = CustomAccountBlueprint::at(package_address, "CustomAccount");
///     blueprint.instantiate_global("account_name")
/// }
///
/// ```
///
/// # Related
///
/// - Replaces the import! macro for importing an abi, using a more concise, readable syntax.
/// - Similar to the `external_component` macro, which is used for making cross-component calls to an already-instantiated component.
#[macro_export]
macro_rules! external_blueprint {
    (
        $blueprint_ident:ident {
            $($blueprint_contents:tt)*
        }
    ) => {

        #[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
        struct $blueprint_ident {
            package_address: ::scrypto::model::PackageAddress,
            blueprint_name: ::sbor::rust::string::String,
        }

        // We allow dead code because it's used for importing interfaces, and not all the interface might be used
        #[allow(dead_code, unused_imports)]
        impl $blueprint_ident {
            fn at<S>(package_address: ::scrypto::model::PackageAddress, blueprint_name: S) -> Self
            where
                S: Into<::sbor::rust::string::String>
            {
                Self {
                    package_address,
                    blueprint_name: blueprint_name.into(),
                }
            }

            ::scrypto::external_blueprint_members!(
                $($blueprint_contents)*
            );
        }

        impl From<$blueprint_ident> for ::scrypto::model::PackageAddress {
            fn from(a: $blueprint_ident) -> ::scrypto::model::PackageAddress {
                a.package_address
            }
        }
    };
}

// This is a TT-Muncher, a useful guide for this type of use case is here: https://adventures.michaelfbryan.com/posts/non-trivial-macros/
#[macro_export]
macro_rules! external_blueprint_members {
    (
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        compile_error!("The external_blueprint! macro cannot be used to define component methods which take &self or &mut self. For these component methods, use a separate external_component! macro.");
    };
    (
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        compile_error!("The external_blueprint! macro cannot be used to define component methods which take &self or &mut self. For these component methods, use a separate external_component! macro.");
    };
    (
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        compile_error!("The external_blueprint! macro cannot be used to define component methods which take &self or &mut self. For these component methods, use a separate external_component! macro.");
    };
    (
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        compile_error!("The external_blueprint! macro cannot be used to define component methods which take &self or &mut self. For these component methods, use a separate external_component! macro.");
    };
    (
        fn $method_name:ident(self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        compile_error!("The external_blueprint! macro cannot be used to define component methods which take &self or &mut self. Also, just self is not supported. For these component methods, use a separate external_component! macro.");
    };
    (
        fn $method_name:ident(self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        compile_error!("The external_blueprint! macro cannot be used to define component methods which take &self or &mut self. Also, just self is not supported. For these component methods, use a separate external_component! macro.");
    };
    (
        fn $func_name:ident($($func_args:ident: $func_types:ty),*) -> $func_output:ty;
        $($rest:tt)*
    ) => {
        pub fn $func_name(&self, $($func_args: $func_types),*) -> $func_output {
            ::scrypto::runtime::Runtime::call_function(
                self.package_address,
                &self.blueprint_name,
                stringify!($func_name),
                args!($($func_args),*)
            )
        }
        ::scrypto::external_blueprint_members!($($rest)*);
    };
    (
        fn $func_name:ident($($func_args:ident: $func_types:ty),*);
        $($rest:tt)*
    ) => {
        pub fn $func_name(&self, $($func_args: $func_types),*) {
            use ::scrypto::rust::str::FromStr;
            ::scrypto::runtime::Runtime::call_function(
                self.package_address,
                &self.blueprint_name,
                stringify!($func_name),
                args!($($func_args),*)
            )
        }
        ::scrypto::external_blueprint_members!($($rest)*);
    };
    () => {}
}

/// Generates a bridge/stub to make cross-component calls.
///
/// # Examples
/// ```no_run
/// use scrypto::prelude::*;
///
/// #[derive(Categorize, Encode, Decode, LegacyDescribe)]
/// enum DepositResult {
///     Success,
///     Failure
/// }
///
/// external_component! {
///     AccountInterface {
///         fn deposit(&mut self, b: Bucket) -> DepositResult;
///         fn deposit_no_return(&mut self, b: Bucket);
///         fn read_balance(&self) -> Decimal;
///     }
/// }
///
/// fn bridge_to_existing_account(component_address: ComponentAddress) {
///     let existing_account = AccountInterface::at(component_address);
///     let balance = existing_account.read_balance();
///     // ...
/// }
/// ```
///
/// # Related
///
/// - Similar to the [external_blueprint] macro, but the external_component can be used without knowing the package and blueprint addresses.
#[macro_export]
macro_rules! external_component {
    (
        $component_ident:ident {
            $($component_methods:tt)*
        }
    ) => {
        #[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode, LegacyDescribe)]
        struct $component_ident {
            component_address: ::scrypto::model::ComponentAddress,
        }

        // We allow dead code because it's used for importing interfaces, and not all the interface might be used
        #[allow(dead_code, unused_imports)]
        impl $component_ident {
            fn at(component_address: ::scrypto::model::ComponentAddress) -> Self {
                Self {
                    component_address,
                }
            }

            ::scrypto::external_component_members!($($component_methods)*);
        }

        impl From<::scrypto::model::ComponentAddress> for $component_ident {
            fn from(component_address: ::scrypto::model::ComponentAddress) -> Self {
                Self {
                    component_address
                }
            }
        }

        impl From<$component_ident> for ::scrypto::model::ComponentAddress {
            fn from(a: $component_ident) -> ::scrypto::model::ComponentAddress {
                a.component_address
            }
        }
    };
}

// This is a TT-Muncher, a useful guide for this type of use case is here: https://adventures.michaelfbryan.com/posts/non-trivial-macros/
#[macro_export]
macro_rules! external_component_members {
    (
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        pub fn $method_name(&self $(, $method_args: $method_types)*) -> $method_output {
            ::scrypto::runtime::Runtime::call_method(
                self.component_address,
                stringify!($method_name),
                args!($($method_args),*)
            )
        }
        ::scrypto::external_component_members!($($rest)*);
    };
    (
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        pub fn $method_name(&self $(, $method_args: $method_types)*) {
            ::scrypto::runtime::Runtime::call_method(
                self.component_address,
                stringify!($method_name),
                args!($($method_args),*)
            )
        }
        ::scrypto::external_component_members!($($rest)*);
    };
    (
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        pub fn $method_name(&mut self $(, $method_args: $method_types)*) -> $method_output {
            ::scrypto::runtime::Runtime::call_method(
                self.component_address,
                stringify!($method_name),
                args!($($method_args),*)
            )
        }
        ::scrypto::external_component_members!($($rest)*);
    };
    (
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        pub fn $method_name(&mut self $(, $method_args: $method_types)*) {
            ::scrypto::runtime::Runtime::call_method(
                self.component_address,
                stringify!($method_name),
                args!($($method_args),*)
            )
        }
        ::scrypto::external_component_members!($($rest)*);
    };
    (
        fn $method_name:ident(self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        compile_error!("Components cannot define methods taking self. Did you mean &self or &mut self instead?");
    };
    (
        fn $method_name:ident(self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        compile_error!("Components cannot define methods taking self. Did you mean &self or &mut self instead?");
    };
    (
        fn $method_name:ident($($method_args:ident: $method_types:ty),*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        compile_error!("The external_component! macro cannot be used to define static blueprint methods which don't take &self or &mut self. For these package methods, use a separate external_blueprint! macro.");
    };
    (
        fn $method_name:ident($($method_args:ident: $method_types:ty),*);
        $($rest:tt)*
    ) => {
        compile_error!("The external_component! macro cannot be used to define static blueprint methods which don't take &self or &mut self. For these package methods, use a separate external_blueprint! macro.");
    };
    () => {}
}
