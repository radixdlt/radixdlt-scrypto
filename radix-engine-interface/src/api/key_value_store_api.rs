use radix_common::prelude::{
    replace_self_package_address, ScryptoCustomTypeKind, ScryptoDescribe, VersionedScryptoSchema,
};
use radix_common::types::*;
use radix_common::{ManifestSbor, ScryptoSbor};
use radix_engine_interface::api::key_value_entry_api::KeyValueEntryHandle;
use radix_engine_interface::api::LockFlags;
use sbor::rust::prelude::*;
use sbor::LocalTypeId;
use sbor::{generate_full_schema, TypeAggregator};

pub const KV_STORE_DATA_SCHEMA_VARIANT_LOCAL: u8 = 0;
pub const KV_STORE_DATA_SCHEMA_VARIANT_REMOTE: u8 = 1;

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
        key_type: BlueprintTypeIdentifier,
        value_type: BlueprintTypeIdentifier,
        allow_ownership: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct LocalKeyValueStoreDataSchema {
    pub additional_schema: VersionedScryptoSchema,
    pub key_type: LocalTypeId,
    pub value_type: LocalTypeId,
    pub allow_ownership: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct RemoteKeyValueStoreDataSchema {
    pub key_type: BlueprintTypeIdentifier,
    pub value_type: BlueprintTypeIdentifier,
    pub allow_ownership: bool,
}

impl LocalKeyValueStoreDataSchema {
    pub fn new_with_self_package_replacement<K: ScryptoDescribe, V: ScryptoDescribe>(
        package_address: PackageAddress,
        allow_ownership: bool,
    ) -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let key_type_id = aggregator.add_child_type_and_descendents::<K>();
        let value_type_id = aggregator.add_child_type_and_descendents::<V>();
        let mut schema = generate_full_schema(aggregator);
        replace_self_package_address(&mut schema, package_address);
        Self {
            additional_schema: schema,
            key_type: key_type_id,
            value_type: value_type_id,
            allow_ownership,
        }
    }

    pub fn new_without_self_package_replacement<K: ScryptoDescribe, V: ScryptoDescribe>(
        allow_ownership: bool,
    ) -> Self {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let key_type_id = aggregator.add_child_type_and_descendents::<K>();
        let value_type_id = aggregator.add_child_type_and_descendents::<V>();
        let schema = generate_full_schema(aggregator);
        Self {
            additional_schema: schema,
            key_type: key_type_id,
            value_type: value_type_id,
            allow_ownership,
        }
    }
}

impl RemoteKeyValueStoreDataSchema {
    pub fn new(
        key_type: BlueprintTypeIdentifier,
        value_type: BlueprintTypeIdentifier,
        allow_ownership: bool,
    ) -> Self {
        Self {
            key_type,
            value_type,
            allow_ownership,
        }
    }
}

impl KeyValueStoreDataSchema {
    pub fn new_local_with_self_package_replacement<K: ScryptoDescribe, V: ScryptoDescribe>(
        package_address: PackageAddress,
        allow_ownership: bool,
    ) -> Self {
        let schema = LocalKeyValueStoreDataSchema::new_with_self_package_replacement::<K, V>(
            package_address,
            allow_ownership,
        );
        Self::Local {
            additional_schema: schema.additional_schema,
            key_type: schema.key_type,
            value_type: schema.value_type,
            allow_ownership: schema.allow_ownership,
        }
    }

    pub fn new_local_without_self_package_replacement<K: ScryptoDescribe, V: ScryptoDescribe>(
        allow_ownership: bool,
    ) -> Self {
        let schema = LocalKeyValueStoreDataSchema::new_without_self_package_replacement::<K, V>(
            allow_ownership,
        );
        Self::Local {
            additional_schema: schema.additional_schema,
            key_type: schema.key_type,
            value_type: schema.value_type,
            allow_ownership: schema.allow_ownership,
        }
    }

    pub fn new_remote(
        key_type: BlueprintTypeIdentifier,
        value_type: BlueprintTypeIdentifier,
        allow_ownership: bool,
    ) -> Self {
        Self::Remote {
            key_type,
            value_type,
            allow_ownership,
        }
    }
}

pub trait SystemKeyValueStoreApi<E> {
    /// Creates a new key value store with a given schema
    fn key_value_store_new(&mut self, data_schema: KeyValueStoreDataSchema) -> Result<NodeId, E>;

    /// Open a key value store entry for reading/writing
    fn key_value_store_open_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
        flags: LockFlags,
    ) -> Result<KeyValueEntryHandle, E>;

    /// Removes an entry from a key value store
    fn key_value_store_remove_entry(
        &mut self,
        node_id: &NodeId,
        key: &Vec<u8>,
    ) -> Result<Vec<u8>, E>;
}
