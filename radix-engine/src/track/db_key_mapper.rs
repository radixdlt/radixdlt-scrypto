use radix_engine_common::data::scrypto::{scrypto_decode, ScryptoDecode};
use radix_engine_common::types::{MapKey, SortedU16Key, TupleKey};
use radix_engine_interface::crypto::hash;
use radix_engine_interface::types::{ModuleId, NodeId, SubstateKey};
use radix_engine_store_interface::interface::{DbPartitionKey, DbSortKey, SubstateDatabase};
use sbor::rust::prelude::*;
use utils::copy_u8_array;

/// A mapper between the business RE Node / Module / Substate IDs and database keys.
pub trait DatabaseKeyMapper {
    /// Converts the given RE Node and Module ID to the database partition's key.
    /// Note: contrary to the sort key, we do not provide the inverse mapping here (i.e. if you
    /// find yourself needing to map the database partition key back to RE Node and Module ID, then
    /// you are most likely using the "partition vs sort key" construct in a wrong way).
    fn to_db_partition_key(node_id: &NodeId, module_id: ModuleId) -> DbPartitionKey;
}

/// A [`DatabaseKeyMapper`] tailored for databases which cannot tolerate long common prefixes
/// among keys (for performance reasons). In other words, it spreads the keys "evenly" (i.e.
/// pseudo-randomly) across the key space. For context: our use-case for this is the Jellyfish
/// Merkle Tree.
///
/// This implementation is the actual, protocol-enforced one, to be used in public Radix networks.
///
/// This implementation achieves the prefix-spreading by:
/// - using a (long, ~100% unique) hash instead of plain RE Node and Module ID (please note that it
///   makes this mapping effectively irreversible);
/// - using a (shorter, but hard to crack) hash prefix for Substate key.
pub struct SpreadPrefixKeyMapper;

impl DatabaseKeyMapper for SpreadPrefixKeyMapper {
    fn to_db_partition_key(node_id: &NodeId, module_id: ModuleId) -> DbPartitionKey {
        let mut buffer = Vec::new();
        buffer.extend(node_id.as_ref());
        buffer.push(module_id.0);
        let hash_bytes = hash(buffer).0[..Self::PARTITION_KEY_HASH_LENGTH].to_vec();
        DbPartitionKey(hash_bytes)
    }
}

impl SpreadPrefixKeyMapper {
    /// Length of hashes that are used in the database *instead* of plain RE Node and Module IDs.
    const PARTITION_KEY_HASH_LENGTH: usize = 26;

    /// A number of leading bytes populated with a hash of the sort key (for spreading purposes).
    /// This number should be:
    /// - high enough to avoid being cracked (for crafting arbitrarily long key prefixes);
    /// - low enough to avoid inflating database key sizes.
    ///
    /// Note: hashing will not be applied to [`TupleKey`] (which is a single byte, and hence does
    /// not create the risk of long common prefixes).
    const HASHED_PREFIX_LENGTH: usize = 20;

    /// Returns the given bytes prefixed by their known-length hash (see [`HASHED_PREFIX_LENGTH`]).
    fn to_hash_prefixed(plain_bytes: &[u8]) -> Vec<u8> {
        let hashed_prefix = &hash(plain_bytes).0[..Self::HASHED_PREFIX_LENGTH];
        [hashed_prefix, plain_bytes].concat()
    }

    /// Returns the given slice without its known-length hash prefix (see [`HASHED_PREFIX_LENGTH`]).
    fn from_hash_prefixed(prefixed_bytes: &[u8]) -> &[u8] {
        &prefixed_bytes[Self::HASHED_PREFIX_LENGTH..]
    }
}

pub trait SortKey {
    fn from_db_sort_key(key: DbSortKey) -> Self;
    fn to_db_sort_key(&self) -> DbSortKey;
}

/// This SubstateKey enum is probably the wrong interface for this. The proper way of
/// exposing for execution is by using the SortKey generic all the way up the stack
/// (and then replacing the current SubstateKey with SortKey). This would enforce the
/// invariant that different types of keys should not be mixed together.
///
/// This may be more of a big implementation undertaking so this hack to use an enum
/// is good enough for now.
impl SortKey for SubstateKey {
    /// Once abstraction is cleaned up we should be able to remove this
    fn from_db_sort_key(_key: DbSortKey) -> Self {
        panic!("Should never be called");
    }

    fn to_db_sort_key(&self) -> DbSortKey {
        match self {
            SubstateKey::Tuple(field) => field.to_db_sort_key(),
            SubstateKey::Map(key) => key.to_db_sort_key(),
            SubstateKey::Sorted(sorted_key) => sorted_key.to_db_sort_key(),
        }
    }
}

impl SortKey for MapKey {
    fn from_db_sort_key(key: DbSortKey) -> Self {
        SpreadPrefixKeyMapper::from_hash_prefixed(&key.0).to_vec()
    }

    fn to_db_sort_key(&self) -> DbSortKey {
        DbSortKey(SpreadPrefixKeyMapper::to_hash_prefixed(self))
    }
}

impl SortKey for TupleKey {
    fn from_db_sort_key(key: DbSortKey) -> Self {
        key.0[0]
    }

    fn to_db_sort_key(&self) -> DbSortKey {
        DbSortKey(vec![*self])
    }
}

impl SortKey for SortedU16Key {
    fn from_db_sort_key(key: DbSortKey) -> Self {
        (
            u16::from_be_bytes(copy_u8_array(&key.0[..2])),
            SpreadPrefixKeyMapper::from_hash_prefixed(&key.0[2..]).to_vec(),
        )
    }

    fn to_db_sort_key(&self) -> DbSortKey {
        DbSortKey(
            [
                self.0.to_be_bytes().as_slice(),
                &SpreadPrefixKeyMapper::to_hash_prefixed(&self.1),
            ]
            .concat(),
        )
    }
}

/// Convenience methods for direct `SubstateDatabase` readers.
pub trait MappedSubstateDatabase {
    /// Gets a scrypto-decoded value by the given business key.
    fn get_mapped<M: DatabaseKeyMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<D>;

    /// Lists fully-mapped entries (i.e. business substate keys and scrypto-decoded values) of the
    /// given node module.
    fn list_mapped<M: DatabaseKeyMapper, D: ScryptoDecode, K: SortKey>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (K, D)> + '_>;
}

impl<S: SubstateDatabase> MappedSubstateDatabase for S {
    fn get_mapped<M: DatabaseKeyMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<D> {
        self.get_substate(
            &M::to_db_partition_key(node_id, module_id),
            &substate_key.to_db_sort_key(),
        )
        .map(|buf| scrypto_decode(&buf).unwrap())
    }

    fn list_mapped<M: DatabaseKeyMapper, D: ScryptoDecode, K: SortKey>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (K, D)> + '_> {
        let mapped_value_iter = self
            .list_entries(&M::to_db_partition_key(node_id, module_id))
            .map(|(db_sort_key, db_value)| {
                (
                    K::from_db_sort_key(db_sort_key),
                    scrypto_decode(&db_value).unwrap(),
                )
            });
        Box::new(mapped_value_iter)
    }
}
