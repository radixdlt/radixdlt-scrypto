use crate::types::*;
use radix_engine_interface::api::substate_lock_api::LockFlags;
use radix_engine_interface::types::*;
use radix_engine_stores::interface::{
    AcquireLockError, DatabaseMapper, DatabaseUpdate, DatabaseUpdates, NodeSubstates,
    SetSubstateError, SubstateDatabase, SubstateStore, TakeSubstateError,
};
use sbor::rust::collections::btree_map::Entry;
use sbor::rust::mem;

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

#[derive(Clone, Debug)]
pub struct RuntimeSubstate {
    pub value: IndexedScryptoValue,
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

#[derive(Clone, Debug)]
pub enum ReadOnly {
    NonExistent,
    Existent(RuntimeSubstate),
}

#[derive(Clone, Debug)]
pub enum Write {
    Update(RuntimeSubstate),
    Delete,
}

impl Write {
    fn into_value(self) -> Option<IndexedScryptoValue> {
        match self {
            Write::Update(substate) => Some(substate.value),
            Write::Delete => None,
        }
    }
}

// TODO: Add new virtualized
#[derive(Clone, Debug)]
pub enum TrackedSubstateKey {
    New(RuntimeSubstate),
    ReadOnly(ReadOnly),
    ReadExistAndWrite(IndexedScryptoValue, Write),
    ReadNonExistAndWrite(RuntimeSubstate),
    WriteOnly(Write),
    Garbage,
}

impl TrackedSubstateKey {
    pub fn get_runtime_substate_mut(&mut self) -> Option<&mut RuntimeSubstate> {
        match self {
            TrackedSubstateKey::New(substate)
            | TrackedSubstateKey::WriteOnly(Write::Update(substate))
            | TrackedSubstateKey::ReadOnly(ReadOnly::Existent(substate))
            | TrackedSubstateKey::ReadExistAndWrite(_, Write::Update(substate))
            | TrackedSubstateKey::ReadNonExistAndWrite(substate) => Some(substate),
            TrackedSubstateKey::WriteOnly(Write::Delete)
            | TrackedSubstateKey::ReadExistAndWrite(_, Write::Delete)
            | TrackedSubstateKey::ReadOnly(ReadOnly::NonExistent)
            | TrackedSubstateKey::Garbage => None,
        }
    }

    pub fn get(&self) -> Option<&IndexedScryptoValue> {
        match self {
            TrackedSubstateKey::New(substate)
            | TrackedSubstateKey::WriteOnly(Write::Update(substate))
            | TrackedSubstateKey::ReadOnly(ReadOnly::Existent(substate))
            | TrackedSubstateKey::ReadExistAndWrite(_, Write::Update(substate))
            | TrackedSubstateKey::ReadNonExistAndWrite(substate) => Some(&substate.value),
            TrackedSubstateKey::WriteOnly(Write::Delete)
            | TrackedSubstateKey::ReadExistAndWrite(_, Write::Delete)
            | TrackedSubstateKey::ReadOnly(ReadOnly::NonExistent)
            | TrackedSubstateKey::Garbage => None,
        }
    }

    pub fn set(&mut self, value: IndexedScryptoValue) {
        match self {
            TrackedSubstateKey::Garbage => {
                *self = TrackedSubstateKey::WriteOnly(Write::Update(RuntimeSubstate::new(value)));
            }
            TrackedSubstateKey::New(substate)
            | TrackedSubstateKey::WriteOnly(Write::Update(substate))
            | TrackedSubstateKey::ReadExistAndWrite(_, Write::Update(substate))
            | TrackedSubstateKey::ReadNonExistAndWrite(substate) => {
                substate.value = value;
            }
            TrackedSubstateKey::ReadOnly(ReadOnly::NonExistent) => {
                let new_tracked =
                    TrackedSubstateKey::ReadNonExistAndWrite(RuntimeSubstate::new(value));
                let mut old = mem::replace(self, new_tracked);
                self.get_runtime_substate_mut().unwrap().lock_state =
                    old.get_runtime_substate_mut().unwrap().lock_state;
            }
            TrackedSubstateKey::ReadOnly(ReadOnly::Existent(old)) => {
                let new_tracked = TrackedSubstateKey::ReadExistAndWrite(
                    old.value.clone(),
                    Write::Update(RuntimeSubstate::new(value)),
                );
                let mut old = mem::replace(self, new_tracked);
                self.get_runtime_substate_mut().unwrap().lock_state =
                    old.get_runtime_substate_mut().unwrap().lock_state;
            }
            TrackedSubstateKey::ReadExistAndWrite(_, write @ Write::Delete)
            | TrackedSubstateKey::WriteOnly(write @ Write::Delete) => {
                *write = Write::Update(RuntimeSubstate::new(value));
            }
        };
    }

    pub fn take(&mut self) -> Option<IndexedScryptoValue> {
        match self {
            TrackedSubstateKey::Garbage => None,
            TrackedSubstateKey::New(..) => {
                let old = mem::replace(self, TrackedSubstateKey::Garbage);
                old.into_value()
            }
            TrackedSubstateKey::WriteOnly(_) => {
                let old = mem::replace(self, TrackedSubstateKey::WriteOnly(Write::Delete));
                old.into_value()
            }
            TrackedSubstateKey::ReadExistAndWrite(_, write) => {
                let write = mem::replace(write, Write::Delete);
                write.into_value()
            }
            TrackedSubstateKey::ReadNonExistAndWrite(..) => {
                let old = mem::replace(self, TrackedSubstateKey::ReadOnly(ReadOnly::NonExistent));
                old.into_value()
            }
            TrackedSubstateKey::ReadOnly(ReadOnly::Existent(v)) => {
                let new_tracked =
                    TrackedSubstateKey::ReadExistAndWrite(v.value.clone(), Write::Delete);
                let old = mem::replace(self, new_tracked);
                old.into_value()
            }
            TrackedSubstateKey::ReadOnly(ReadOnly::NonExistent) => None,
        }
    }

    fn revert_writes(&mut self) {
        match self {
            TrackedSubstateKey::ReadOnly(..) | TrackedSubstateKey::Garbage => {}
            TrackedSubstateKey::New(..) | TrackedSubstateKey::WriteOnly(_) => {
                *self = TrackedSubstateKey::Garbage;
            }
            TrackedSubstateKey::ReadExistAndWrite(read, _) => {
                *self = TrackedSubstateKey::ReadOnly(ReadOnly::Existent(RuntimeSubstate::new(
                    read.clone(),
                )));
            }
            TrackedSubstateKey::ReadNonExistAndWrite(..) => {
                *self = TrackedSubstateKey::ReadOnly(ReadOnly::NonExistent);
            }
        }
    }

    pub fn into_value(self) -> Option<IndexedScryptoValue> {
        match self {
            TrackedSubstateKey::New(substate)
            | TrackedSubstateKey::WriteOnly(Write::Update(substate))
            | TrackedSubstateKey::ReadOnly(ReadOnly::Existent(substate))
            | TrackedSubstateKey::ReadNonExistAndWrite(substate)
            | TrackedSubstateKey::ReadExistAndWrite(_, Write::Update(substate)) => {
                Some(substate.value)
            }
            TrackedSubstateKey::WriteOnly(Write::Delete)
            | TrackedSubstateKey::ReadExistAndWrite(_, Write::Delete)
            | TrackedSubstateKey::ReadOnly(ReadOnly::NonExistent)
            | TrackedSubstateKey::Garbage => None,
        }
    }
}

#[derive(Debug)]
pub struct TrackedModule {
    pub substates: BTreeMap<Vec<u8>, TrackedSubstateKey>,
    pub range_read: Option<u32>,
}

impl TrackedModule {
    pub fn new() -> Self {
        Self {
            substates: BTreeMap::new(),
            range_read: None,
        }
    }

    pub fn new_with_substates(substates: BTreeMap<Vec<u8>, TrackedSubstateKey>) -> Self {
        Self {
            substates,
            range_read: None,
        }
    }

    pub fn revert_writes(&mut self) {
        for (_key, tracked_key) in &mut self.substates {
            tracked_key.revert_writes();
        }
    }
}

#[derive(Debug)]
pub struct TrackedNode {
    pub tracked_modules: IndexMap<ModuleId, TrackedModule>,
    // If true, then all SubstateUpdates under this NodeUpdate must be inserts
    // The extra information, though awkward structurally, makes for a much
    // simpler iteration implementation as long as the invariant is maintained
    pub is_new: bool,
}

impl TrackedNode {
    pub fn new(is_new: bool) -> Self {
        Self {
            tracked_modules: index_map_new(),
            is_new,
        }
    }

    pub fn revert_writes(&mut self) {
        for (_, tracked_module) in &mut self.tracked_modules {
            tracked_module.revert_writes();
        }
    }
}

pub fn to_database_updates<M: DatabaseMapper>(
    index: IndexMap<NodeId, TrackedNode>,
) -> DatabaseUpdates {
    let mut database_updates: IndexMap<Vec<u8>, IndexMap<Vec<u8>, DatabaseUpdate>> =
        index_map_new();
    for (node_id, tracked_node) in index {
        for (module_id, tracked_module) in tracked_node.tracked_modules {
            let mut index_updates = index_map_new();
            for (db_key, tracked) in tracked_module.substates {
                let update = match tracked {
                    TrackedSubstateKey::ReadOnly(..) | TrackedSubstateKey::Garbage => None,
                    TrackedSubstateKey::ReadNonExistAndWrite(substate)
                    | TrackedSubstateKey::New(substate) => {
                        Some(DatabaseUpdate::Set(substate.value.into()))
                    }
                    TrackedSubstateKey::ReadExistAndWrite(_, write)
                    | TrackedSubstateKey::WriteOnly(write) => match write {
                        Write::Delete => Some(DatabaseUpdate::Delete),
                        Write::Update(substate) => Some(DatabaseUpdate::Set(substate.value.into())),
                    },
                };
                if let Some(update) = update {
                    index_updates.insert(db_key, update);
                }
            }

            let index_id = M::map_to_index_id(&node_id, module_id);
            database_updates.insert(index_id, index_updates);
        }
    }

    DatabaseUpdates { database_updates }
}

struct TrackedIter<'a> {
    iter: Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>,
    num_iterations: u32,
}

impl<'a> TrackedIter<'a> {
    fn new(iter: Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + 'a>) -> Self {
        Self {
            iter,
            num_iterations: 0u32,
        }
    }
}

impl<'a> Iterator for TrackedIter<'a> {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        self.num_iterations = self.num_iterations + 1;
        self.iter.next()
    }
}
/// Transaction-wide states and side effects
pub struct Track<'s, S: SubstateDatabase, M: DatabaseMapper> {
    substate_db: &'s S,
    tracked_nodes: IndexMap<NodeId, TrackedNode>,
    force_write_tracked_nodes: IndexMap<NodeId, TrackedNode>,

    locks: IndexMap<u32, (NodeId, ModuleId, Vec<u8>, LockFlags)>,
    next_lock_id: u32,
    phantom_data: PhantomData<M>,
}

impl<'s, S: SubstateDatabase, M: DatabaseMapper> Track<'s, S, M> {
    pub fn new(substate_db: &'s S) -> Self {
        Self {
            substate_db,
            force_write_tracked_nodes: index_map_new(),
            tracked_nodes: index_map_new(),
            locks: index_map_new(),
            next_lock_id: 0,
            phantom_data: PhantomData::default(),
        }
    }

    fn new_lock_handle(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        db_key: &Vec<u8>,
        flags: LockFlags,
    ) -> u32 {
        let new_lock = self.next_lock_id;
        self.locks
            .insert(new_lock, (*node_id, module_id, db_key.clone(), flags));
        self.next_lock_id += 1;
        new_lock
    }

    /// Reverts all non force write changes.
    ///
    /// Note that dependencies will never be reverted.
    pub fn revert_non_force_write_changes(&mut self) {
        self.tracked_nodes
            .retain(|_, tracked_node| !tracked_node.is_new);
        for (_, tracked_node) in &mut self.tracked_nodes {
            tracked_node.revert_writes();
        }

        let force_writes = mem::take(&mut self.force_write_tracked_nodes);

        for (node_id, force_track_node) in force_writes {
            for (module_id, force_track_module) in force_track_node.tracked_modules {
                for (db_key, force_track_key) in force_track_module.substates {
                    let tracked = self.get_tracked_substate(&node_id, module_id, &db_key);
                    *tracked = force_track_key;
                }
            }
        }
    }

    /// Finalizes changes captured by this substate store.
    ///
    ///  Returns the state changes and dependencies.
    pub fn finalize(self) -> IndexMap<NodeId, TrackedNode> {
        self.tracked_nodes
    }

    fn get_tracked_module(&mut self, node_id: &NodeId, module_id: ModuleId) -> &mut TrackedModule {
        self.tracked_nodes
            .entry(*node_id)
            .or_insert(TrackedNode::new(false))
            .tracked_modules
            .entry(module_id)
            .or_insert(TrackedModule::new());

        self.tracked_nodes
            .get_mut(node_id)
            .unwrap()
            .tracked_modules
            .get_mut(&module_id)
            .unwrap()
    }

    fn get_tracked_substate_virtualize<F: FnOnce() -> Option<IndexedScryptoValue>>(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        db_key: &Vec<u8>,
        virtualize: F,
    ) -> &mut TrackedSubstateKey {
        let module_substates = &mut self
            .tracked_nodes
            .entry(*node_id)
            .or_insert(TrackedNode::new(false))
            .tracked_modules
            .entry(module_id)
            .or_insert(TrackedModule::new())
            .substates;
        let entry = module_substates.entry(db_key.clone());

        match entry {
            Entry::Vacant(e) => {
                let index_id = M::map_to_index_id(node_id, module_id);
                let value = self
                    .substate_db
                    .get_substate(&index_id, db_key)
                    .map(|e| IndexedScryptoValue::from_vec(e).expect("Failed to decode substate"));
                if let Some(value) = value {
                    e.insert(TrackedSubstateKey::ReadOnly(ReadOnly::Existent(
                        RuntimeSubstate::new(value),
                    )));
                } else {
                    let value = virtualize();
                    if let Some(value) = value {
                        e.insert(TrackedSubstateKey::ReadNonExistAndWrite(
                            RuntimeSubstate::new(value),
                        ));
                    } else {
                        e.insert(TrackedSubstateKey::ReadOnly(ReadOnly::NonExistent));
                    }
                }
            }
            Entry::Occupied(..) => {}
        };

        module_substates.get_mut(db_key).unwrap()
    }

    fn get_tracked_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        db_key: &Vec<u8>,
    ) -> &mut TrackedSubstateKey {
        self.get_tracked_substate_virtualize(node_id, module_id, db_key, || None)
    }
}

impl<'s, S: SubstateDatabase, M: DatabaseMapper> SubstateStore for Track<'s, S, M> {
    fn create_node(&mut self, node_id: NodeId, node_substates: NodeSubstates) {
        let tracked_modules = node_substates
            .into_iter()
            .map(|(module_id, module_substates)| {
                let module_substates = module_substates
                    .into_iter()
                    .map(|(key, value)| {
                        let key = M::map_to_db_key(key);
                        (key, TrackedSubstateKey::New(RuntimeSubstate::new(value)))
                    })
                    .collect();
                let tracked_module = TrackedModule::new_with_substates(module_substates);
                (module_id, tracked_module)
            })
            .collect();

        self.tracked_nodes.insert(
            node_id,
            TrackedNode {
                tracked_modules: tracked_modules,
                is_new: true,
            },
        );
    }

    fn set_substate(
        &mut self,
        node_id: NodeId,
        module_id: ModuleId,
        substate_key: SubstateKey,
        substate_value: IndexedScryptoValue,
    ) -> Result<(), SetSubstateError> {
        let db_key = M::map_to_db_key(substate_key.clone());

        let tracked_module = self
            .tracked_nodes
            .entry(node_id)
            .or_insert(TrackedNode::new(false))
            .tracked_modules
            .entry(module_id)
            .or_insert(TrackedModule::new());

        let entry = tracked_module.substates.entry(db_key.clone());

        match entry {
            Entry::Vacant(e) => {
                e.insert(TrackedSubstateKey::WriteOnly(Write::Update(
                    RuntimeSubstate::new(substate_value),
                )));
            }
            Entry::Occupied(mut e) => {
                let tracked = e.get_mut();
                if let Some(runtime) = tracked.get_runtime_substate_mut() {
                    if runtime.lock_state.is_locked() {
                        return Err(SetSubstateError::SubstateLocked(
                            node_id,
                            module_id,
                            substate_key.clone(),
                        ));
                    }
                }

                tracked.set(substate_value);
            }
        }

        Ok(())
    }

    // Should not use on virtualized substates
    fn take_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Result<Option<IndexedScryptoValue>, TakeSubstateError> {
        let db_key = M::map_to_db_key(substate_key.clone());

        let tracked = self.get_tracked_substate(node_id, module_id, &db_key);
        if let Some(runtime) = tracked.get_runtime_substate_mut() {
            if runtime.lock_state.is_locked() {
                return Err(TakeSubstateError::SubstateLocked(
                    *node_id,
                    module_id,
                    substate_key.clone(),
                ));
            }
        }

        Ok(tracked.take())
    }

    fn scan_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        count: u32,
    ) -> Vec<IndexedScryptoValue> {
        let count: usize = count.try_into().unwrap();
        let mut items = Vec::new();

        let node_updates = self.tracked_nodes.get(node_id);
        let is_new = node_updates
            .map(|tracked_node| tracked_node.is_new)
            .unwrap_or(false);
        let tracked_module = node_updates.and_then(|n| n.tracked_modules.get(&module_id));

        if let Some(tracked_module) = tracked_module {
            for (_key, tracked) in tracked_module.substates.iter() {
                if items.len() == count {
                    return items;
                }

                // TODO: Check that substate is not write locked
                if let Some(substate) = tracked.get() {
                    items.push(substate.clone());
                }
            }
        }

        // Optimization, no need to go into database if the node is just created
        if is_new {
            return items;
        }

        let index_id = M::map_to_index_id(node_id, module_id);
        let mut tracked_iter = TrackedIter::new(self.substate_db.list_substates(&index_id));
        for (key, substate) in &mut tracked_iter {
            if items.len() == count {
                break;
            }

            if tracked_module
                .map(|tracked_module| tracked_module.substates.contains_key(&key))
                .unwrap_or(false)
            {
                continue;
            }

            items.push(IndexedScryptoValue::from_vec(substate).unwrap());
        }

        // Update track
        let num_iterations = tracked_iter.num_iterations;
        let tracked_module = self.get_tracked_module(node_id, module_id);
        let next_range_read = tracked_module
            .range_read
            .map(|cur| u32::max(cur, num_iterations))
            .unwrap_or(num_iterations);
        tracked_module.range_read = Some(next_range_read);

        items
    }

    fn take_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        count: u32,
    ) -> Vec<IndexedScryptoValue> {
        let count: usize = count.try_into().unwrap();
        let mut items = Vec::new();

        let node_updates = self.tracked_nodes.get_mut(node_id);
        let is_new = node_updates
            .as_ref()
            .map(|tracked_node| tracked_node.is_new)
            .unwrap_or(false);

        // Check what we've currently got so far without going into database
        let mut tracked_module = node_updates.and_then(|n| n.tracked_modules.get_mut(&module_id));
        if let Some(tracked_module) = tracked_module.as_mut() {
            for (_key, tracked) in tracked_module.substates.iter_mut() {
                if items.len() == count {
                    return items;
                }

                // TODO: Check that substate is not locked
                if let Some(value) = tracked.take() {
                    items.push(value);
                }
            }
        }

        // Optimization, no need to go into database if the node is just created
        if is_new {
            return items;
        }

        // Read from database
        let index_id = M::map_to_index_id(node_id, module_id);
        let mut tracked_iter = TrackedIter::new(self.substate_db.list_substates(&index_id));
        let new_updates = {
            let mut new_updates = Vec::new();
            for (key, substate) in &mut tracked_iter {
                if items.len() == count {
                    break;
                }

                if tracked_module
                    .as_ref()
                    .map(|tracked_module| tracked_module.substates.contains_key(&key))
                    .unwrap_or(false)
                {
                    continue;
                }

                let value = IndexedScryptoValue::from_vec(substate).unwrap();
                new_updates.push((
                    key,
                    TrackedSubstateKey::ReadExistAndWrite(value.clone(), Write::Delete),
                ));
                items.push(value);
            }
            new_updates
        };

        // Update track
        {
            let num_iterations = tracked_iter.num_iterations;
            let tracked_module = self.get_tracked_module(node_id, module_id);
            let next_range_read = tracked_module
                .range_read
                .map(|cur| u32::max(cur, num_iterations))
                .unwrap_or(num_iterations);
            tracked_module.range_read = Some(next_range_read);
            for (key, tracked) in new_updates {
                tracked_module.substates.insert(key, tracked);
            }
        }

        items
    }

    fn scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        count: u32,
    ) -> Vec<IndexedScryptoValue> {
        // TODO: Add module dependencies/lock
        let count: usize = count.try_into().unwrap();
        let node_updates = self.tracked_nodes.get_mut(node_id);
        let is_new = node_updates
            .as_ref()
            .map(|tracked_node| tracked_node.is_new)
            .unwrap_or(false);
        let tracked_module = node_updates.and_then(|n| n.tracked_modules.get(&module_id));

        if is_new {
            let mut items = Vec::new();
            if let Some(tracked_module) = tracked_module {
                for (_key, tracked) in tracked_module.substates.iter() {
                    if items.len() == count {
                        break;
                    }

                    // TODO: Check that substate is not write locked
                    if let Some(substate) = tracked.get() {
                        items.push(substate.clone());
                    }
                }
            }

            return items;
        }

        // TODO: Add interleaving updates
        let index_id = M::map_to_index_id(node_id, module_id);
        let tracked_iter = TrackedIter::new(self.substate_db.list_substates(&index_id));
        let items: Vec<IndexedScryptoValue> = tracked_iter
            .take(count)
            .map(|(_key, buf)| IndexedScryptoValue::from_vec(buf).unwrap())
            .collect();

        // Update track
        {
            let num_iterations: u32 = items.len().try_into().unwrap();
            let tracked_module = self.get_tracked_module(node_id, module_id);
            let next_range_read = tracked_module
                .range_read
                .map(|cur| u32::max(cur, num_iterations))
                .unwrap_or(num_iterations);
            tracked_module.range_read = Some(next_range_read);
        }

        items
    }

    fn acquire_lock_virtualize<F: FnOnce() -> Option<IndexedScryptoValue>>(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
        virtualize: F,
    ) -> Result<u32, AcquireLockError> {
        let db_key = M::map_to_db_key(substate_key.clone());

        // Load the substate from state track
        let tracked = self.get_tracked_substate_virtualize(node_id, module_id, &db_key, virtualize);

        // Check substate state
        if flags.contains(LockFlags::UNMODIFIED_BASE) {
            match tracked {
                TrackedSubstateKey::New(..) | TrackedSubstateKey::Garbage => {
                    return Err(AcquireLockError::LockUnmodifiedBaseOnNewSubstate(
                        *node_id,
                        module_id,
                        substate_key.clone(),
                    ));
                }
                TrackedSubstateKey::WriteOnly(..)
                | TrackedSubstateKey::ReadExistAndWrite(..)
                | TrackedSubstateKey::ReadNonExistAndWrite(..) => {
                    return Err(AcquireLockError::LockUnmodifiedBaseOnOnUpdatedSubstate(
                        *node_id,
                        module_id,
                        substate_key.clone(),
                    ));
                }
                TrackedSubstateKey::ReadOnly(..) => {
                    // Okay
                }
            }
        }

        let substate = tracked
            .get_runtime_substate_mut()
            .ok_or(AcquireLockError::NotFound(
                *node_id,
                module_id,
                substate_key.clone(),
            ))?;

        // Check read/write permission
        substate.lock_state.try_lock(flags).map_err(|_| {
            AcquireLockError::SubstateLocked(*node_id, module_id, substate_key.clone())
        })?;

        Ok(self.new_lock_handle(node_id, module_id, &db_key, flags))
    }

    fn release_lock(&mut self, handle: u32) {
        let (node_id, module_id, db_key, flags) =
            self.locks.remove(&handle).expect("Invalid lock handle");

        let tracked = self.get_tracked_substate(&node_id, module_id, &db_key);

        let substate = tracked
            .get_runtime_substate_mut()
            .expect("Could not have created lock on non-existent subsate");

        substate.lock_state.unlock();

        if flags.contains(LockFlags::FORCE_WRITE) {
            let cloned_track = tracked.clone();

            self.force_write_tracked_nodes
                .entry(node_id)
                .or_insert(TrackedNode {
                    tracked_modules: index_map_new(),
                    is_new: false,
                })
                .tracked_modules
                .entry(module_id)
                .or_insert(TrackedModule::new())
                .substates
                .insert(db_key.clone(), cloned_track);
        }
    }

    fn read_substate(&mut self, handle: u32) -> &IndexedScryptoValue {
        let (node_id, module_id, db_key, _flags) =
            self.locks.get(&handle).expect("Invalid lock handle");

        let node_id = *node_id;
        let module_id = *module_id;
        let db_key = db_key.clone();

        let tracked = self.get_tracked_substate(&node_id, module_id, &db_key);
        tracked
            .get()
            .expect("Could not have created lock on non existent substate")
    }

    fn update_substate(&mut self, handle: u32, substate_value: IndexedScryptoValue) {
        let (node_id, module_id, db_key, flags) =
            self.locks.get(&handle).expect("Invalid lock handle");

        if !flags.contains(LockFlags::MUTABLE) {
            panic!("No write permission for {}", handle);
        }

        let node_id = *node_id;
        let module_id = *module_id;
        let db_key = db_key.clone();

        let tracked = self.get_tracked_substate(&node_id, module_id, &db_key);

        match tracked {
            TrackedSubstateKey::New(substate)
            | TrackedSubstateKey::WriteOnly(Write::Update(substate))
            | TrackedSubstateKey::ReadExistAndWrite(_, Write::Update(substate))
            | TrackedSubstateKey::ReadNonExistAndWrite(substate) => {
                substate.value = substate_value;
            }
            TrackedSubstateKey::ReadOnly(ReadOnly::NonExistent) => {
                let new_tracked =
                    TrackedSubstateKey::ReadNonExistAndWrite(RuntimeSubstate::new(substate_value));
                let mut old = mem::replace(tracked, new_tracked);
                tracked.get_runtime_substate_mut().unwrap().lock_state =
                    old.get_runtime_substate_mut().unwrap().lock_state;
            }
            TrackedSubstateKey::ReadOnly(ReadOnly::Existent(substate)) => {
                let new_tracked = TrackedSubstateKey::ReadExistAndWrite(
                    substate.value.clone(),
                    Write::Update(RuntimeSubstate::new(substate_value)),
                );
                let mut old = mem::replace(tracked, new_tracked);
                tracked.get_runtime_substate_mut().unwrap().lock_state =
                    old.get_runtime_substate_mut().unwrap().lock_state;
            }
            TrackedSubstateKey::WriteOnly(Write::Delete)
            | TrackedSubstateKey::ReadExistAndWrite(_, Write::Delete)
            | TrackedSubstateKey::Garbage => {
                panic!("Could not have created lock on non existent substate")
            }
        };
    }
}
