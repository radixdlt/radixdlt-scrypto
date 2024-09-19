use radix_common::prelude::*;
use radix_rust::prelude::borrow::*;
use radix_substate_store_interface::interface::*;

pub type UnmergeableSubstateDatabaseOverlay<'a, S> = SubstateDatabaseOverlay<&'a S, S>;
pub type MergeableSubstateDatabaseOverlay<'a, S> = SubstateDatabaseOverlay<&'a mut S, S>;
pub type OwnedSubstateDatabaseOverlay<S> = SubstateDatabaseOverlay<S, S>;

pub struct SubstateDatabaseOverlay<S, D> {
    /// The database overlay. All commits made to the database are written to the overlay. This
    /// covers new values and deletions too.
    overlay: StagingDatabaseUpdates,

    /// A mutable or immutable reference to the root database that this type overlays.
    /// It only needs to be mutable if you wish to commit to the root store.
    /// To be useful, `S` should implement at least `Borrow<D>`.
    root: S,

    /// The concrete type of the underlying substate database.
    substate_database_type: PhantomData<D>,
}

impl<'a, D> UnmergeableSubstateDatabaseOverlay<'a, D> {
    pub fn new_unmergeable(root_database: &'a D) -> Self {
        Self::new(root_database)
    }
}

impl<'a, D> MergeableSubstateDatabaseOverlay<'a, D> {
    pub fn new_mergeable(root_database: &'a mut D) -> Self {
        Self::new(root_database)
    }
}

impl<D> OwnedSubstateDatabaseOverlay<D> {
    pub fn new_owned(root_database: D) -> Self {
        Self::new(root_database)
    }
}

impl<S, D> SubstateDatabaseOverlay<S, D> {
    pub fn new(root_database: S) -> Self {
        Self {
            overlay: Default::default(),
            root: root_database,
            substate_database_type: PhantomData,
        }
    }

    pub fn deconstruct(self) -> (S, DatabaseUpdates) {
        (self.root, self.overlay.into())
    }

    pub fn database_updates(&self) -> DatabaseUpdates {
        self.overlay.clone().into()
    }

    pub fn into_database_updates(self) -> DatabaseUpdates {
        self.overlay.into()
    }
}

impl<S: Borrow<D>, D> SubstateDatabaseOverlay<S, D> {
    fn get_readable_root(&self) -> &D {
        self.root.borrow()
    }
}

impl<S: BorrowMut<D>, D> SubstateDatabaseOverlay<S, D> {
    fn get_writable_root(&mut self) -> &mut D {
        self.root.borrow_mut()
    }
}

impl<S: BorrowMut<D>, D: CommittableSubstateDatabase> SubstateDatabaseOverlay<S, D> {
    pub fn commit_overlay_into_root_store(&mut self) {
        let overlay = mem::replace(&mut self.overlay, StagingDatabaseUpdates::default());
        self.get_writable_root().commit(&overlay.into());
    }
}

impl<S: Borrow<D>, D: SubstateDatabase> SubstateDatabase for SubstateDatabaseOverlay<S, D> {
    fn get_raw_substate_by_db_key(
        &self,
        partition_key @ DbPartitionKey {
            node_key,
            partition_num,
        }: &DbPartitionKey,
        sort_key: &DbSortKey,
    ) -> Option<DbSubstateValue> {
        let overlay_lookup_result = match self.overlay.node_updates.get(node_key) {
            // This particular node key exists in the overlay and probably has some partitions
            // written to the overlay.
            Some(StagingNodeDatabaseUpdates { partition_updates }) => {
                match partition_updates.get(partition_num) {
                    // This partition has some data written to the overlay
                    Some(StagingPartitionDatabaseUpdates::Delta { substate_updates }) => {
                        match substate_updates.get(sort_key) {
                            // The substate value is written to the overlay. It is a database set
                            // so we return the new value.
                            Some(DatabaseUpdate::Set(substate_value)) => {
                                OverlayLookupResult::Found(Some(substate_value))
                            }
                            // The substate value is written to the overlay. It is a database delete
                            // so we return a `Found(None)`.
                            Some(DatabaseUpdate::Delete) => OverlayLookupResult::Found(None),
                            // This particular substate was not written to the overlay and should be
                            // read from the underlying database.
                            None => OverlayLookupResult::NotFound,
                        }
                    }
                    Some(StagingPartitionDatabaseUpdates::Reset {
                        new_substate_values,
                    }) => match new_substate_values.get(sort_key) {
                        // The substate value is written to the overlay.
                        Some(substate_value) => OverlayLookupResult::Found(Some(substate_value)),
                        // In a partition reset we delete all substates in a partition and can also
                        // write new substates there. If the substate that we're looking for can't
                        // be found in the new substate values of a partition delete then it is
                        // one of the deleted substates. Therefore, the following will report that
                        // it has found the substate value in the overlay and that the substate
                        // does not exist.
                        None => OverlayLookupResult::Found(None),
                    },
                    // This particular partition for the specified node key does not exist in the
                    // overlay and should be read from the underlying database.
                    None => OverlayLookupResult::NotFound,
                }
            }
            // This particular node key does not exist in the overlay. The substate must be read
            // from the underlying database.
            None => OverlayLookupResult::NotFound,
        };

        match overlay_lookup_result {
            OverlayLookupResult::Found(substate_value) => substate_value.cloned(),
            OverlayLookupResult::NotFound => self
                .get_readable_root()
                .get_raw_substate_by_db_key(partition_key, sort_key),
        }
    }

    fn list_raw_values_from_db_key(
        &self,
        partition_key @ DbPartitionKey {
            node_key,
            partition_num,
        }: &DbPartitionKey,
        from_sort_key: Option<&DbSortKey>,
    ) -> Box<dyn Iterator<Item = PartitionEntry> + '_> {
        // This function iterates over entries of the specified partition. Therefore, we don't need
        // to think about other partitions here. We first check if there are any partition updates
        // for the specified partition. If there is not, no overlaying is needed and we can just
        // return the iterator of the root store.
        let from_sort_key = from_sort_key.cloned();
        match self.overlay.node_updates.get(node_key) {
            // There is a partition update in the overlay.
            Some(StagingNodeDatabaseUpdates { partition_updates }) => {
                match partition_updates.get(partition_num) {
                    // The partition was reset. None of the substates of this partition that exist
                    // in the root store "exist" anymore. We just need an iterator over the new
                    // substates in the reset action.
                    Some(StagingPartitionDatabaseUpdates::Reset {
                        new_substate_values,
                    }) => {
                        match from_sort_key {
                            // A `from_sort_key` is specified. Only return sort keys that are larger
                            // than or equal to the from sort key. We do this through BTreeMap's
                            // range function instead of doing filtering. We're able to do this
                            // since a `BTreeMap`'s keys are always sorted.
                            Some(from_sort_key) => {
                                Box::new(new_substate_values.range(from_sort_key..).map(
                                    |(sort_key, substate_value)| {
                                        (sort_key.clone(), substate_value.clone())
                                    },
                                ))
                            }
                            // No `from_sort_key` is specified. Start iterating from the beginning.
                            None => Box::new(new_substate_values.iter().map(
                                |(sort_key, substate_value)| {
                                    (sort_key.clone(), substate_value.clone())
                                },
                            )),
                        }
                    }
                    // There are some changes that need to be overlayed.
                    Some(StagingPartitionDatabaseUpdates::Delta { substate_updates }) => {
                        let underlying = self
                            .get_readable_root()
                            .list_raw_values_from_db_key(partition_key, from_sort_key.as_ref());

                        match from_sort_key {
                            // A `from_sort_key` is specified. Only return sort keys that are larger
                            // than or equal to the from sort key. We do this through BTreeMap's
                            // range function instead of doing filtering. We're able to do this
                            // since a `BTreeMap`'s keys are always sorted.
                            Some(from_sort_key) => {
                                let overlaying = substate_updates.range(from_sort_key..).map(
                                    |(sort_key, database_update)| match database_update {
                                        DatabaseUpdate::Set(substate_value) => {
                                            (sort_key.clone(), Some(substate_value.clone()))
                                        }
                                        DatabaseUpdate::Delete => (sort_key.clone(), None),
                                    },
                                );
                                Box::new(OverlayingIterator::new(underlying, overlaying))
                            }
                            // No `from_sort_key` is specified. Start iterating from the beginning.
                            None => {
                                let overlaying =
                                    substate_updates.iter().map(|(sort_key, database_update)| {
                                        match database_update {
                                            DatabaseUpdate::Set(substate_value) => {
                                                (sort_key.clone(), Some(substate_value.clone()))
                                            }
                                            DatabaseUpdate::Delete => (sort_key.clone(), None),
                                        }
                                    });
                                Box::new(OverlayingIterator::new(underlying, overlaying))
                            }
                        }
                    }
                    // Overlay doesn't contain anything for the provided partition number. Return an
                    // iterator over the data in the root store.
                    None => self
                        .get_readable_root()
                        .list_raw_values_from_db_key(partition_key, from_sort_key.as_ref()),
                }
            }
            // Overlay doesn't contain anything for the provided node key. Return an iterator over
            // the data in the root store.
            None => self
                .get_readable_root()
                .list_raw_values_from_db_key(partition_key, from_sort_key.as_ref()),
        }
    }
}

impl<S, D> CommittableSubstateDatabase for SubstateDatabaseOverlay<S, D> {
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        merge_database_updates(&mut self.overlay, database_updates.clone())
    }
}

impl<S: Borrow<D>, D: ListableSubstateDatabase> ListableSubstateDatabase
    for SubstateDatabaseOverlay<S, D>
{
    fn list_partition_keys(&self) -> Box<dyn Iterator<Item = DbPartitionKey> + '_> {
        let overlying = self
            .overlay
            .node_updates
            .iter()
            .flat_map(
                |(node_key, StagingNodeDatabaseUpdates { partition_updates })| {
                    partition_updates
                        .keys()
                        .map(|partition_num| DbPartitionKey {
                            node_key: node_key.clone(),
                            partition_num: *partition_num,
                        })
                },
            )
            .map(|partition_key| (partition_key, Some(())));
        let underlying = self
            .get_readable_root()
            .list_partition_keys()
            .map(|partition_key| (partition_key, ()));

        Box::new(OverlayingIterator::new(underlying, overlying).map(|(value, _)| value))
    }
}

pub enum OverlayLookupResult<T> {
    Found(T),
    NotFound,
}

fn merge_database_updates(this: &mut StagingDatabaseUpdates, other: DatabaseUpdates) {
    for (
        other_node_key,
        NodeDatabaseUpdates {
            partition_updates: other_partition_updates,
        },
    ) in other.node_updates.into_iter()
    {
        // Check if the other node key exists in `this` database updates.
        match this.node_updates.get_mut(&other_node_key) {
            // The node key exists in `this` database updates.
            Some(StagingNodeDatabaseUpdates {
                partition_updates: this_partition_updates,
            }) => {
                for (other_partition_num, other_partition_database_updates) in
                    other_partition_updates.into_iter()
                {
                    // Check if the partition num exists in `this` database updates
                    match this_partition_updates.get_mut(&other_partition_num) {
                        // The partition exists in both `this` and `other` and now we must combine
                        // both the partition database updates together
                        Some(this_partition_database_updates) => {
                            match (
                                this_partition_database_updates,
                                other_partition_database_updates,
                            ) {
                                // This and other are both `Delta`. We insert all entries in the
                                // other state updates into this substate updates. This will also
                                // override anything in `this` with anything in `other`.
                                (
                                    StagingPartitionDatabaseUpdates::Delta {
                                        substate_updates: this_substate_updates,
                                    },
                                    PartitionDatabaseUpdates::Delta {
                                        substate_updates: other_substate_updates,
                                    },
                                ) => this_substate_updates.extend(other_substate_updates),
                                // We need to apply the delta on the reset. 
                                (
                                    StagingPartitionDatabaseUpdates::Reset {
                                        new_substate_values: this_new_substate_values,
                                    },
                                    PartitionDatabaseUpdates::Delta {
                                        substate_updates: other_substate_updates,
                                    },
                                ) => {
                                    for (other_sort_key, other_database_update) in
                                        other_substate_updates.into_iter()
                                    {
                                        match other_database_update {
                                            DatabaseUpdate::Set(other_substate_value) => {
                                                this_new_substate_values
                                                    .insert(other_sort_key, other_substate_value);
                                            }
                                            DatabaseUpdate::Delete => {
                                                this_new_substate_values.remove(&other_sort_key);
                                            }
                                        }
                                    }
                                }
                                // Whatever the current state is, if the other database update is
                                // a partition reset then it takes precedence.
                                (
                                    this_partition_database_updates,
                                    other_partition_database_updates @ PartitionDatabaseUpdates::Reset { .. },
                                ) => {
                                    *this_partition_database_updates = other_partition_database_updates.into();
                                }
                            }
                        }
                        // The partition num does not exist in `this` database updates. This merge
                        // is simple, just insert it.
                        None => {
                            this_partition_updates.insert(
                                other_partition_num,
                                other_partition_database_updates.into(),
                            );
                        }
                    }
                }
            }
            // The node key does not exist in `this` database updates. This merge is simple, just
            // insert it.
            None => {
                this.node_updates.insert(
                    other_node_key,
                    NodeDatabaseUpdates {
                        partition_updates: other_partition_updates,
                    }
                    .into(),
                );
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor, Default)]
struct StagingDatabaseUpdates {
    node_updates: BTreeMap<DbNodeKey, StagingNodeDatabaseUpdates>,
}

impl From<StagingDatabaseUpdates> for DatabaseUpdates {
    fn from(value: StagingDatabaseUpdates) -> Self {
        Self {
            node_updates: value
                .node_updates
                .into_iter()
                .map(|(key, value)| (key, NodeDatabaseUpdates::from(value)))
                .collect(),
        }
    }
}

impl From<DatabaseUpdates> for StagingDatabaseUpdates {
    fn from(value: DatabaseUpdates) -> Self {
        Self {
            node_updates: value
                .node_updates
                .into_iter()
                .map(|(key, value)| (key, StagingNodeDatabaseUpdates::from(value)))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor, Default)]
struct StagingNodeDatabaseUpdates {
    partition_updates: BTreeMap<DbPartitionNum, StagingPartitionDatabaseUpdates>,
}

impl From<StagingNodeDatabaseUpdates> for NodeDatabaseUpdates {
    fn from(value: StagingNodeDatabaseUpdates) -> Self {
        Self {
            partition_updates: value
                .partition_updates
                .into_iter()
                .map(|(key, value)| (key, PartitionDatabaseUpdates::from(value)))
                .collect(),
        }
    }
}

impl From<NodeDatabaseUpdates> for StagingNodeDatabaseUpdates {
    fn from(value: NodeDatabaseUpdates) -> Self {
        Self {
            partition_updates: value
                .partition_updates
                .into_iter()
                .map(|(key, value)| (key, StagingPartitionDatabaseUpdates::from(value)))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
enum StagingPartitionDatabaseUpdates {
    Delta {
        substate_updates: BTreeMap<DbSortKey, DatabaseUpdate>,
    },

    Reset {
        new_substate_values: BTreeMap<DbSortKey, DbSubstateValue>,
    },
}

impl From<StagingPartitionDatabaseUpdates> for PartitionDatabaseUpdates {
    fn from(value: StagingPartitionDatabaseUpdates) -> Self {
        match value {
            StagingPartitionDatabaseUpdates::Delta { substate_updates } => Self::Delta {
                substate_updates: substate_updates.into_iter().collect(),
            },
            StagingPartitionDatabaseUpdates::Reset {
                new_substate_values,
            } => Self::Reset {
                new_substate_values: new_substate_values.into_iter().collect(),
            },
        }
    }
}

impl From<PartitionDatabaseUpdates> for StagingPartitionDatabaseUpdates {
    fn from(value: PartitionDatabaseUpdates) -> Self {
        match value {
            PartitionDatabaseUpdates::Delta { substate_updates } => Self::Delta {
                substate_updates: substate_updates.into_iter().collect(),
            },
            PartitionDatabaseUpdates::Reset {
                new_substate_values,
            } => Self::Reset {
                new_substate_values: new_substate_values.into_iter().collect(),
            },
        }
    }
}
