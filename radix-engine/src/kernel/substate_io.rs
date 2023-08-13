use crate::kernel::call_frame::{
    CallFrameDrainSubstatesError, CallFrameRemoveSubstateError, CallFrameScanKeysError,
    CallFrameScanSortedSubstatesError, CallFrameSetSubstateError, CloseSubstateError,
    CreateNodeError, DropNodeError, MoveModuleError, NonGlobalNodeRefs, OpenSubstateError,
    PersistNodeError, WriteSubstateError,
};
use crate::kernel::heap::{Heap, HeapRemoveNodeError};
use crate::kernel::substate_locks::SubstateLocks;
use crate::track::interface::{
    CallbackError, NodeSubstates, StoreAccess, SubstateStore, TrackedSubstateInfo,
};
use radix_engine_common::prelude::{NodeId, PartitionNumber};
use radix_engine_common::types::{SortedU16Key, SubstateKey};
use radix_engine_common::ScryptoSbor;
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::types::IndexedScryptoValue;
use radix_engine_store_interface::db_key_mapper::SubstateKeyContent;
use sbor::prelude::Vec;
use sbor::rust::collections::LinkedList;
use utils::prelude::NonIterMap;

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

pub trait SubstateIOHandler<E> {
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ProcessSubstateIOWriteError {
    NonGlobalRefNotAllowed(NodeId),
    PersistNodeError(PersistNodeError),
}

pub struct SubstateIO<'g, S: SubstateStore> {
    pub heap: Heap,
    pub store: &'g mut S,
    pub non_global_node_refs: NonGlobalNodeRefs,
    pub substate_locks: SubstateLocks<LockData>,
    pub stick_to_heap: NonIterMap<NodeId, ()>,
    pub substate_heap_mount: NonIterMap<(NodeId, PartitionNumber, SubstateKey), ()>,
}

impl<'g, S: SubstateStore + 'g> SubstateIO<'g, S> {
    pub fn new(store: &'g mut S) -> Self {
        Self {
            heap: Heap::new(),
            store,
            non_global_node_refs: NonGlobalNodeRefs::new(),
            substate_locks: SubstateLocks::new(),
            stick_to_heap: NonIterMap::new(),
            substate_heap_mount: NonIterMap::new(),
        }
    }

    /// Creates a new node with partitions/substates at the given device.
    /// No additional node movement occurs (For example, owned nodes in node substates
    /// are not moved and must be done manually using other interfaces)
    pub fn create_node<E>(
        &mut self,
        handler: &mut impl SubstateIOHandler<E>,
        device: SubstateDevice,
        node_id: NodeId,
        node_substates: NodeSubstates,
    ) -> Result<(), CallbackError<CreateNodeError, E>> {
        match device {
            SubstateDevice::Heap => {
                self.heap.create_node(node_id, node_substates);
            }
            SubstateDevice::Store => {
                self.store
                    .create_node(node_id, node_substates, &mut |store_access| {
                        handler.on_store_access(&self.heap, store_access)
                    })
                    .map_err(CallbackError::CallbackError)?;
            }
        }

        Ok(())
    }

    pub fn drop_node(
        &mut self,
        device: SubstateDevice,
        node_id: &NodeId,
    ) -> Result<NodeSubstates, DropNodeError> {
        if self.substate_locks.node_is_locked(node_id) {
            return Err(DropNodeError::SubstateBorrowed(*node_id));
        }

        if self.non_global_node_refs.node_is_referenced(node_id) {
            return Err(DropNodeError::NodeBorrowed(*node_id));
        }

        let node_substates = match device {
            SubstateDevice::Heap => match self.heap.remove_node(node_id) {
                Ok(substates) => substates,
                Err(HeapRemoveNodeError::NodeNotFound(node_id)) => {
                    panic!("Frame owned node {:?} not found in heap", node_id)
                }
            },
            SubstateDevice::Store => {
                panic!("Node drops not supported for store")
            }
        };

        Ok(node_substates)
    }

    pub fn move_node_from_heap_to_store<E>(
        &mut self,
        handler: &mut impl SubstateIOHandler<E>,
        node_id: &NodeId,
    ) -> Result<(), CallbackError<PersistNodeError, E>> {
        // TODO: Add locked substate checks, though this is not required since
        // the system layer currently maintains the invariant that a call frame cannot
        // open a substate of an owned node

        let mut queue = LinkedList::new();
        queue.push_back(node_id.clone());

        while let Some(node_id) = queue.pop_front() {
            if self.stick_to_heap.contains_key(&node_id) {
                return Err(CallbackError::Error(
                    PersistNodeError::CannotPersistHeapMountedNode(node_id),
                ));
            }

            if self.non_global_node_refs.node_is_referenced(&node_id) {
                return Err(CallbackError::Error(PersistNodeError::NodeBorrowed(
                    node_id,
                )));
            }

            let mut node_substates = match self.heap.remove_node(&node_id) {
                Ok(substates) => substates,
                Err(HeapRemoveNodeError::NodeNotFound(node_id)) => {
                    panic!("Frame owned node {:?} not found in heap", node_id)
                }
            };

            let mut mounted = Vec::new();

            for (partition_number, module_substates) in &node_substates {
                for (substate_key, substate_value) in module_substates {
                    for reference in substate_value.references() {
                        if !reference.is_global() {
                            return Err(CallbackError::Error(
                                PersistNodeError::ContainsNonGlobalRef(*reference),
                            ));
                        }
                    }

                    let substate_id = (node_id, *partition_number, substate_key.clone());
                    if self.substate_heap_mount.contains_key(&substate_id) {
                        mounted.push(substate_id);
                        continue;
                    }

                    for node_id in substate_value.owned_nodes() {
                        queue.push_back(*node_id);
                    }
                }
            }

            for (node_id, partition_num, key) in mounted {
                let substate = node_substates
                    .get_mut(&partition_num)
                    .unwrap()
                    .remove(&key)
                    .unwrap();
                self.heap
                    .set_substate(node_id, partition_num, key, substate);
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
        handler: &mut impl SubstateIOHandler<E>,
        src_device: SubstateDevice,
        src_node_id: &NodeId,
        src_partition_number: PartitionNumber,
        dest_device: SubstateDevice,
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
        let module = match src_device {
            SubstateDevice::Heap => self
                .heap
                .remove_module(src_node_id, src_partition_number)
                .map_err(|e| CallbackError::Error(MoveModuleError::HeapRemoveModuleErr(e)))?,
            SubstateDevice::Store => {
                panic!("Partition moves from store not supported.");
            }
        };

        for (substate_key, substate_value) in module {
            match dest_device {
                SubstateDevice::Heap => {
                    self.heap.set_substate(
                        *dest_node_id,
                        dest_partition_number,
                        substate_key,
                        substate_value,
                    );
                }
                SubstateDevice::Store => {
                    if self.substate_heap_mount.contains_key(&(
                        *src_node_id,
                        src_partition_number,
                        substate_key.clone(),
                    )) {
                        continue;
                    }

                    // Recursively move nodes to store
                    for own in substate_value.owned_nodes() {
                        self.move_node_from_heap_to_store(handler, own)
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
        }

        Ok(())
    }

    pub fn open_substate<
        E,
        F: FnMut(&Heap, StoreAccess) -> Result<(), E>,
        D: FnOnce() -> IndexedScryptoValue,
    >(
        &mut self,
        device: SubstateDevice,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        on_store_access: &mut F,
        default: Option<D>,
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
            on_store_access,
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

    pub fn read_substate<H: SubstateReadHandler>(
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
            SubstateDevice::Heap => {
                self.heap
                    .set_substate(node_id, partition_num, substate_key, substate);
            }
            SubstateDevice::Store => {
                self.store
                    .set_substate(node_id, partition_num, substate_key, substate, &mut |_| {
                        Err(())
                    })
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
        device: SubstateDevice,
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

        match device {
            SubstateDevice::Heap => {
                self.heap
                    .set_substate(*node_id, partition_num, substate_key, value);
            }
            SubstateDevice::Store => self
                .store
                .set_substate(
                    *node_id,
                    partition_num,
                    substate_key,
                    value,
                    on_store_access,
                )
                .map_err(CallbackError::CallbackError)?,
        }

        Ok(())
    }

    pub fn remove_substate<'f, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        device: SubstateDevice,
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

        let removed = match device {
            SubstateDevice::Heap => self.heap.remove_substate(node_id, partition_num, key),
            SubstateDevice::Store => self
                .store
                .remove_substate(node_id, partition_num, key, on_store_access)
                .map_err(CallbackError::CallbackError)?,
        };

        Ok(removed)
    }

    pub fn scan_keys<'f, K: SubstateKeyContent, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        device: SubstateDevice,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        on_store_access: &mut F,
    ) -> Result<Vec<SubstateKey>, CallbackError<CallFrameScanKeysError, E>> {
        let keys = match device {
            SubstateDevice::Heap => self.heap.scan_keys(node_id, partition_num, count),
            SubstateDevice::Store => self
                .store
                .scan_keys::<K, E, F>(node_id, partition_num, count, on_store_access)
                .map_err(|e| CallbackError::CallbackError(e))?,
        };

        Ok(keys)
    }

    pub fn drain_substates<'f, K: SubstateKeyContent, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        device: SubstateDevice,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        on_store_access: &mut F,
    ) -> Result<
        Vec<(SubstateKey, IndexedScryptoValue)>,
        CallbackError<CallFrameDrainSubstatesError, E>,
    > {
        let substates = match device {
            SubstateDevice::Heap => self.heap.drain_substates(node_id, partition_num, count),
            SubstateDevice::Store => self
                .store
                .drain_substates::<K, E, F>(node_id, partition_num, count, on_store_access)
                .map_err(|e| CallbackError::CallbackError(e))?,
        };

        // TODO: Should check if any substate is locked

        Ok(substates)
    }

    // Substate Virtualization does not apply to this call
    // Should this be prevented at this layer?
    pub fn scan_sorted<'f, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        device: SubstateDevice,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        on_store_access: &mut F,
    ) -> Result<
        Vec<(SortedU16Key, IndexedScryptoValue)>,
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
                .scan_sorted_substates(node_id, partition_num, count, on_store_access)
                .map_err(|e| CallbackError::CallbackError(e))?,
        };

        // TODO: Should check if any substate is locked

        Ok(substates)
    }

    fn get_substate_internal<'a, E, F: FnMut(&Heap, StoreAccess) -> Result<(), E>>(
        heap: &'a mut Heap,
        store: &'a mut S,
        location: SubstateDevice,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        on_store_access: &mut F,
    ) -> Result<Option<&'a IndexedScryptoValue>, CallbackError<OpenSubstateError, E>> {
        let value = match location {
            SubstateDevice::Heap => heap.get_substate(node_id, partition_num, substate_key),
            SubstateDevice::Store => store
                .get_substate(node_id, partition_num, substate_key, &mut |store_access| {
                    on_store_access(heap, store_access)
                })
                .map_err(|e| CallbackError::CallbackError(e))?,
        };

        Ok(value)
    }
}
