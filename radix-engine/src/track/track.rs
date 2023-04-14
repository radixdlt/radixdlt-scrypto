use std::collections::btree_map::Entry;
use std::mem;
use crate::types::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::types::*;
use radix_engine_stores::interface::{
    AcquireLockError, NodeSubstates, StateUpdate, StateUpdates, SubstateDatabase, SubstateStore,
};

pub struct SubstateLockError;

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

    pub fn try_lock(&mut self, flags: LockFlags) -> Result<(), SubstateLockError> {
        match self {
            SubstateLockState::Read(n) => {
                if flags.contains(LockFlags::MUTABLE) {
                    if *n != 0 {
                        return Err(SubstateLockError);

                    }
                    *self = SubstateLockState::Write;
                } else {
                    *n = *n + 1;
                }
            }
            SubstateLockState::Write => {
                return Err(SubstateLockError);
            }
        }

        Ok(())
    }

    fn unlock(&mut self) {
        match self {
            SubstateLockState::Read(n) => {
                *n = *n - 1;
            }
            SubstateLockState::Write => {
                *self = SubstateLockState::no_lock();
            }
        }
    }
}

#[derive(Debug)]
pub struct RuntimeSubstate {
    value: IndexedScryptoValue,
    lock_state: SubstateLockState,
}

impl RuntimeSubstate {
    fn new(value: IndexedScryptoValue) -> Self {
        Self {
            value,
            lock_state: SubstateLockState::no_lock(),
        }
    }
}

#[derive(Debug)]
pub enum ReadOnly {
    NonExistent,
    Existent(RuntimeSubstate),
}

#[derive(Debug)]
pub enum TrackedSubstateKey {
    ReadOnly(ReadOnly),
    Update(RuntimeSubstate),
    WriteOnly(RuntimeSubstate),
    Delete,
}

impl TrackedSubstateKey {
    fn get_substate(&mut self) -> Option<&mut RuntimeSubstate> {
        match self {
            TrackedSubstateKey::WriteOnly(substate)
            | TrackedSubstateKey::ReadOnly(ReadOnly::Existent(substate))
            | TrackedSubstateKey::Update(substate) => {
                Some(substate)
            },
            TrackedSubstateKey::Delete
            | TrackedSubstateKey::ReadOnly(ReadOnly::NonExistent) => {
                None
            },
        }
    }
}

pub struct TrackedNode {
    modules: IndexMap<ModuleId, BTreeMap<SubstateKey, TrackedSubstateKey>>,
    // If true, then all SubstateUpdates under this NodeUpdate must be inserts
    // The extra information, though awkward structurally, makes for a much
    // simpler implementation as long as the invariant is maintained
    is_new: bool,
}

impl TrackedNode {
    pub fn new(is_new: bool) -> Self{
        Self {
            modules: index_map_new(),
            is_new,
        }
    }
}

/// Transaction-wide states and side effects
pub struct Track<'s> {
    substate_db: &'s dyn SubstateDatabase,
    updates: IndexMap<NodeId, TrackedNode>,
    force_updates: IndexMap<NodeId, TrackedNode>,

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
            for (module_id, module) in node_update.modules {
                for (substate_key, tracked) in module {
                    let update = match tracked {
                        TrackedSubstateKey::WriteOnly(substate) => {
                            StateUpdate::Create(substate.value.into())
                        }
                        TrackedSubstateKey::Update(substate) => {
                            StateUpdate::Update(substate.value.into())
                        }
                        TrackedSubstateKey::ReadOnly(ReadOnly::Existent(substate)) => {
                            // TODO: Fix
                            StateUpdate::Update(substate.value.into())
                        }
                        TrackedSubstateKey::Delete | TrackedSubstateKey::ReadOnly(ReadOnly::NonExistent) => {
                            todo!()
                        }
                    };
                    substate_changes.insert((node_id, module_id, substate_key.clone()), update);
                }
            }
        }

        StateUpdates { substate_changes }
    }

    fn get_tracked_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey
    ) -> &mut TrackedSubstateKey {
        let module_substates = self.updates.entry(*node_id).or_insert(TrackedNode::new(false))
            .modules.entry(module_id).or_insert(BTreeMap::new());
        let entry = module_substates.entry(substate_key.clone());

        match entry {
            Entry::Vacant(e) => {
                let value = self.substate_db
                    .get_substate(node_id, module_id, substate_key)
                    .expect("Database misconfigured")
                    .map(|e| IndexedScryptoValue::from_vec(e).expect("Failed to decode substate"));
                if let Some(value) = value {
                    e.insert(TrackedSubstateKey::ReadOnly(ReadOnly::Existent(RuntimeSubstate::new(value))));
                } else {
                    e.insert(TrackedSubstateKey::ReadOnly(ReadOnly::NonExistent));
                }
            },
            Entry::Occupied(..) => {}
        };

        module_substates.get_mut(substate_key).unwrap()
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
        let tracked = self.get_tracked_substate(node_id, module_id, substate_key);

        // Check substate state
        let substate = match tracked {
            TrackedSubstateKey::WriteOnly(substate) => {
                if flags.contains(LockFlags::UNMODIFIED_BASE) {
                    return Err(AcquireLockError::LockUnmodifiedBaseOnNewSubstate(
                        *node_id,
                        module_id,
                        substate_key.clone(),
                    ))
                }
                substate
            }
            TrackedSubstateKey::Update(substate) => {
                if flags.contains(LockFlags::UNMODIFIED_BASE) {
                    return Err(AcquireLockError::LockUnmodifiedBaseOnOnUpdatedSubstate(
                        *node_id,
                        module_id,
                        substate_key.clone(),
                    ))
                }
                substate
            }
            TrackedSubstateKey::ReadOnly(ReadOnly::Existent(substate)) => {
               substate
            }
            TrackedSubstateKey::Delete | TrackedSubstateKey::ReadOnly(ReadOnly::NonExistent) => {
                return Err(AcquireLockError::NotFound(
                    *node_id,
                    module_id,
                    substate_key.clone(),
                ));
            }
        };

        // Check read/write permission
        substate.lock_state.try_lock(flags).map_err(|_| AcquireLockError::SubstateLocked(
            *node_id,
            module_id,
            substate_key.clone(),
        ))?;

        Ok(self.new_lock_handle(node_id, module_id, substate_key, flags))
    }

    fn release_lock(&mut self, handle: u32) {
        let (node_id, module_id, substate_key, flags) =
            self.locks.remove(&handle).expect("Invalid lock handle");

        let tracked = self.get_tracked_substate(&node_id, module_id, &substate_key);

        let substate = tracked.get_substate()
            .expect("Could not have created lock on non-existent subsate");

        substate.lock_state.unlock();

        if flags.contains(LockFlags::FORCE_WRITE) {
            let value = substate.value.clone();
            self.force_updates
                .entry(node_id)
                .or_insert(TrackedNode {
                    modules: index_map_new(),
                    is_new: false,
                }).modules.entry(module_id)
                .or_default()
                .insert(
                    substate_key.clone(),
                    TrackedSubstateKey::Update(RuntimeSubstate::new(value)),
                );
        }
    }

    fn create_node(&mut self, node_id: NodeId, node_substates: NodeSubstates) {
        let node_runtime = node_substates.into_iter().map(|(module_id, module_substates)| {
            let module_substates = module_substates.into_iter()
                .map(|(key, value)| (key, TrackedSubstateKey::WriteOnly(RuntimeSubstate::new(value))))
                .collect();
            (module_id, module_substates)
        }).collect();

        self.updates.insert(node_id, TrackedNode {
            modules: node_runtime,
            is_new: true,
        });
    }

    fn upsert_substate(
        &mut self,
        node_id: NodeId,
        module_id: ModuleId,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
    ) -> Result<(), AcquireLockError> {
        let node_update = self.updates.entry(node_id).or_insert(TrackedNode::new(false));
        let prev = node_update.modules.entry(module_id).or_insert(BTreeMap::new())
            .insert(substate_key.clone(), TrackedSubstateKey::WriteOnly(RuntimeSubstate::new(substate_value)));

        if let Some(mut prev) = prev {
            if let Some(runtime) = prev.get_substate() {
                if runtime.lock_state.is_locked() {
                    return Err(AcquireLockError::SubstateLocked(
                        node_id,
                        module_id,
                        substate_key.clone(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn read_substate(&mut self, handle: u32) -> &IndexedScryptoValue {
        let (node_id, module_id, substate_key, _flags) =
            self.locks.get(&handle).expect("Invalid lock handle");

        let node_id = *node_id;
        let module_id = *module_id;
        let substate_key = substate_key.clone();

        let tracked = self.get_tracked_substate(&node_id, module_id, &substate_key);
        &tracked.get_substate().expect("Could not have created lock on non existent substate").value
    }

    fn update_substate(&mut self, handle: u32, substate_value: IndexedScryptoValue) {
        let (node_id, module_id, substate_key, flags) =
            self.locks.get(&handle).expect("Invalid lock handle");

        if !flags.contains(LockFlags::MUTABLE) {
            panic!("No write permission for {}", handle);
        }

        let node_id = *node_id;
        let module_id = *module_id;
        let substate_key = substate_key.clone();

        let tracked = self.get_tracked_substate(&node_id, module_id, &substate_key);


        match tracked {
            TrackedSubstateKey::WriteOnly(substate) | TrackedSubstateKey::Update(substate) => {
                substate.value = substate_value;
            },
            track @ TrackedSubstateKey::ReadOnly(ReadOnly::Existent(..)) => {
                let mut old = mem::replace(track, TrackedSubstateKey::Update(RuntimeSubstate::new(substate_value)));
                track.get_substate().unwrap().lock_state = old.get_substate().unwrap().lock_state;
            }
            TrackedSubstateKey::Delete | TrackedSubstateKey::ReadOnly(ReadOnly::NonExistent) => {
                panic!("Could not have created lock on non existent substate")
            }
        };
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
        node_id: &NodeId,
        module_id: ModuleId,
        count: u32,
    ) -> Vec<(SubstateKey, IndexedScryptoValue)> {
        if let Some(update) = self.updates.get_mut(node_id) {
            if update.is_new {
                let substates = update.modules.get_mut(&module_id).unwrap();
                let count: usize = count.try_into().unwrap();
                return substates.into_iter()
                    .take(count)
                    .map(|(key, tracked)| (key.clone(), tracked.get_substate().unwrap().value.clone())).collect();
            }
        }

        let substates = self.substate_db.list_substates(node_id, module_id, count).unwrap();
        substates.into_iter()
            .map(|(key, buf)| (key, IndexedScryptoValue::from_vec(buf).unwrap())).collect()
    }
}
