use crate::kernel::call_frame::{CallFrameDrainSubstatesError, CallFrameRemoveSubstateError, CallFrameScanKeysError, CallFrameScanSortedSubstatesError, CallFrameSetSubstateError, OpenSubstateError, PersistNodeError, ReadSubstateError, WriteSubstateError};
use crate::kernel::heap::{Heap, HeapRemoveNodeError};
use crate::kernel::substate_locks::SubstateLocks;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::track::interface::{CallbackError, StoreAccess, SubstateStore, TrackedSubstateInfo};
use radix_engine_common::prelude::{NodeId, PartitionNumber, RESOURCE_PACKAGE};
use radix_engine_common::types::SubstateKey;
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::blueprints::resource::{
    FUNGIBLE_BUCKET_BLUEPRINT, FUNGIBLE_PROOF_BLUEPRINT, NON_FUNGIBLE_BUCKET_BLUEPRINT,
    NON_FUNGIBLE_PROOF_BLUEPRINT,
};
use radix_engine_interface::prelude::{BlueprintInfo, ObjectInfo};
use radix_engine_interface::types::{
    IndexedScryptoValue, TypeInfoField, TYPE_INFO_FIELD_PARTITION,
};
use radix_engine_store_interface::db_key_mapper::SubstateKeyContent;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SubstateLocation {
    Heap,
    Store,
}

pub struct SubstateIO<'g, S: SubstateStore> {
    pub heap: Heap,
    pub store: &'g mut S,
    pub substate_locks: SubstateLocks,
}

impl<'g, S: SubstateStore> SubstateIO<'g, S> {
    pub fn open_substate<E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        on_store_access: F,
        default: Option<fn() -> IndexedScryptoValue>,
    ) -> Result<(u32, &IndexedScryptoValue, SubstateLocation), CallbackError<OpenSubstateError, E>>
    {
        let substate_location = if self.heap.contains_node(node_id) {
            if flags.contains(LockFlags::UNMODIFIED_BASE) {
                return Err(CallbackError::Error(
                    OpenSubstateError::LockUnmodifiedBaseOnHeapNode,
                ));
            }

            SubstateLocation::Heap
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
            SubstateLocation::Store
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

        let global_lock_handle = match self.substate_locks.lock(
            node_id,
            partition_num,
            substate_key,
            flags,
            substate_location,
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

    pub fn read_substate(&mut self, global_lock_handle: u32) -> &IndexedScryptoValue {
        let (node_id, partition_num, substate_key, _, substate_location) =
            self.substate_locks.get(global_lock_handle);

        let substate = match substate_location {
            SubstateLocation::Heap => self
                .heap
                .get_substate(node_id, *partition_num, substate_key)
                .unwrap(),
            SubstateLocation::Store => self
                .store
                .get_substate::<(), _>(node_id, *partition_num, substate_key, &mut |store_access| {
                    panic!("Getting substate on handled substate should not incur a store access.")
                })
                .unwrap(),
        };

        substate
    }

    pub fn write_substate(
        &mut self,
        global_lock_handle: u32,
        substate: IndexedScryptoValue,
    ) -> Result<(), WriteSubstateError> {
        let (node_id, partition_num, substate_key, flags, substate_location) =
            self.substate_locks.get(global_lock_handle);
        if !flags.contains(LockFlags::MUTABLE) {
            return Err(WriteSubstateError::NoWritePermission);
        }

        match substate_location {
            SubstateLocation::Heap => {
                self.heap.set_substate(
                    node_id.clone(),
                    partition_num.clone(),
                    substate_key.clone(),
                    substate,
                );
            }
            SubstateLocation::Store => {
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
    ) -> (
        NodeId,
        PartitionNumber,
        SubstateKey,
        LockFlags,
        SubstateLocation,
    ) {
        let (node_id, partition_num, substate_key, flags, location) =
            self.substate_locks.unlock(global_lock_handle);

        if flags.contains(LockFlags::FORCE_WRITE) {
            self.store
                .force_write(&node_id, &partition_num, &substate_key);
        }

        (node_id, partition_num, substate_key, flags, location)
    }

    pub fn set_substate<'f, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        value: IndexedScryptoValue,
        on_store_access: &mut F,
    ) -> Result<(), CallbackError<CallFrameSetSubstateError, E>> {
        if self.substate_locks.is_locked(
            node_id,
            partition_num,
            &substate_key,
        ) {
            return Err(CallbackError::Error(CallFrameSetSubstateError::SubstateLocked(node_id.clone(), partition_num, substate_key)));
        }

        if self.heap.contains_node(node_id) {
            self
                .heap
                .set_substate(*node_id, partition_num, substate_key, value);
        } else {
            self
                .store
                .set_substate(*node_id, partition_num, substate_key, value, on_store_access)
                .map_err(CallbackError::CallbackError)?
        };

        Ok(())
    }

    pub fn remove_substate<'f, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        key: &SubstateKey,
        on_store_access: F,
    ) -> Result<Option<IndexedScryptoValue>, CallbackError<CallFrameRemoveSubstateError, E>> {
        if self
            .substate_locks
            .is_locked(node_id, partition_num, key)
        {
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
            self
                .store
                .remove_substate(node_id, partition_num, key, on_store_access)
                .map_err(CallbackError::CallbackError)?
        };

        Ok(removed)
    }

    pub fn scan_keys<
        'f,
        K: SubstateKeyContent,
        E,
        F: FnMut(StoreAccess) -> Result<(), E>,
    >(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        on_store_access: F,
    ) -> Result<Vec<SubstateKey>, CallbackError<CallFrameScanKeysError, E>> {
        let keys = if self.heap.contains_node(node_id) {
            self.heap.scan_keys(node_id, partition_num, count)
        } else {
            self
                .store
                .scan_keys::<K, E, F>(node_id, partition_num, count, on_store_access)
                .map_err(|e| CallbackError::CallbackError(e))?
        };

        Ok(keys)
    }

    pub fn drain_substates<
        'f,
        K: SubstateKeyContent,
        E,
        F: FnMut(StoreAccess) -> Result<(), E>,
    >(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
        on_store_access: F,
    ) -> Result<
        Vec<(SubstateKey, IndexedScryptoValue)>,
        CallbackError<CallFrameDrainSubstatesError, E>,
    > {
        let substates = if self.heap.contains_node(node_id) {
            self
                .heap
                .drain_substates(node_id, partition_num, count)
        } else {
            self
                .store
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
        on_store_access: F,
    ) -> Result<Vec<IndexedScryptoValue>, CallbackError<CallFrameScanSortedSubstatesError, E>> {
        let substates = if self.heap.contains_node(node_id) {
            // This should never be triggered because sorted index store is
            // used by consensus manager only.
            panic!("Unexpected code path")
        } else {
            self
                .store
                .scan_sorted_substates(node_id, partition_num, count, on_store_access)
                .map_err(|e| CallbackError::CallbackError(e))?
        };

        // TODO: Should check if any substate is locked

        Ok(substates)
    }



    pub fn move_node_to_store<E, F: FnMut(StoreAccess) -> Result<(), E>>(
        &mut self,
        node_id: &NodeId,
        on_store_access: &mut F,
    ) -> Result<(), CallbackError<PersistNodeError, E>> {
        // FIXME: Use unified approach to node configuration
        let can_be_stored = if node_id.is_global() {
            true
        } else {
            let type_info = Self::get_heap_type_info(node_id, &mut self.heap);

            if let Some(type_info) = type_info {
                match type_info {
                    TypeInfoSubstate::Object(ObjectInfo {
                        blueprint_info: BlueprintInfo { blueprint_id, .. },
                        ..
                    }) if blueprint_id.package_address == RESOURCE_PACKAGE
                        && (blueprint_id.blueprint_name == FUNGIBLE_BUCKET_BLUEPRINT
                            || blueprint_id.blueprint_name == NON_FUNGIBLE_BUCKET_BLUEPRINT
                            || blueprint_id.blueprint_name == FUNGIBLE_PROOF_BLUEPRINT
                            || blueprint_id.blueprint_name == NON_FUNGIBLE_PROOF_BLUEPRINT) =>
                    {
                        false
                    }
                    _ => true,
                }
            } else {
                false
            }
        };
        if !can_be_stored {
            return Err(CallbackError::Error(PersistNodeError::CantBeStored(
                node_id.clone(),
            )));
        }

        let node_substates = match self.heap.remove_node(node_id) {
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
                            PersistNodeError::NonGlobalRefNotAllowed(*reference),
                        ));
                    }
                }

                for node in substate_value.owned_nodes() {
                    self.move_node_to_store(node, on_store_access)?;
                }
            }
        }

        self.store
            .create_node(node_id.clone(), node_substates, on_store_access)
            .map_err(CallbackError::CallbackError)?;

        Ok(())
    }

    // TODO: Remove
    fn get_heap_type_info(node_id: &NodeId, heap: &mut Heap) -> Option<TypeInfoSubstate> {
        if let Some(substate) = heap.get_substate(
            node_id,
            TYPE_INFO_FIELD_PARTITION,
            &TypeInfoField::TypeInfo.into(),
        ) {
            let type_info: TypeInfoSubstate = substate.as_typed().unwrap();
            Some(type_info)
        } else {
            None
        }
    }

    fn get_substate_internal<'a, E, F: FnMut(StoreAccess) -> Result<(), E>>(
        heap: &'a mut Heap,
        store: &'a mut S,
        location: SubstateLocation,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        on_store_access: F,
        default: Option<fn() -> IndexedScryptoValue>,
    ) -> Result<&'a IndexedScryptoValue, CallbackError<OpenSubstateError, E>> {
        let value = match location {
            SubstateLocation::Heap => heap
                .get_substate_or_default(node_id, partition_num, substate_key, || {
                    default.map(|f| f())
                })
                .map_err(|e| CallbackError::Error(OpenSubstateError::HeapError(e)))?,
            SubstateLocation::Store => store
                .get_substate_or_default(
                    node_id,
                    partition_num,
                    substate_key,
                    on_store_access,
                    || default.map(|f| f()),
                )
                .map_err(|x| x.map(|e| OpenSubstateError::TrackError(Box::new(e))))?,
        };

        Ok(value)
    }
}
