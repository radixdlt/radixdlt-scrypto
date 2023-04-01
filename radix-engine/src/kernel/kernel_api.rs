use super::call_frame::RENodeVisibilityOrigin;
use super::heap::HeapRENode;
use super::module_mixer::KernelModuleMixer;
use crate::errors::*;
use crate::kernel::actor::Actor;
use crate::system::kernel_modules::execution_trace::BucketSnapshot;
use crate::system::kernel_modules::execution_trace::ProofSnapshot;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_substates::SubstateRef;
use crate::system::node_substates::SubstateRefMut;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::*;

pub struct LockInfo {
    pub node_id: RENodeId,
    pub module_id: NodeModuleId,
    pub offset: SubstateOffset,
    pub flags: LockFlags,
}

// Following the convention of Linux Kernel API, https://www.kernel.org/doc/htmldocs/kernel-api/,
// all methods are prefixed by the subsystem of kernel.

pub trait KernelNodeApi {
    /// Removes an RENode and all of it's children from the Heap
    fn kernel_drop_node(&mut self, node_id: &RENodeId) -> Result<HeapRENode, RuntimeError>;

    /// TODO: Cleanup
    fn kernel_allocate_virtual_node_id(&mut self, node_id: RENodeId) -> Result<(), RuntimeError>;

    /// Allocates a new node id useable for create_node
    fn kernel_allocate_node_id(
        &mut self,
        node_type: AllocateEntityType,
    ) -> Result<RENodeId, RuntimeError>;

    /// Creates a new RENode
    fn kernel_create_node(
        &mut self,
        node_id: RENodeId,
        init: RENodeInit,
        node_module_init: BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError>;
}

pub trait KernelSubstateApi {
    /// Locks a visible substate
    fn kernel_lock_substate(
        &mut self,
        node_id: &RENodeId,
        module_id: NodeModuleId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError>;

    fn kernel_get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError>;

    /// Drops a lock
    fn kernel_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError>;

    /// Get a non-mutable reference to a locked substate
    fn kernel_read_substate(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<IndexedScryptoValue, RuntimeError>;

    fn kernel_get_substate_ref<'a, 'b, S>(
        &'b mut self,
        lock_handle: LockHandle,
    ) -> Result<&'a S, RuntimeError>
    where
        &'a S: From<SubstateRef<'a>>,
        'b: 'a;

    fn kernel_get_substate_ref_mut<'a, 'b, S>(
        &'b mut self,
        lock_handle: LockHandle,
    ) -> Result<&'a mut S, RuntimeError>
    where
        &'a mut S: From<SubstateRefMut<'a>>,
        'b: 'a;
}

pub trait KernelWasmApi<W: WasmEngine> {
    fn kernel_create_wasm_instance(
        &mut self,
        package_address: PackageAddress,
        handle: LockHandle,
    ) -> Result<W::WasmInstance, RuntimeError>;
}

pub trait KernelInvokeApi<I: Invocation, E> {
    fn kernel_invoke(&mut self, invocation: Box<I>) -> Result<I::Output, E>;
}

/// Interface of the Kernel, for Kernel modules.
pub trait KernelApi<W: WasmEngine, E>:
    KernelNodeApi
    + KernelSubstateApi
    + KernelWasmApi<W>
    + KernelInvokeApi<FunctionInvocation, E>
    + KernelInvokeApi<MethodInvocation, E>
{
}

/// Internal API for kernel modules.
/// No kernel state changes are expected as of a result of invoking such APIs, except updating returned references.
pub trait KernelInternalApi {
    fn kernel_get_module_state(&mut self) -> &mut KernelModuleMixer;

    // TODO: Cleanup
    fn kernel_get_node_info(&self, node_id: RENodeId) -> Option<(RENodeVisibilityOrigin, bool)>;

    fn kernel_get_current_depth(&self) -> usize;

    // TODO: Remove
    fn kernel_get_current_actor(&mut self) -> Option<Actor>;

    /* Super unstable interface, specifically for `ExecutionTrace` kernel module */
    fn kernel_read_bucket(&mut self, bucket_id: ObjectId) -> Option<BucketSnapshot>;
    fn kernel_read_proof(&mut self, proof_id: ObjectId) -> Option<ProofSnapshot>;
}

pub trait KernelModuleApi<E>:
    KernelNodeApi
    + KernelSubstateApi
    + KernelInternalApi
    + KernelInvokeApi<VirtualLazyLoadInvocation, E>
    + ClientObjectApi<E>
{
}
