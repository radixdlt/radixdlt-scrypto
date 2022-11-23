use crate::api::types::ScryptoActor;
use crate::crypto::Hash;
use crate::model::*;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

use super::types::*;

pub trait Invocation: Debug {
    type Output: Debug;
}

pub trait SysNativeInvokable<I: Invocation, E> {
    fn sys_invoke(&mut self, invocation: I) -> Result<I::Output, E>;
}

pub trait SysNativeInvokable2<I: Invocation, E> {
    fn sys_invoke2(&mut self, invocation: I) -> Result<I::Output, E>;
}

pub trait EngineApi<E: Debug> {
    fn sys_invoke_scrypto_function(
        &mut self,
        fn_ident: ScryptoFunctionIdent,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;
    fn sys_invoke_scrypto_method(
        &mut self,
        method_ident: ScryptoMethodIdent,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;
    fn sys_create_node(&mut self, node: ScryptoRENode) -> Result<RENodeId, E>;
    fn sys_drop_node(&mut self, node_id: RENodeId) -> Result<(), E>;
    fn sys_get_visible_nodes(&mut self) -> Result<Vec<RENodeId>, E>;
    fn sys_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        mutable: bool,
    ) -> Result<LockHandle, E>;
    fn sys_read(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, E>;
    fn sys_write(&mut self, lock_handle: LockHandle, buffer: Vec<u8>) -> Result<(), E>;
    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), E>;
    fn sys_get_actor(&mut self) -> Result<ScryptoActor, E>;
    fn sys_generate_uuid(&mut self) -> Result<u128, E>;
    fn sys_get_transaction_hash(&mut self) -> Result<Hash, E>;
    fn sys_emit_log(&mut self, level: Level, message: String) -> Result<(), E>;
}

pub trait SysInvokableNative2<E>:
    SysNativeInvokable2<EpochManagerCreateInvocation, E>
    + SysNativeInvokable2<EpochManagerSetEpochInvocation, E>
    + SysNativeInvokable2<EpochManagerGetCurrentEpochInvocation, E>
    + SysNativeInvokable2<MetadataSetInvocation, E>
    + SysNativeInvokable2<AccessRulesAddAccessCheckInvocation, E>
    + SysNativeInvokable2<AuthZonePopInvocation, E>
    + SysNativeInvokable2<AuthZonePushInvocation, E>
    + SysNativeInvokable2<AuthZoneCreateProofInvocation, E>
    + SysNativeInvokable2<AuthZoneCreateProofByAmountInvocation, E>
    + SysNativeInvokable2<AuthZoneCreateProofByIdsInvocation, E>
    + SysNativeInvokable2<AuthZoneClearInvocation, E>
    + SysNativeInvokable2<AuthZoneDrainInvocation, E>
    + SysNativeInvokable2<PackagePublishNoOwnerInvocation, E>
    + SysNativeInvokable2<PackagePublishWithOwnerInvocation, E>
    + SysNativeInvokable2<BucketTakeInvocation, E>
    + SysNativeInvokable2<BucketPutInvocation, E>
    + SysNativeInvokable2<BucketTakeNonFungiblesInvocation, E>
    + SysNativeInvokable2<BucketGetNonFungibleIdsInvocation, E>
    + SysNativeInvokable2<BucketGetAmountInvocation, E>
    + SysNativeInvokable2<BucketGetResourceAddressInvocation, E>
    + SysNativeInvokable2<BucketCreateProofInvocation, E>
    + SysNativeInvokable2<BucketCreateProofInvocation, E>
    + SysNativeInvokable2<ProofCloneInvocation, E>
    + SysNativeInvokable2<ProofGetAmountInvocation, E>
    + SysNativeInvokable2<ProofGetNonFungibleIdsInvocation, E>
    + SysNativeInvokable2<ProofGetResourceAddressInvocation, E>
    + SysNativeInvokable2<ResourceManagerBucketBurnInvocation, E>
    + SysNativeInvokable2<ResourceManagerCreateInvocation, E>
    + SysNativeInvokable2<ResourceManagerBurnInvocation, E>
    + SysNativeInvokable2<ResourceManagerUpdateAuthInvocation, E>
    + SysNativeInvokable2<ResourceManagerLockAuthInvocation, E>
    + SysNativeInvokable2<ResourceManagerCreateVaultInvocation, E>
    + SysNativeInvokable2<ResourceManagerCreateBucketInvocation, E>
    + SysNativeInvokable2<ResourceManagerMintInvocation, E>
    + SysNativeInvokable2<ResourceManagerGetMetadataInvocation, E>
    + SysNativeInvokable2<ResourceManagerGetResourceTypeInvocation, E>
    + SysNativeInvokable2<ResourceManagerGetTotalSupplyInvocation, E>
    + SysNativeInvokable2<ResourceManagerUpdateMetadataInvocation, E>
    + SysNativeInvokable2<ResourceManagerUpdateNonFungibleDataInvocation, E>
    + SysNativeInvokable2<ResourceManagerNonFungibleExistsInvocation, E>
    + SysNativeInvokable2<ResourceManagerGetNonFungibleInvocation, E>
{
}

pub trait SysInvokableNative<E>:
    SysNativeInvokable<VaultTakeInvocation, E>
    + SysNativeInvokable<VaultPutInvocation, E>
    + SysNativeInvokable<VaultLockFeeInvocation, E>
    + SysNativeInvokable<VaultTakeNonFungiblesInvocation, E>
    + SysNativeInvokable<VaultGetAmountInvocation, E>
    + SysNativeInvokable<VaultGetResourceAddressInvocation, E>
    + SysNativeInvokable<VaultGetNonFungibleIdsInvocation, E>
    + SysNativeInvokable<VaultCreateProofInvocation, E>
    + SysNativeInvokable<VaultCreateProofByAmountInvocation, E>
    + SysNativeInvokable<VaultCreateProofByIdsInvocation, E>
    + SysNativeInvokable<WorktopPutInvocation, E>
    + SysNativeInvokable<WorktopTakeAmountInvocation, E>
    + SysNativeInvokable<WorktopTakeAllInvocation, E>
    + SysNativeInvokable<WorktopTakeNonFungiblesInvocation, E>
    + SysNativeInvokable<WorktopAssertContainsInvocation, E>
    + SysNativeInvokable<WorktopAssertContainsAmountInvocation, E>
    + SysNativeInvokable<WorktopAssertContainsNonFungiblesInvocation, E>
    + SysNativeInvokable<WorktopDrainInvocation, E>
{
}
