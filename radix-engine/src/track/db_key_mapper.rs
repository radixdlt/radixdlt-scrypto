use radix_engine_common::data::scrypto::{scrypto_decode, ScryptoDecode};
use radix_engine_interface::crypto::hash;
use radix_engine_interface::types::{ModuleId, NodeId, SubstateKey};
use radix_engine_store_interface::interface::{
    DbPartitionKey, DbSortKey, DbSubstateValue, SubstateDatabase,
};
use sbor::rust::prelude::*;
use utils::{combine, copy_u8_array};

/// A mapper between the business RE Node / Module / Substate IDs and database keys.
pub trait DatabaseKeyMapper {
    /// Converts the given RE Node and Module ID to the database partition's key.
    /// Note: contrary to the sort key, we do not provide the inverse mapping here (i.e. if you
    /// find yourself needing to map the database partition key back to RE Node and Module ID, then
    /// you are most likely using the "partition vs sort key" construct in a wrong way).
    fn to_db_partition_key(node_id: &NodeId, module_id: ModuleId) -> DbPartitionKey;

    /// Converts the given Substate key to the database's sort key.
    fn to_db_sort_key(key: &SubstateKey) -> DbSortKey;

    /// Converts the given database's sort key to a Substate key.
    fn from_db_sort_key(db_sort_key: &DbSortKey) -> SubstateKey;
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

    fn to_db_sort_key(key: &SubstateKey) -> DbSortKey {
        let (discriminator_byte, rest_bytes) = match key {
            SubstateKey::Tuple(field) => (Self::TUPLE_DISCRIMINATOR, vec![*field]),
            SubstateKey::Map(key) => (Self::MAP_DISCRIMINATOR, Self::to_hash_prefixed(key)),
            SubstateKey::Sorted((bucket, key)) => (
                Self::SORTED_DISCRIMINATOR,
                [
                    bucket.to_be_bytes().as_slice(),
                    &Self::to_hash_prefixed(key),
                ]
                .concat(),
            ),
        };
        DbSortKey(combine(discriminator_byte, &rest_bytes))
    }

    fn from_db_sort_key(db_sort_key: &DbSortKey) -> SubstateKey {
        let bytes = &db_sort_key.0;
        let (discriminator_byte, rest_bytes) = (bytes[0], &bytes[1..]);
        match discriminator_byte {
            Self::TUPLE_DISCRIMINATOR => SubstateKey::Tuple(rest_bytes[0]),
            Self::MAP_DISCRIMINATOR => {
                SubstateKey::Map(Self::from_hash_prefixed(rest_bytes).to_vec())
            }
            Self::SORTED_DISCRIMINATOR => SubstateKey::Sorted((
                u16::from_be_bytes(copy_u8_array(&rest_bytes[..2])),
                Self::from_hash_prefixed(&rest_bytes[2..]).to_vec(),
            )),
            unexpected_byte => panic!("unexpected discriminator byte: {}", unexpected_byte),
        }
    }
}

impl SpreadPrefixKeyMapper {
    /// Length of hashes that are used in the database *instead* of plain RE Node and Module IDs.
    const PARTITION_KEY_HASH_LENGTH: usize = 26;

    /// Discriminator bytes for telling apart different types of sort keys (needed for reversible
    /// encoding).
    const TUPLE_DISCRIMINATOR: u8 = 1;
    const MAP_DISCRIMINATOR: u8 = 2;
    const SORTED_DISCRIMINATOR: u8 = 3;

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
    fn list_mapped<M: DatabaseKeyMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (SubstateKey, D)> + '_> {
        let mapped_value_iter = self
            .list_raw_with_mapped_keys::<M>(node_id, module_id)
            .map(|(substate_key, db_value)| (substate_key, scrypto_decode(&db_value).unwrap()));
        Box::new(mapped_value_iter)
    }

    /// Lists partially-mapped entries (i.e. business substate keys and raw database byte values) of
    /// the given node module.
    /// TODO(resolve during review): this seems like a "half-way abstraction" - can be refactored?
    fn list_raw_with_mapped_keys<M: DatabaseKeyMapper>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (SubstateKey, DbSubstateValue)> + '_>;
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

    fn list_raw_with_mapped_keys<M: DatabaseKeyMapper>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (SubstateKey, DbSubstateValue)> + '_> {
        let mapped_key_iter = self
            .list_entries(&M::to_db_partition_key(node_id, module_id))
            .map(|(db_sort_key, db_value)| (M::from_db_sort_key(&db_sort_key), db_value));
        Box::new(mapped_key_iter)
    }
}
