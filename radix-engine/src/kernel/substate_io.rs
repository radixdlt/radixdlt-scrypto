use crate::kernel::call_frame::{
    CallFrameDrainSubstatesError, CallFrameRemoveSubstateError, CallFrameScanKeysError,
    CallFrameScanSortedSubstatesError, CallFrameSetSubstateError, CloseSubstateError,
    CreateNodeError, DropNodeError, MovePartitionError, NonGlobalNodeRefs, OpenSubstateError,
    PersistNodeError, TransientSubstates, WriteSubstateError,
};
use crate::kernel::heap::{Heap, HeapRemoveNodeError};
use crate::kernel::substate_locks::SubstateLocks;
use crate::track::interface::{
    CallbackError, CommitableSubstateStore, NodeSubstates, StoreAccess, TrackedSubstateInfo,
};
use radix_engine_common::prelude::{NodeId, PartitionNumber};
use radix_engine_common::types::{SortedKey, SubstateKey};
use radix_engine_common::ScryptoSbor;
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::types::IndexedScryptoValue;
use radix_engine_store_interface::db_key_mapper::SubstateKeyContent;
use sbor::prelude::Vec;
use sbor::rust::collections::BTreeSet;
use sbor::rust::collections::LinkedList;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubstateDevice {
    Heap,
    Store,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockData {
    pub flags: LockFlags,
    device: SubstateDevice,
    virtualized: Option<IndexedScryptoValue>,
}

/// Callback for store access, from SubstateIO
pub trait IOStoreAccessHandler<E> {
    fn on_store_access(&mut self, heap: &Heap, store_access: StoreAccess) -> Result<(), E>;
}

/// Callback for substate read, from SubstateIO
pub trait IOSubstateReadHandler {
    type Error;

    fn on_read_substate(
        &mut self,
        heap: &Heap,
        value: &IndexedScryptoValue,
        location: SubstateDevice,
    ) -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ProcessSubstateIOWriteError {
    NonGlobalRefNotAllowed(NodeId),
    PersistNodeError(PersistNodeError),
}

pub struct SubstateIO<'g, S: CommitableSubstateStore> {
    pub heap: Heap,
    pub store: &'g mut S,
    pub non_global_node_refs: NonGlobalNodeRefs,
    pub substate_locks: SubstateLocks<LockData>,
    pub heap_transient_substates: TransientSubstates,
    pub pinned_nodes: BTreeSet<NodeId>,
}

impl<'g, S: CommitableSubstateStore + 'g> SubstateIO<'g, S> {
    pub fn new(store: &'g mut S) -> Self {
        Self {
            heap: Heap::new(),
            store,
            non_global_node_refs: NonGlobalNodeRefs::new(),
            substate_locks: SubstateLocks::new(),
            heap_transient_substates: TransientSubstates::new(),
            pinned_nodes: BTreeSet::new(),
        }
    }

    /// Creates a new node with partitions/substates at the given device.
    /// No additional node movement occurs (For example, owned nodes in node substates
    /// are not moved and must be done manually using other interfaces)
    pub fn create_node<E>(
        &mut self,
        device: SubstateDevice,
        node_id: NodeId,
        node_substates: NodeSubstates,
        handler: &mut impl IOStoreAccessHandler<E>,
    ) -> Result<(), CallbackError<CreateNodeError, E>> {
        match device {
            SubstateDevice::Heap => {
                self.heap
                    .create_node(node_id, node_substates, &mut |heap, store_access| {
                        handler.on_store_access(heap, store_access)
                    })
            }
            SubstateDevice::Store => {
                self.store
                    .create_node(node_id, node_substates, &mut |store_access| {
                        handler.on_store_access(&self.heap, store_access)
                    })
            }
        }
        .map_err(CallbackError::CallbackError)?;

        Ok(())
    }

    pub fn drop_node<E>(
        &mut self,
        device: SubstateDevice,
        node_id: &NodeId,
        handler: &mut impl IOStoreAccessHandler<E>,
    ) -> Result<NodeSubstates, CallbackError<DropNodeError, E>> {
        if self.substate_locks.node_is_locked(node_id) {
            return Err(CallbackError::Error(DropNodeError::SubstateBorrowed(
                *node_id,
            )));
        }

        if self.non_global_node_refs.node_is_referenced(node_id) {
            return Err(CallbackError::Error(DropNodeError::NodeBorrowed(*node_id)));
        }

        let node_substates = match device {
            SubstateDevice::Heap => {
                match self.heap.remove_node(node_id, &mut |heap, store_access| {
                    handler.on_store_access(heap, store_access)
                }) {
                    Ok(substates) => substates,
                    Err(CallbackError::Error(HeapRemoveNodeError::NodeNotFound(node_id))) => {
                        panic!("Frame owned node {:?} not found in heap", node_id)
                    }
                    Err(CallbackError::CallbackError(e)) => {
                        return Err(CallbackError::CallbackError(e));
                    }
                }
            }
            SubstateDevice::Store => {
                panic!("Node drops not supported for store")
            }
        };

        Ok(node_substates)
    }

    pub fn move_node_from_heap_to_store<E>(
        &mut self,
        node_id: &NodeId,
        handler: &mut impl IOStoreAccessHandler<E>,
    ) -> Result<(), CallbackError<PersistNodeError, E>> {
        // TODO: Add locked substate checks, though this is not required since
        // the system layer currently maintains the invariant that a call frame cannot
        // open a substate of an owned node

        let mut queue = LinkedList::new();
        queue.push_back(node_id.clone());

        while let Some(node_id) = queue.pop_front() {
            if self.non_global_node_refs.node_is_referenced(&node_id) {
                return Err(CallbackError::Error(PersistNodeError::NodeBorrowed(
                    node_id,
                )));
            }

            if self.pinned_nodes.contains(&node_id) {
                return Err(CallbackError::Error(
                    PersistNodeError::CannotPersistPinnedNode(node_id),
                ));
            }

            let node_substates = match self.heap.remove_node(&node_id, &mut |heap, store_access| {
                handler.on_store_access(heap, store_access)
            }) {
                Ok(substates) => substates,
                Err(CallbackError::Error(HeapRemoveNodeError::NodeNotFound(node_id))) => {
                    panic!("Frame owned node {:?} not found in heap", node_id)
                }
                Err(CallbackError::CallbackError(e)) => {
                    return Err(CallbackError::CallbackError(e));
                }
            };

            for (_partition_num, module_substates) in &node_substates {
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

            if let Some(transient_substates) = self
                .heap_transient_substates
                .transient_substates
                .remove(&node_id)
            {
                for (partition_num, substate_key) in transient_substates {
                    self.store
                        .mark_as_transient(node_id, partition_num, substate_key);
                }
            }

            self.store
                .create_node(node_id.clone(), node_substates, &mut |store_access| {
                    handler.on_store_access(&self.heap, store_access)
                })
                .map_err(CallbackError::CallbackError)?;
        }

        Ok(())
    }

    pub fn move_partition<'f, E>(
        &mut self,
        src_device: SubstateDevice,
        src_node_id: &NodeId,
        src_partition_number: PartitionNumber,
        dest_device: SubstateDevice,
        dest_node_id: &NodeId,
        dest_partition_number: PartitionNumber,
        handler: &mut impl IOStoreAccessHandler<E>,
    ) -> Result<(), CallbackError<MovePartitionError, E>> {
        // TODO: Use more granular partition lock checks?
        if self.substate_locks.node_is_locked(src_node_id) {
            return Err(CallbackError::Error(MovePartitionError::SubstateBorrowed(
                *src_node_id,
            )));
        }

        // Move
        let partition_substates = match src_device {
            SubstateDevice::Heap => self
                .heap
                .remove_module(
                    src_node_id,
                    src_partition_number,
                    &mut |heap, store_access| handler.on_store_access(heap, store_access),
                )
                .map_err(|e| match e {
                    CallbackError::Error(e) => {
                        CallbackError::Error(MovePartitionError::HeapRemovePartitionError(e))
                    }
                    CallbackError::CallbackError(e) => CallbackError::CallbackError(e),
                })?,
            SubstateDevice::Store => {
                panic!("Partition moves from store not supported.");
            }
        };

        for (substate_key, substate_value) in partition_substates {
            match dest_device {
                SubstateDevice::Heap => {
                    self.heap.set_substate(
                        *dest_node_id,
                        dest_partition_number,
                        substate_key,
                        substate_value,
                        &mut |heap, store_access| handler.on_store_access(heap, store_access),
                    );
                }
                SubstateDevice::Store => {
                    if self.heap_transient_substates.is_transient(
                        src_node_id,
                        src_partition_number,
                        &substate_key,
                    ) {
                        continue;
                    }

                    // Recursively move nodes to store
                    for own in substate_value.owned_nodes() {
                        self.move_node_from_heap_to_store(own, handler)
                            .map_err(|e| e.map(|e| MovePartitionError::PersistNodeError(e)))?;
                    }

                    for reference in substate_value.references() {
                        if !reference.is_global() {
                            return Err(CallbackError::Error(
                                MovePartitionError::NonGlobalRefNotAllowed(reference.clone()),
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
        }

        Ok(())
    }

    pub fn open_substate<E, D: FnOnce() -> IndexedScryptoValue>(
        &mut self,
        device: SubstateDevice,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<D>,
        handler: &mut impl IOStoreAccessHandler<E>,
    ) -> Result<(u32, &IndexedScryptoValue), CallbackError<OpenSubstateError, E>> {
        match device {
            SubstateDevice::Heap => {
                if flags.contains(LockFlags::UNMODIFIED_BASE) {
                    return Err(CallbackError::Error(
                        OpenSubstateError::LockUnmodifiedBaseOnHeapNode,
                    ));
                }
            }
            SubstateDevice::Store => {
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
            }
        }

        let substate_value = Self::get_substate_internal(
            &mut self.heap,
            &mut self.store,
            device,
            node_id,
            partition_num,
            substate_key,
            handler,
        )?;

        let (lock_data, substate_value) = if let Some(substate_value) = substate_value {
            let lock_data = LockData {
                flags,
                device,
                virtualized: None,
            };

            (lock_data, Some(substate_value))
        } else if let Some(compute_default) = default {
            let default_value = compute_default();
            if !default_value.owned_nodes().is_empty() {
                return Err(CallbackError::Error(OpenSubstateError::InvalidDefaultValue));
            }

            let lock_data = LockData {
                flags,
                device,
                virtualized: Some(default_value),
            };

            (lock_data, None)
        } else {
            return Err(CallbackError::Error(OpenSubstateError::SubstateFault));
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

        let substate_value = substate_value.unwrap_or_else(|| {
            let (.., data) = self.substate_locks.get(global_lock_handle);
            data.virtualized.as_ref().unwrap()
        });

        Ok((global_lock_handle, substate_value))
    }

    pub fn read_substate<H: IOSubstateReadHandler>(
        &mut self,
        global_lock_handle: u32,
        handler: &mut H,
    ) -> Result<&IndexedScryptoValue, H::Error> {
        let (node_id, partition_num, substate_key, lock_data) =
            self.substate_locks.get(global_lock_handle);

        // If substate is current virtualized, just return it
        if let Some(virtualized) = &lock_data.virtualized {
            // TODO: Should we callback for costing in this case?
            return Ok(virtualized);
        }

        let substate = match lock_data.device {
            SubstateDevice::Heap => self
                .heap
                .get_substate(node_id, *partition_num, substate_key)
                .unwrap(),
            SubstateDevice::Store => self
                .store
                .get_substate(node_id, *partition_num, substate_key, &mut |_| Err(()))
                .expect("Getting substate on handled substate should not incur a store access.")
                .unwrap(),
        };

        handler.on_read_substate(&self.heap, substate, lock_data.device)?;

        Ok(substate)
    }

    pub fn write_substate<E>(
        &mut self,
        global_lock_handle: u32,
        substate: IndexedScryptoValue,
        handler: &mut impl IOStoreAccessHandler<E>,
    ) -> Result<(), CallbackError<WriteSubstateError, E>> {
        let (node_id, partition_num, substate_key, lock_data) =
            self.substate_locks.get_mut(global_lock_handle);
        if !lock_data.flags.contains(LockFlags::MUTABLE) {
            return Err(CallbackError::Error(WriteSubstateError::NoWritePermission));
        }

        // Remove any virtualized state if it exists
        let _ = lock_data.virtualized.take();

        let node_id = node_id.clone();
        let partition_num = partition_num.clone();
        let substate_key = substate_key.clone();

        match lock_data.device {
            SubstateDevice::Heap => self.heap.set_substate(
                node_id,
                partition_num,
                substate_key,
                substate,
                &mut |heap, store_access| handler.on_store_access(heap, store_access),
            ),
            SubstateDevice::Store => self.store.set_substate(
                node_id,
                partition_num,
                substate_key,
                substate,
                &mut |store_access| handler.on_store_access(&self.heap, store_access),
            ),
        }
        .map_err(|e| CallbackError::CallbackError(e))?;

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

    pub fn set_substate<'f, E>(
        &mut self,
        device: SubstateDevice,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        value: IndexedScryptoValue,
        handler: &mut impl IOStoreAccessHandler<E>,
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

        match device {
            SubstateDevice::Heap => self.heap.set_substate(
                *node_id,
                partition_num,
                substate_key,
                value,
                &mut |heap, store_access| handler.on_store_access(heap, store_access),
            ),
            SubstateDevice::Store => self.store.set_substate(
                *node_id,
                partition_num,
                substate_key,
                value,
                &mut |store_access| handler.on_store_access(&self.heap, store_access),
            ),
        }
        .map_err(CallbackError::CallbackError)?;

        Ok(())
    }

    pub fn remove_substate<'f, E>(
        &mut self,
        device: SubstateDevice,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: &SubstateKey,
        handler: &mut impl IOStoreAccessHandler<E>,
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

        let removed = match device {
            SubstateDevice::Heap => {
                self.heap
                    .remove_substate(node_id, partition_num, key, &mut |heap, store_access| {
                        handler.on_store_access(heap, store_access)
                    })
            }
            SubstateDevice::Store => {
                self.store
                    .remove_substate(node_id, partition_num, key, &mut |store_access| {
                        handler.on_store_access(&self.heap, store_access)
                    })
            }
        }
        .map_err(CallbackError::CallbackError)?;

        Ok(removed)
    }

    pub fn scan_keys<K: SubstateKeyContent + 'static, E>(
        &mut self,
        device: SubstateDevice,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        handler: &mut impl IOStoreAccessHandler<E>,
    ) -> Result<Vec<SubstateKey>, CallbackError<CallFrameScanKeysError, E>> {
        let keys = match device {
            SubstateDevice::Heap => self.heap.scan_keys(node_id, partition_num, count),
            SubstateDevice::Store => self
                .store
                .scan_keys::<K, E, _>(node_id, partition_num, count, &mut |store_access| {
                    handler.on_store_access(&self.heap, store_access)
                })
                .map_err(|e| CallbackError::CallbackError(e))?,
        };

        Ok(keys)
    }

    pub fn drain_substates<K: SubstateKeyContent + 'static, E>(
        &mut self,
        device: SubstateDevice,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        handler: &mut impl IOStoreAccessHandler<E>,
    ) -> Result<
        Vec<(SubstateKey, IndexedScryptoValue)>,
        CallbackError<CallFrameDrainSubstatesError, E>,
    > {
        let substates = match device {
            SubstateDevice::Heap => self.heap.drain_substates(
                node_id,
                partition_num,
                count,
                &mut |heap, store_access| handler.on_store_access(heap, store_access),
            ),
            SubstateDevice::Store => self.store.drain_substates::<K, E, _>(
                node_id,
                partition_num,
                count,
                &mut |store_access| handler.on_store_access(&self.heap, store_access),
            ),
        }
        .map_err(|e| CallbackError::CallbackError(e))?;

        // TODO: Should check if any substate is locked

        Ok(substates)
    }

    // Substate Virtualization does not apply to this call
    // Should this be prevented at this layer?
    pub fn scan_sorted<'f, E>(
        &mut self,
        device: SubstateDevice,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        handler: &mut impl IOStoreAccessHandler<E>,
    ) -> Result<
        Vec<(SortedKey, IndexedScryptoValue)>,
        CallbackError<CallFrameScanSortedSubstatesError, E>,
    > {
        let substates = match device {
            SubstateDevice::Heap => {
                // This should never be triggered because sorted index store is
                // used by consensus manager only.
                panic!("Unexpected code path")
            }
            SubstateDevice::Store => self
                .store
                .scan_sorted_substates(node_id, partition_num, count, &mut |store_access| {
                    handler.on_store_access(&self.heap, store_access)
                })
                .map_err(|e| CallbackError::CallbackError(e))?,
        };

        // TODO: Should check if any substate is locked

        Ok(substates)
    }

    fn get_substate_internal<'a, E>(
        heap: &'a mut Heap,
        store: &'a mut S,
        location: SubstateDevice,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        handler: &mut impl IOStoreAccessHandler<E>,
    ) -> Result<Option<&'a IndexedScryptoValue>, CallbackError<OpenSubstateError, E>> {
        let value = match location {
            SubstateDevice::Heap => heap.get_substate(node_id, partition_num, substate_key),
            SubstateDevice::Store => store
                .get_substate(node_id, partition_num, substate_key, &mut |store_access| {
                    handler.on_store_access(heap, store_access)
                })
                .map_err(|e| CallbackError::CallbackError(e))?,
        };

        Ok(value)
    }
}
