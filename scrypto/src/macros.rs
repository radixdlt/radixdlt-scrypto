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
        fn $func_name:ident($($func_args:ident: $func_types:ty),*) -> $func_output:ty;
        $($rest:tt)*
    ) => {
        fn $func_name($($func_args: $func_types),*) -> $func_output {
            Self::call_function_raw(stringify!($func_name), scrypto_args!($($func_args),*))
        }

        $crate::external_functions!($($rest)*);
    };
    (
        fn $func_name:ident($($func_args:ident: $func_types:ty),*);
        $($rest:tt)*
    ) => {
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
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        pub fn $method_name(&self $(, $method_args: $method_types)*) -> $method_output {
            self.call_raw(stringify!($method_name), scrypto_args!($($method_args),*))
        }
        $crate::external_methods!($($rest)*);
    };
    (
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        pub fn $method_name(&self $(, $method_args: $method_types)*) {
            self.call_raw(stringify!($method_name), scrypto_args!($($method_args),*))
        }
        $crate::external_methods!($($rest)*);
    };
    (
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)*) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        pub fn $method_name(&mut self $(, $method_args: $method_types)*) -> $method_output {
            self.call_raw(stringify!($method_name), scrypto_args!($($method_args),*))
        }
        $crate::external_methods!($($rest)*);
    };
    (
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)*);
        $($rest:tt)*
    ) => {
        pub fn $method_name(&mut self $(, $method_args: $method_types)*) {
            self.call_raw(stringify!($method_name), scrypto_args!($($method_args),*))
        }
        $crate::external_methods!($($rest)*);
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
    ($permissions:expr, $module_methods:ident, $($method:ident => $($permission:ident),+ ;)*) => ({
        let permissions = method_permissions!($module_methods, $($method => $($permission),+ ;)*);
        for (method, permission) in permissions.to_mapping() {
            $permissions.insert(MethodKey::new(method), permission);
        }
    })
}

#[macro_export]
macro_rules! module_permissions {
    ($permissions:expr, methods { $($method:ident => $($permission:ident),+ ;)* }) => ({
        main_permissions!($permissions, Methods, $($method => $($permission),+ ;)*);
    });
}

#[macro_export]
macro_rules! internal_add_role {
    ($roles:ident, $role:ident) => {{
        $roles.insert(stringify!($role).into(), RoleList::none());
    }};
    ($roles:ident, $role:ident => updaters: $($updaters:ident),+) => {{
        let role_list = [
            $(
                ROLE_STRINGS.$updaters
            ),+
        ];
        $roles.insert(stringify!($role).into(), role_list.into());
    }};
}

#[macro_export]
macro_rules! internal_role_mutability {
    () => {{
        false
    }};
    (=> updaters: $($updaters:ident),+) => {{
        true
    }};
}

#[macro_export]
macro_rules! enable_method_auth {
    (
        roles {
            $($role:ident $( => updaters: $($updaters:ident),+)?;)*
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

        const ROLE_MUTABLE: MethodRoles<bool> = MethodRoles {
            $(
                $role: internal_role_mutability!($( => updaters: $($updaters),+)?)
            ),*
        };

        fn method_auth_template() -> scrypto::blueprints::package::MethodAuthTemplate {
            let mut methods: BTreeMap<MethodKey, MethodPermission> = BTreeMap::new();
            $(
                module_permissions!(methods, $module { $($method => $($permission),+ ;)* });
            )*

            let mut roles: BTreeMap<RoleKey, RoleList> = BTreeMap::new();
            $(
                internal_add_role!(roles, $role $( => updaters: $($updaters),+)?);
            )*

            let static_roles = scrypto::blueprints::package::StaticRoles {
                methods,
                roles,
            };

            scrypto::blueprints::package::MethodAuthTemplate::Static(static_roles)
        }
    );

    (
        $($module:ident { $($method:ident => $($permission:ident),+ ;)* }),*
    ) => (
        fn method_auth_template() -> scrypto::blueprints::package::MethodAuthTemplate {
            let mut methods: BTreeMap<MethodKey, MethodPermission> = BTreeMap::new();
            $(
                module_permissions!(methods, $module { $($method => $($permission),+ ;)* });
            )*

            let roles = scrypto::blueprints::package::StaticRoles {
                methods,
                roles: BTreeMap::new(),
            };

            scrypto::blueprints::package::MethodAuthTemplate::Static(roles)
        }
    );
}

#[macro_export]
macro_rules! enable_function_auth {
    (
        $($function:ident => $rule:expr;)*
    ) => (
        fn function_auth() -> BTreeMap<String, AccessRule> {
            let rules = Functions::<AccessRule> {
                $( $function: $rule, )*
            };

            rules.to_mapping().into_iter().collect()
        }
    );
}

#[macro_export]
macro_rules! enable_package_royalties {
    ($($function:ident => $royalty:expr,)*) => (
        fn package_royalty_config() -> RoyaltyConfig {
            let royalties = Fns::<RoyaltyAmount> {
                $( $function: $royalty, )*
            };

            RoyaltyConfig {
                rules: royalties.to_mapping().into_iter().collect()
            }
        }
    );
}

#[macro_export]
macro_rules! role_definition_entry {
    ($rule:expr, updaters: $($mutators:ident),+) => {{
        let mut list = RoleList::none();
        permission_role_list!(list, $($mutators),+);
        RoleEntry::new($rule, list)
    }};
    ($rule:expr) => {{
        RoleEntry::immutable($rule)
    }};
}

#[macro_export]
macro_rules! roles_internal {
    ($module_roles:ident, $mutability:ident, $($role:ident => $rule:expr $(, updaters: $($mutators:ident),+)? ;)* ) => ({
        let method_roles = $module_roles::<(AccessRule, bool)> {
            $(
                $role: {
                    ($rule, $mutability.$role)
                }
            ),*
        };

        let mut roles = $crate::blueprints::resource::Roles::new();
        for (name, (rule, mutable)) in method_roles.list() {
            if mutable {
                roles.define_mutable_role(name, RoleEntry::new(rule, RoleList::none()));
            } else {
                roles.define_immutable_role(name, rule);
            }
        }

        roles
    });
}

#[macro_export]
macro_rules! roles {
    ( $($role:ident => $rule:expr $(, updaters: $($mutators:ident),+)? ;)* ) => ({
        roles_internal!(MethodRoles, ROLE_MUTABLE, $($role => $rule $(, updaters: $($mutators),+)? ;)*)
    });
}

#[macro_export]
macro_rules! royalty_config {
    ($($method:ident => $royalty:expr),*) => ({
        Methods::<RoyaltyAmount> {
            $(
                $method: $royalty.into(),
            )*
        }
    });
    ($($method:ident => $royalty:expr,)*) => ({
        royalty_config!($($method => $royalty),*)
    });
}

#[macro_export]
macro_rules! metadata_config {
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

#[macro_export]
macro_rules! metadata {
    {
        roles {
            $($role:ident => $rule:expr $(, updaters: $($mutators:ident),+)? ;)*
        },
        init {
            $($key:expr => $value:expr),*
        }
    } => ({
        let metadata_roles = roles_internal!(MetadataRoles, METADATA_MUTABLE, $($role => $rule $(, updaters: $($mutators),+)? ;)*);
        let metadata = metadata_config!($($key => $value),*);
        (metadata, metadata_roles)
    });

    {
        init {
            $($key:expr => $value:expr),*
        }
    } => ({
        let metadata = metadata_config!($($key => $value),*);
        (metadata, Roles::new())
    });

    {
        roles {
            $($role:ident => $rule:expr $(, updaters: $($mutators:ident),+)? ;)*
        }
    } => ({
        let metadata_roles = roles_internal!(MetadataRoles, METADATA_MUTABLE, $($role => $rule $(, updaters: $($mutators),+)? ;)*);
        let metadata = metadata_config!();
        (metadata, metadata_roles)
    });

}

#[macro_export]
macro_rules! royalties {
    {
        roles {
            $($role:ident => $rule:expr $(, updaters: $($mutators:ident),+)? ;)*
        },
        init {
            $($init:tt)*
        }
    } => ({
        let royalty_roles = roles_internal!(RoyaltyRoles, ROYALTY_MUTABLE, $($role => $rule $(, updaters: $($mutators),+)? ;)*);
        let royalties = royalty_config!($($init)*);
        (royalties, royalty_roles)
    });
    {
        init {
            $($init:tt)*
        }
    } => ({
        let royalties = royalty_config!($($init)*);
        (royalties, Roles::new())
    })
}
