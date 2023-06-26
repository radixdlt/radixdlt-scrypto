use super::call_frame::NodeVisibility;
use crate::errors::*;
use crate::kernel::actor::Actor;
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::system_modules::execution_trace::BucketSnapshot;
use crate::system::system_modules::execution_trace::ProofSnapshot;
use crate::track::interface::NodeSubstates;
use crate::types::*;
use radix_engine_interface::api::field_lock_api::LockFlags;

// Following the convention of Linux Kernel API, https://www.kernel.org/doc/htmldocs/kernel-api/,
// all methods are prefixed by the subsystem of kernel.

/// API for managing nodes
pub trait KernelNodeApi {
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
    fn kernel_move_module(
        &mut self,
        src_node_id: &NodeId,
        src_partition_number: PartitionNumber,
        dest_node_id: &NodeId,
        dest_partition_number: PartitionNumber,
    ) -> Result<(), RuntimeError>;
}

/// Info regarding the substate locked as well as what type of lock
pub struct LockInfo<L> {
    pub node_id: NodeId,
    pub partition_num: PartitionNumber,
    pub substate_key: SubstateKey,
    pub flags: LockFlags,
    pub data: L,
}

/// API for managing substates within nodes
pub trait KernelSubstateApi<L> {
    /// Locks a substate to make available for reading and/or writing
    fn kernel_open_substate_with_default(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<fn() -> IndexedScryptoValue>,
        lock_data: L,
    ) -> Result<LockHandle, RuntimeError>;

    fn kernel_open_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        lock_data: L,
    ) -> Result<LockHandle, RuntimeError> {
        self.kernel_open_substate_with_default(
            node_id,
            partition_num,
            substate_key,
            flags,
            None,
            lock_data,
        )
    }

    /// Retrieves info related to a lock
    fn kernel_get_lock_info(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<LockInfo<L>, RuntimeError>;

    /// Drops a lock on some substate, if the lock is writable, updates are flushed to
    /// the store at this point.
    fn kernel_close_substate(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError>;

    /// Reads the value of the substate locked by the given lock handle
    fn kernel_read_substate(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<&IndexedScryptoValue, RuntimeError>;

    /// Writes a value to the substate locked by the given lock handle
    fn kernel_write_substate(
        &mut self,
        lock_handle: LockHandle,
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
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError>;

    fn kernel_scan_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError>;

    fn kernel_take_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError>;
}

#[derive(Debug)]
pub struct KernelInvocation {
    /// FIXME: redo actor generification
    /// Temporarily restored as there's a large conflict with `develop` branch
    pub actor: Actor,
    pub args: IndexedScryptoValue,
}

impl KernelInvocation {
    pub fn len(&self) -> usize {
        self.actor.len() + self.args.len()
    }
}

/// API for invoking a function creating a new call frame and passing
/// control to the callee
pub trait KernelInvokeApi {
    fn kernel_invoke(
        &mut self,
        invocation: Box<KernelInvocation>,
    ) -> Result<IndexedScryptoValue, RuntimeError>;
}

pub struct SystemState<'a, M: KernelCallbackObject> {
    pub system: &'a mut M,
    pub current: &'a Actor,
    pub caller: &'a Actor,
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

    // TODO: Cleanup
    fn kernel_get_node_visibility(&self, node_id: &NodeId) -> NodeVisibility;

    /* Super unstable interface, specifically for `ExecutionTrace` kernel module */
    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot>;
    fn kernel_read_proof(&mut self, proof_id: &NodeId) -> Option<ProofSnapshot>;
}

pub trait KernelApi<M: KernelCallbackObject>:
    KernelNodeApi + KernelSubstateApi<M::LockData> + KernelInvokeApi + KernelInternalApi<M>
{
}
