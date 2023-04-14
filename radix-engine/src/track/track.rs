use std::mem;
use crate::types::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::types::*;
use radix_engine_stores::interface::{
    AcquireLockError, NodeSubstates, StateUpdate, StateUpdates, SubstateDatabase, SubstateStore,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
pub enum SubstateLockState {
    Read(usize),
    Write,
}

impl SubstateLockState {
    pub fn no_lock() -> Self {
        Self::Read(0)
    }

    pub fn is_locked(&self) -> bool {
        !matches!(self, SubstateLockState::Read(0usize))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SubstateMeta {
    New,
    Read,
    Updated,
}

#[derive(Debug)]
pub struct SubstateUpdate {
    substate: Substate,
    meta_state: SubstateMeta,
}

#[derive(Debug)]
pub struct Substate {
    value: IndexedScryptoValue,
    lock_state: SubstateLockState,
}

pub enum NodeUpdate {
    New(BTreeMap<ModuleId, BTreeMap<SubstateKey, Substate>>),
    Update(IndexMap<ModuleId, IndexMap<SubstateKey, SubstateUpdate>>),
}

/// Transaction-wide states and side effects
pub struct Track<'s> {
    substate_db: &'s dyn SubstateDatabase,
    updates: IndexMap<NodeId, NodeUpdate>,
    force_updates: IndexMap<NodeId, NodeUpdate>,

    locks: IndexMap<u32, (NodeId, ModuleId, SubstateKey, LockFlags)>,
    next_lock_id: u32,
}

impl<'s> Track<'s> {
    pub fn new(substate_db: &'s dyn SubstateDatabase) -> Self {
        Self {
            substate_db,
            force_updates: index_map_new(),
            updates: index_map_new(),
            locks: index_map_new(),
            next_lock_id: 0,
        }
    }

    fn new_lock_handle(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> u32 {
        let new_lock = self.next_lock_id;
        self.locks
            .insert(new_lock, (*node_id, module_id, substate_key.clone(), flags));
        self.next_lock_id += 1;
        new_lock
    }

    fn loaded_substate<'a>(
        updates: &'a IndexMap<NodeId, NodeUpdate>,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<&'a IndexedScryptoValue> {
        updates
            .get(node_id)
            .and_then(|update| match update {
                NodeUpdate::New(modules) => {
                    modules.get(&module_id)
                        .and_then(|substates| substates.get(substate_key))
                        .map(|s| &s.value)
                }
                NodeUpdate::Update(module_updates) => {
                    module_updates.get(&module_id)
                        .and_then(|substates| substates.get(substate_key))
                        .map(|s| &s.substate.value)
                }
            })
    }

    fn loaded_substate_mut<'a>(
        updates: &'a mut IndexMap<NodeId, NodeUpdate>,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<(&'a mut Substate, Option<&'a mut SubstateMeta>)> {
        updates
            .get_mut(node_id)
            .and_then(|update| match update {
                NodeUpdate::New(modules) => {
                    modules.get_mut(&module_id)
                        .and_then(|substates| substates.get_mut(substate_key))
                        .map(|s| (s, None))
                }
                NodeUpdate::Update(module_updates) => {
                    module_updates.get_mut(&module_id)
                        .and_then(|substates| substates.get_mut(substate_key))
                        .map(|s| (&mut s.substate, Some(&mut s.meta_state)))
                }
            })
    }

    fn load_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<IndexedScryptoValue> {
        self.substate_db
            .get_substate(node_id, module_id, substate_key)
            .expect("Database misconfigured")
            .map(|e| IndexedScryptoValue::from_vec(e).expect("Failed to decode substate"))
    }

    fn add_loaded_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        substate_value: IndexedScryptoValue,
    ) {
        let node_update = self.updates
            .entry(*node_id)
            .or_insert(NodeUpdate::Update(IndexMap::new()));

        match node_update {
            NodeUpdate::Update(update) => {
                update.entry(module_id)
                    .or_default()
                    .insert(
                        substate_key.clone(),
                        SubstateUpdate {
                            substate: Substate {
                                value: substate_value,
                                lock_state: SubstateLockState::no_lock(),
                            },
                            meta_state: SubstateMeta::Read,
                        },
                    );
            },
            NodeUpdate::New(..) => panic!("unexpected"),
        }
    }

    /// Reverts all non force write changes.
    ///
    /// Note that dependencies will never be reverted.
    pub fn revert_non_force_write_changes(&mut self) {
        let updates = mem::take(&mut self.force_updates);
        self.updates = updates;
    }

    /// Finalizes changes captured by this substate store.
    ///
    ///  Returns the state changes and dependencies.
    pub fn finalize(self) -> StateUpdates {
        // TODO:
        // - Remove version from state updates
        // - Split read,
        // - Track dependencies

        let mut substate_changes: IndexMap<(NodeId, ModuleId, SubstateKey), StateUpdate> =
            index_map_new();
        for (node_id, node_update) in self.updates {
            match node_update {
                NodeUpdate::New(substates) => {
                    for (module_id, module) in substates {
                        for (substate_key, substate) in module {
                            substate_changes.insert((node_id, module_id, substate_key.clone()), StateUpdate::Create(substate.value.into()));
                        }
                    }
                }
                NodeUpdate::Update(node_update) => {
                    for (module_id, module) in node_update {
                        for (substate_key, loaded) in module {
                            let update = match loaded.meta_state {
                                SubstateMeta::New => StateUpdate::Create(loaded.substate.value.into()),
                                SubstateMeta::Updated => {
                                    StateUpdate::Update(loaded.substate.value.into())
                                }
                                SubstateMeta::Read => {
                                    // TODO: Fix
                                    StateUpdate::Update(loaded.substate.value.into())
                                }
                            };
                            substate_changes.insert((node_id, module_id, substate_key.clone()), update);
                        }
                    }
                }
            }
        }

        StateUpdates { substate_changes }
    }
}

impl<'s> SubstateStore for Track<'s> {
    fn acquire_lock(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<u32, AcquireLockError> {
        // Load the substate from state track
        if Self::loaded_substate(&self.updates, node_id, module_id, substate_key).is_none()
        {
            let maybe_substate = self.load_substate(node_id, module_id, substate_key);
            if let Some(output) = maybe_substate {
                self.add_loaded_substate(node_id, module_id, substate_key, output);
            } else {
                return Err(AcquireLockError::NotFound(
                    *node_id,
                    module_id,
                    substate_key.clone(),
                ));
            }
        }

        // Check substate state
        let (substate, meta) =
            Self::loaded_substate_mut(&mut self.updates, node_id, module_id, substate_key)
                .unwrap();
        if flags.contains(LockFlags::UNMODIFIED_BASE) {
            match meta {
                None | Some(SubstateMeta::New) => {
                    return Err(AcquireLockError::LockUnmodifiedBaseOnNewSubstate(
                        *node_id,
                        module_id,
                        substate_key.clone(),
                    ))
                }
                Some(SubstateMeta::Updated) => {
                    return Err(AcquireLockError::LockUnmodifiedBaseOnOnUpdatedSubstate(
                        *node_id,
                        module_id,
                        substate_key.clone(),
                    ))
                }
                Some(SubstateMeta::Read) => {}
            }
        }

        // Check read/write permission
        match substate.lock_state {
            SubstateLockState::Read(n) => {
                if flags.contains(LockFlags::MUTABLE) {
                    if n != 0 {
                        return Err(AcquireLockError::SubstateLocked(
                            *node_id,
                            module_id,
                            substate_key.clone(),
                        ));
                    }
                    substate.lock_state = SubstateLockState::Write;
                } else {
                    substate.lock_state = SubstateLockState::Read(n + 1);
                }
            }
            SubstateLockState::Write => {
                return Err(AcquireLockError::SubstateLocked(
                    *node_id,
                    module_id,
                    substate_key.clone(),
                ));
            }
        }

        Ok(self.new_lock_handle(node_id, module_id, substate_key, flags))
    }

    fn release_lock(&mut self, handle: u32) {
        let (node_id, module_id, substate_key, flags) =
            self.locks.remove(&handle).expect("Invalid lock handle");

        let (substate, meta) = Self::loaded_substate_mut(
            &mut self.updates,
            &node_id,
            module_id,
            &substate_key,
        )
        .expect("Substate missing for valid lock handle");

        match substate.lock_state {
            SubstateLockState::Read(n) => {
                substate.lock_state = SubstateLockState::Read(n - 1)
            }
            SubstateLockState::Write => {
                substate.lock_state = SubstateLockState::no_lock();

                if flags.contains(LockFlags::FORCE_WRITE) {

                    match meta {
                        Some(meta @ (SubstateMeta::Read | SubstateMeta::Updated)) => {
                            *meta = SubstateMeta::Updated;
                        }
                        _ => {
                            panic!("Unexpected");
                        }
                    }

                    let node_update = self.force_updates
                        .entry(node_id)
                        .or_insert(NodeUpdate::Update(IndexMap::new()));

                    match node_update {
                        NodeUpdate::Update(update) => {
                            update.entry(module_id)
                                .or_default()
                                .insert(
                                    substate_key.clone(),
                                    SubstateUpdate {
                                        substate: Substate {
                                            value: substate.value.clone(),
                                            lock_state: SubstateLockState::no_lock(),
                                        },
                                        meta_state: SubstateMeta::Updated,
                                    },
                                );
                        },
                        NodeUpdate::New(..) => panic!("unexpected"),
                    }
                } else {
                    match meta {
                        Some(meta@ SubstateMeta::Read) => {
                            *meta = SubstateMeta::Updated;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn create_node(&mut self, node_id: NodeId, node_substates: NodeSubstates) {
        let node_runtime = node_substates.into_iter().map(|(module_id, module_substates)| {
            let module_substates = module_substates.into_iter().map(|(key, value)| (key, Substate {
                value, lock_state: SubstateLockState::no_lock(),
            })).collect();
            (module_id, module_substates)
        }).collect();

        self.updates.insert(node_id, NodeUpdate::New(node_runtime));
    }

    fn upsert_substate(
        &mut self,
        node_id: NodeId,
        module_id: ModuleId,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
    ) -> Result<(), AcquireLockError> {
        let substate = Substate {
            value: substate_value,
            lock_state: SubstateLockState::no_lock(),
        };
        let node_update = self.updates.entry(node_id).or_insert(NodeUpdate::Update(IndexMap::new()));
        let prev = match node_update {
            NodeUpdate::New(substates) => {
                substates.entry(module_id)
                    .or_insert(BTreeMap::new())
                    .insert(substate_key.clone(), substate)
            }
            NodeUpdate::Update(node_update) => {
                node_update.entry(module_id).or_insert(IndexMap::new())
                    .insert(substate_key.clone(), SubstateUpdate {
                        substate,
                        meta_state: SubstateMeta::New,
                    }).map(|s| s.substate)
            }
        };

        if let Some(prev) = prev {
            if prev.lock_state.is_locked() {
                return Err(AcquireLockError::SubstateLocked(
                    node_id,
                    module_id,
                    substate_key.clone(),
                ));
            }
        }

        Ok(())
    }

    fn read_substate(&self, handle: u32) -> &IndexedScryptoValue {
        let (node_id, module_id, substate_key, _flags) =
            self.locks.get(&handle).expect("Invalid lock handle");

        &Self::loaded_substate(&self.updates, node_id, *module_id, substate_key)
            .expect("Substate missing for valid lock handle")
    }

    fn update_substate(&mut self, handle: u32, substate_value: IndexedScryptoValue) {
        let (node_id, module_id, substate_key, flags) =
            self.locks.get(&handle).expect("Invalid lock handle");

        if !flags.contains(LockFlags::MUTABLE) {
            panic!("No write permission for {}", handle);
        }

        Self::loaded_substate_mut(
            &mut self.updates,
            node_id,
            *module_id,
            substate_key,
        )
        .expect("Substate missing for valid lock handle")
        .0.value = substate_value;
    }

    fn delete_substate(
        &mut self,
        _node_id: &NodeId,
        _module_id: ModuleId,
        _substate_key: &SubstateKey,
    ) -> Option<IndexedScryptoValue> {
        todo!()
    }

    fn read_sorted_substates(
        &mut self,
        _node_id: &NodeId,
        _module_id: ModuleId,
        _count: u32,
    ) -> Vec<(SubstateKey, IndexedScryptoValue)> {
        todo!()
    }
}
