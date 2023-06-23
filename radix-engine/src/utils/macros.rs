#[macro_export]
macro_rules! event_schema {
    ($aggregator: ident, [$($event_type: ty),*]) => {
        {
            let mut event_schema = sbor::rust::collections::BTreeMap::new();
            $(
                event_schema.insert(
                    <$event_type as radix_engine_interface::traits::ScryptoEvent>::event_name().to_string(),
                    TypeRef::Static($aggregator.add_child_type_and_descendents::<$event_type>()),
                );
            )*
            radix_engine_interface::schema::BlueprintEventSchemaInit {
                event_schema
            }
        }
    };
}

#[macro_export]
macro_rules! add_role {
    ($roles:expr, $role:expr) => {{
        $roles.insert(
            $role.into(),
            radix_engine_interface::blueprints::resource::RoleList::none(),
        );
    }};
    ($roles:expr, $role:expr => updaters: $updaters:expr) => {{
        $roles.insert($role.into(), $updaters.into());
    }};
}

#[macro_export]
macro_rules! method_auth_template {
    () => ({
        let methods: BTreeMap<radix_engine_interface::blueprints::resource::MethodKey, radix_engine_interface::blueprints::resource::MethodAccessibility>
            = BTreeMap::new();
        methods
    });
    ($($method:expr => $entry:expr;)*) => ({
        let mut methods: BTreeMap<radix_engine_interface::blueprints::resource::MethodKey, radix_engine_interface::blueprints::resource::MethodAccessibility>
            = BTreeMap::new();
        $(
            methods.insert($method.into(), $entry.into());
        )*
        methods
    });
}

#[macro_export]
macro_rules! roles_template {
    () => ({
        radix_engine_interface::blueprints::package::StaticRoles {
            roles: radix_engine_interface::blueprints::package::RoleSpecification::Normal(BTreeMap::new()),
            methods: BTreeMap::new(),
        }
    });
    (
        roles { $($role:expr $( => updaters: $updaters:expr)?;)* },
        methods { $($method:expr => $entry:expr; )* }
    ) => ({
        let mut methods: BTreeMap<radix_engine_interface::blueprints::resource::MethodKey, radix_engine_interface::blueprints::resource::MethodAccessibility>
            = BTreeMap::new();
        $(
            methods.insert($method.into(), $entry.into());
        )*

        let mut roles: BTreeMap<radix_engine_interface::blueprints::resource::RoleKey, radix_engine_interface::blueprints::resource::RoleList> = BTreeMap::new();
        $(
            crate::add_role!(roles, $role $( => updaters: $updaters)?);
        )*

        radix_engine_interface::blueprints::package::StaticRoles {
            roles: radix_engine_interface::blueprints::package::RoleSpecification::Normal(roles),
            methods,
        }
    });
    ( methods { $($method:expr => $entry:expr;)* }) => ({
        let mut methods: BTreeMap<radix_engine_interface::blueprints::resource::MethodKey, radix_engine_interface::blueprints::resource::MethodAccessibility>
            = BTreeMap::new();
        $(
            methods.insert($method.into(), $entry.into());
        )*

        radix_engine_interface::blueprints::package::StaticRoles {
            roles: radix_engine_interface::blueprints::package::RoleSpecification::Normal(BTreeMap::new()),
            methods,
        }
    });
}
