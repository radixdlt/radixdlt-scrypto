use radix_engine_common::data::scrypto::{scrypto_decode, ScryptoDecode};
use radix_engine_common::types::{MapKey, SortedU16Key, TupleKey};
use radix_engine_interface::crypto::hash;
use radix_engine_interface::types::{ModuleId, NodeId, SubstateKey};
use radix_engine_store_interface::interface::{
    DbPartitionKey, DbSortKey, DbSubstateValue, SubstateDatabase,
};
use sbor::rust::prelude::*;
use utils::copy_u8_array;

/// A mapper between the business RE Node / Module / Substate IDs and database keys.
pub trait DatabaseKeyMapper {
    /// Converts the given RE Node and Module ID to the database partition's key.
    /// Note: contrary to the sort key, we do not provide the inverse mapping here (i.e. if you
    /// find yourself needing to map the database partition key back to RE Node and Module ID, then
    /// you are most likely using the "partition vs sort key" construct in a wrong way).
    fn to_db_partition_key(node_id: &NodeId, module_id: ModuleId) -> DbPartitionKey;

    /// Converts the given Substate key to the database's sort key.
    fn to_db_sort_key(key: &SubstateKey) -> DbSortKey;

    /// Converts the given database's sort key to a concrete type of Substate key.
    fn from_db_sort_key<K: ConcreteSubstateKeyDecoding>(db_sort_key: &DbSortKey) -> K;
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

    fn to_db_sort_key(key: &SubstateKey) -> DbSortKey {
        let bytes = match key {
            SubstateKey::Tuple(field) => vec![*field],
            SubstateKey::Map(key) => Self::to_hash_prefixed(key),
            SubstateKey::Sorted((bucket, key)) => [
                bucket.to_be_bytes().as_slice(),
                &Self::to_hash_prefixed(key),
            ]
            .concat(),
        };
        DbSortKey(bytes)
    }

    fn from_db_sort_key<K: ConcreteSubstateKeyDecoding>(db_sort_key: &DbSortKey) -> K {
        K::decode(&db_sort_key.0)
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
    fn list_mapped_values<M: DatabaseKeyMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = D> + '_> {
        let mapped_value_iter = self
            .list_raw_with_mapped_keys::<M, IrrelevantSubstateKey>(node_id, module_id)
            .map(|(_substate_key, db_value)| scrypto_decode(&db_value).unwrap());
        Box::new(mapped_value_iter)
    }

    /// Lists partially-mapped entries (i.e. business substate keys and raw database byte values) of
    /// the given node module.
    fn list_raw_with_mapped_keys<M: DatabaseKeyMapper, K: ConcreteSubstateKeyDecoding>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (K, DbSubstateValue)> + '_>;
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
            &M::to_db_sort_key(&substate_key),
        )
        .map(|buf| scrypto_decode(&buf).unwrap())
    }

    fn list_raw_with_mapped_keys<M: DatabaseKeyMapper, K: ConcreteSubstateKeyDecoding>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (K, DbSubstateValue)> + '_> {
        let mapped_key_iter = self
            .list_entries(&M::to_db_partition_key(node_id, module_id))
            .map(|(db_sort_key, db_value)| (M::from_db_sort_key::<K>(&db_sort_key), db_value));
        Box::new(mapped_key_iter)
    }
}

/// A "decode self" extension trait for concrete contents of the [`SubstateKey`] enum (used only
/// internally by [`SpreadPrefixKeyMapper::from_db_sort_key()`]).
pub trait ConcreteSubstateKeyDecoding {
    /// Decodes self from the given bytes.
    fn decode(bytes: &[u8]) -> Self;
}

impl ConcreteSubstateKeyDecoding for TupleKey {
    fn decode(bytes: &[u8]) -> Self {
        if bytes.len() != 1 {
            panic!("unexpected length: {}", bytes.len());
        }
        bytes[0]
    }
}

impl ConcreteSubstateKeyDecoding for MapKey {
    fn decode(bytes: &[u8]) -> Self {
        SpreadPrefixKeyMapper::from_hash_prefixed(bytes).to_vec()
    }
}

impl ConcreteSubstateKeyDecoding for SortedU16Key {
    fn decode(bytes: &[u8]) -> Self {
        if bytes.len() < 2 {
            panic!("insufficient length: {}", bytes.len());
        }
        (
            u16::from_be_bytes(copy_u8_array(&bytes[..2])),
            SpreadPrefixKeyMapper::from_hash_prefixed(&bytes[2..]).to_vec(),
        )
    }
}

/// A special "no-op" key decoding for cases where concrete key type is not important.
type IrrelevantSubstateKey = ();

impl ConcreteSubstateKeyDecoding for IrrelevantSubstateKey {
    fn decode(_bytes: &[u8]) -> Self {
        ()
    }
}
