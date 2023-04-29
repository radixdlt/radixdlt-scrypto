use radix_engine_common::data::scrypto::{scrypto_decode, ScryptoDecode};
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::types::{ModuleId, NodeId, SubstateKey};
use radix_engine_store_interface::interface::{
    DbPartitionKey, DbSortKey, PartitionEntry, SubstateDatabase,
};
use sbor::rust::prelude::*;

/// A mapper between the business RE Node / Module / Substate IDs and database keys.
pub trait DatabaseKeyMapper {
    /// Converts the given RE Node and Module ID to the database partition's key.
    fn to_db_partition_key(node_id: &NodeId, module_id: ModuleId) -> DbPartitionKey;

    /// Converts the given Substate key to the database's sort key.
    fn to_db_sort_key(key: &SubstateKey) -> DbSortKey;
}

/// A [`DatabaseKeyMapper`] tailored for databases which cannot tolerate long common prefixes
/// among keys (for performance reasons). In other words, it spreads the keys "evenly" (i.e.
/// pseudo-randomly) across the key space.
/// For context: our actual use-case for it is the Jellyfish Merkle Tree.
pub struct SpreadPrefixKeyMapper;

impl DatabaseKeyMapper for SpreadPrefixKeyMapper {
    fn to_db_partition_key(node_id: &NodeId, module_id: ModuleId) -> DbPartitionKey {
        let mut buffer = Vec::new();
        buffer.extend(node_id.as_ref());
        buffer.push(module_id.0);
        let hash_bytes = hash(buffer).0[(Hash::LENGTH - 26)..Hash::LENGTH].to_vec(); // 26 bytes
        DbPartitionKey(hash_bytes)
    }

    fn to_db_sort_key(key: &SubstateKey) -> DbSortKey {
        let bytes = match key {
            SubstateKey::Tuple(field) => {
                vec![*field]
            }
            SubstateKey::Map(key) => {
                hash(key).0[12..Hash::LENGTH].to_vec() // 20 bytes
            }
            SubstateKey::Sorted((bucket, key)) => {
                let mut bytes = bucket.to_be_bytes().to_vec();
                bytes.extend(hash(key).0[12..Hash::LENGTH].to_vec()); // 20 bytes
                bytes
            }
        };
        DbSortKey(bytes)
    }
}

/// Convenience methods for direct `SubstateDatabase` readers.
pub trait MappedSubstateDatabase {
    fn get_mapped_substate<M: DatabaseKeyMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<D>;

    fn list_mapped_substates<M: DatabaseKeyMapper>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_>;
}

impl<S: SubstateDatabase> MappedSubstateDatabase for S {
    fn get_mapped_substate<M: DatabaseKeyMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<D> {
        self.get_substate(
            &M::to_db_partition_key(node_id, module_id),
            &M::to_db_sort_key(&substate_key),
        )
        .map(|buf| scrypto_decode(&buf).unwrap())
    }

    fn list_mapped_substates<M: DatabaseKeyMapper>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        self.list_entries(&M::to_db_partition_key(node_id, module_id))
    }
}
