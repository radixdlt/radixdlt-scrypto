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
    ( $($method:expr => $entry:expr );* ) => ({
        let mut methods: BTreeMap<SchemaMethodKey, SchemaMethodPermission>
            = BTreeMap::new();
        $(
            methods.insert($method, $entry.into());
        )*
        methods
    });
    ( $($key:expr => $entry:expr;)* ) => (
        method_auth_template!{$($key => $entry);*}
    );
}
