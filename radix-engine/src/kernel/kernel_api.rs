use super::call_frame::RefType;
use super::heap::HeapNode;
use super::module_mixer::KernelModuleMixer;
use crate::errors::*;
use crate::kernel::actor::{Actor, ExecutionMode};
use crate::kernel::call_frame::CallFrameUpdate;
use crate::system::system_upstream::SystemInvocation;
use crate::system::kernel_modules::execution_trace::BucketSnapshot;
use crate::system::kernel_modules::execution_trace::ProofSnapshot;
use crate::system::node_init::NodeInit;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::*;
use crate::vm::ScryptoInterpreter;

pub struct LockInfo {
    pub node_id: NodeId,
    pub module_id: SysModuleId,
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
        module_init: BTreeMap<SysModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>,
    ) -> Result<(), RuntimeError>;
}

pub trait KernelSubstateApi {
    fn kernel_lock_substate(
        &mut self,
        node_id: &NodeId,
        module_id: SysModuleId,
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

/// Interface of the Kernel, for Kernel modules.
pub trait KernelApi<M: KernelUpstream>:
    KernelNodeApi + KernelSubstateApi + KernelInvokeDownstreamApi
{
}

/// Internal API for kernel modules.
/// No kernel state changes are expected as of a result of invoking such APIs, except updating returned references.
pub trait KernelInternalApi<M: KernelUpstream> {
    fn kernel_get_system(&self) -> &M;

    fn kernel_get_module_state(&mut self) -> &mut KernelModuleMixer;

    // TODO: Cleanup
    fn kernel_get_node_info(&self, node_id: &NodeId) -> Option<(RefType, bool)>;

    fn kernel_get_current_depth(&self) -> usize;

    // TODO: Remove
    fn kernel_get_current_actor(&mut self) -> Option<Actor>;

    fn kernel_set_mode(&mut self, mode: ExecutionMode);

    // TODO: Remove
    fn kernel_load_package_package_dependencies(&mut self);
    fn kernel_load_common(&mut self);

    /* Super unstable interface, specifically for `ExecutionTrace` kernel module */
    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot>;
    fn kernel_read_proof(&mut self, proof_id: &NodeId) -> Option<ProofSnapshot>;
}

pub trait KernelModuleApi<M: KernelUpstream>:
    KernelNodeApi
    + KernelSubstateApi
    + KernelInternalApi<M>
    + KernelInvokeDownstreamApi
    //+ ClientObjectApi<RuntimeError>
{
}

#[derive(Debug)]
pub struct KernelInvocation {
    pub sys_invocation: SystemInvocation,

    // TODO: Remove
    pub payload_size: usize,

    // TODO: Make these two RENodes / Substates
    pub resolved_actor: Actor,
    pub args: IndexedScryptoValue,
}

impl KernelInvocation {
    pub fn get_update(&self) -> CallFrameUpdate {
        let nodes_to_move = self.args.owned_node_ids().clone();
        let mut node_refs_to_copy = self.args.references().clone();
        match self.resolved_actor {
            Actor::Method { node_id, .. } => {
                node_refs_to_copy.insert(node_id);
            }
            Actor::Function { .. } | Actor::VirtualLazyLoad { .. } => {}
        }

        CallFrameUpdate {
            nodes_to_move,
            node_refs_to_copy,
        }
    }
}

pub trait KernelInvokeDownstreamApi {
    fn kernel_invoke_downstream(
        &mut self,
        invocation: Box<KernelInvocation>,
    ) -> Result<IndexedScryptoValue, RuntimeError>;
}

pub trait KernelUpstream: Sized {
    fn on_init<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where Y: KernelModuleApi<Self>;

    fn on_teardown<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where Y: KernelModuleApi<Self>;

    fn before_drop_node<Y>(
        node_id: &NodeId,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where Y: KernelModuleApi<Self>;

    fn after_drop_node<Y>(api: &mut Y) -> Result<(), RuntimeError>
        where Y: KernelModuleApi<Self>;

    fn before_create_node<Y>(
        node_id: &NodeId,
        node_init: &NodeInit,
        node_module_init: &BTreeMap<SysModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where Y: KernelModuleApi<Self>;

    fn after_create_node<Y>(
        node_id: &NodeId,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where Y: KernelModuleApi<Self>;

    fn before_lock_substate<Y>(
        node_id: &NodeId,
        module_id: &SysModuleId,
        substate_key: &SubstateKey,
        flags: &LockFlags,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where Y: KernelModuleApi<Self>;

    fn after_lock_substate<Y>(
        handle: LockHandle,
        size: usize,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where Y: KernelModuleApi<Self>;

    fn on_drop_lock<Y>(
        lock_handle: LockHandle,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where Y: KernelModuleApi<Self>;

    fn on_read_substate<Y>(
        lock_handle: LockHandle,
        size: usize,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where Y: KernelModuleApi<Self>;

    fn on_write_substate<Y>(
        lock_handle: LockHandle,
        size: usize,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where Y: KernelModuleApi<Self>;

    fn before_invoke<Y>(
        identifier: &KernelInvocation,
        input_size: usize,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where Y: KernelModuleApi<Self>;

    fn after_invoke<Y>(
        output_size: usize,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
        where Y: KernelModuleApi<Self>;

    fn before_push_frame<Y>(
        callee: &Actor,
        update: &mut CallFrameUpdate,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where Y: KernelModuleApi<Self>;

    fn on_execution_start<Y>(
        caller: &Option<Actor>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where Y: KernelModuleApi<Self>;

    fn invoke_upstream<Y>(
        invocation: SystemInvocation,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelModuleApi<Self>;

    /*
    Y: KernelNodeApi
            + KernelSubstateApi
            + KernelInternalApi<Self>;
            + ClientApi<RuntimeError>;
     */

    fn on_execution_finish<Y>(
        caller: &Option<Actor>,
        update: &CallFrameUpdate,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where Y: KernelModuleApi<Self>;

    fn auto_drop<Y>(nodes: Vec<NodeId>, api: &mut Y) -> Result<(), RuntimeError>
        where Y: KernelModuleApi<Self>;

    fn after_pop_frame<Y>(api: &mut Y) -> Result<(), RuntimeError>
        where Y: KernelModuleApi<Self>;

    fn on_substate_lock_fault<Y>(
        node_id: NodeId,
        module_id: SysModuleId,
        offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
        where Y: KernelModuleApi<Self>;
}
