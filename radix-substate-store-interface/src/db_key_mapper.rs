use crate::interface::*;
use radix_common::prelude::*;
use radix_rust::copy_u8_array;

/// A mapper between the business ReNode / Partition / Substate IDs and database keys.
pub trait DatabaseKeyMapper: 'static {
    /// Converts the given Node ID and Partition number to the database partition's key.
    fn to_db_partition_key(node_id: &NodeId, partition_num: PartitionNumber) -> DbPartitionKey {
        DbPartitionKey {
            node_key: Self::to_db_node_key(node_id),
            partition_num: Self::to_db_partition_num(partition_num),
        }
    }

    /// Converts database partition's key back to ReNode ID and Partition number.
    fn from_db_partition_key(partition_key: &DbPartitionKey) -> (NodeId, PartitionNumber) {
        (
            Self::from_db_node_key(&partition_key.node_key),
            Self::from_db_partition_num(partition_key.partition_num),
        )
    }

    /// Converts the given Node ID to the database Node key.
    fn to_db_node_key(node_id: &NodeId) -> DbNodeKey;

    /// Converts the database Node key back to ReNode ID.
    fn from_db_node_key(db_node_key: &DbNodeKey) -> NodeId;

    /// Converts the given Partition number to the database Partition number.
    fn to_db_partition_num(partition_num: PartitionNumber) -> DbPartitionNum;

    /// Converts the database Partition number back to a Partition number.
    fn from_db_partition_num(db_partition_num: DbPartitionNum) -> PartitionNumber;

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

    /// Converts the given [`SubstateKeyRef`] to the database's sort key.
    /// This is a convenience method, which simply unwraps the [`SubstateKeyRef`] and maps any specific
    /// type found inside (see `*_to_db_sort_key()` family).
    fn to_db_sort_key_from_ref(key: SubstateKeyRef) -> DbSortKey {
        match key {
            SubstateKeyRef::Field(fields_key) => Self::field_to_db_sort_key(fields_key),
            SubstateKeyRef::Map(map_key) => Self::map_to_db_sort_key(map_key),
            SubstateKeyRef::Sorted(sorted_key) => Self::sorted_to_db_sort_key(sorted_key),
        }
    }

    /// Converts the given database's sort key to a [`SubstateKey`].
    /// This is a convenience method, which simply wraps the type-specific result of an appropriate
    /// `*_from_db_sort_key()` method into a [`SubstateKey`].
    ///
    /// ## Examples
    /// ```ignore
    /// Mapper::from_db_sort_key::<FieldKey>(&db_sort_key);
    /// Mapper::from_db_sort_key::<MapKey>(&db_sort_key);
    /// Mapper::from_db_sort_key::<SortedKey>(&db_sort_key);
    /// ```
    fn from_db_sort_key<K: SubstateKeyContent>(db_sort_key: &DbSortKey) -> SubstateKey {
        K::from_db_key::<Self>(db_sort_key).into_typed_key()
    }

    /// ## Examples
    /// ```ignore
    /// Mapper::from_db_sort_key_to_inner::<FieldKey>(&db_sort_key);
    /// Mapper::from_db_sort_key_to_inner::<MapKey>(&db_sort_key);
    /// Mapper::from_db_sort_key_to_inner::<SortedKey>(&db_sort_key);
    /// ```
    fn from_db_sort_key_to_inner<K: SubstateKeyContent>(db_sort_key: &DbSortKey) -> K {
        K::from_db_key::<Self>(db_sort_key)
    }

    // Type-specific methods for mapping the `SubstateKey` inner data to/from `DbSortKey`:

    fn field_to_db_sort_key(field_key: &FieldKey) -> DbSortKey;
    fn field_from_db_sort_key(db_sort_key: &DbSortKey) -> FieldKey;

    fn map_to_db_sort_key(map_key: &MapKey) -> DbSortKey;
    fn map_from_db_sort_key(db_sort_key: &DbSortKey) -> MapKey;

    fn sorted_to_db_sort_key(sorted_key: &SortedKey) -> DbSortKey;
    fn sorted_from_db_sort_key(db_sort_key: &DbSortKey) -> SortedKey;
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
/// - the ReNode part of DB Partition key
/// - the Map key and Sorted key flavors of SubstateKey
pub struct SpreadPrefixKeyMapper;

impl DatabaseKeyMapper for SpreadPrefixKeyMapper {
    fn to_db_node_key(node_id: &NodeId) -> DbNodeKey {
        SpreadPrefixKeyMapper::to_hash_prefixed(node_id.as_bytes())
    }

    fn from_db_node_key(db_node_key: &DbNodeKey) -> NodeId {
        NodeId(copy_u8_array(SpreadPrefixKeyMapper::from_hash_prefixed(
            db_node_key,
        )))
    }

    fn to_db_partition_num(partition_num: PartitionNumber) -> DbPartitionNum {
        partition_num.0
    }

    fn from_db_partition_num(db_partition_num: DbPartitionNum) -> PartitionNumber {
        PartitionNumber(db_partition_num)
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

    fn sorted_to_db_sort_key(sorted_key: &SortedKey) -> DbSortKey {
        DbSortKey(
            [
                sorted_key.0.as_slice(),
                &SpreadPrefixKeyMapper::to_hash_prefixed(&sorted_key.1),
            ]
            .concat(),
        )
    }

    fn sorted_from_db_sort_key(db_sort_key: &DbSortKey) -> SortedKey {
        (
            copy_u8_array(&db_sort_key.0[..2]),
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

    /// Returns the given bytes prefixed by their known-length hash (see [`Self::HASHED_PREFIX_LENGTH`]).
    fn to_hash_prefixed(plain_bytes: &[u8]) -> Vec<u8> {
        let hashed_prefix = &hash(plain_bytes).0[..Self::HASHED_PREFIX_LENGTH];
        [hashed_prefix, plain_bytes].concat()
    }

    /// Returns the given slice without its known-length hash prefix (see [`HASHED_PREFIX_LENGTH`]).
    fn from_hash_prefixed(prefixed_bytes: &[u8]) -> &[u8] {
        &prefixed_bytes[Self::HASHED_PREFIX_LENGTH..]
    }
}

// Internal-only trait enabling the concrete `DatabaseKeyMapper` implementations to drive their
// logic by `SubstateKey`'s inner data type.

pub trait SubstateKeyContent: Sized + 'static {
    fn from_db_key<M: DatabaseKeyMapper + ?Sized>(db_sort_key: &DbSortKey) -> Self;
    fn into_typed_key(self) -> SubstateKey;
}

impl SubstateKeyContent for MapKey {
    fn from_db_key<M: DatabaseKeyMapper + ?Sized>(db_sort_key: &DbSortKey) -> Self {
        M::map_from_db_sort_key(db_sort_key)
    }

    fn into_typed_key(self) -> SubstateKey {
        SubstateKey::Map(self)
    }
}

impl SubstateKeyContent for FieldKey {
    fn from_db_key<M: DatabaseKeyMapper + ?Sized>(db_sort_key: &DbSortKey) -> Self {
        M::field_from_db_sort_key(db_sort_key)
    }

    fn into_typed_key(self) -> SubstateKey {
        SubstateKey::Field(self)
    }
}

impl SubstateKeyContent for SortedKey {
    fn from_db_key<M: DatabaseKeyMapper + ?Sized>(db_sort_key: &DbSortKey) -> Self {
        M::sorted_from_db_sort_key(db_sort_key)
    }

    fn into_typed_key(self) -> SubstateKey {
        SubstateKey::Sorted(self)
    }
}
