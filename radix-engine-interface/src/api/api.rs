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

pub trait SysInvokableNative<E>:
    SysNativeInvokable<EpochManagerCreateInvocation, E>
    + SysNativeInvokable<EpochManagerSetEpochInvocation, E>
    + SysNativeInvokable<EpochManagerGetCurrentEpochInvocation, E>
    + SysNativeInvokable<MetadataSetInvocation, E>
    + SysNativeInvokable<MetadataGetInvocation, E>
    + SysNativeInvokable<AccessRulesAddAccessCheckInvocation, E>
    + SysNativeInvokable<AccessRulesSetAccessRuleInvocation, E>
    + SysNativeInvokable<AccessRulesSetMutabilityInvocation, E>
    + SysNativeInvokable<AuthZonePopInvocation, E>
    + SysNativeInvokable<AuthZonePushInvocation, E>
    + SysNativeInvokable<AuthZoneCreateProofInvocation, E>
    + SysNativeInvokable<AuthZoneCreateProofByAmountInvocation, E>
    + SysNativeInvokable<AuthZoneCreateProofByIdsInvocation, E>
    + SysNativeInvokable<AuthZoneClearInvocation, E>
    + SysNativeInvokable<AuthZoneDrainInvocation, E>
    + SysNativeInvokable<AuthZoneAssertAccessRuleInvocation, E>
    + SysNativeInvokable<AccessRulesAddAccessCheckInvocation, E>
    + SysNativeInvokable<ComponentSetRoyaltyConfigInvocation, E>
    + SysNativeInvokable<PackageSetRoyaltyConfigInvocation, E>
    + SysNativeInvokable<PackagePublishNoOwnerInvocation, E>
    + SysNativeInvokable<PackagePublishWithOwnerInvocation, E>
    + SysNativeInvokable<BucketTakeInvocation, E>
    + SysNativeInvokable<BucketPutInvocation, E>
    + SysNativeInvokable<BucketTakeNonFungiblesInvocation, E>
    + SysNativeInvokable<BucketGetNonFungibleIdsInvocation, E>
    + SysNativeInvokable<BucketGetAmountInvocation, E>
    + SysNativeInvokable<BucketGetResourceAddressInvocation, E>
    + SysNativeInvokable<BucketCreateProofInvocation, E>
    + SysNativeInvokable<BucketCreateProofInvocation, E>
    + SysNativeInvokable<ProofCloneInvocation, E>
    + SysNativeInvokable<ProofGetAmountInvocation, E>
    + SysNativeInvokable<ProofGetNonFungibleIdsInvocation, E>
    + SysNativeInvokable<ProofGetResourceAddressInvocation, E>
    + SysNativeInvokable<ResourceManagerBucketBurnInvocation, E>
    + SysNativeInvokable<ResourceManagerCreateInvocation, E>
    + SysNativeInvokable<ResourceManagerBurnInvocation, E>
    + SysNativeInvokable<ResourceManagerUpdateVaultAuthInvocation, E>
    + SysNativeInvokable<ResourceManagerSetVaultAuthMutabilityInvocation, E>
    + SysNativeInvokable<ResourceManagerCreateVaultInvocation, E>
    + SysNativeInvokable<ResourceManagerCreateBucketInvocation, E>
    + SysNativeInvokable<ResourceManagerMintInvocation, E>
    + SysNativeInvokable<ResourceManagerGetMetadataInvocation, E>
    + SysNativeInvokable<ResourceManagerGetResourceTypeInvocation, E>
    + SysNativeInvokable<ResourceManagerGetTotalSupplyInvocation, E>
    + SysNativeInvokable<ResourceManagerUpdateMetadataInvocation, E>
    + SysNativeInvokable<ResourceManagerUpdateNonFungibleDataInvocation, E>
    + SysNativeInvokable<ResourceManagerNonFungibleExistsInvocation, E>
    + SysNativeInvokable<ResourceManagerGetNonFungibleInvocation, E>
    + SysNativeInvokable<VaultTakeInvocation, E>
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
