use crate::resource::*;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::{Decode, Encode};
use scrypto::core::*;
use crate::component::ComponentAddAccessCheckInvocation;
use crate::crypto::Hash;

use super::types::*;

pub trait SysInvocation: Encode {
    type Output: Debug + Decode;
    fn native_fn() -> NativeFn;
}

pub trait SysInvokable<I: SysInvocation, E> {
    fn sys_invoke(&mut self, invocation: I) -> Result<I::Output, E>;
}

pub trait ScryptoSyscalls<E: Debug> {
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
    SysInvokable<AuthZonePopInvocation, E>
    + SysInvokable<AuthZonePushInvocation, E>
    + SysInvokable<AuthZoneCreateProofInvocation, E>
    + SysInvokable<AuthZoneCreateProofByAmountInvocation, E>
    + SysInvokable<AuthZoneCreateProofByIdsInvocation, E>
    + SysInvokable<AuthZoneClearInvocation, E>
    + SysInvokable<AuthZoneDrainInvocation, E>
    + SysInvokable<ComponentAddAccessCheckInvocation, E>
    + SysInvokable<BucketTakeInvocation, E>
    + SysInvokable<BucketPutInvocation, E>
    + SysInvokable<BucketTakeNonFungiblesInvocation, E>
    + SysInvokable<BucketGetNonFungibleIdsInvocation, E>
    + SysInvokable<BucketGetAmountInvocation, E>
    + SysInvokable<BucketGetResourceAddressInvocation, E>
    + SysInvokable<BucketCreateProofInvocation, E>
    + SysInvokable<ResourceManagerCreateBucketInvocation, E>
    + SysInvokable<ResourceManagerBurnInvocation, E>
    + SysInvokable<BucketCreateProofInvocation, E>
    + SysInvokable<ProofCloneInvocation, E>
    + SysInvokable<ProofGetAmountInvocation, E>
    + SysInvokable<ProofGetNonFungibleIdsInvocation, E>
    + SysInvokable<ProofGetResourceAddressInvocation, E>
    + SysInvokable<VaultTakeInvocation, E>
    + SysInvokable<VaultPutInvocation, E>
    + SysInvokable<VaultLockFeeInvocation, E>
    + SysInvokable<VaultTakeNonFungiblesInvocation, E>
    + SysInvokable<VaultGetAmountInvocation, E>
    + SysInvokable<VaultGetResourceAddressInvocation, E>
    + SysInvokable<VaultGetNonFungibleIdsInvocation, E>
    + SysInvokable<VaultCreateProofInvocation, E>
    + SysInvokable<VaultCreateProofByAmountInvocation, E>
    + SysInvokable<VaultCreateProofByIdsInvocation, E>
    + SysInvokable<ResourceManagerBurnInvocation, E>
    + SysInvokable<ResourceManagerUpdateAuthInvocation, E>
    + SysInvokable<ResourceManagerLockAuthInvocation, E>
    + SysInvokable<ResourceManagerCreateVaultInvocation, E>
    + SysInvokable<ResourceManagerCreateBucketInvocation, E>
    + SysInvokable<ResourceManagerMintInvocation, E>
    + SysInvokable<ResourceManagerGetMetadataInvocation, E>
    + SysInvokable<ResourceManagerGetResourceTypeInvocation, E>
    + SysInvokable<ResourceManagerGetTotalSupplyInvocation, E>
    + SysInvokable<ResourceManagerUpdateMetadataInvocation, E>
    + SysInvokable<ResourceManagerUpdateNonFungibleDataInvocation, E>
    + SysInvokable<ResourceManagerNonFungibleExistsInvocation, E>
    + SysInvokable<ResourceManagerGetNonFungibleInvocation, E>
    + SysInvokable<ResourceManagerSetResourceAddressInvocation, E>
    + SysInvokable<EpochManagerSetEpochInvocation, E>
    + SysInvokable<EpochManagerGetCurrentEpochInvocation, E>
    + SysInvokable<WorktopPutInvocation, E>
    + SysInvokable<WorktopTakeAmountInvocation, E>
    + SysInvokable<WorktopTakeAllInvocation, E>
    + SysInvokable<WorktopTakeNonFungiblesInvocation, E>
    + SysInvokable<WorktopAssertContainsInvocation, E>
    + SysInvokable<WorktopAssertContainsAmountInvocation, E>
    + SysInvokable<WorktopAssertContainsNonFungiblesInvocation, E>
    + SysInvokable<WorktopDrainInvocation, E>
{
}
