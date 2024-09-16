use crate::db_key_mapper::*;
use radix_common::prelude::*;

pub type DbNodeKey = Vec<u8>;

pub type DbPartitionNum = u8;

/// A database-level key of an entire partition.
/// Seen from the higher-level API: it represents a pair (RE Node ID, Module ID).
/// Seen from the lower-level implementation: it is used as a key in the upper-layer tree of our
/// two-layered JMT.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd, Sbor)]
pub struct DbPartitionKey {
    pub node_key: DbNodeKey,
    pub partition_num: DbPartitionNum,
}

/// A database-level key of a substate within a known partition.
/// Seen from the higher-level API: it represents a local Substate Key.
/// Seen from the lower-level implementation: it is used as a key in the Substate-Tier JMT.
#[derive(Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd, Sbor)]
pub struct DbSortKey(pub Vec<u8>);

/// A fully-specified key of a substate (i.e. specifying its partition and sort key).
pub type DbSubstateKey = (DbPartitionKey, DbSortKey);

/// A key-value entry of a substate within a known partition.
pub type PartitionEntry = (DbSortKey, DbSubstateValue);

pub trait CreateDatabaseUpdates {
    type DatabaseUpdates;

    /// Uses the default [`DatabaseKeyMapper`], [`SpreadPrefixKeyMapper`], to express self using database-level key encoding.
    fn create_database_updates(&self) -> Self::DatabaseUpdates {
        self.create_database_updates_with_mapper::<SpreadPrefixKeyMapper>()
    }

    /// Uses the given [`DatabaseKeyMapper`] to express self using database-level key encoding.
    fn create_database_updates_with_mapper<M: DatabaseKeyMapper>(&self) -> Self::DatabaseUpdates;
}

/// A canonical description of all database updates to be applied.
/// Note: this struct can be migrated to an enum if we ever have a need for database-wide batch
/// changes (see [`PartitionDatabaseUpdates`] enum).
#[derive(Debug, Clone, PartialEq, Eq, Sbor, Default)]
pub struct DatabaseUpdates {
    /// Node-level updates.
    pub node_updates: IndexMap<DbNodeKey, NodeDatabaseUpdates>,
}

impl DatabaseUpdates {
    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.node_updates
            .keys()
            .map(|key| SpreadPrefixKeyMapper::from_db_node_key(key))
    }
}

impl CreateDatabaseUpdates for StateUpdates {
    type DatabaseUpdates = DatabaseUpdates;

    fn create_database_updates_with_mapper<M: DatabaseKeyMapper>(&self) -> DatabaseUpdates {
        DatabaseUpdates {
            node_updates: self
                .by_node
                .iter()
                .map(|(node_id, node_state_updates)| {
                    (
                        M::to_db_node_key(node_id),
                        node_state_updates.create_database_updates_with_mapper::<M>(),
                    )
                })
                .collect(),
        }
    }
}

/// A canonical description of specific Node's updates to be applied.
/// Note: this struct can be migrated to an enum if we ever have a need for Node-wide batch changes
/// (see [`PartitionDatabaseUpdates`] enum).
#[derive(Debug, Clone, PartialEq, Eq, Sbor, Default)]
pub struct NodeDatabaseUpdates {
    /// Partition-level updates.
    pub partition_updates: IndexMap<DbPartitionNum, PartitionDatabaseUpdates>,
}

impl CreateDatabaseUpdates for NodeStateUpdates {
    type DatabaseUpdates = NodeDatabaseUpdates;

    fn create_database_updates_with_mapper<M: DatabaseKeyMapper>(&self) -> NodeDatabaseUpdates {
        match self {
            NodeStateUpdates::Delta { by_partition } => NodeDatabaseUpdates {
                partition_updates: by_partition
                    .iter()
                    .map(|(partition_num, partition_state_updates)| {
                        (
                            M::to_db_partition_num(*partition_num),
                            partition_state_updates.create_database_updates_with_mapper::<M>(),
                        )
                    })
                    .collect(),
            },
        }
    }
}

/// A canonical description of specific Partition's updates to be applied.
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum PartitionDatabaseUpdates {
    /// A delta change, touching just selected substates.
    Delta {
        substate_updates: IndexMap<DbSortKey, DatabaseUpdate>,
    },

    /// A reset, dropping all Substates of a partition and replacing them with a new set.
    Reset {
        new_substate_values: IndexMap<DbSortKey, DbSubstateValue>,
    },
}

impl PartitionDatabaseUpdates {
    /// Returns an effective change applied to the given Substate by this Partition update.
    /// May return [`None`] only if the Substate was unaffected.
    ///
    /// This method is useful for index-updating logic which does not care about the nature of the
    /// Partition update (i.e. delta vs reset).
    pub fn get_substate_change(&self, sort_key: &DbSortKey) -> Option<DatabaseUpdateRef> {
        match self {
            Self::Delta { substate_updates } => {
                substate_updates.get(sort_key).map(|update| match update {
                    DatabaseUpdate::Set(value) => DatabaseUpdateRef::Set(value),
                    DatabaseUpdate::Delete => DatabaseUpdateRef::Delete,
                })
            }
            Self::Reset {
                new_substate_values,
            } => new_substate_values
                .get(sort_key)
                .map(|value| DatabaseUpdateRef::Set(value))
                .or_else(|| Some(DatabaseUpdateRef::Delete)),
        }
    }
}

impl CreateDatabaseUpdates for PartitionStateUpdates {
    type DatabaseUpdates = PartitionDatabaseUpdates;

    fn create_database_updates_with_mapper<M: DatabaseKeyMapper>(
        &self,
    ) -> PartitionDatabaseUpdates {
        match self {
            PartitionStateUpdates::Delta { by_substate } => PartitionDatabaseUpdates::Delta {
                substate_updates: by_substate
                    .iter()
                    .map(|(key, update)| (M::to_db_sort_key(key), update.clone()))
                    .collect(),
            },
            PartitionStateUpdates::Batch(batch) => batch.create_database_updates_with_mapper::<M>(),
        }
    }
}

impl CreateDatabaseUpdates for BatchPartitionStateUpdate {
    type DatabaseUpdates = PartitionDatabaseUpdates;

    fn create_database_updates_with_mapper<M: DatabaseKeyMapper>(
        &self,
    ) -> PartitionDatabaseUpdates {
        match self {
            BatchPartitionStateUpdate::Reset {
                new_substate_values,
            } => PartitionDatabaseUpdates::Reset {
                new_substate_values: new_substate_values
                    .iter()
                    .map(|(key, value)| (M::to_db_sort_key(key), value.clone()))
                    .collect(),
            },
        }
    }
}

impl Default for PartitionDatabaseUpdates {
    fn default() -> Self {
        Self::Delta {
            substate_updates: index_map_new(),
        }
    }
}

impl DatabaseUpdates {
    /// Constructs an instance from the given legacy representation (a map of maps), which is only
    /// capable of specifying "deltas" (i.e. individual substate changes; no partition deletes).
    ///
    /// Note: This method is only meant for tests/demos - with regular Engine usage, the
    /// [`DatabaseUpdates`] can be obtained directly from the receipt.
    pub fn from_delta_maps(
        maps: IndexMap<DbPartitionKey, IndexMap<DbSortKey, DatabaseUpdate>>,
    ) -> DatabaseUpdates {
        let mut database_updates = DatabaseUpdates::default();
        for (
            DbPartitionKey {
                node_key,
                partition_num,
            },
            substate_updates,
        ) in maps
        {
            database_updates
                .node_updates
                .entry(node_key)
                .or_default()
                .partition_updates
                .insert(
                    partition_num,
                    PartitionDatabaseUpdates::Delta { substate_updates },
                );
        }
        database_updates
    }
}

/// A read interface between Track and a database vendor.
pub trait SubstateDatabase {
    /// Reads a substate value by its partition and sort key, or [`Option::None`] if missing.
    ///
    /// ## Alternatives
    ///
    /// It's likely easier to use the [`read_substate`][SubstateDatabaseExtensions::read_substate] or
    /// [`read_substate_typed`][SubstateDatabaseExtensions::read_substate_typed] methods instead.
    /// These should exist on the substate database as long as the [`SubstateDatabaseExtensions`] trait
    /// is in scope.
    fn get_substate(
        &self,
        partition_key: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue>;

    /// Iterates over all entries of the given partition (starting either from the beginning, or
    /// from the given [`DbSortKey`]), in a lexicographical order (ascending) of the [`DbSortKey`]s.
    /// Note: If the exact given starting key does not exist, the iteration starts with its
    /// immediate successor.
    ///
    /// ## Alternatives
    ///
    /// There are lots of methods starting `read_` which allow reading entries more easily.
    /// These methods are present as long as the [`SubstateDatabaseExtensions`] trait is in scope.
    fn list_entries_from(
        &self,
        partition_key: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_>;

    /// Iterates over all entries of the given partition, in a lexicographical order (ascending)
    /// of the [`DbSortKey`]s.
    /// This is a convenience method, equivalent to [`Self::list_entries_from()`] with the starting
    /// key set to [`None`].
    ///
    /// ## Alternatives
    ///
    /// There are lots of methods starting `read_` which allow reading entries more easily.
    /// These methods are present as long as the [`SubstateDatabaseExtensions`] trait is in scope.
    fn list_entries(
        &self,
        partition_key: &DbPartitionKey,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        self.list_entries_from(partition_key, None)
    }
}

impl<T: SubstateDatabase + ?Sized> SubstateDatabaseExtensions for T {}

/// These are a separate trait so that [`SubstateDatabase`] stays object-safe,
/// and can be used as `dyn SubstateDatabase`.
///
/// Generic parameters aren't permitted on object-safe traits.
pub trait SubstateDatabaseExtensions: SubstateDatabase {
    fn db_partition_key(
        node_id: impl AsRef<NodeId>,
        partition_number: PartitionNumber,
    ) -> DbPartitionKey {
        SpreadPrefixKeyMapper::to_db_partition_key(node_id.as_ref(), partition_number)
    }

    fn db_sort_key<'a>(substate_key: impl ResolvableSubstateKey<'a>) -> DbSortKey {
        SpreadPrefixKeyMapper::to_db_sort_key_from_ref(
            substate_key.into_substate_key_or_ref().as_ref(),
        )
    }

    fn optional_db_sort_key<'a>(
        optional_substate_key: impl ResolvableOptionalSubstateKey<'a>,
    ) -> Option<DbSortKey> {
        optional_substate_key
            .into_optional_substate_key_or_ref()
            .map(|key_or_ref| SpreadPrefixKeyMapper::to_db_sort_key_from_ref(key_or_ref.as_ref()))
    }

    /// Reads the substate using the default [`SpreadPrefixKeyMapper`].
    ///
    /// ## Example use:
    /// ```ignore
    /// let is_bootstrapped = store.read_substate(
    ///     PACKAGE_PACKAGE,
    ///     TYPE_INFO_FIELD_PARTITION,
    ///     TypeInfoField::TypeInfo,
    /// ).is_some();
    /// ```
    fn read_substate<'a>(
        &self,
        node_id: impl AsRef<NodeId>,
        partition_number: PartitionNumber,
        substate_key: impl ResolvableSubstateKey<'a>,
    ) -> Option<Vec<u8>> {
        self.get_substate(
            &Self::db_partition_key(node_id, partition_number),
            &Self::db_sort_key(substate_key),
        )
    }

    /// Reads the substate using the default [`SpreadPrefixKeyMapper`], and then
    /// decodes the substate value. It panics if the value has a decode error.
    ///
    /// ## Example use:
    /// ```ignore
    /// let type_info_substate: TypeInfoSubstate = store.read_substate_typed(
    ///     PACKAGE_PACKAGE,
    ///     TYPE_INFO_FIELD_PARTITION,
    ///     TypeInfoField::TypeInfo,
    /// );
    /// ```
    fn read_substate_typed<'a, T: ScryptoDecode>(
        &self,
        node_id: impl AsRef<NodeId>,
        partition_number: PartitionNumber,
        substate_key: impl ResolvableSubstateKey<'a>,
    ) -> Option<T> {
        let raw = self.read_substate(node_id, partition_number, substate_key)?;
        Some(decode_value(&raw))
    }

    /// Use `from_substate_key_inclusive: None::<SubstateKey>` to read from start.
    #[inline]
    fn read_entries_unknown_key<'a>(
        &self,
        node_id: impl AsRef<NodeId>,
        partition_number: PartitionNumber,
        from_substate_key_inclusive: impl ResolvableOptionalSubstateKey<'a>,
    ) -> Box<dyn Iterator<Item = (DbSortKey, Vec<u8>)> + '_> {
        self.list_entries_from(
            &Self::db_partition_key(node_id, partition_number),
            Self::optional_db_sort_key(from_substate_key_inclusive).as_ref(),
        )
    }

    /// Use `from_substate_key_inclusive: None::<SubstateKey>` to read from start.
    fn read_entries<'a, K: SubstateKeyContent>(
        &self,
        node_id: impl AsRef<NodeId>,
        partition_number: PartitionNumber,
        from_substate_key_inclusive: impl ResolvableOptionalSubstateKey<'a>,
    ) -> Box<dyn Iterator<Item = (K, Vec<u8>)> + '_> {
        let iterable = self
            .list_entries_from(
                &Self::db_partition_key(node_id, partition_number),
                Self::optional_db_sort_key(from_substate_key_inclusive).as_ref(),
            )
            .map(|(db_sort_key, raw_value)| {
                (
                    SpreadPrefixKeyMapper::from_db_sort_key_to_inner::<K>(&db_sort_key),
                    raw_value,
                )
            });
        Box::new(iterable)
    }

    /// Use `from_substate_key_inclusive: None::<SubstateKey>` to read from start.
    fn read_entries_values_typed<'a, K: SubstateKeyContent, V: ScryptoDecode>(
        &self,
        node_id: impl AsRef<NodeId>,
        partition_number: PartitionNumber,
        from_substate_key_inclusive: impl ResolvableOptionalSubstateKey<'a>,
    ) -> Box<dyn Iterator<Item = (K, V)> + '_> {
        let iterator = self
            .read_entries_unknown_key(node_id, partition_number, from_substate_key_inclusive)
            .map(|(db_sort_key, raw_value)| {
                (
                    SpreadPrefixKeyMapper::from_db_sort_key_to_inner::<K>(&db_sort_key),
                    decode_value::<V>(&raw_value),
                )
            });
        Box::new(iterator)
    }

    /// Leaves the key and value raw.
    /// Use `from_substate_key_inclusive: None::<SubstateKey>` to read from start.
    fn read_map_entries<'a>(
        &self,
        node_id: impl AsRef<NodeId>,
        partition_number: PartitionNumber,
        from_substate_key_inclusive: impl ResolvableOptionalSubstateKey<'a>,
    ) -> Box<dyn Iterator<Item = (MapKey, Vec<u8>)> + '_> {
        self.read_entries::<MapKey>(node_id, partition_number, from_substate_key_inclusive)
    }

    /// Decodes just the value, leaving the key raw.
    /// Use `from_substate_key_inclusive: None::<SubstateKey>` to read from start.
    fn read_map_entries_values_typed<'a, V: ScryptoDecode>(
        &self,
        node_id: impl AsRef<NodeId>,
        partition_number: PartitionNumber,
        from_substate_key_inclusive: impl ResolvableOptionalSubstateKey<'a>,
    ) -> Box<dyn Iterator<Item = (MapKey, V)> + '_> {
        self.read_entries_values_typed::<MapKey, V>(
            node_id,
            partition_number,
            from_substate_key_inclusive,
        )
    }

    /// Decodes both the key and value.
    /// Use `from_substate_key_inclusive: None::<SubstateKey>` to read from start.
    fn read_map_entries_typed<'a, K: ScryptoDecode, V: ScryptoDecode>(
        &self,
        node_id: impl AsRef<NodeId>,
        partition_number: PartitionNumber,
        from_substate_key_inclusive: impl ResolvableOptionalSubstateKey<'a>,
    ) -> Box<dyn Iterator<Item = (K, V)> + '_> {
        let iterator = self
            .read_map_entries(node_id, partition_number, from_substate_key_inclusive)
            .map(|(raw_key, raw_value)| (decode_key::<K>(&raw_key), decode_value::<V>(&raw_value)));
        Box::new(iterator)
    }

    /// Use `from_substate_key_inclusive: None::<SubstateKey>` to read from start.
    fn read_sorted_entries<'a>(
        &self,
        node_id: impl AsRef<NodeId>,
        partition_number: PartitionNumber,
        from_substate_key_inclusive: impl ResolvableOptionalSubstateKey<'a>,
    ) -> Box<dyn Iterator<Item = (SortedKey, Vec<u8>)> + '_> {
        self.read_entries::<SortedKey>(node_id, partition_number, from_substate_key_inclusive)
    }

    /// Decodes just the value, leaving the key raw.
    /// Use `from_substate_key_inclusive: None::<SubstateKey>` to read from start.
    fn read_sorted_entries_values_typed<'a, V: ScryptoDecode>(
        &self,
        node_id: impl AsRef<NodeId>,
        partition_number: PartitionNumber,
        from_substate_key_inclusive: impl ResolvableOptionalSubstateKey<'a>,
    ) -> Box<dyn Iterator<Item = (SortedKey, V)> + '_> {
        self.read_entries_values_typed::<SortedKey, V>(
            node_id,
            partition_number,
            from_substate_key_inclusive,
        )
    }
}

fn decode_key<K: ScryptoDecode>(raw: &[u8]) -> K {
    scrypto_decode::<K>(&raw).unwrap_or_else(|err| {
        panic!(
            "Expected key to be decodable as {}. Error: {:?}.",
            core::any::type_name::<K>(),
            err,
        )
    })
}

fn decode_value<V: ScryptoDecode>(raw: &[u8]) -> V {
    scrypto_decode::<V>(&raw).unwrap_or_else(|err| {
        panic!(
            "Expected value to be decodable as {}. Error: {:?}.",
            core::any::type_name::<V>(),
            err,
        )
    })
}

/// A write interface between Track and a database vendor.
pub trait CommittableSubstateDatabase {
    /// Commits state changes to the database.
    fn commit(&mut self, database_updates: &DatabaseUpdates);
}

impl<T: CommittableSubstateDatabase + ?Sized> CommittableSubstateDatabaseExtensions for T {}

pub trait CommittableSubstateDatabaseExtensions: CommittableSubstateDatabase {
    fn update_substate<'a>(
        &mut self,
        node_id: impl AsRef<NodeId>,
        partition_number: PartitionNumber,
        substate_key: impl ResolvableSubstateKey<'a>,
        value: Vec<u8>,
    ) {
        self.commit(&DatabaseUpdates::from_delta_maps(indexmap!(
            SpreadPrefixKeyMapper::to_db_partition_key(
                node_id.as_ref(),
                partition_number,
            ) => indexmap!(
                SpreadPrefixKeyMapper::to_db_sort_key_from_ref(
                    substate_key.into_substate_key_or_ref().as_ref(),
                ) => DatabaseUpdate::Set(
                    value
                )
            )
        )))
    }

    fn delete_substate<'a>(
        &mut self,
        node_id: impl AsRef<NodeId>,
        partition_number: PartitionNumber,
        substate_key: impl ResolvableSubstateKey<'a>,
    ) {
        self.commit(&DatabaseUpdates::from_delta_maps(indexmap!(
            SpreadPrefixKeyMapper::to_db_partition_key(
                node_id.as_ref(),
                partition_number,
            ) => indexmap!(
                SpreadPrefixKeyMapper::to_db_sort_key_from_ref(
                    substate_key.into_substate_key_or_ref().as_ref(),
                ) => DatabaseUpdate::Delete,
            )
        )))
    }

    fn update_substate_typed<'a, E: ScryptoEncode>(
        &mut self,
        node_id: impl AsRef<NodeId>,
        partition_number: PartitionNumber,
        substate_key: impl ResolvableSubstateKey<'a>,
        value: E,
    ) {
        let encoded_value = scrypto_encode(&value).unwrap_or_else(|err| {
            panic!(
                "Expected value to be encodable as {}. Error: {:?}.",
                core::any::type_name::<E>(),
                err,
            )
        });
        self.update_substate(node_id, partition_number, substate_key, encoded_value)
    }
}

/// A partition listing interface between Track and a database vendor.
pub trait ListableSubstateDatabase {
    /// Iterates over all partition keys, in an arbitrary order.
    ///
    /// ## Alternatives
    /// You likely want to use the [`read_partition_keys`][ListableSubstateDatabaseExtensions::read_partition_keys]
    /// method instead, which returns an unmapped key. This is available if
    /// the trait [`ListableSubstateDatabaseExtensions`] is in scope.
    fn list_partition_keys(&self) -> Box<dyn Iterator<Item = DbPartitionKey> + '_>;
}

impl<T: ListableSubstateDatabase + ?Sized> ListableSubstateDatabaseExtensions for T {}

/// These are a separate trait so that [`ListableSubstateDatabase`] stays object-safe,
/// and can be used as `dyn ListableSubstateDatabase`.
///
/// Generic parameters aren't permitted on object-safe traits.
pub trait ListableSubstateDatabaseExtensions: ListableSubstateDatabase {
    fn read_partition_keys(&self) -> Box<dyn Iterator<Item = (NodeId, PartitionNumber)> + '_> {
        let iterator = self
            .list_partition_keys()
            .map(|key| SpreadPrefixKeyMapper::from_db_partition_key(&key));
        Box::new(iterator)
    }
}
