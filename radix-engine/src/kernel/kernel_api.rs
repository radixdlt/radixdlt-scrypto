use super::actor::ResolvedActor;
use super::call_frame::CallFrameUpdate;
use super::call_frame::RENodeVisibilityOrigin;
use super::heap::HeapRENode;
use super::module_mixer::KernelModuleMixer;
use crate::errors::*;
use crate::system::kernel_modules::execution_trace::BucketSnapshot;
use crate::system::kernel_modules::execution_trace::ProofSnapshot;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_substates::SubstateRef;
use crate::system::node_substates::SubstateRefMut;
use crate::types::*;
use crate::wasm::WasmEngine;
use bitflags::bitflags;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::ClientComponentApi;

bitflags! {
    #[derive(Sbor)]
    pub struct LockFlags: u32 {
        /// Allows the locked substate to be mutated
        const MUTABLE = 0b00000001;
        /// Checks that the substate locked is unmodified from the beginning of
        /// the transaction. This is used mainly for locking fees in vaults which
        /// requires this in order to be able to support rollbacks
        const UNMODIFIED_BASE = 0b00000010;
        /// Forces a write of a substate even on a transaction failure
        /// Currently used for vault fees.
        const FORCE_WRITE = 0b00000100;
    }
}

impl LockFlags {
    pub fn read_only() -> Self {
        LockFlags::empty()
    }
}

pub struct LockInfo {
    pub offset: SubstateOffset,
}

// Following the convention of Linux Kernel API, https://www.kernel.org/doc/htmldocs/kernel-api/,
// all methods are prefixed by the subsystem of kernel.

pub trait KernelNodeApi {
    /// Removes an RENode and all of it's children from the Heap
    fn kernel_drop_node(&mut self, node_id: RENodeId) -> Result<HeapRENode, RuntimeError>;

    /// Allocates a new node id useable for create_node
    fn kernel_allocate_node_id(&mut self, node_type: RENodeType) -> Result<RENodeId, RuntimeError>;

    /// Creates a new RENode
    /// TODO: Remove, replace with lock_substate + get_ref_mut use
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
        node_id: RENodeId,
        module_id: NodeModuleId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError>;

    fn kernel_get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError>;

    /// Drops a lock
    fn kernel_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError>;

    /// Get a non-mutable reference to a locked substate
    fn kernel_get_substate_ref(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<SubstateRef, RuntimeError>;

    /// Get a mutable reference to a locked substate
    fn kernel_get_substate_ref_mut(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<SubstateRefMut, RuntimeError>;
}

pub trait KernelWasmApi<W: WasmEngine> {
    fn kernel_create_wasm_instance(
        &mut self,
        package_address: PackageAddress,
        handle: LockHandle,
    ) -> Result<W::WasmInstance, RuntimeError>;
}

pub trait Invokable<I: Invocation, E> {
    fn kernel_invoke(&mut self, invocation: I) -> Result<I::Output, E>;
}

pub trait Executor {
    type Output: Debug;

    fn execute<Y, W>(self, api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + KernelWasmApi<W> + ClientApi<RuntimeError>,
        W: WasmEngine;
}

pub trait ExecutableInvocation: Invocation {
    type Exec: Executor<Output = Self::Output>;

    fn resolve<Y: KernelSubstateApi>(
        self,
        api: &mut Y,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>;
}

/// Interface of the Kernel, for Kernel modules.
pub trait KernelApi<W: WasmEngine, E>:
    KernelNodeApi + KernelSubstateApi + KernelWasmApi<W>
{
}

/// Internal API for kernel modules.
/// No kernel state changes are expected as of a result of invoking such APIs, except updating returned references.
pub trait KernelInternalApi {
    fn kernel_get_module_state(&mut self) -> &mut KernelModuleMixer;
    fn kernel_get_node_visibility_origin(
        &self,
        node_id: RENodeId,
    ) -> Option<RENodeVisibilityOrigin>;
    fn kernel_get_current_depth(&self) -> usize;

    // TODO: Remove
    fn kernel_get_current_actor(&self) -> Option<ResolvedActor>;

    /* Super unstable interface, specifically for `ExecutionTrace` kernel module */
    fn kernel_read_bucket(&mut self, bucket_id: BucketId) -> Option<BucketSnapshot>;
    fn kernel_read_proof(&mut self, proof_id: BucketId) -> Option<ProofSnapshot>;
}

pub trait KernelModuleApi<E>:
    KernelNodeApi + KernelSubstateApi + KernelInternalApi + ClientComponentApi<E>
{
}
