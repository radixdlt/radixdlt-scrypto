use super::call_frame::*;
use crate::errors::*;
use crate::kernel::kernel_callback_api::*;
use crate::system::system_modules::execution_trace::*;
use crate::track::interface::*;
use crate::types::*;
use radix_engine_interface::api::field_api::*;
use radix_engine_store_interface::db_key_mapper::*;

#[cfg(feature = "radix_engine_tests")]
use super::id_allocator::*;
#[cfg(feature = "radix_engine_tests")]
use super::kernel::*;
#[cfg(feature = "radix_engine_tests")]
use super::substate_io::*;

// Following the convention of Linux Kernel API, https://www.kernel.org/doc/htmldocs/kernel-api/,
// all methods are prefixed by the subsystem of kernel.

/// API for managing nodes
pub trait KernelNodeApi {
    /// Pin a node to it's current device.
    fn kernel_pin_node(&mut self, node_id: NodeId) -> Result<(), RuntimeError>;

    /// Marks a substate as transient, or a substate which was never and will never be persisted
    fn kernel_mark_substate_as_transient(
        &mut self,
        node_id: NodeId,
        partition_num: PartitionNumber,
        key: SubstateKey,
    ) -> Result<(), RuntimeError>;

    /// Allocates a new node id useable for create_node
    fn kernel_allocate_node_id(&mut self, entity_type: EntityType) -> Result<NodeId, RuntimeError>;

    /// Creates a new RENode
    fn kernel_create_node(
        &mut self,
        node_id: NodeId,
        node_substates: NodeSubstates,
    ) -> Result<(), RuntimeError>;

    /// Removes an RENode. Owned children will be possessed by the call frame.
    ///
    /// Dropped substates can't necessary be added back due to visibility loss.
    /// Clients should consider the return value as "raw data".
    fn kernel_drop_node(&mut self, node_id: &NodeId) -> Result<NodeSubstates, RuntimeError>;

    /// Moves module substates from one node to another node.
    ///
    /// The source node must be in heap and lock-free; otherwise a runtime error is returned.
    ///
    /// Note that implementation will not check if the destination already exists.
    fn kernel_move_partition(
        &mut self,
        src_node_id: &NodeId,
        src_partition_number: PartitionNumber,
        dest_node_id: &NodeId,
        dest_partition_number: PartitionNumber,
    ) -> Result<(), RuntimeError>;
}

/// API for managing substates within nodes
pub trait KernelSubstateApi<L> {
    /// Locks a substate to make available for reading and/or writing
    fn kernel_open_substate_with_default<F: FnOnce() -> IndexedScryptoValue>(
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
            None::<fn() -> IndexedScryptoValue>,
            lock_data,
        )
    }

    /// Retrieves info related to a lock
    fn kernel_get_lock_data(&mut self, lock_handle: SubstateHandle) -> Result<L, RuntimeError>;

    /// Drops a lock on some substate, if the lock is writable, updates are flushed to
    /// the store at this point.
    fn kernel_close_substate(&mut self, lock_handle: SubstateHandle) -> Result<(), RuntimeError>;

    /// Reads the value of the substate locked by the given lock handle
    fn kernel_read_substate(
        &mut self,
        lock_handle: SubstateHandle,
    ) -> Result<&IndexedScryptoValue, RuntimeError>;

    /// Writes a value to the substate locked by the given lock handle
    fn kernel_write_substate(
        &mut self,
        lock_handle: SubstateHandle,
        value: IndexedScryptoValue,
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
        value: IndexedScryptoValue,
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
    ) -> Result<Option<IndexedScryptoValue>, RuntimeError>;

    /// Reads substates under a node in sorted lexicographical order
    ///
    /// Clients must ensure that this isn't used in conjunction with virtualized
    /// substates; otherwise, the behavior is undefined
    fn kernel_scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<(SortedKey, IndexedScryptoValue)>, RuntimeError>;

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
    ) -> Result<Vec<(SubstateKey, IndexedScryptoValue)>, RuntimeError>;
}

#[derive(Debug)]
pub struct KernelInvocation<C> {
    pub call_frame_data: C,
    pub args: IndexedScryptoValue,
}

impl<C: CallFrameReferences> KernelInvocation<C> {
    pub fn len(&self) -> usize {
        self.call_frame_data.len() + self.args.len()
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

#[cfg(feature = "radix_engine_tests")]
pub trait KernelTestingApi<'g, M, S>
where
    M: KernelCallbackObject,
    S: SubstateStore,
{
    fn kernel_create_kernel_for_testing(
        substate_io: SubstateIO<'g, S>,
        id_allocator: &'g mut IdAllocator,
        current_frame: CallFrame<M::CallFrameData, M::LockData>,
        prev_frame_stack: Vec<CallFrame<M::CallFrameData, M::LockData>>,
        callback: &'g mut M,
    ) -> Kernel<'g, M, S>;

    fn kernel_current_frame(&self) -> &CallFrame<M::CallFrameData, M::LockData>;
    fn kernel_current_frame_mut(&mut self) -> &mut CallFrame<M::CallFrameData, M::LockData>;

    fn kernel_prev_frame_stack(&self) -> &Vec<CallFrame<M::CallFrameData, M::LockData>>;
    fn kernel_prev_frame_stack_mut(&mut self)
        -> &mut Vec<CallFrame<M::CallFrameData, M::LockData>>;

    fn kernel_substate_io(&self) -> &SubstateIO<'g, S>;
    fn kernel_substate_io_mut(&mut self) -> &mut SubstateIO<'g, S>;

    fn kernel_id_allocator(&self) -> &IdAllocator;
    fn kernel_id_allocator_mut(&mut self) -> &mut &'g mut IdAllocator;

    fn kernel_callback(&self) -> &M;
    fn kernel_callback_mut(&mut self) -> &mut M;
}
