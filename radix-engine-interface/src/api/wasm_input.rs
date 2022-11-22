use crate::api::types::*;
use crate::scrypto;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum RadixEngineInput {
    InvokeScryptoFunction(ScryptoFunctionIdent, Vec<u8>),
    InvokeScryptoMethod(ScryptoMethodIdent, Vec<u8>),
    InvokeNativeFn(NativeFnInvocation),

    CreateNode(ScryptoRENode),
    GetVisibleNodeIds(),
    DropNode(RENodeId),

    LockSubstate(RENodeId, SubstateOffset, bool),
    DropLock(LockHandle),
    Read(LockHandle),
    Write(LockHandle, Vec<u8>),

    GetActor(),
    EmitLog(Level, String),
    GenerateUuid(),
    GetTransactionHash(),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum NativeFnInvocation {
    Method(NativeMethodInvocation),
    Function(NativeFunctionInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum NativeMethodInvocation {
    AccessRules(AccessRulesMethodInvocation),
    Component(ComponentMethodInvocation),
    Package(PackageMethodInvocation),
    EpochManager(EpochManagerMethodInvocation),
    AuthZone(AuthZoneMethodInvocation),
    ResourceManager(ResourceManagerMethodInvocation),
    Bucket(BucketMethodInvocation),
    Vault(VaultMethodInvocation),
    Proof(ProofMethodInvocation),
    Worktop(WorktopMethodInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum NativeFunctionInvocation {
    EpochManager(EpochManagerFunctionInvocation),
    ResourceManager(ResourceManagerFunctionInvocation),
    Package(PackageFunctionInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum AccessRulesMethodInvocation {
    AddAccessCheck(AccessRulesAddAccessCheckInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ComponentMethodInvocation {
    SetRoyaltyConfig(ComponentSetRoyaltyConfigInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum PackageMethodInvocation {
    SetRoyaltyConfig(PackageSetRoyaltyConfigInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum EpochManagerFunctionInvocation {
    Create(EpochManagerCreateInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum EpochManagerMethodInvocation {
    GetCurrentEpoch(EpochManagerGetCurrentEpochInvocation),
    SetEpoch(EpochManagerSetEpochInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum AuthZoneMethodInvocation {
    Pop(AuthZonePopInvocation),
    Push(AuthZonePushInvocation),
    CreateProof(AuthZoneCreateProofInvocation),
    CreateProofByAmount(AuthZoneCreateProofByAmountInvocation),
    CreateProofByIds(AuthZoneCreateProofByIdsInvocation),
    Clear(AuthZoneClearInvocation),
    Drain(AuthZoneDrainInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ResourceManagerFunctionInvocation {
    Create(ResourceManagerCreateInvocation),
    BurnBucket(ResourceManagerBucketBurnInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ResourceManagerMethodInvocation {
    Burn(ResourceManagerBurnInvocation),
    UpdateAuth(ResourceManagerUpdateAuthInvocation),
    LockAuth(ResourceManagerLockAuthInvocation),
    Mint(ResourceManagerMintInvocation),
    UpdateNonFungibleData(ResourceManagerUpdateNonFungibleDataInvocation),
    GetNonFungible(ResourceManagerGetNonFungibleInvocation),
    GetMetadata(ResourceManagerGetMetadataInvocation),
    GetResourceType(ResourceManagerGetResourceTypeInvocation),
    GetTotalSupply(ResourceManagerGetTotalSupplyInvocation),
    UpdateMetadata(ResourceManagerUpdateMetadataInvocation),
    NonFungibleExists(ResourceManagerNonFungibleExistsInvocation),
    CreateBucket(ResourceManagerCreateBucketInvocation),
    CreateVault(ResourceManagerCreateVaultInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum BucketMethodInvocation {
    Take(BucketTakeInvocation),
    TakeNonFungibles(BucketTakeNonFungiblesInvocation),
    Put(BucketPutInvocation),
    GetNonFungibleIds(BucketGetNonFungibleIdsInvocation),
    GetAmount(BucketGetAmountInvocation),
    GetResourceAddress(BucketGetResourceAddressInvocation),
    CreateProof(BucketCreateProofInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum VaultMethodInvocation {
    Take(VaultTakeInvocation),
    LockFee(VaultLockFeeInvocation),
    LockRoyalty(VaultLockRoyaltyInvocation),
    Put(VaultPutInvocation),
    TakeNonFungibles(VaultTakeNonFungiblesInvocation),
    GetAmount(VaultGetAmountInvocation),
    GetResourceAddress(VaultGetResourceAddressInvocation),
    GetNonFungibleIds(VaultGetNonFungibleIdsInvocation),
    CreateProof(VaultCreateProofInvocation),
    CreateProofByAmount(VaultCreateProofByAmountInvocation),
    CreateProofByIds(VaultCreateProofByIdsInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ProofMethodInvocation {
    Clone(ProofCloneInvocation),
    GetAmount(ProofGetAmountInvocation),
    GetNonFungibleIds(ProofGetNonFungibleIdsInvocation),
    GetResourceAddress(ProofGetResourceAddressInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum WorktopMethodInvocation {
    TakeAll(WorktopTakeAllInvocation),
    TakeAmount(WorktopTakeAmountInvocation),
    TakeNonFungibles(WorktopTakeNonFungiblesInvocation),
    Put(WorktopPutInvocation),
    AssertContains(WorktopAssertContainsInvocation),
    AssertContainsAmount(WorktopAssertContainsAmountInvocation),
    AssertContainsNonFungibles(WorktopAssertContainsNonFungiblesInvocation),
    Drain(WorktopDrainInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum PackageFunctionInvocation {
    Publish(PackagePublishInvocation),
}
