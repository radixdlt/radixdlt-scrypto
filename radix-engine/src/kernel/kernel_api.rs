use super::call_frame::*;
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::kernel_callback_api::*;
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

    /// Drops the handle on some substate, if the handle is a force write, updates are flushed.
    /// No updates should occur if an error is returned.
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

    fn kernel_scan_keys<K: SubstateKeyContent>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<SubstateKey>, RuntimeError>;

    fn kernel_drain_substates<K: SubstateKeyContent>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<(SubstateKey, IndexedScryptoValue)>, RuntimeError>;
}

#[derive(Debug, Clone)]
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

/// API for managing multiple call frame stacks
pub trait KernelStackApi {
    type CallFrameData;

    /// Gets the stack id which is currently being used
    fn kernel_get_stack_id(&mut self) -> Result<usize, RuntimeError>;

    /// Achieves a context switch by switching the underlying callframe/stack
    fn kernel_switch_stack(&mut self, id: usize) -> Result<(), RuntimeError>;

    /// Moves the objects in a scrypto value from the current call frame to another stack
    fn kernel_send_to_stack(
        &mut self,
        id: usize,
        value: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError>;

    /// Sets the call frame data for the current call frame
    fn kernel_set_call_frame_data(&mut self, data: Self::CallFrameData)
        -> Result<(), RuntimeError>;

    /// Returns the owned nodes of the current call frame
    fn kernel_get_owned_nodes(&mut self) -> Result<Vec<NodeId>, RuntimeError>;
}

pub struct SystemState<'a, M: KernelCallbackObject> {
    pub system: &'a mut M,
    pub current_call_frame: &'a M::CallFrameData,
    pub caller_call_frame: &'a M::CallFrameData,
}

/// Internal API for system modules only.
///
/// TODO: Do not use uncosted API within protocol
/// The uncosted APIs should be used by non-consensus related system modules only, i.e. kernel
/// trace and execution trace. All other usages should be migrated away, ideally as a whole.
pub trait KernelInternalApi {
    type System: KernelCallbackObject;

    /// Returns the system.
    #[inline]
    fn kernel_get_system(&mut self) -> &mut Self::System {
        self.kernel_get_system_state().system
    }

    /// Returns the system state.
    fn kernel_get_system_state(&mut self) -> SystemState<'_, Self::System>;

    /// Returns the current stack depth.
    ///
    /// Used by kernel trace, execution trace and costing system modules only.
    fn kernel_get_current_stack_depth_uncosted(&self) -> usize;

    /// Returns the current stack id.
    ///
    /// Used by kernel trace, execution trace, costing system modules and `start_lock_fee` system function only.
    fn kernel_get_current_stack_id_uncosted(&self) -> usize;

    /// Returns the visibility of a node.
    ///
    /// Used by auth system module and `actor_get_node_id` system function only.
    fn kernel_get_node_visibility_uncosted(&self, node_id: &NodeId) -> NodeVisibility;

    /// Returns the value of a substate.
    ///
    /// Used by execution trace system module only.
    fn kernel_read_substate_uncosted(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Option<&IndexedScryptoValue>;
}

pub trait KernelApi:
    KernelNodeApi
    + KernelSubstateApi<<Self::CallbackObject as KernelCallbackObject>::LockData>
    + KernelInvokeApi<<Self::CallbackObject as KernelCallbackObject>::CallFrameData>
    + KernelStackApi<CallFrameData = <Self::CallbackObject as KernelCallbackObject>::CallFrameData>
    + KernelInternalApi<System = Self::CallbackObject>
{
    type CallbackObject: KernelCallbackObject;
}
