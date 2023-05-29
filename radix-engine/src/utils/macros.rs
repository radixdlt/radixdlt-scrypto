#[macro_export]
macro_rules! event_schema {
    ($aggregator: ident, [$($type_name: ty),*]) => {
        {
            let mut schema = sbor::rust::collections::BTreeMap::new();
            $(
                schema.insert(
                    stringify!($type_name).to_owned(),
                    $aggregator.add_child_type_and_descendents::<$type_name>(),
                );
            )*
            schema
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
