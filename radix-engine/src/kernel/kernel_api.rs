use super::actor::MethodActor;
use super::call_frame::RefType;
use crate::errors::*;
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::system_callback::SystemInvocation;
use crate::system::system_modules::execution_trace::BucketSnapshot;
use crate::system::system_modules::execution_trace::ProofSnapshot;
use crate::track::interface::NodeSubstates;
use crate::types::*;
use radix_engine_interface::api::field_lock_api::LockFlags;

// Following the convention of Linux Kernel API, https://www.kernel.org/doc/htmldocs/kernel-api/,
// all methods are prefixed by the subsystem of kernel.

/// API for managing nodes
pub trait KernelNodeApi {
    /// Removes an RENode and all of it's children from the Heap
    fn kernel_drop_node(&mut self, node_id: &NodeId) -> Result<NodeSubstates, RuntimeError>;

    /// TODO: Remove
    fn kernel_allocate_virtual_node_id(&mut self, node_id: NodeId) -> Result<(), RuntimeError>;

    /// Allocates a new node id useable for create_node
    fn kernel_allocate_node_id(&mut self, node_type: EntityType) -> Result<NodeId, RuntimeError>;

    /// Creates a new RENode
    fn kernel_create_node(
        &mut self,
        node_id: NodeId,
        node_substates: NodeSubstates,
    ) -> Result<(), RuntimeError>;
}

/// Info regarding the substate locked as well as what type of lock
pub struct LockInfo<L> {
    pub node_id: NodeId,
    pub module_num: ModuleNumber,
    pub substate_key: SubstateKey,
    pub flags: LockFlags,
    pub data: L,
}

/// API for managing substates within nodes
pub trait KernelSubstateApi<L> {
    /// Locks a substate to make available for reading and/or writing
    fn kernel_lock_substate_with_default(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<fn() -> IndexedScryptoValue>,
        lock_data: L,
    ) -> Result<LockHandle, RuntimeError>;

    fn kernel_lock_substate(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        lock_data: L,
    ) -> Result<LockHandle, RuntimeError> {
        self.kernel_lock_substate_with_default(
            node_id,
            module_num,
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
    fn kernel_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError>;

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
        module_num: ModuleNumber,
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
        module_num: ModuleNumber,
        substate_key: &SubstateKey,
    ) -> Result<Option<IndexedScryptoValue>, RuntimeError>;

    /// Reads substates under a node in sorted lexicographical order
    ///
    /// Clients must ensure that this isn't used in conjunction with virtualized
    /// substates; otherwise, the behavior is undefined
    fn kernel_scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError>;

    fn kernel_scan_substates(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError>;

    fn kernel_take_substates(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError>;
}

#[derive(Debug)]
pub struct KernelInvocation<I: Debug> {
    pub sys_invocation: I,
    // TODO: Make these two RENodes / Substates
    pub resolved_actor: Actor,
    pub args: IndexedScryptoValue,
}

impl KernelInvocation<SystemInvocation> {
    pub fn len(&self) -> usize {
        self.sys_invocation.blueprint.len() + self.sys_invocation.ident.len() + self.args.len()
    }
}

impl<I: Debug> KernelInvocation<I> {
    pub fn get_update(
        &self,
        allow_local_references: bool,
    ) -> Result<CallFrameUpdate, RuntimeError> {
        let mut call_frame_update =
            CallFrameUpdate::from_indexed_scrypto_value(&self.args, allow_local_references)
                .map_err(|e| RuntimeError::KernelError(KernelError::CallFrameUpdateError(e)))?;
        // TODO: remove
        match self.resolved_actor {
            Actor::Method(MethodActor { node_id, .. }) => {
                call_frame_update.add_reference(node_id);
            }
            Actor::Function { .. } | Actor::VirtualLazyLoad { .. } => {}
        }
        Ok(call_frame_update)
    }
}

/// API for invoking a function creating a new call frame and passing
/// control to the callee
pub trait KernelInvokeApi<I: Debug> {
    fn kernel_invoke(
        &mut self,
        invocation: Box<KernelInvocation<I>>,
    ) -> Result<IndexedScryptoValue, RuntimeError>;
}

pub struct SystemState<'a, M: KernelCallbackObject> {
    pub system: &'a mut M,
    pub current: Option<&'a Actor>,
    pub caller: Option<&'a Actor>,
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
    fn kernel_get_node_info(&self, node_id: &NodeId) -> Option<(RefType, bool)>;

    /* Super unstable interface, specifically for `ExecutionTrace` kernel module */
    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot>;
    fn kernel_read_proof(&mut self, proof_id: &NodeId) -> Option<ProofSnapshot>;
}

pub trait KernelApi<M: KernelCallbackObject>:
    KernelNodeApi
    + KernelSubstateApi<M::LockData>
    + KernelInvokeApi<M::Invocation>
    + KernelInternalApi<M>
{
}
