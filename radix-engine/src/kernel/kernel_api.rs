use crate::errors::*;
use crate::kernel::*;
use crate::system::kernel_modules::execution_trace::ProofSnapshot;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_substates::{SubstateRef, SubstateRefMut};
use crate::types::*;
use crate::wasm::WasmEngine;
use bitflags::bitflags;
use radix_engine_interface::api::component::*;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::package::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::ClientDerefApi;
use radix_engine_interface::blueprints::logger::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::transaction_runtime::*;

bitflags! {
    #[derive(Encode, Decode, Categorize)]
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

pub trait KernelActorApi<E> {
    fn fn_identifier(&mut self) -> Result<FnIdentifier, E>;
}

pub trait KernelNodeApi {
    fn get_visible_node_data(
        &mut self,
        node_id: RENodeId,
    ) -> Result<RENodeVisibilityOrigin, RuntimeError>;

    /// Removes an RENode and all of it's children from the Heap
    fn drop_node(&mut self, node_id: RENodeId) -> Result<HeapRENode, RuntimeError>;

    /// Allocates a new node id useable for create_node
    fn allocate_node_id(&mut self, node_type: RENodeType) -> Result<RENodeId, RuntimeError>;

    /// Creates a new RENode
    /// TODO: Remove, replace with lock_substate + get_ref_mut use
    fn create_node(
        &mut self,
        node_id: RENodeId,
        init: RENodeInit,
        node_module_init: BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError>;
}

/// Internal API for kernel modules.
/// No kernel state changes are expected as of a result of invoking such APIs, except updating returned references.
pub trait KernelInternalApi {
    fn get_module_state(&mut self) -> &mut KernelModuleMixer;
    fn get_current_depth(&self) -> usize;
    fn get_current_actor(&self) -> ResolvedActor;

    /* Temporary interface, specifically for `ExecutionTrace` kernel module */
    fn read_bucket(&mut self, bucket_id: BucketId) -> Option<Resource>;
    fn read_proof(&mut self, proof_id: ProofId) -> Option<ProofSnapshot>;
}

#[repr(u8)]
pub enum KernelModuleId {
    KernelDebug,
    Costing,
    NodeMove,
    Auth,
    Logger,
    TransactionRuntime,
    ExecutionTrace,
}

pub trait KernelSubstateApi {
    /// Locks a visible substate
    fn lock_substate(
        &mut self,
        node_id: RENodeId,
        module_id: NodeModuleId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError>;

    fn get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError>;

    /// Drops a lock
    fn drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError>;

    /// Get a non-mutable reference to a locked substate
    fn get_ref(&mut self, lock_handle: LockHandle) -> Result<SubstateRef, RuntimeError>;

    /// Get a mutable reference to a locked substate
    fn get_ref_mut(&mut self, lock_handle: LockHandle) -> Result<SubstateRefMut, RuntimeError>;
}

pub trait KernelWasmApi<W: WasmEngine> {
    fn scrypto_interpreter(&mut self) -> &ScryptoInterpreter<W>;

    fn emit_wasm_instantiation_event(&mut self, code: &[u8]) -> Result<(), RuntimeError>;
}

pub trait Invokable<I: Invocation, E> {
    fn invoke(&mut self, invocation: I) -> Result<I::Output, E>;
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

    fn resolve<Y: KernelSubstateApi + ClientDerefApi<RuntimeError>>(
        self,
        api: &mut Y,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>;
}

pub trait KernelInvokeApi<E>:
    Invokable<MetadataSetInvocation, E>
    + Invokable<MetadataGetInvocation, E>
    + Invokable<AccessRulesAddAccessCheckInvocation, E>
    + Invokable<AccessRulesSetMethodAccessRuleInvocation, E>
    + Invokable<AccessRulesSetMethodMutabilityInvocation, E>
    + Invokable<AccessRulesSetGroupAccessRuleInvocation, E>
    + Invokable<AccessRulesSetGroupMutabilityInvocation, E>
    + Invokable<AccessRulesGetLengthInvocation, E>
    + Invokable<AuthZonePopInvocation, E>
    + Invokable<AuthZonePushInvocation, E>
    + Invokable<AuthZoneCreateProofInvocation, E>
    + Invokable<AuthZoneCreateProofByAmountInvocation, E>
    + Invokable<AuthZoneCreateProofByIdsInvocation, E>
    + Invokable<AuthZoneClearInvocation, E>
    + Invokable<AuthZoneDrainInvocation, E>
    + Invokable<AuthZoneAssertAccessRuleInvocation, E>
    + Invokable<AccessRulesAddAccessCheckInvocation, E>
    + Invokable<ComponentGlobalizeInvocation, E>
    + Invokable<ComponentGlobalizeWithOwnerInvocation, E>
    + Invokable<ComponentSetRoyaltyConfigInvocation, E>
    + Invokable<ComponentClaimRoyaltyInvocation, E>
    + Invokable<PackageSetRoyaltyConfigInvocation, E>
    + Invokable<PackageClaimRoyaltyInvocation, E>
    + Invokable<PackagePublishInvocation, E>
    + Invokable<PackagePublishNativeInvocation, E>
    + Invokable<WorktopAssertContainsInvocation, E>
    + Invokable<WorktopAssertContainsAmountInvocation, E>
    + Invokable<WorktopAssertContainsNonFungiblesInvocation, E>
    + Invokable<WorktopDrainInvocation, E>
    + Invokable<TransactionRuntimeGetHashInvocation, E>
    + Invokable<TransactionRuntimeGenerateUuidInvocation, E>
    + Invokable<LoggerLogInvocation, E>
{
}

/// Interface of the Kernel, for Kernel modules.
pub trait KernelApi<W: WasmEngine, E>:
    KernelActorApi<E> + KernelNodeApi + KernelSubstateApi + KernelWasmApi<W> + KernelInvokeApi<E>
{
}

pub trait KernelModuleApi<E>: KernelNodeApi + KernelSubstateApi + KernelInternalApi {}
