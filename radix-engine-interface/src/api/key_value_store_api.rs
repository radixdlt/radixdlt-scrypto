use radix_engine_common::prelude::{
    replace_self_package_address, ScryptoCustomTypeKind, ScryptoDescribe, VersionedScryptoSchema,
};
use radix_engine_common::types::*;
use radix_engine_derive::{ManifestSbor, ScryptoSbor};
use radix_engine_interface::api::key_value_entry_api::KeyValueEntryHandle;
use radix_engine_interface::api::LockFlags;
use sbor::rust::prelude::*;
use sbor::LocalTypeId;
use sbor::{generate_full_schema, TypeAggregator};

/// Less flexible than previous revision, as mixed type origin is not allowed, but
/// better for client-side optimization
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum KeyValueStoreDataSchema {
    // TODO: ignore this variant in Scrypto for smaller code size
    Local {
        additional_schema: VersionedScryptoSchema,
        key_type: LocalTypeId,
        value_type: LocalTypeId,
        allow_ownership: bool,
    },
    Remote {
        key_type: BlueprintTypeId,
        value_type: BlueprintTypeId,
        allow_ownership: bool,
    },
}

impl KeyValueStoreDataSchema {
    /// Arguments:
    /// * [`package_address`] - The package address to use for replacing `None` package address in type validation
    /// * [`allow_ownership`] - Whether to allow ownership in value
    pub fn new_local_with_self_package_replacement<K: ScryptoDescribe, V: ScryptoDescribe>(
        package_address: PackageAddress,
        allow_ownership: bool,
    ) -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let key_type_id = aggregator.add_child_type_and_descendents::<K>();
        let value_type_id = aggregator.add_child_type_and_descendents::<V>();
        let mut schema = generate_full_schema(aggregator);
        replace_self_package_address(&mut schema, package_address);
        Self::Local {
            additional_schema: schema,
            key_type: key_type_id,
            value_type: value_type_id,
            allow_ownership,
        }
    }

    pub fn new_local_without_self_package_replacement<K: ScryptoDescribe, V: ScryptoDescribe>(
        allow_ownership: bool,
    ) -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let key_type_id = aggregator.add_child_type_and_descendents::<K>();
        let value_type_id = aggregator.add_child_type_and_descendents::<V>();
        let schema = generate_full_schema(aggregator);
        Self::Local {
            additional_schema: schema,
            key_type: key_type_id,
            value_type: value_type_id,
            allow_ownership,
        }
    }

    pub fn new_remote(
        key_type: BlueprintTypeId,
        value_type: BlueprintTypeId,
        allow_ownership: bool,
    ) -> Self {
        Self::Remote {
            key_type,
            value_type,
            allow_ownership,
        }
    }
}

pub trait ClientKeyValueStoreApi<E> {
    /// Creates a new key value store with a given schema
    fn key_value_store_new(&mut self, data_schema: KeyValueStoreDataSchema) -> Result<NodeId, E>;

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
