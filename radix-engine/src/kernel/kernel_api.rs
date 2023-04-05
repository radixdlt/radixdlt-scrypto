use super::call_frame::RefType;
use super::heap::HeapNode;
use super::module_mixer::KernelModuleMixer;
use crate::errors::*;
use crate::kernel::actor::Actor;
use crate::system::kernel_modules::execution_trace::BucketSnapshot;
use crate::system::kernel_modules::execution_trace::ProofSnapshot;
use crate::system::node_init::NodeInit;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::*;

pub struct LockInfo {
    pub node_id: NodeId,
    pub module_id: TypedModuleId,
    pub substate_key: SubstateKey,
    pub flags: LockFlags,
}

// Following the convention of Linux Kernel API, https://www.kernel.org/doc/htmldocs/kernel-api/,
// all methods are prefixed by the subsystem of kernel.

pub trait KernelNodeApi {
    /// Removes an RENode and all of it's children from the Heap
    fn kernel_drop_node(&mut self, node_id: &NodeId) -> Result<HeapNode, RuntimeError>;

    /// TODO: Cleanup
    fn kernel_allocate_virtual_node_id(&mut self, node_id: NodeId) -> Result<(), RuntimeError>;

    /// Allocates a new node id useable for create_node
    fn kernel_allocate_node_id(&mut self, node_type: EntityType) -> Result<NodeId, RuntimeError>;

    /// Creates a new RENode
    /// TODO: merge `node_init` and `module_init`?
    fn kernel_create_node(
        &mut self,
        node_id: NodeId,
        node_init: NodeInit,
        module_init: BTreeMap<TypedModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>,
    ) -> Result<(), RuntimeError>;
}

pub trait KernelSubstateApi {
    fn kernel_lock_substate(
        &mut self,
        node_id: &NodeId,
        module_id: TypedModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError>;

    fn kernel_get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError>;

    fn kernel_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError>;

    fn kernel_read_substate(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<&IndexedScryptoValue, RuntimeError>;

    fn kernel_write_substate(
        &mut self,
        lock_handle: LockHandle,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError>;
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
    fn kernel_get_node_info(&self, node_id: &NodeId) -> Option<(RefType, bool)>;

    fn kernel_get_current_depth(&self) -> usize;

    // TODO: Remove
    fn kernel_get_current_actor(&mut self) -> Option<Actor>;

    /* Super unstable interface, specifically for `ExecutionTrace` kernel module */
    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot>;
    fn kernel_read_proof(&mut self, proof_id: &NodeId) -> Option<ProofSnapshot>;
}

pub trait KernelModuleApi<E>:
    KernelNodeApi
    + KernelSubstateApi
    + KernelInternalApi
    + KernelInvokeApi<VirtualLazyLoadInvocation, E>
    + ClientObjectApi<E> // This is to allow invocation for kernel modules
{
}
