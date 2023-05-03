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

    /// Converts the given [`SubstateKey`] to the database's sort key.
    /// This is a convenience method, which simply unwraps the [`SubstateKey`] and maps any specific
    /// type found inside (see `*_to_db_sort_key()` family).
    fn to_db_sort_key(key: &SubstateKey) -> DbSortKey {
        match key {
            SubstateKey::Tuple(tuple_key) => Self::tuple_to_db_sort_key(tuple_key),
            SubstateKey::Map(map_key) => Self::map_to_db_sort_key(map_key),
            SubstateKey::Sorted(sorted_key) => Self::sorted_to_db_sort_key(sorted_key),
        }
    }

    /// Converts the given database's sort key to a [`SubstateKey`].
    /// This is a convenience method, which simply wraps the type-specific result of an appropriate
    /// `*_from_db_sort_key()` method into a [`SubstateKey`].
    fn from_db_sort_key<K: SubstateKeyContent>(db_sort_key: &DbSortKey) -> SubstateKey {
        match K::get_type() {
            SubstateKeyTypeContentType::Tuple => {
                SubstateKey::Tuple(Self::tuple_from_db_sort_key(db_sort_key))
            }
            SubstateKeyTypeContentType::Map => {
                SubstateKey::Map(Self::map_from_db_sort_key(db_sort_key))
            }
            SubstateKeyTypeContentType::Sorted => {
                SubstateKey::Sorted(Self::sorted_from_db_sort_key(db_sort_key))
            }
        }
    }

    // Type-specific methods for mapping the `SubstateKey` inner data to/from `DbSortKey`:

    fn tuple_to_db_sort_key(tuple_key: &TupleKey) -> DbSortKey;
    fn tuple_from_db_sort_key(db_sort_key: &DbSortKey) -> TupleKey;

    fn map_to_db_sort_key(map_key: &MapKey) -> DbSortKey;
    fn map_from_db_sort_key(db_sort_key: &DbSortKey) -> MapKey;

    fn sorted_to_db_sort_key(sorted_key: &SortedU16Key) -> DbSortKey;
    fn sorted_from_db_sort_key(db_sort_key: &DbSortKey) -> SortedU16Key;
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
        DbPartitionKey(hash(buffer).to_vec())
    }

    fn tuple_to_db_sort_key(tuple_key: &TupleKey) -> DbSortKey {
        DbSortKey(vec![*tuple_key])
    }

    fn tuple_from_db_sort_key(db_sort_key: &DbSortKey) -> TupleKey {
        db_sort_key.0[0]
    }

    fn map_to_db_sort_key(map_key: &MapKey) -> DbSortKey {
        DbSortKey(SpreadPrefixKeyMapper::to_hash_prefixed(map_key))
    }

    fn map_from_db_sort_key(db_sort_key: &DbSortKey) -> MapKey {
        SpreadPrefixKeyMapper::from_hash_prefixed(&db_sort_key.0).to_vec()
    }

    fn sorted_to_db_sort_key(sorted_key: &SortedU16Key) -> DbSortKey {
        DbSortKey(
            [
                sorted_key.0.to_be_bytes().as_slice(),
                &SpreadPrefixKeyMapper::to_hash_prefixed(&sorted_key.1),
            ]
            .concat(),
        )
    }

    fn sorted_from_db_sort_key(db_sort_key: &DbSortKey) -> SortedU16Key {
        (
            u16::from_be_bytes(copy_u8_array(&db_sort_key.0[..2])),
            SpreadPrefixKeyMapper::from_hash_prefixed(&db_sort_key.0[2..]).to_vec(),
        )
    }
}

impl SpreadPrefixKeyMapper {
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
    fn list_mapped<M: DatabaseKeyMapper, D: ScryptoDecode, K: SubstateKeyContent>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (SubstateKey, D)> + '_>;
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
            &M::to_db_sort_key(substate_key),
        )
        .map(|buf| scrypto_decode(&buf).unwrap())
    }

    fn list_mapped<M: DatabaseKeyMapper, D: ScryptoDecode, K: SubstateKeyContent>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (SubstateKey, D)> + '_> {
        let mapped_value_iter = self
            .list_entries(&M::to_db_partition_key(node_id, module_id))
            .map(|(db_sort_key, db_value)| {
                (
                    M::from_db_sort_key::<K>(&db_sort_key),
                    scrypto_decode(&db_value).unwrap(),
                )
            });
        Box::new(mapped_value_iter)
    }
}

// Internal-only trait enabling the concrete `DatabaseKeyMapper` implementations to drive their
// logic by `SubstateKey`'s inner data type.

pub trait SubstateKeyContent {
    fn get_type() -> SubstateKeyTypeContentType;
}

pub enum SubstateKeyTypeContentType {
    Tuple,
    Map,
    Sorted,
}

impl SubstateKeyContent for MapKey {
    fn get_type() -> SubstateKeyTypeContentType {
        SubstateKeyTypeContentType::Map
    }
}

impl SubstateKeyContent for TupleKey {
    fn get_type() -> SubstateKeyTypeContentType {
        SubstateKeyTypeContentType::Tuple
    }
}

impl SubstateKeyContent for SortedU16Key {
    fn get_type() -> SubstateKeyTypeContentType {
        SubstateKeyTypeContentType::Sorted
    }
}
