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
macro_rules! method_permissions {
    ( $($key:expr => $value:expr),* ) => ({
        let mut temp: BTreeMap<MethodKey, (MethodPermission, RoleList)>
            = BTreeMap::new();
        $(
            temp.insert($key, ($value.into(), RoleList::none()));
        )*
        temp
    });
    ( $($key:expr => $value:expr,)* ) => (
        method_permissions!{$($key => $value),*}
    );
}