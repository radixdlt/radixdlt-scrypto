#[macro_export]
macro_rules! event_schema {
    ($($type_name: ty),*) => {
        {
            let mut schema = sbor::rust::collections::BTreeMap::new();
            $(
                schema.insert(
                    stringify!($type_name).to_owned(),
                    sbor::generate_full_schema_from_single_type::<$type_name, radix_engine_common::data::scrypto::ScryptoCustomTypeExtension>()
                );
            )*
            schema
        }
    };
}
