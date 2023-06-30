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
        $crate::runtime::Logger::emit_log($crate::types::Level::Error, ::sbor::rust::format!($($args),+));
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
        $crate::runtime::Logger::emit_log($crate::types::Level::Warn, ::sbor::rust::format!($($args),+));
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
        $crate::runtime::Logger::emit_log($crate::types::Level::Info, ::sbor::rust::format!($($args),+));
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
        $crate::runtime::Logger::emit_log($crate::types::Level::Debug, ::sbor::rust::format!($($args),+));
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
        $crate::runtime::Logger::emit_log($crate::types::Level::Trace, ::sbor::rust::format!($($args),+));
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

// This is a TT-Muncher, a useful guide for this type of use case is here: https://adventures.michaelfbryan.com/posts/non-trivial-macros/
#[macro_export]
macro_rules! external_functions {
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
        $(#[$meta: meta])*
        fn $func_name:ident($($func_args:ident: $func_types:ty),*) -> $func_output:ty;
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        fn $func_name($($func_args: $func_types),*) -> $func_output {
            Self::call_function_raw(stringify!($func_name), scrypto_args!($($func_args),*))
        }

        $crate::external_functions!($($rest)*);
    };
    (
        $(#[$meta: meta])*
        fn $func_name:ident($($func_args:ident: $func_types:ty),*);
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        fn $func_name($($func_args: $func_types),*) {
            Self::call_function_raw(stringify!($func_name), scrypto_args!($($func_args),*))
        }

        $crate::external_functions!($($rest)*);
    };
    () => {
    };
}

// This is a TT-Muncher, a useful guide for this type of use case is here: https://adventures.michaelfbryan.com/posts/non-trivial-macros/
#[macro_export]
macro_rules! external_methods {
    (
        $(#[$meta: meta])*
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $method_name(&self $(, $method_args: $method_types)*) -> $method_output {
            self.call_raw(stringify!($method_name), scrypto_args!($($method_args),*))
        }
        $crate::external_methods!($($rest)*);
    };
    (
        $(#[$meta: meta])*
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $method_name(&self $(, $method_args: $method_types)*) {
            self.call_raw(stringify!($method_name), scrypto_args!($($method_args),*))
        }
        $crate::external_methods!($($rest)*);
    };
    (
        $(#[$meta: meta])*
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $method_name(&mut self $(, $method_args: $method_types)*) -> $method_output {
            self.call_raw(stringify!($method_name), scrypto_args!($($method_args),*))
        }
        $crate::external_methods!($($rest)*);
    };
    (
        $(#[$meta: meta])*
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $method_name(&mut self $(, $method_args: $method_types)*) {
            self.call_raw(stringify!($method_name), scrypto_args!($($method_args),*))
        }
        $crate::external_methods!($($rest)*);
    };
    (
        $(#[$meta: meta])*
        fn $method_name:ident(self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        compile_error!("Components cannot define methods taking self. Did you mean &self or &mut self instead?");
    };
    (
        $(#[$meta: meta])*
        fn $method_name:ident(self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        compile_error!("Components cannot define methods taking self. Did you mean &self or &mut self instead?");
    };
    (
        $(#[$meta: meta])*
        fn $method_name:ident($($method_args:ident: $method_types:ty),*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        compile_error!("The external_component! macro cannot be used to define static blueprint methods which don't take &self or &mut self. For these package methods, use a separate external_blueprint! macro.");
    };
    (
        $(#[$meta: meta])*
        fn $method_name:ident($($method_args:ident: $method_types:ty),*);
        $($rest:tt)*
    ) => {
        compile_error!("The external_component! macro cannot be used to define static blueprint methods which don't take &self or &mut self. For these package methods, use a separate external_blueprint! macro.");
    };
    () => {}
}

#[macro_export]
macro_rules! extern_blueprint_internal {
    (
        $package_address:expr, $blueprint:ident, $blueprint_name:expr, $owned_type_name:expr, $global_type_name: expr, $functions:ident {
            $($function_contents:tt)*
        }, {
            $($method_contents:tt)*
        }
    ) => {
        #[derive(Copy, Clone)]
        pub struct $blueprint {
            pub handle: ::scrypto::component::ObjectStubHandle,
        }

        impl HasTypeInfo for $blueprint {
            const PACKAGE_ADDRESS: Option<PackageAddress> = Some($package_address);
            const BLUEPRINT_NAME: &'static str = $blueprint_name;
            const OWNED_TYPE_NAME: &'static str = $owned_type_name;
            const GLOBAL_TYPE_NAME: &'static str = $global_type_name;
        }

        pub trait $functions {
            $($function_contents)*
        }

        impl $functions for ::scrypto::component::Blueprint<$blueprint> {
            $crate::external_functions!($($function_contents)*);
        }

        impl ::scrypto::component::ObjectStub for $blueprint {
            type AddressType = ComponentAddress;

            fn new(handle: ::scrypto::component::ObjectStubHandle) -> Self {
                Self {
                    handle
                }
            }
            fn handle(&self) -> &::scrypto::component::ObjectStubHandle {
                &self.handle
            }
        }

        impl HasStub for $blueprint {
            type Stub = $blueprint;
        }

        // We allow dead code because it's used for importing interfaces, and not all the interface might be used
        #[allow(dead_code, unused_imports)]
        impl $blueprint {
            $crate::external_methods!($($method_contents)*);
        }
    };
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
macro_rules! role_list {
    () => ({
        RoleList::none()
    });
    ($($role:ident),*) => ({
        let mut list = RoleList::none();
        $(
            list.insert(to_role_key!($role));
        )*
        list
    });
}

#[macro_export]
macro_rules! method_accessibility {
    (PUBLIC) => ({
        MethodAccessibility::Public
    });
    (NOBODY) => ({
        [].into()
    });
    (restrict_to: [$($roles:ident),+]) => ({
        let list = role_list!($($roles),+);
        MethodAccessibility::RoleProtected(list)
    });
}

#[macro_export]
macro_rules! method_accessibilities {
    ($module_methods:ident, $($method:ident => $accessibility:ident $(: [$($allow_role:ident),+])?;)*) => ({
        $module_methods::<MethodAccessibility> {
            $(
                $method: method_accessibility!($accessibility $(: [$($allow_role),+])?),
            )*
        }
    })
}

#[macro_export]
macro_rules! main_accessibility {
    ($permissions:expr, $module_methods:ident, $($method:ident => $accessibility:ident $(: [$($allow_role:ident),+])?;)*) => ({
        let permissions = method_accessibilities!(
            $module_methods,
            $($method => $accessibility $(: [$($allow_role),+])?;)*
        );
        for (method, permission) in permissions.to_mapping() {
            $permissions.insert(MethodKey::new(method), permission);
        }
    })
}

#[macro_export]
macro_rules! internal_add_role {
    ($roles:ident, $role:ident => updatable_by: [$($updaters:ident),*]) => {{
        let updaters = role_list!($($updaters),*);
        $roles.insert(stringify!($role).into(), updaters);
    }};
}

#[macro_export]
macro_rules! enable_method_auth {
    (
        roles {
            $($role:ident => updatable_by: [$($updaters:ident),*];)*
        },
        methods {
            $($method:ident => $accessibility:ident $(: [$($allow_role:ident),+])?;)*
        }
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

        fn method_auth_template() -> scrypto::blueprints::package::MethodAuthTemplate {
            let mut methods: BTreeMap<MethodKey, MethodAccessibility> = BTreeMap::new();
            main_accessibility!(
                methods,
                Methods,
                $($method => $accessibility $(: [$($allow_role),+])?;)*
            );

            let mut roles: BTreeMap<RoleKey, RoleList> = BTreeMap::new();
            $(
                internal_add_role!(roles, $role => updatable_by: [$($updaters),*]);
            )*

            let static_roles = scrypto::blueprints::package::StaticRoles {
                methods,
                roles: scrypto::blueprints::package::RoleSpecification::Normal(roles),
            };

            scrypto::blueprints::package::MethodAuthTemplate::StaticRoles(static_roles)
        }
    );

    (
        methods {
            $($method:ident => $accessibility:ident $(: [$($allow_role:ident),+])?;)*
        }
    ) => (
        fn method_auth_template() -> scrypto::blueprints::package::MethodAuthTemplate {
            let mut methods: BTreeMap<MethodKey, MethodAccessibility> = BTreeMap::new();
            main_accessibility!(
                methods,
                Methods,
                $($method => $accessibility $(: [$($allow_role),+])?;)*
            );

            let roles = scrypto::blueprints::package::StaticRoles {
                methods,
                roles: scrypto::blueprints::package::RoleSpecification::Normal(BTreeMap::new()),
            };

            scrypto::blueprints::package::MethodAuthTemplate::StaticRoles(roles)
        }
    );
}

#[macro_export]
macro_rules! enable_function_auth {
    (
        $($function:ident => $rule:expr;)*
    ) => (
        fn function_auth() -> scrypto::blueprints::package::FunctionAuth {
            let rules = Functions::<AccessRule> {
                $( $function: $rule, )*
            };

            scrypto::blueprints::package::FunctionAuth::AccessRules(rules.to_mapping().into_iter().collect())
        }
    );
}

#[macro_export]
macro_rules! enable_package_royalties {
    ($($function:ident => $royalty:expr;)*) => (
        fn package_royalty_config() -> PackageRoyaltyConfig {
            let royalties = Fns::<RoyaltyAmount> {
                $( $function: $royalty, )*
            };

            PackageRoyaltyConfig::Enabled(royalties.to_mapping().into_iter().collect())
        }
    );
}

#[macro_export]
macro_rules! component_royalties {
    {
        roles {
            $($role:ident => $rule:expr $(, $updatable:ident)?;)*
        },
        init {
            $($init:tt)*
        }
    } => ({
        let royalty_roles = internal_roles!(RoyaltyRoles, $($role => $rule $(, $updatable)?;)*);
        let royalties = component_royalty_config!($($init)*);
        (royalties, royalty_roles)
    });
    {
        init {
            $($init:tt)*
        }
    } => ({
        let royalties = component_royalty_config!($($init)*);
        (royalties, RolesInit::new())
    })
}

/// Roles macro for main module
#[macro_export]
macro_rules! roles {
    ( $($role:ident => $rule:expr, $locked:ident;)* ) => ({
        internal_roles!(MethodRoles, $($role => $rule, $locked;)*)
    });
}

#[macro_export]
macro_rules! component_royalty_config {
    ($($method:ident => $royalty:expr, $locked:ident;)*) => ({
        Methods::<(RoyaltyAmount, bool)> {
            $(
                $method: internal_component_royalty_entry!($royalty, $locked),
            )*
        }
    });
}

#[macro_export]
macro_rules! internal_component_royalty_entry {
    ($royalty:expr, locked) => {{
        ($royalty.into(), false)
    }};
    ($royalty:expr, updatable) => {{
        ($royalty.into(), true)
    }};
}
