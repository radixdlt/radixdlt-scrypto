use super::call_frame::*;
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::kernel_callback_api::*;
use crate::system::system_modules::execution_trace::*;
use crate::track::interface::*;
use radix_engine_interface::api::field_api::*;
use radix_substate_store_interface::db_key_mapper::*;

pub struct DroppedNode {
    pub substates: NodeSubstates,
    pub pinned_to_heap: bool,
}

// Following the convention of Linux Kernel API, https://www.kernel.org/doc/htmldocs/kernel-api/,
// all methods are prefixed by the subsystem of kernel.

/// API for managing nodes
pub trait KernelNodeApi {
    /// Pin a node to it's current device.
    fn kernel_pin_node(&mut self, node_id: NodeId) -> Result<(), RuntimeError>;

    /// Allocates a new node id useable for create_node
    fn kernel_allocate_node_id(&mut self, entity_type: EntityType) -> Result<NodeId, RuntimeError>;

    /// Creates a new RENode
    fn kernel_create_node(
        &mut self,
        node_id: NodeId,
        node_substates: NodeSubstates,
    ) -> Result<(), RuntimeError>;

    fn kernel_create_node_from(
        &mut self,
        node_id: NodeId,
        partitions: BTreeMap<PartitionNumber, (NodeId, PartitionNumber)>,
    ) -> Result<(), RuntimeError>;

    /// Removes an RENode. Owned children will be possessed by the call frame.
    ///
    /// Dropped substates can't necessary be added back due to visibility loss.
    /// Clients should consider the return value as "raw data".
    fn kernel_drop_node(&mut self, node_id: &NodeId) -> Result<DroppedNode, RuntimeError>;
}

/// API for managing substates within nodes
pub trait KernelSubstateApi<L> {
    /// Marks a substate as transient, or a substate which was never and will never be persisted
    fn kernel_mark_substate_as_transient(
        &mut self,
        node_id: NodeId,
        partition_num: PartitionNumber,
        key: SubstateKey,
    ) -> Result<(), RuntimeError>;

    /// Locks a substate to make available for reading and/or writing
    fn kernel_open_substate_with_default<F: FnOnce() -> IndexedOwnedScryptoValue>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<F>,
        lock_data: L,
    ) -> Result<SubstateHandle, RuntimeError>;

    fn kernel_open_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        lock_data: L,
    ) -> Result<SubstateHandle, RuntimeError> {
        self.kernel_open_substate_with_default(
            node_id,
            partition_num,
            substate_key,
            flags,
            None::<fn() -> IndexedOwnedScryptoValue>,
            lock_data,
        )
    }

    /// Retrieves info related to a lock
    fn kernel_get_lock_data(&mut self, lock_handle: SubstateHandle) -> Result<L, RuntimeError>;

    /// Drops the handle on some substate, if the handle is a force write, updates are flushed.
    /// No updates should occur if an error is returned.
    fn kernel_close_substate(&mut self, lock_handle: SubstateHandle) -> Result<(), RuntimeError>;

    /// Reads the value of the substate locked by the given lock handle
    fn kernel_read_substate(
        &mut self,
        lock_handle: SubstateHandle,
    ) -> Result<&IndexedOwnedScryptoValue, RuntimeError>;

    /// Writes a value to the substate locked by the given lock handle
    fn kernel_write_substate(
        &mut self,
        lock_handle: SubstateHandle,
        value: IndexedOwnedScryptoValue,
    ) -> Result<(), RuntimeError>;

    /// Sets a value to a substate without checking for the original value.
    ///
    /// Clients must ensure that this isn't used in conjunction with shardable
    /// substates; otherwise, the behavior is undefined
    fn kernel_set_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        value: IndexedOwnedScryptoValue,
    ) -> Result<(), RuntimeError>;

    /// Removes a substate from a node and returns the original value.
    ///
    /// Clients must ensure that this isn't used in conjunction with virtualized
    /// substates; otherwise, the behavior is undefined
    fn kernel_remove_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<Option<IndexedOwnedScryptoValue>, RuntimeError>;

    /// Reads substates under a node in sorted lexicographical order
    ///
    /// Clients must ensure that this isn't used in conjunction with virtualized
    /// substates; otherwise, the behavior is undefined
    fn kernel_scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<(SortedKey, IndexedOwnedScryptoValue)>, RuntimeError>;

    fn kernel_scan_keys<K: SubstateKeyContent + 'static>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<SubstateKey>, RuntimeError>;

    fn kernel_drain_substates<K: SubstateKeyContent + 'static>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<(SubstateKey, IndexedOwnedScryptoValue)>, RuntimeError>;
}

#[derive(Debug, Clone)]
pub struct KernelInvocation<'v, C> {
    pub call_frame_data: C,
    pub args: IndexedScryptoValue<'v>,
}

impl<'v, C: CallFrameReferences> KernelInvocation<'v, C> {
    pub fn len(&self) -> usize {
        self.call_frame_data.len() + self.args.payload_len()
    }
}

/// API for invoking a function creating a new call frame and passing
/// control to the callee
pub trait KernelInvokeApi<C> {
    fn kernel_invoke(
        &mut self,
        invocation: Box<KernelInvocation<C>>,
    ) -> Result<IndexedScryptoValue, RuntimeError>;
}

pub struct SystemState<'a, M: KernelCallbackObject> {
    pub system: &'a mut M,
    pub current_call_frame: &'a M::CallFrameData,
    pub caller_call_frame: &'a M::CallFrameData,
}

/// Internal API for kernel modules.
/// No kernel state changes are expected as of a result of invoking such APIs, except updating returned references.
pub trait KernelInternalApi<M: KernelCallbackObject> {
    /// Retrieves data associated with the kernel upstream layer (system)
    fn kernel_get_system(&mut self) -> &mut M {
        self.kernel_get_system_state().system
    }

    fn kernel_get_system_state(&mut self) -> SystemState<'_, M>;

    /// Gets the number of call frames that are currently in the call frame stack
    fn kernel_get_current_depth(&self) -> usize;

    /// Returns the visibility of a node
    fn kernel_get_node_visibility(&self, node_id: &NodeId) -> NodeVisibility;

    /* Super unstable interface, specifically for `ExecutionTrace` kernel module */
    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot>;
    fn kernel_read_proof(&mut self, proof_id: &NodeId) -> Option<ProofSnapshot>;
}

pub trait KernelApi<M: KernelCallbackObject>:
    KernelNodeApi
    + KernelSubstateApi<M::LockData>
    + KernelInvokeApi<M::CallFrameData>
    + KernelInternalApi<M>
{
}
