use radix_engine_common::prelude::{NodeId, PartitionNumber};
use radix_engine_common::types::SubstateKey;
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::types::IndexedScryptoValue;
use crate::kernel::call_frame::{OpenSubstateError, SubstateLocation};
use crate::kernel::heap::Heap;
use crate::kernel::substate_locks::SubstateLocks;
use crate::track::interface::{CallbackError, StoreAccess, SubstateStore, TrackedSubstateInfo};

pub struct SubstateIO<'g, S: SubstateStore> {
    /// Heap
    pub heap: Heap,
    /// Store
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
    ) -> Result<(u32, &IndexedScryptoValue, SubstateLocation), CallbackError<OpenSubstateError, E>> {
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
                match self.store.get_tracked_substate_info(node_id, partition_num, substate_key) {
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

        let substate_value = Self::get_substate_internal(&mut self.heap, &mut self.store, substate_location, node_id, partition_num, substate_key, on_store_access, default)?;

        let global_lock_handle =
            match self.substate_locks.lock(node_id, partition_num, substate_key, flags, substate_location) {
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
            SubstateLocation::Heap => {
                heap
                    .get_substate_or_default(node_id, partition_num, substate_key, || {
                        default.map(|f| f())
                    })
                    .map_err(|e| {
                        CallbackError::Error(OpenSubstateError::HeapError(e))
                    })?
            }
            SubstateLocation::Store => {
                store
                    .get_substate_or_default(
                        node_id,
                        partition_num,
                        substate_key,
                        on_store_access,
                        || default.map(|f| f()),
                    )
                    .map_err(|x| {
                        x.map(|e| OpenSubstateError::TrackError(Box::new(e)))
                    })?
            }
        };

        Ok(value)
    }
}
