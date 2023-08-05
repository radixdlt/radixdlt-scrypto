use crate::kernel::call_frame::{
    CallFrameDrainSubstatesError, CallFrameRemoveSubstateError, CallFrameScanKeysError,
    CallFrameScanSortedSubstatesError, CallFrameSetSubstateError, CloseSubstateError,
    CreateNodeError, DropNodeError, MoveModuleError, OpenSubstateError, PersistNodeError,
    WriteSubstateError,
};
use crate::kernel::heap::{Heap, HeapRemoveNodeError};
use crate::kernel::substate_locks::SubstateLocks;
use crate::track::interface::{
    CallbackError, NodeSubstates, StoreAccess, SubstateStore, TrackedSubstateInfo,
};
use radix_engine_common::prelude::{NodeId, PartitionNumber};
use radix_engine_common::types::{SortedU16Key, SubstateKey};
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::types::IndexedScryptoValue;
use radix_engine_store_interface::db_key_mapper::SubstateKeyContent;
use sbor::prelude::Box;
use sbor::prelude::Vec;
use sbor::rust::collections::LinkedList;
use utils::prelude::index_set_new;
use utils::rust::prelude::IndexSet;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SubstateDevice {
    Heap,
    Store,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockData {
    flags: LockFlags,
    location: SubstateDevice,
    owned_nodes: IndexSet<NodeId>,
    non_global_references: IndexSet<NodeId>,
}

pub trait SubstateIOHandler<E> {
    fn on_persist_node(&mut self, heap: &Heap, node_id: &NodeId) -> Result<(), E>;

    fn on_store_access(&mut self, heap: &Heap, store_access: StoreAccess) -> Result<(), E>;
}

pub trait SubstateReadHandler {
    type Error;

    fn on_read_substate(
        &mut self,
        heap: &Heap,
        value: &IndexedScryptoValue,
        location: SubstateDevice,
    ) -> Result<(), Self::Error>;
}

pub struct SubstateIO<'g, S: SubstateStore> {
    pub heap: Heap,
    pub store: &'g mut S,
    pub substate_locks: SubstateLocks<LockData>,
}

impl<'g, S: SubstateStore + 'g> SubstateIO<'g, S> {
    pub fn new(store: &'g mut S) -> Self {
        Self {
            heap: Heap::new(),
            store,
            substate_locks: SubstateLocks::new(),
        }
    }

    pub fn create_node<'f, E>(
        &mut self,
        handler: &mut impl SubstateIOHandler<E>,
        node_id: NodeId,
        node_substates: NodeSubstates,
        substate_device: SubstateDevice,
    ) -> Result<(), CallbackError<CreateNodeError, E>> {
        for (_partition_number, module) in &node_substates {
            for (_substate_key, substate_value) in module {
                for own in substate_value.owned_nodes() {
                    if substate_device.eq(&SubstateDevice::Store) {
                        Self::move_node_to_store(&mut self.heap, self.store, handler, own)
                            .map_err(|e| e.map(CreateNodeError::PersistNodeError))?
                    }
                }
                for reference in substate_value.references() {
                    if substate_device.eq(&SubstateDevice::Store) && !reference.is_global() {
                        return Err(CallbackError::Error(
                            CreateNodeError::NonGlobalRefNotAllowed(*reference),
                        ));
                    }

                    if self.heap.contains_node(reference) {
                        self.heap.increase_borrow_count(reference);
                    }
                }
            }
        }

        match substate_device {
            SubstateDevice::Store => {
                self.store
                    .create_node(node_id, node_substates, &mut |store_access| {
                        handler.on_store_access(&self.heap, store_access)
                    })
                    .map_err(CallbackError::CallbackError)?;
            }
            SubstateDevice::Heap => {
                self.heap.create_node(node_id, node_substates);
            }
        }

        Ok(())
    }

    pub fn drop_node(&mut self, node_id: &NodeId) -> Result<NodeSubstates, DropNodeError> {
        if self.substate_locks.node_is_locked(node_id) {
            return Err(DropNodeError::SubstateBorrowed(*node_id));
        }

        let node_substates = match self.heap.remove_node(node_id) {
            Ok(substates) => substates,
            Err(HeapRemoveNodeError::NodeNotFound(node_id)) => {
                panic!("Frame owned node {:?} not found in heap", node_id)
            }
            Err(HeapRemoveNodeError::NodeBorrowed(node_id, count)) => {
                return Err(DropNodeError::NodeBorrowed(node_id, count));
            }
        };
        for (_, module) in &node_substates {
            for (_, substate_value) in module {
                for reference in substate_value.references() {
                    if !reference.is_global() {
                        if self.heap.contains_node(reference) {
                            self.heap.decrease_borrow_count(reference);
                        }
                    }
                }
            }
        }
        Ok(node_substates)
    }

    pub fn move_partition<'f, E>(
        &mut self,
        handler: &mut impl SubstateIOHandler<E>,
        src_node_id: &NodeId,
        src_partition_number: PartitionNumber,
        dest_node_id: &NodeId,
        dest_partition_number: PartitionNumber,
    ) -> Result<(), CallbackError<MoveModuleError, E>> {
        // TODO: Use more granular partition lock checks?
        if self.substate_locks.node_is_locked(src_node_id) {
            return Err(CallbackError::Error(MoveModuleError::SubstateBorrowed(
                *src_node_id,
            )));
        }

        // Move
        let module = self
            .heap
            .remove_module(src_node_id, src_partition_number)
            .map_err(|e| CallbackError::Error(MoveModuleError::HeapRemoveModuleErr(e)))?;
        let to_heap = self.heap.contains_node(dest_node_id);
        for (substate_key, substate_value) in module {
            if to_heap {
                self.heap.set_substate(
                    *dest_node_id,
                    dest_partition_number,
                    substate_key,
                    substate_value,
                );
            } else {
                // Recursively move nodes to store
                for own in substate_value.owned_nodes() {
                    Self::move_node_to_store(&mut self.heap, self.store, handler, own)
                        .map_err(|e| e.map(|e| MoveModuleError::PersistNodeError(e)))?;
                }

                for reference in substate_value.references() {
                    if !reference.is_global() {
                        return Err(CallbackError::Error(
                            MoveModuleError::NonGlobalRefNotAllowed(reference.clone()),
                        ));
                    }
                }

                self.store
                    .set_substate(
                        *dest_node_id,
                        dest_partition_number,
                        substate_key,
                        substate_value,
                        &mut |store_access| handler.on_store_access(&self.heap, store_access),
                    )
                    .map_err(CallbackError::CallbackError)?
            }
        }

        Ok(())
    }

    pub fn open_substate<E, F: FnMut(&Heap, StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        on_store_access: &mut F,
        default: Option<fn() -> IndexedScryptoValue>,
    ) -> Result<(u32, &IndexedScryptoValue, SubstateDevice), CallbackError<OpenSubstateError, E>>
    {
        let substate_location = if self.heap.contains_node(node_id) {
            if flags.contains(LockFlags::UNMODIFIED_BASE) {
                return Err(CallbackError::Error(
                    OpenSubstateError::LockUnmodifiedBaseOnHeapNode,
                ));
            }

            SubstateDevice::Heap
        } else {
            // Check substate state
            if flags.contains(LockFlags::UNMODIFIED_BASE) {
                match self
                    .store
                    .get_tracked_substate_info(node_id, partition_num, substate_key)
                {
                    TrackedSubstateInfo::New => {
                        return Err(CallbackError::Error(
                            OpenSubstateError::LockUnmodifiedBaseOnNewSubstate(
                                node_id.clone(),
                                partition_num,
                                substate_key.clone(),
                            ),
                        ));
                    }
                    TrackedSubstateInfo::Updated => {
                        return Err(CallbackError::Error(
                            OpenSubstateError::LockUnmodifiedBaseOnOnUpdatedSubstate(
                                node_id.clone(),
                                partition_num,
                                substate_key.clone(),
                            ),
                        ));
                    }
                    TrackedSubstateInfo::Unmodified => {
                        // Okay
                    }
                }
            }
            SubstateDevice::Store
        };

        let substate_value = Self::get_substate_internal(
            &mut self.heap,
            &mut self.store,
            substate_location,
            node_id,
            partition_num,
            substate_key,
            on_store_access,
            default,
        )?;

        let mut owned_nodes = index_set_new();
        for node_id in substate_value.owned_nodes() {
            owned_nodes.insert(*node_id);
        }
        let mut non_global_references = index_set_new(); // du-duplicated
        for node_id in substate_value.references() {
            if !node_id.is_global() {
                non_global_references.insert(*node_id);
            }
        }

        let lock_data = LockData {
            flags,
            location: substate_location,
            owned_nodes,
            non_global_references,
        };

        let global_lock_handle = match self.substate_locks.lock(
            node_id,
            partition_num,
            substate_key,
            !flags.contains(LockFlags::MUTABLE),
            lock_data,
        ) {
            Some(handle) => handle,
            None => {
                return Err(CallbackError::Error(OpenSubstateError::SubstateLocked(
                    *node_id,
                    partition_num,
                    substate_key.clone(),
                )));
            }
        };

        Ok((global_lock_handle, substate_value, substate_location))
    }

    pub fn read_substate<H: SubstateReadHandler>(
        &mut self,
        global_lock_handle: u32,
        handler: &mut H,
    ) -> Result<&IndexedScryptoValue, H::Error> {
        let (node_id, partition_num, substate_key, lock_data) =
            self.substate_locks.get(global_lock_handle);

        let substate = match lock_data.location {
            SubstateDevice::Heap => self
                .heap
                .get_substate(node_id, *partition_num, substate_key)
                .unwrap(),
            SubstateDevice::Store => self
                .store
                .get_substate(node_id, *partition_num, substate_key, &mut |_| Err(()))
                .expect("Getting substate on handled substate should not incur a store access."),
        };

        handler.on_read_substate(&self.heap, substate, lock_data.location)?;

        Ok(substate)
    }

    pub fn write_substate<E>(
        &mut self,
        handler: &mut impl SubstateIOHandler<E>,
        global_lock_handle: u32,
        substate: IndexedScryptoValue,
    ) -> Result<(), CallbackError<WriteSubstateError, E>> {
        let mut new_owned_nodes = index_set_new();
        for own in substate.owned_nodes() {
            if !new_owned_nodes.insert(own.clone()) {
                return Err(CallbackError::Error(
                    WriteSubstateError::ContainsDuplicatedOwns,
                ));
            }
        }
        let mut new_non_global_references = index_set_new();
        let mut new_references = index_set_new();
        for own in substate.references() {
            // Deduplicate
            new_references.insert(own.clone());
            if !own.is_global() {
                new_non_global_references.insert(own.clone());
            }
        }

        let (node_id, partition_num, substate_key, lock_data) =
            self.substate_locks.get_mut(global_lock_handle);
        if !lock_data.flags.contains(LockFlags::MUTABLE) {
            return Err(CallbackError::Error(WriteSubstateError::NoWritePermission));
        }

        // Process owned
        {
            for own in &new_owned_nodes {
                if !lock_data.owned_nodes.contains(own) {
                    // Move the node to store, if its owner is already in store
                    if lock_data.location.eq(&SubstateDevice::Store) {
                        Self::move_node_to_store(&mut self.heap, self.store, handler, own)
                            .map_err(|e| e.map(WriteSubstateError::PersistNodeError))?;
                    }
                }
            }

            lock_data.owned_nodes = new_owned_nodes;
        }

        // Process references
        {
            for reference in &new_references {
                if !lock_data.non_global_references.contains(reference) {
                    if lock_data.location.eq(&SubstateDevice::Store) && !reference.is_global() {
                        return Err(CallbackError::Error(
                            WriteSubstateError::NonGlobalRefNotAllowed(*reference),
                        ));
                    }

                    if self.heap.contains_node(reference) {
                        self.heap.increase_borrow_count(reference);
                    }
                }
            }
            for reference in &lock_data.non_global_references {
                if !new_references.contains(reference) {
                    // handle removed references
                    if self.heap.contains_node(reference) {
                        self.heap.decrease_borrow_count(reference);
                    }
                }
            }

            lock_data.non_global_references = new_non_global_references;
        }

        match lock_data.location {
            SubstateDevice::Heap => {
                self.heap.set_substate(
                    node_id.clone(),
                    partition_num.clone(),
                    substate_key.clone(),
                    substate,
                );
            }
            SubstateDevice::Store => {
                self.store
                    .set_substate(
                        node_id.clone(),
                        partition_num.clone(),
                        substate_key.clone(),
                        substate,
                        &mut |_| Err(()),
                    )
                    .expect(
                        "Setting substate on handled substate should not incur a store access.",
                    );
            }
        }

        Ok(())
    }

    pub fn close_substate(
        &mut self,
        global_lock_handle: u32,
    ) -> Result<(NodeId, PartitionNumber, SubstateKey, LockFlags), CloseSubstateError> {
        let (node_id, partition_num, substate_key, lock_data) =
            self.substate_locks.unlock(global_lock_handle);

        if lock_data.flags.contains(LockFlags::FORCE_WRITE) {
            self.store
                .force_write(&node_id, &partition_num, &substate_key);
        }

        Ok((node_id, partition_num, substate_key, lock_data.flags))
    }

    pub fn set_substate<'f, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        value: IndexedScryptoValue,
        on_store_access: &mut F,
    ) -> Result<(), CallbackError<CallFrameSetSubstateError, E>> {
        if self
            .substate_locks
            .is_locked(node_id, partition_num, &substate_key)
        {
            return Err(CallbackError::Error(
                CallFrameSetSubstateError::SubstateLocked(
                    node_id.clone(),
                    partition_num,
                    substate_key,
                ),
            ));
        }

        if self.heap.contains_node(node_id) {
            self.heap
                .set_substate(*node_id, partition_num, substate_key, value);
        } else {
            self.store
                .set_substate(
                    *node_id,
                    partition_num,
                    substate_key,
                    value,
                    on_store_access,
                )
                .map_err(CallbackError::CallbackError)?
        };

        Ok(())
    }

    pub fn remove_substate<'f, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: &SubstateKey,
        on_store_access: &mut F,
    ) -> Result<Option<IndexedScryptoValue>, CallbackError<CallFrameRemoveSubstateError, E>> {
        if self.substate_locks.is_locked(node_id, partition_num, key) {
            return Err(CallbackError::Error(
                CallFrameRemoveSubstateError::SubstateLocked(
                    node_id.clone(),
                    partition_num,
                    key.clone(),
                ),
            ));
        }

        let removed = if self.heap.contains_node(node_id) {
            self.heap.remove_substate(node_id, partition_num, key)
        } else {
            self.store
                .remove_substate(node_id, partition_num, key, on_store_access)
                .map_err(CallbackError::CallbackError)?
        };

        Ok(removed)
    }

    pub fn scan_keys<'f, K: SubstateKeyContent, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        on_store_access: &mut F,
    ) -> Result<Vec<SubstateKey>, CallbackError<CallFrameScanKeysError, E>> {
        let keys = if self.heap.contains_node(node_id) {
            self.heap.scan_keys(node_id, partition_num, count)
        } else {
            self.store
                .scan_keys::<K, E, F>(node_id, partition_num, count, on_store_access)
                .map_err(|e| CallbackError::CallbackError(e))?
        };

        Ok(keys)
    }

    pub fn drain_substates<'f, K: SubstateKeyContent, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        on_store_access: &mut F,
    ) -> Result<
        Vec<(SubstateKey, IndexedScryptoValue)>,
        CallbackError<CallFrameDrainSubstatesError, E>,
    > {
        let substates = if self.heap.contains_node(node_id) {
            self.heap.drain_substates(node_id, partition_num, count)
        } else {
            self.store
                .drain_substates::<K, E, F>(node_id, partition_num, count, on_store_access)
                .map_err(|e| CallbackError::CallbackError(e))?
        };

        // TODO: Should check if any substate is locked

        Ok(substates)
    }

    // Substate Virtualization does not apply to this call
    // Should this be prevented at this layer?
    pub fn scan_sorted<'f, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        on_store_access: &mut F,
    ) -> Result<
        Vec<(SortedU16Key, IndexedScryptoValue)>,
        CallbackError<CallFrameScanSortedSubstatesError, E>,
    > {
        let substates = if self.heap.contains_node(node_id) {
            // This should never be triggered because sorted index store is
            // used by consensus manager only.
            panic!("Unexpected code path")
        } else {
            self.store
                .scan_sorted_substates(node_id, partition_num, count, on_store_access)
                .map_err(|e| CallbackError::CallbackError(e))?
        };

        // TODO: Should check if any substate is locked

        Ok(substates)
    }

    pub fn move_node_to_store<E>(
        heap: &mut Heap,
        store: &mut S,
        handler: &mut impl SubstateIOHandler<E>,
        node_id: &NodeId,
    ) -> Result<(), CallbackError<PersistNodeError, E>> {
        // TODO: Add locked substate checks, though this is not required since
        // the system layer currently maintains the invariant that a call frame cannot
        // open a substate of an owned node

        let mut queue = LinkedList::new();
        queue.push_back(node_id.clone());

        while let Some(node_id) = queue.pop_front() {
            handler
                .on_persist_node(heap, &node_id)
                .map_err(CallbackError::CallbackError)?;

            let node_substates = match heap.remove_node(&node_id) {
                Ok(substates) => substates,
                Err(HeapRemoveNodeError::NodeNotFound(node_id)) => {
                    panic!("Frame owned node {:?} not found in heap", node_id)
                }
                Err(HeapRemoveNodeError::NodeBorrowed(node_id, count)) => {
                    return Err(CallbackError::Error(PersistNodeError::NodeBorrowed(
                        node_id, count,
                    )));
                }
            };
            for (_partition_number, module_substates) in &node_substates {
                for (_substate_key, substate_value) in module_substates {
                    for reference in substate_value.references() {
                        if !reference.is_global() {
                            return Err(CallbackError::Error(
                                PersistNodeError::ContainsNonGlobalRef(*reference),
                            ));
                        }
                    }

                    for node_id in substate_value.owned_nodes() {
                        queue.push_back(*node_id);
                    }
                }
            }

            store
                .create_node(node_id.clone(), node_substates, &mut |store_access| {
                    handler.on_store_access(heap, store_access)
                })
                .map_err(CallbackError::CallbackError)?;
        }

        Ok(())
    }

    fn get_substate_internal<'a, E, F: FnMut(&Heap, StoreAccess) -> Result<(), E>>(
        heap: &'a mut Heap,
        store: &'a mut S,
        location: SubstateDevice,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        on_store_access: &mut F,
        default: Option<fn() -> IndexedScryptoValue>,
    ) -> Result<&'a IndexedScryptoValue, CallbackError<OpenSubstateError, E>> {
        let value = match location {
            SubstateDevice::Heap => heap
                .get_substate_or_default(node_id, partition_num, substate_key, || {
                    default.map(|f| f())
                })
                .map_err(|e| CallbackError::Error(OpenSubstateError::HeapError(e)))?,
            SubstateDevice::Store => store
                .get_substate_or_default(
                    node_id,
                    partition_num,
                    substate_key,
                    &mut |store_access| on_store_access(heap, store_access),
                    || default.map(|f| f()),
                )
                .map_err(|x| x.map(|e| OpenSubstateError::TrackError(Box::new(e))))?,
        };

        Ok(value)
    }
}
