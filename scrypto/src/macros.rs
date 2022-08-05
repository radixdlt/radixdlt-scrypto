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

/// Creates a `Decimal` from literals.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// let a = dec!(1);
/// let b = dec!("1.1");
/// ```
#[macro_export]
macro_rules! dec {
    ($x:literal) => {
        ::scrypto::math::Decimal::from($x)
    };

    ($base:literal, $shift:literal) => {
        // Base can be any type that converts into a Decimal, and shift must support
        // comparison and `-` unary operation, enforced by rustc.
        {
            let base = ::scrypto::math::Decimal::from($base);
            if $shift >= 0 {
                base * ::scrypto::math::Decimal::try_from(::scrypto::math::I256::from(10u8).pow(u32::try_from($shift).expect("Shift overflow"))).expect("Shift overflow")
            } else {
                base / ::scrypto::math::Decimal::try_from(::scrypto::math::I256::from(10u8).pow(u32::try_from(-$shift).expect("Shift overflow"))).expect("Shift overflow")
            }
        }
    };
}

/// Creates a `PreciseDecimal` from literals.
///
/// # Example
/// ```ignore
/// use scrypto::prelude::*;
///
/// let a = pdec!(1);
/// let b = pdec!("1.1");
/// ```
#[macro_export]
macro_rules! pdec {
    ($x:literal) => {
        ::scrypto::math::PreciseDecimal::from($x)
    };

    ($base:literal, $shift:literal) => {
        // Base can be any type that converts into a PreciseDecimal, and shift must support
        // comparison and `-` unary operation, enforced by rustc.
        {
            let base = ::scrypto::math::PreciseDecimal::from($base);
            if $shift >= 0 {
                base * ::scrypto::math::PreciseDecimal::try_from(::scrypto::math::I512::from(10u8).pow(u32::try_from($shift).expect("Shift overflow"))).expect("Shift overflow")
            } else {
                base / ::scrypto::math::PreciseDecimal::try_from(::scrypto::math::I512::from(10u8).pow(u32::try_from(-$shift).expect("Shift overflow"))).expect("Shift overflow")
            }
        }
    };
}

#[macro_export]
macro_rules! to_struct {
    ($($args: expr),*) => {{
        let mut fields = Vec::new();
        $(
            let encoded = ::scrypto::buffer::scrypto_encode(&$args);
            fields.push(::sbor::decode_any(&encoded).unwrap());
        )*
        let input_struct = ::sbor::Value::Struct {
            fields,
        };
        ::sbor::encode_any(&input_struct)
    }};
}

#[macro_export]
macro_rules! vec_to_struct {
    ($args: expr) => {{
        let input_struct = ::sbor::Value::Struct { fields: $args };
        ::sbor::encode_any(&input_struct)
    }};
}

#[macro_export]
macro_rules! bytes_vec_to_struct {
    ($args: expr) => {{
        let mut fields = Vec::new();
        for arg in $args {
            fields.push(::sbor::decode_any(&arg).unwrap());
        }
        let input_struct = ::sbor::Value::Struct { fields };
        ::sbor::encode_any(&input_struct)
    }};
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

/**
Generates a bridge/stub to make package calls to a blueprint.

You can also use this to make calls to the component itself.
If you just wish to make calls to an instantiated component, see the [external_component]! macro.

# Examples
```
use scrypto::prelude::*;
use sbor::{TypeId, Encode, Decode, Describe};

#[derive(TypeId, Encode, Decode, Describe)]
enum DepositResult {
    Success,
    Failure
}

external_blueprint! {
    {
        package: "package_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsnznk7n",
        blueprint: "CustomAccount"
    },
    CustomAccount {
        fn instantiate_global(account_name: &str) -> ComponentAddress;
        fn deposit(&mut self, b: Bucket) -> DepositResult;
        fn deposit_no_return(&mut self, b: Bucket);
        fn read_balance(&self) -> Decimal;
    }
}

fn create_custom_accounts() {
    let new_account_address = CustomAccount::instantiate_global("account_name");
    let mut account = CustomAccount::from(new_account_address);

    let empty_bucket = Bucket::new(ResourceAddress::from_str("resource_sim1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzqu57yag").unwrap());
    account.deposit(empty_bucket);
}

fn bridge_to_existing_account() {
    let existing_account = CustomAccount::from(ComponentAddress::from_str("account_sim1qdencrktc8r4f2ek5qh98uvfn7ugyjlpsw6ayusqyx6syqd3ly").unwrap());
    let balance = existing_account.read_balance();
    // ...
}
```

# Related

- Replaces the import! macro for importing an abi, using a more concise, readable syntax.
- Similar to the [external_component]! macro, which is used for making cross-component calls to an already-instantiated component.

*/
#[macro_export]
macro_rules! external_blueprint {
    (
        $blueprint_context:tt,
        $blueprint_ident:ident {
            $($blueprint_contents:tt)*
        }
    ) => {

        #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
        struct $blueprint_ident {
            component_address: ::scrypto::component::ComponentAddress,
        }

        // We allow dead code because it's used for importing interfaces, and not all the interface might be used
        #[allow(dead_code, unused_imports)]
        impl $blueprint_ident {
            ::scrypto::external_interface_members!(
                $blueprint_context,
                $($blueprint_contents)*
            );
        }

        impl From<::scrypto::component::ComponentAddress> for $blueprint_ident {
            fn from(component_address: ::scrypto::component::ComponentAddress) -> Self {
                Self {
                    component_address
                }
            }
        }

        impl From<$blueprint_ident> for ::scrypto::component::ComponentAddress {
            fn from(a: $blueprint_ident) -> ::scrypto::component::ComponentAddress {
                a.component_address
            }
        }
    };
}

/**
Generates a bridge/stub to make cross-component calls.

# Examples
```
use scrypto::prelude::*;
use sbor::{TypeId, Encode, Decode, Describe};

#[derive(TypeId, Encode, Decode, Describe)]
enum DepositResult {
    Success,
    Failure
}

external_component! {
    AccountInterface {
        fn deposit(&mut self, b: Bucket) -> DepositResult;
        fn deposit_no_return(&mut self, b: Bucket);
        fn read_balance(&self) -> Decimal;
    }
}

fn bridge_to_existing_account() {
    let existing_account = AccountInterface::from(ComponentAddress::from_str("account_sim1q02r73u7nv47h80e30pc3q6ylsj7mgvparm3pnsm780qgsy064").unwrap());
    let balance = existing_account.read_balance();
    // ...
}
```

# Related

- Similar to the [external_blueprint] macro, but the external_component can be used without knowing the package and blueprint addresses.

*/
#[macro_export]
macro_rules! external_component {
    (
        $component_ident:ident {
            $($component_methods:tt)*
        }
    ) => {

        #[derive(::sbor::TypeId, ::sbor::Encode, ::sbor::Decode, ::sbor::Describe)]
        struct $component_ident {
            component_address: ::scrypto::component::ComponentAddress,
        }

        // We allow dead code because it's used for importing interfaces, and not all the interface might be used
        #[allow(dead_code, unused_imports)]
        impl $component_ident {
            ::scrypto::external_interface_members!((), $($component_methods)*);
        }

        impl From<::scrypto::component::ComponentAddress> for $component_ident {
            fn from(component_address: ::scrypto::component::ComponentAddress) -> Self {
                Self {
                    component_address
                }
            }
        }

        impl From<$component_ident> for ::scrypto::component::ComponentAddress {
            fn from(a: $component_ident) -> ::scrypto::component::ComponentAddress {
                a.component_address
            }
        }
    };
}

// This is a TT-Muncher, a useful guide for this type of use case is here: https://adventures.michaelfbryan.com/posts/non-trivial-macros/
#[macro_export]
macro_rules! external_interface_members {
    (
        $blueprint_context:tt,
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        pub fn $method_name(&self $(, $method_args: $method_types)*) -> $method_output {
            ::scrypto::core::Runtime::call_method(
                self.component_address,
                stringify!($method_name),
                ::scrypto::args!($($method_args),*)
            )
        }
        ::scrypto::external_interface_members!($blueprint_context, $($rest)*);
    };
    (
        $blueprint_context:tt,
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        pub fn $method_name(&self $(, $method_args: $method_types)*) {
            ::scrypto::core::Runtime::call_method(
                self.component_address,
                stringify!($method_name),
                ::scrypto::args!($($method_args),*)
            )
        }
        ::scrypto::external_interface_members!($blueprint_context, $($rest)*);
    };
    (
        $blueprint_context:tt,
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        pub fn $method_name(&mut self $(, $method_args: $method_types)*) -> $method_output {
            ::scrypto::core::Runtime::call_method(
                self.component_address,
                stringify!($method_name),
                ::scrypto::args!($($method_args),*)
            )
        }
        ::scrypto::external_interface_members!($blueprint_context, $($rest)*);
    };
    (
        $blueprint_context:tt,
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        pub fn $method_name(&mut self $(, $method_args: $method_types)*) {
            ::scrypto::core::Runtime::call_method(
                self.component_address,
                stringify!($method_name),
                ::scrypto::args!($($method_args),*)
            )
        }
        ::scrypto::external_interface_members!($blueprint_context, $($rest)*);
    };
    (
        $blueprint_context:tt,
        fn $method_name:ident(self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        compile_error!("Components cannot define methods taking self. Did you mean &self or &mut self instead?");
    };
    (
        $blueprint_context:tt,
        fn $method_name:ident(self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        compile_error!("Components cannot define methods taking self. Did you mean &self or &mut self instead?");
    };
    (
        $blueprint_context:tt,
        fn $func_name:ident($($func_args:ident: $func_types:ty),*) -> $func_output:ty;
        $($rest:tt)*
    ) => {
        pub fn $func_name($($func_args: $func_types),*) -> $func_output {
            ::scrypto::core::Runtime::call_function(
                ::scrypto::component::PackageAddress::from_str(::scrypto::package_address_from_context!($blueprint_context)).unwrap(),
                ::scrypto::blueprint_name_from_context!($blueprint_context),
                stringify!($func_name),
                ::scrypto::args!($($func_args),*)
            )
        }
        ::scrypto::external_interface_members!($blueprint_context, $($rest)*);
    };
    (
        $blueprint_context:tt,
        fn $func_name:ident($($func_args:ident: $func_types:ty),*);
        $($rest:tt)*
    ) => {
        pub fn $func_name($($func_args: $func_types),*) {
            use ::scrypto::rust::str::FromStr;
            ::scrypto::core::Runtime::call_function(
                ::scrypto::component::PackageAddress::from_str(::scrypto::package_address_from_context!($blueprint_context)).unwrap(),
                ::scrypto::blueprint_name_from_context!($blueprint_context),
                stringify!($func_name),
                ::scrypto::args!($($func_args),*)
            )
        }
        ::scrypto::external_interface_members!($blueprint_context, $($rest)*);
    };
    (
        $blueprint_context:tt,
    ) => {}
}

#[macro_export]
macro_rules! package_address_from_context {
    (
        {
            package: $package_address:literal,
            blueprint: $blueprint_logical_name:literal $(,)?
        }
    ) => {
        $package_address
    };
    // This macro is only called when used for blueprint function calls (instead of component method calls)
    //    () is the context passed into the external_interface_members macro by the external_component macro,
    //    which doesn't support function calls. So when we detect a () parameter, then we assume we're
    //    being called from the external_component macro, and can give a more useful error message.
    () => {
        compile_error!("Cannot call package functions (ie without &self or &mut self) on a external_component - use a external_blueprint instead.");
    };
    (
        $blueprint_context:tt
    ) => {
        compile_error!("Unknown blueprint context - use { package: \"<PACKAGE_HEX_ADDRESS>\", blueprint: \"<BLUEPRINT_LOGICAL_NAME>\" }")
    }
}

#[macro_export]
macro_rules! blueprint_name_from_context {
    (
        {
            package: $package_address:literal,
            blueprint: $blueprint_logical_name:literal $(,)?
        }
    ) => {
        $blueprint_logical_name
    };
    // This macro is only called when used for blueprint function calls (instead of component method calls)
    //    () is the context passed into the external_interface_members macro by the external_component macro,
    //    which doesn't support function calls. So when we detect a () parameter, then we assume we're
    //    being called from the external_component macro, and can give a more useful error message.
    () => {
        compile_error!("Cannot call package functions (ie without &self or &mut self) on a external_component - use a external_blueprint instead.");
    };
    (
        $blueprint_context:tt
    ) => {
        compile_error!("Unknown blueprint context - use { package: \"<PACKAGE_HEX_ADDRESS>\", blueprint: \"<BLUEPRINT_LOGICAL_NAME>\" }")
    }
}
