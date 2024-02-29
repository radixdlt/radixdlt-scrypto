use itertools::Itertools;
use radix_engine_common::prelude::*;
use substate_store_interface::interface::*;

pub struct SubstateDatabaseStaging<'s, S> {
    /// The database overlay. All commits made to the database are written to the overlay. This
    /// covers new values and deletions too.
    overlay: DatabaseUpdates,

    /// The root database that this type overlays. There is no restriction on what type this is at
    /// this level. This is an immutable reference to a substate store as there is no need for this
    /// type to own the underlying database or to mutate it in any way.
    root: &'s S,
}

impl<'s, S> SubstateDatabaseStaging<'s, S> {
    pub fn new(root_database: &'s S) -> Self {
        Self {
            overlay: Default::default(),
            root: root_database,
        }
    }

    pub fn root(&self) -> &S {
        self.root
    }

    pub fn into_overlay(self) -> DatabaseUpdates {
        self.overlay
    }

    pub fn overlay_ref(&self) -> &DatabaseUpdates {
        &self.overlay
    }
}

impl<'s, S> SubstateDatabase for SubstateDatabaseStaging<'s, S>
where
    S: SubstateDatabase,
{
    fn get_substate(
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
            Some(NodeDatabaseUpdates { partition_updates }) => {
                match partition_updates.get(partition_num) {
                    // This partition has some data written to the overlay
                    Some(PartitionDatabaseUpdates::Delta { substate_updates }) => {
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
                    Some(PartitionDatabaseUpdates::Reset {
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
            OverlayLookupResult::NotFound => self.root.get_substate(partition_key, sort_key),
        }
    }

    fn list_entries_from(
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
        match self.overlay.node_updates.get(node_key) {
            // There is a partition update in the overlay.
            Some(NodeDatabaseUpdates { partition_updates }) => {
                match partition_updates.get(partition_num) {
                    // The partition was reset. None of the substates of this partition that exist
                    // in the root store "exist" anymore. We just need an iterator over the new
                    // substates in the reset action.
                    Some(PartitionDatabaseUpdates::Reset {
                        new_substate_values,
                    }) => Box::new(
                        new_substate_values
                            .iter()
                            .map(|(sort_key, substate_value)| {
                                (sort_key.clone(), substate_value.clone())
                            })
                            .sorted(),
                    ),
                    // There are some changes that need to be overlayed.
                    Some(PartitionDatabaseUpdates::Delta { substate_updates }) => {
                        let underlying = self.root.list_entries_from(partition_key, from_sort_key);
                        let overlaying = substate_updates
                            .iter()
                            .map(|(sort_key, database_update)| match database_update {
                                DatabaseUpdate::Set(substate_value) => {
                                    (sort_key.clone(), Some(substate_value.clone()))
                                }
                                DatabaseUpdate::Delete => (sort_key.clone(), None),
                            })
                            .sorted_by(|(sort_key_a, _), (sort_key_b, _)| {
                                sort_key_a.cmp(sort_key_b)
                            });
                        Box::new(OverlayingIterator::new(underlying, overlaying))
                    }
                    // Overlay doesn't contain anything for the provided partition number. Return an
                    // iterator over the data in the root store.
                    None => self.root.list_entries_from(partition_key, from_sort_key),
                }
            }
            // Overlay doesn't contain anything for the provided node key. Return an iterator over
            // the data in the root store.
            None => self.root.list_entries_from(partition_key, from_sort_key),
        }
    }
}

impl<'s, S> CommittableSubstateDatabase for SubstateDatabaseStaging<'s, S> {
    fn commit(&mut self, database_updates: &DatabaseUpdates) {
        merge_database_updates(&mut self.overlay, database_updates.clone())
    }
}

impl<'s, S> ListableSubstateDatabase for SubstateDatabaseStaging<'s, S>
where
    S: ListableSubstateDatabase,
{
    fn list_partition_keys(&self) -> Box<dyn Iterator<Item = DbPartitionKey> + '_> {
        Box::new(
            self.root
                .list_partition_keys()
                .chain(self.overlay.node_updates.iter().flat_map(
                    |(node_key, NodeDatabaseUpdates { partition_updates })| {
                        partition_updates
                            .keys()
                            .map(|partition_num| DbPartitionKey {
                                node_key: node_key.clone(),
                                partition_num: *partition_num,
                            })
                    },
                ))
                .unique()
                .sorted(),
        )
    }
}

pub enum OverlayLookupResult<T> {
    Found(T),
    NotFound,
}

fn merge_database_updates(this: &mut DatabaseUpdates, other: DatabaseUpdates) {
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
            Some(NodeDatabaseUpdates {
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
                                    PartitionDatabaseUpdates::Delta {
                                        substate_updates: this_substate_updates,
                                    },
                                    PartitionDatabaseUpdates::Delta {
                                        substate_updates: other_substate_updates,
                                    },
                                ) => this_substate_updates.extend(other_substate_updates),
                                // We need to apply the delta on the reset. The strategy will be as
                                // follows.
                                (
                                    PartitionDatabaseUpdates::Reset {
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
                                    *this_partition_database_updates = other_partition_database_updates;
                                }
                            }
                        }
                        // The partition num does not exist in `this` database updates. This merge
                        // is simple, just insert it.
                        None => {
                            this_partition_updates
                                .insert(other_partition_num, other_partition_database_updates);
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
                    },
                );
            }
        }
    }
}
