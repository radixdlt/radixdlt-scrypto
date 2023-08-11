use radix_engine_common::prelude::{
    replace_self_package_address, HasSchemaHash, ScryptoCustomTypeKind, ScryptoDescribe,
    ScryptoSchema,
};
use radix_engine_common::types::*;
use radix_engine_derive::{ManifestSbor, ScryptoSbor};
use radix_engine_interface::api::key_value_entry_api::KeyValueEntryHandle;
use radix_engine_interface::api::LockFlags;
use sbor::rust::prelude::*;
use sbor::{generate_full_schema, TypeAggregator};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct KeyValueStoreGenericArgs {
    pub additional_schema: Option<ScryptoSchema>,
    pub key_type: TypeSubstitutionRef,
    pub value_type: TypeSubstitutionRef,
    pub can_own: bool,
}

impl KeyValueStoreGenericArgs {
    pub fn new<K: ScryptoDescribe, V: ScryptoDescribe>(can_own: bool) -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let key_type_index = aggregator.add_child_type_and_descendents::<K>();
        let value_type_index = aggregator.add_child_type_and_descendents::<V>();
        let schema = generate_full_schema(aggregator);
        let schema_hash = schema.generate_schema_hash();
        Self {
            additional_schema: Some(schema),
            key_type: TypeSubstitutionRef::Local(TypeIdentifier(schema_hash, key_type_index)),
            value_type: TypeSubstitutionRef::Local(TypeIdentifier(schema_hash, value_type_index)),
            can_own,
        }
    }

    pub fn new_with_self_package<K: ScryptoDescribe, V: ScryptoDescribe>(
        can_own: bool,
        package_address: PackageAddress,
    ) -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let key_type_index = aggregator.add_child_type_and_descendents::<K>();
        let value_type_index = aggregator.add_child_type_and_descendents::<V>();
        let mut schema = generate_full_schema(aggregator);
        replace_self_package_address(&mut schema, package_address);
        let schema_hash = schema.generate_schema_hash();
        Self {
            additional_schema: Some(schema),
            key_type: TypeSubstitutionRef::Local(TypeIdentifier(schema_hash, key_type_index)),
            value_type: TypeSubstitutionRef::Local(TypeIdentifier(schema_hash, value_type_index)),
            can_own,
        }
    }
}

pub trait ClientKeyValueStoreApi<E> {
    /// Creates a new key value store with a given schema
    fn key_value_store_new(&mut self, generic_args: KeyValueStoreGenericArgs) -> Result<NodeId, E>;

    /// Lock a key value store entry for reading/writing
    fn key_value_store_open_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, E>;

    fn key_value_store_remove_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
    ) -> Result<Vec<u8>, E>;
}
