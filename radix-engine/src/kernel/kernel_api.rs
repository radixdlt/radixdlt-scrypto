use crate::errors::*;
use crate::system::kernel_modules::execution_trace::ProofSnapshot;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_substates::{SubstateRef, SubstateRefMut};
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;

use super::actor::ResolvedActor;
use super::call_frame::CallFrameUpdate;
use super::call_frame::RENodeVisibilityOrigin;
use super::heap::HeapRENode;
use super::module_mixer::KernelModuleMixer;

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

    fn kernel_get_substate_ref_mut2<'a, 'b, S>(
        &'b mut self,
        lock_handle: LockHandle,
    ) -> Result<&'a mut S, RuntimeError>
        where
            &'a mut S: From<SubstateRefMut<'a>>,
            'b: 'a;


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

    fn execute<Y, W>(
        self,
        arg: ScryptoValue,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + KernelWasmApi<W> + ClientApi<RuntimeError>,
        W: WasmEngine;
}

pub struct TemporaryResolvedInvocation<E: Executor> {
    pub executor: E,
    pub update: CallFrameUpdate,

    // TODO: Make these two RENodes / Substates
    pub resolved_actor: ResolvedActor,
    pub args: ScryptoValue,
}

pub trait ExecutableInvocation: Invocation {
    type Exec: Executor<Output = Self::Output>;

    fn resolve<Y: KernelSubstateApi>(
        self,
        api: &mut Y,
    ) -> Result<TemporaryResolvedInvocation<Self::Exec>, RuntimeError>;
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
    fn kernel_read_bucket(&mut self, bucket_id: BucketId) -> Option<Resource>;
    fn kernel_read_proof(&mut self, proof_id: BucketId) -> Option<ProofSnapshot>;
}

pub trait KernelModuleApi<E>: KernelNodeApi + KernelSubstateApi + KernelInternalApi {}
