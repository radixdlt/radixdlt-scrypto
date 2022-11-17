/// Creates a `Decimal` from literals.
///
/// # Example
/// ```no_run
/// use scrypto::prelude::*;
///
/// let a = dec!(1);
/// let b = dec!("1.1");
/// ```
#[macro_export]
macro_rules! dec {
    ($x:literal) => {
        scrypto::math::Decimal::from($x)
    };

    ($base:literal, $shift:literal) => {
        // Base can be any type that converts into a Decimal, and shift must support
        // comparison and `-` unary operation, enforced by rustc.
        {
            let base = scrypto::math::Decimal::from($base);
            if $shift >= 0 {
                base * scrypto::math::Decimal::try_from(
                    scrypto::math::I256::from(10u8)
                        .pow(u32::try_from($shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            } else {
                base / scrypto::math::Decimal::try_from(
                    scrypto::math::I256::from(10u8)
                        .pow(u32::try_from(-$shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            }
        }
    };
}

/// Creates a safe integer from literals.
/// You must specify the type of the
/// integer you want to create.
///
/// # Example
/// ```no_run
/// use scrypto::prelude::*;
///
/// let a: I256 = i!(21);
/// let b: U512 = i!("1156");
/// ```
#[macro_export]
macro_rules! i {
    ($x:expr) => {
        $x.try_into().expect("Parse Error")
    };
}

/// Creates a `PreciseDecimal` from literals.
///
/// # Example
/// ```no_run
/// use scrypto::prelude::*;
///
/// let a = pdec!(1);
/// let b = pdec!("1.1");
/// ```
#[macro_export]
macro_rules! pdec {
    ($x:literal) => {
        scrypto::math::PreciseDecimal::from($x)
    };

    ($base:literal, $shift:literal) => {
        // Base can be any type that converts into a PreciseDecimal, and shift must support
        // comparison and `-` unary operation, enforced by rustc.
        {
            let base = scrypto::math::PreciseDecimal::from($base);
            if $shift >= 0 {
                base * scrypto::math::PreciseDecimal::try_from(
                    scrypto::math::I512::from(10u8)
                        .pow(u32::try_from($shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            } else {
                base / scrypto::math::PreciseDecimal::try_from(
                    scrypto::math::I512::from(10u8)
                        .pow(u32::try_from(-$shift).expect("Shift overflow")),
                )
                .expect("Shift overflow")
            }
        }
    };
}

/// Constructs argument list for Scrypto function/method invocation.
///
/// # Example
/// ```no_run
/// use scrypto::prelude::*;
///
/// let args = args!("1.1", 100u32);
/// ```
#[macro_export]
macro_rules! args {
    ($($args: expr),*) => {{
        let mut fields = Vec::new();
        $(
            let encoded = ::scrypto::buffer::scrypto_encode(&$args);
            fields.push(::sbor::decode_any::<::scrypto::data::ScryptoCustomTypeId, ::scrypto::data::ScryptoCustomValue>(&encoded).unwrap());
        )*
        let input_struct = ::sbor::SborValue::Struct {
            fields,
        };
        ::sbor::encode_any::<::scrypto::data::ScryptoCustomTypeId, ::scrypto::data::ScryptoCustomValue>(&input_struct)
    }};
}

#[macro_export]
macro_rules! args_from_value_vec {
    ($args: expr) => {{
        let input_struct = ::sbor::SborValue::Struct { fields: $args };
        ::sbor::encode_any(&input_struct)
    }};
}

#[macro_export]
macro_rules! args_from_bytes_vec {
    ($args: expr) => {{
        let mut fields = Vec::new();
        for arg in $args {
            fields.push(
                ::sbor::decode_any::<
                    ::scrypto::data::ScryptoCustomTypeId,
                    ::scrypto::data::ScryptoCustomValue,
                >(&arg)
                .unwrap(),
            );
        }
        let input_struct = ::sbor::SborValue::Struct { fields };
        ::sbor::encode_any(&input_struct)
    }};
}

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
        ::scrypto::core::Logger::log(scrypto::engine_lib::engine::types::Level::Error, ::sbor::rust::format!($($args),+));
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
        ::scrypto::core::Logger::log(scrypto::engine_lib::engine::types::Level::Warn, ::sbor::rust::format!($($args),+));
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
        ::scrypto::core::Logger::log(scrypto::engine_lib::engine::types::Level::Info, ::sbor::rust::format!($($args),+));
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
        ::scrypto::core::Logger::log(scrypto::engine_lib::engine::types::Level::Debug, ::sbor::rust::format!($($args),+));
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
        ::scrypto::core::Logger::log(scrypto::engine_lib::engine::types::Level::Trace, ::sbor::rust::format!($($args),+));
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
/// If you just wish to instead make calls to an instantiated component, see the [external_component]! macro.
///
/// # Examples
/// ```no_run
/// use radix_engine_lib::address::Bech32Decoder;
/// use scrypto::prelude::*;
///
/// external_blueprint! {
///     CustomAccountBlueprint {
///         fn instantiate_global(account_name: &str) -> ComponentAddress;
///     }
/// }
///
/// #[derive(TypeId, Encode, Decode, Describe)]
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
/// - Similar to the [external_component]! macro, which is used for making cross-component calls to an already-instantiated component.
#[macro_export]
macro_rules! external_blueprint {
    (
        $blueprint_ident:ident {
            $($blueprint_contents:tt)*
        }
    ) => {

        #[scrypto(TypeId, Encode, Decode, Describe)]
        struct $blueprint_ident {
            package_address: ::scrypto::engine_lib::component::PackageAddress,
            blueprint_name: ::sbor::rust::string::String,
        }

        // We allow dead code because it's used for importing interfaces, and not all the interface might be used
        #[allow(dead_code, unused_imports)]
        impl $blueprint_ident {
            fn at<S>(package_address: ::scrypto::engine_lib::component::PackageAddress, blueprint_name: S) -> Self
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

        impl From<$blueprint_ident> for ::scrypto::engine_lib::component::PackageAddress {
            fn from(a: $blueprint_ident) -> ::scrypto::engine_lib::component::PackageAddress {
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
            ::scrypto::core::Runtime::call_function(
                self.package_address,
                &self.blueprint_name,
                stringify!($func_name),
                ::scrypto::args!($($func_args),*)
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
            ::scrypto::core::Runtime::call_function(
                self.package_address,
                &self.blueprint_name,
                stringify!($func_name),
                ::scrypto::args!($($func_args),*)
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
/// #[derive(TypeId, Encode, Decode, Describe)]
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
        #[scrypto(TypeId, Encode, Decode, Describe)]
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
            ::scrypto::core::Runtime::call_method(
                self.component_address,
                stringify!($method_name),
                ::scrypto::args!($($method_args),*)
            )
        }
        ::scrypto::external_component_members!($($rest)*);
    };
    (
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
        ::scrypto::external_component_members!($($rest)*);
    };
    (
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
        ::scrypto::external_component_members!($($rest)*);
    };
    (
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

/// A macro for implementing sbor traits.
#[macro_export]
macro_rules! scrypto_type {
    // static size
    ($t:ty, $type_id:expr, $schema_type: expr, $size: expr) => {
        impl TypeId<::scrypto::data::ScryptoCustomTypeId> for $t {
            #[inline]
            fn type_id() -> SborTypeId<::scrypto::data::ScryptoCustomTypeId> {
                SborTypeId::Custom($type_id)
            }
        }

        impl Encode<::scrypto::data::ScryptoCustomTypeId> for $t {
            #[inline]
            fn encode_type_id(encoder: &mut Encoder<::scrypto::data::ScryptoCustomTypeId>) {
                encoder.write_type_id(Self::type_id());
            }
            #[inline]
            fn encode_value(&self, encoder: &mut Encoder<::scrypto::data::ScryptoCustomTypeId>) {
                encoder.write_slice(&self.to_vec());
            }
        }

        impl Decode<::scrypto::data::ScryptoCustomTypeId> for $t {
            fn check_type_id(
                decoder: &mut Decoder<::scrypto::data::ScryptoCustomTypeId>,
            ) -> Result<(), DecodeError> {
                decoder.check_type_id(Self::type_id())
            }

            fn decode_value(
                decoder: &mut Decoder<::scrypto::data::ScryptoCustomTypeId>,
            ) -> Result<Self, DecodeError> {
                let slice = decoder.read_slice($size)?;
                Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomValue)
            }
        }

        impl Describe for $t {
            fn describe() -> Type {
                $schema_type
            }
        }
    };

    // dynamic size
    ($t:ty, $type_id:expr, $schema_type: expr) => {
        impl TypeId<::scrypto::data::ScryptoCustomTypeId> for $t {
            #[inline]
            fn type_id() -> SborTypeId<::scrypto::data::ScryptoCustomTypeId> {
                SborTypeId::Custom($type_id)
            }
        }

        impl Encode<::scrypto::data::ScryptoCustomTypeId> for $t {
            #[inline]
            fn encode_type_id(encoder: &mut Encoder<::scrypto::data::ScryptoCustomTypeId>) {
                encoder.write_type_id(Self::type_id());
            }
            #[inline]
            fn encode_value(&self, encoder: &mut Encoder<::scrypto::data::ScryptoCustomTypeId>) {
                let bytes = self.to_vec();
                encoder.write_size(bytes.len());
                encoder.write_slice(&bytes);
            }
        }

        impl Decode<::scrypto::data::ScryptoCustomTypeId> for $t {
            fn check_type_id(
                decoder: &mut Decoder<::scrypto::data::ScryptoCustomTypeId>,
            ) -> Result<(), DecodeError> {
                decoder.check_type_id(Self::type_id())
            }

            fn decode_value(
                decoder: &mut Decoder<::scrypto::data::ScryptoCustomTypeId>,
            ) -> Result<Self, DecodeError> {
                let len = decoder.read_size()?;
                let slice = decoder.read_slice(len)?;
                Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomValue)
            }
        }

        impl Describe for $t {
            fn describe() -> Type {
                $schema_type
            }
        }
    };
}
