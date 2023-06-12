#[macro_export]
macro_rules! event_schema {
    ($aggregator: ident, [$($event_type: ty),*]) => {
        {
            let mut event_schema = sbor::rust::collections::BTreeMap::new();
            $(
                event_schema.insert(
                    <$event_type as radix_engine_interface::traits::ScryptoEvent>::event_name().to_string(),
                    $aggregator.add_child_type_and_descendents::<$event_type>(),
                );
            )*
            radix_engine_interface::schema::BlueprintEventSchemaInit {
                event_schema
            }
        }
    };
}

#[macro_export]
macro_rules! method_auth_template {
    () => ({
        let auth: BTreeMap<radix_engine_interface::blueprints::resource::MethodKey, radix_engine_interface::blueprints::resource::MethodPermission> = BTreeMap::new();
        auth
    });
    ( $($method:expr => $entry:expr );* ) => ({
        let mut auth: BTreeMap<radix_engine_interface::blueprints::resource::MethodKey, radix_engine_interface::blueprints::resource::MethodPermission>
            = BTreeMap::new();
        $(
            auth.insert($method, $entry.into());
        )*
        auth
    });
    ( $($key:expr => $entry:expr;)* ) => (
        method_auth_template!{$($key => $entry);*}
    );
}
