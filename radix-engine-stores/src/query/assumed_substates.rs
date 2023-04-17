use radix_engine_interface::schema::KeyValueStoreSchema;
use radix_engine_interface::types::ObjectInfo;
use radix_engine_interface::*;

// TODO: de-dup

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TypeInfoSubstate {
    Object(ObjectInfo),
    KeyValueStore(KeyValueStoreSchema),
    IterableStore,
    SortedStore,
}
