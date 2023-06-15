use crate::interface::{
    CommittableSubstateDatabase, DatabaseUpdate, DbPartitionKey, DbSortKey, SubstateDatabase,
};
use radix_engine_common::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode,
};
use radix_engine_common::types::{FieldKey, MapKey, PartitionNumber, SortedU16Key};
use radix_engine_interface::crypto::hash;
use radix_engine_interface::types::{NodeId, SubstateKey};
use sbor::rust::prelude::*;
use utils::copy_u8_array;

/// A mapper between the business RE Node / Module / Substate IDs and database keys.
pub trait DatabaseKeyMapper {
    /// Converts the given RE Node and Module ID to the database partition's key.
    fn to_db_partition_key(node_id: &NodeId, partition_num: PartitionNumber) -> DbPartitionKey;

    /// Converts database partition's key back to RE Node and Module ID.
    fn from_db_partition_key(partition_key: &DbPartitionKey) -> (NodeId, PartitionNumber);

    /// Converts the given [`SubstateKey`] to the database's sort key.
    /// This is a convenience method, which simply unwraps the [`SubstateKey`] and maps any specific
    /// type found inside (see `*_to_db_sort_key()` family).
    fn to_db_sort_key(key: &SubstateKey) -> DbSortKey {
        match key {
            SubstateKey::Field(fields_key) => Self::field_to_db_sort_key(fields_key),
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
                SubstateKey::Field(Self::field_from_db_sort_key(db_sort_key))
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

    fn field_to_db_sort_key(field_key: &FieldKey) -> DbSortKey;
    fn field_from_db_sort_key(db_sort_key: &DbSortKey) -> FieldKey;

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
/// This implementation achieves the prefix-spreading by adding a hash prefix (shortened hash for
/// performance reasons, but still hard to crack) to:
/// - the PartitionKey (namely a RE Node and Module ID)
/// - the SubstateKey
pub struct SpreadPrefixKeyMapper;

impl DatabaseKeyMapper for SpreadPrefixKeyMapper {
    fn to_db_partition_key(node_id: &NodeId, partition_num: PartitionNumber) -> DbPartitionKey {
        let mut buffer = Vec::new();
        buffer.extend(node_id.as_ref());
        buffer.push(partition_num.0);
        DbPartitionKey(SpreadPrefixKeyMapper::to_hash_prefixed(&buffer[..]))
    }

    fn from_db_partition_key(partition_key: &DbPartitionKey) -> (NodeId, PartitionNumber) {
        let buffer = SpreadPrefixKeyMapper::from_hash_prefixed(&partition_key.0);
        let mut bytes = [0u8; NodeId::LENGTH];
        bytes.copy_from_slice(&buffer[..buffer.len() - 1]);
        let partition_num = PartitionNumber(*buffer.last().unwrap());

        (NodeId(bytes), partition_num)
    }

    fn field_to_db_sort_key(fields_key: &FieldKey) -> DbSortKey {
        DbSortKey(vec![*fields_key])
    }

    fn field_from_db_sort_key(db_sort_key: &DbSortKey) -> FieldKey {
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
    /// Note: hashing will not be applied to [`FieldKey`] (which is a single byte, and hence does
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
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Option<D>;

    /// Lists fully-mapped entries (i.e. business substate keys and scrypto-decoded values) of the
    /// given node partition.
    fn list_mapped<M: DatabaseKeyMapper, D: ScryptoDecode, K: SubstateKeyContent>(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
    ) -> Box<dyn Iterator<Item = (SubstateKey, D)> + '_>;
}

impl<S: SubstateDatabase> MappedSubstateDatabase for S {
    fn get_mapped<M: DatabaseKeyMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Option<D> {
        self.get_substate(
            &M::to_db_partition_key(node_id, partition_num),
            &M::to_db_sort_key(substate_key),
        )
        .map(|buf| scrypto_decode(&buf).unwrap())
    }

    fn list_mapped<M: DatabaseKeyMapper, D: ScryptoDecode, K: SubstateKeyContent>(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
    ) -> Box<dyn Iterator<Item = (SubstateKey, D)> + '_> {
        let mapped_value_iter = self
            .list_entries(&M::to_db_partition_key(node_id, partition_num))
            .map(|(db_sort_key, db_value)| {
                (
                    M::from_db_sort_key::<K>(&db_sort_key),
                    scrypto_decode(&db_value).unwrap(),
                )
            });
        Box::new(mapped_value_iter)
    }
}

/// Convenience methods for direct `SubstateDatabase` writers.
pub trait MappedCommittableSubstateDatabase {
    /// Puts a scrypto-encoded value by the given business key.
    fn put_mapped<M: DatabaseKeyMapper, E: ScryptoEncode>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        value: &E,
    );
}

impl<S: CommittableSubstateDatabase> MappedCommittableSubstateDatabase for S {
    fn put_mapped<M: DatabaseKeyMapper, E: ScryptoEncode>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        value: &E,
    ) {
        self.commit(&indexmap!(
            M::to_db_partition_key(node_id, partition_num) => indexmap!(
                M::to_db_sort_key(substate_key) => DatabaseUpdate::Set(
                    scrypto_encode(value).unwrap()
                )
            )
        ))
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

impl SubstateKeyContent for FieldKey {
    fn get_type() -> SubstateKeyTypeContentType {
        SubstateKeyTypeContentType::Tuple
    }
}

impl SubstateKeyContent for SortedU16Key {
    fn get_type() -> SubstateKeyTypeContentType {
        SubstateKeyTypeContentType::Sorted
    }
}
