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
        $crate::runtime::Logger::log_message($crate::types::Level::Error, ::sbor::rust::format!($($args),+));
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
        $crate::runtime::Logger::log_message($crate::types::Level::Warn, ::sbor::rust::format!($($args),+));
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
        $crate::runtime::Logger::log_message($crate::types::Level::Info, ::sbor::rust::format!($($args),+));
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
        $crate::runtime::Logger::log_message($crate::types::Level::Debug, ::sbor::rust::format!($($args),+));
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
        $crate::runtime::Logger::log_message($crate::types::Level::Trace, ::sbor::rust::format!($($args),+));
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

/// Includes the schema file of a Scrypto package.
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
/// let schema1 = include_schema!("bin_name");
///
/// // Another package
/// let schema2 = include_schema!("/path/to/package", "bin_name");
/// ```
#[macro_export]
macro_rules! include_schema {
    ($bin_name: expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/target/wasm32-unknown-unknown/release/",
            $bin_name,
            ".schema"
        ))
    };
    ($package_dir: expr, $bin_name: expr) => {
        include_bytes!(concat!(
            $package_dir,
            "/target/wasm32-unknown-unknown/release/",
            $bin_name,
            ".schema"
        ))
    };
}

/// Generates a bridge/stub to make package calls to a blueprint.
///
/// If you just wish to instead make calls to an instantiated component, see the `external_component` macro.
///
/// # Examples
/// ```no_run
/// use scrypto::address::Bech32Decoder;
/// use scrypto::prelude::*;
///
/// external_blueprint! {
///     CustomAccountBlueprint {
///         fn instantiate_global(account_name: &str) -> ComponentAddress;
///     }
/// }
///
/// #[derive(Sbor)]
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
///     let package_address = PackageAddress::try_from_bech32(
///         &Bech32Decoder::for_simulator(),
///         "package_sim1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsnznk7n"
///     ).unwrap();
///     let blueprint = CustomAccountBlueprint::at(package_address, "CustomAccount");
///     blueprint.instantiate_global("account_name")
/// }
///
/// ```
///
/// # Related
///
/// - Replaces the import! macro for importing an schema, using a more concise, readable syntax.
/// - Similar to the `external_component` macro, which is used for making cross-component calls to an already-instantiated component.
#[macro_export]
macro_rules! external_blueprint {
    (
        $blueprint_ident:ident {
            $($blueprint_contents:tt)*
        }
    ) => {

        #[derive(ScryptoSbor)]
        struct $blueprint_ident {
            package_address: $crate::types::PackageAddress,
            blueprint_name: ::sbor::rust::string::String,
        }

        // We allow dead code because it's used for importing interfaces, and not all the interface might be used
        #[allow(dead_code, unused_imports)]
        impl $blueprint_ident {
            fn at<S>(package_address: $crate::types::PackageAddress, blueprint_name: S) -> Self
            where
                S: Into<::sbor::rust::string::String>
            {
                Self {
                    package_address,
                    blueprint_name: blueprint_name.into(),
                }
            }

            $crate::external_blueprint_members!(
                $($blueprint_contents)*
            );
        }

        impl From<$blueprint_ident> for $crate::types::PackageAddress {
            fn from(a: $blueprint_ident) -> $crate::types::PackageAddress {
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
            $crate::runtime::Runtime::call_function(
                self.package_address,
                &self.blueprint_name,
                stringify!($func_name),
                scrypto_args!($($func_args),*)
            )
        }
        $crate::external_blueprint_members!($($rest)*);
    };
    (
        fn $func_name:ident($($func_args:ident: $func_types:ty),*);
        $($rest:tt)*
    ) => {
        pub fn $func_name(&self, $($func_args: $func_types),*) {
            use $crate::rust::str::FromStr;
            $crate::runtime::Runtime::call_function(
                self.package_address,
                &self.blueprint_name,
                stringify!($func_name),
                scrypto_args!($($func_args),*)
            )
        }
        $crate::external_blueprint_members!($($rest)*);
    };
    () => {}
}

/// Generates a bridge/stub to make cross-component calls.
///
/// # Examples
/// ```no_run
/// use scrypto::prelude::*;
///
/// #[derive(Sbor)]
/// enum DepositResult {
///     Success,
///     Failure
/// }
///
/// external_component! {
///     Account {
///         fn deposit(&mut self, b: Bucket) -> DepositResult;
///         fn deposit_no_return(&mut self, b: Bucket);
///         fn read_balance(&self) -> Decimal;
///     }
/// }
///
/// fn bridge_to_existing_account(component_address: ComponentAddress) {
///     let existing_account: Global<Account> = component_address.into();
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
        #[derive(Copy, Clone)]
        struct $component_ident {
            pub handle: ::scrypto::component::ObjectStubHandle,
        }

        impl ::scrypto::component::ObjectStub for $component_ident {
            fn new(handle: ::scrypto::component::ObjectStubHandle) -> Self {
                Self {
                    handle
                }
            }
            fn handle(&self) -> &::scrypto::component::ObjectStubHandle {
                &self.handle
            }
        }

        impl HasStub for $component_ident {
            type Stub = $component_ident;
        }

        // We allow dead code because it's used for importing interfaces, and not all the interface might be used
        #[allow(dead_code, unused_imports)]
        impl $component_ident {
            $crate::external_component_members!($($component_methods)*);
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
            self.call_raw(stringify!($method_name), scrypto_args!($($method_args),*))
        }
        $crate::external_component_members!($($rest)*);
    };
    (
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        pub fn $method_name(&self $(, $method_args: $method_types)*) {
            self.call_raw(stringify!($method_name), scrypto_args!($($method_args),*))
        }
        $crate::external_component_members!($($rest)*);
    };
    (
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        pub fn $method_name(&mut self $(, $method_args: $method_types)*) -> $method_output {
            self.call_raw(stringify!($method_name), scrypto_args!($($method_args),*))
        }
        $crate::external_component_members!($($rest)*);
    };
    (
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        pub fn $method_name(&mut self $(, $method_args: $method_types)*) {
            self.call_raw(stringify!($method_name), scrypto_args!($($method_args),*))
        }
        $crate::external_component_members!($($rest)*);
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

#[macro_export]
macro_rules! to_role_key {
    (OWNER) => {{
        OWNER_ROLE
    }};
    (SELF) => {{
        SELF_ROLE
    }};
    ($role:ident) => {{
        ROLE_STRINGS.$role
    }};
}

#[macro_export]
macro_rules! permission_role_list {
    ($list:ident, $role:ident) => ({
        $list.insert(to_role_key!($role));
    });
    ($list:ident, $role:ident, $($roles:ident),*) => ({
        $list.insert(to_role_key!($role));
        permission_role_list!($list, $($roles),*);
    });
}

#[macro_export]
macro_rules! method_permission {
    (PUBLIC) => ({
        MethodPermission::Public
    });
    (NOBODY) => ({
        [].into()
    });
    ($($roles:ident),+) => ({
        let mut list = RoleList::none();
        permission_role_list!(list, $($roles),+);
        MethodPermission::Protected(list)
    });
}

#[macro_export]
macro_rules! method_permissions {
    ($module_methods:ident, $($method:ident => $($permission:ident),+ ;)*) => ({
        $module_methods::<MethodPermission> {
            $(
                $method: method_permission!($($permission),+),
            )*
        }
    })
}

#[macro_export]
macro_rules! main_permissions {
    ($permissions:expr, $module_methods:ident, $key:ident, $($method:ident => $($permission:ident),+ ;)*) => ({
        let permissions = method_permissions!($module_methods, $($method => $($permission),+ ;)*);
        for (method, permission) in permissions.to_mapping() {
            let permission = match permission {
                MethodPermission::Public => scrypto::schema::SchemaMethodPermission::Public,
                MethodPermission::Protected(role_list) => scrypto::schema::SchemaMethodPermission::Protected(role_list.to_list()),
            };
            $permissions.insert(scrypto::schema::SchemaMethodKey::$key(method), permission);
        }
    })
}

#[macro_export]
macro_rules! module_permissions {
    ($permissions:expr, methods { $($method:ident => $($permission:ident),+ ;)* }) => ({
        main_permissions!($permissions, Methods, main, $($method => $($permission),+ ;)*);
    });
    ($permissions:expr, metadata { $($method:ident => $($permission:ident),+ ;)* }) => ({
        main_permissions!($permissions, MetadataMethods, metadata, $($method => $($permission),+ ;)*);
    });
    ($permissions:expr, royalties { $($method:ident => $($permission:ident),+ ;)* }) => ({
        main_permissions!($permissions, RoyaltyMethods, royalty, $($method => $($permission),+ ;)*);
    });
}

#[macro_export]
macro_rules! define_static_auth {
    (
        roles {
            $($role:ident),*
        },
        $($module:ident { $($method:ident => $($permission:ident),+ ;)* }),*
    ) => (
        pub struct MethodRoles<T> {
            $($role: T),*
        }

        impl<T> MethodRoles<T> {
            fn list(self) -> Vec<(&'static str, T)> {
                vec![
                    $((stringify!($role), self.$role)),*
                ]
            }
        }

        const ROLE_STRINGS: MethodRoles<&str> = MethodRoles {
            $($role: stringify!($role)),*
        };

        fn method_auth_template() -> BTreeMap<scrypto::schema::SchemaMethodKey, scrypto::schema::SchemaMethodPermission> {
            let mut permissions: BTreeMap<scrypto::schema::SchemaMethodKey, scrypto::schema::SchemaMethodPermission> = BTreeMap::new();
            $(
                module_permissions!(permissions, $module { $($method => $($permission),+ ;)* });
            )*
            permissions
        }
    );

    (
        $($module:ident { $($method:ident => $($permission:ident),+ ;)* }),*
    ) => (
        fn method_auth_template() -> BTreeMap<scrypto::schema::SchemaMethodKey, scrypto::schema::SchemaMethodPermission> {
            let mut permissions: BTreeMap<scrypto::schema::SchemaMethodKey, scrypto::schema::SchemaMethodPermission> = BTreeMap::new();
            $(
                module_permissions!(permissions, $module { $($method => $($permission),+ ;)* });
            )*
            permissions
        }
    );
}

#[macro_export]
macro_rules! role_definition_entry {

    ($rule:expr, mutable_by: $($mutators:ident),+) => {{
        let mut list = RoleList::none();
        permission_role_list!(list, $($mutators),+);
        RoleEntry::new($rule, list, true)
    }};
    ($rule:expr) => {{
        RoleEntry::immutable($rule)
    }};
}

#[macro_export]
macro_rules! roles {
    ( $($role:ident => $rule:expr $(, mutable_by: $($mutators:ident),+)? ;)* ) => ({
        let method_roles = MethodRoles::<RoleEntry> {
            $($role: role_definition_entry!($rule $(, mutable_by: $($mutators),+)?)),*
        };

        let mut roles = $crate::blueprints::resource::Roles::new();
        for (name, entry) in method_roles.list() {
            roles.define_role(name, entry);
        }

        roles
    });
}

#[macro_export]
macro_rules! royalties {
    ($($method:ident => $royalty:expr),*) => ({
        Methods::<MethodRoyalty> {
            $(
                $method: $royalty.into(),
            )*
        }
    });
    ($($method:ident => $royalty:expr,)*) => ({
        royalties!($($method => $royalty),*)
    });
}

#[macro_export]
macro_rules! metadata {
    ( ) => ({
        ::scrypto::prelude::Metadata::new()
    });
    ( $($key:expr => $value:expr),* ) => ({
        let mut metadata = ::scrypto::prelude::Metadata::new();
        $(
            metadata.set($key, $value);
        )*
        metadata
    });
    ( $($key:expr => $value:expr,)* ) => ({
        metadata!{$($key => $value),*}
    });
}
