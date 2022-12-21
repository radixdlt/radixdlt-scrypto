use crate::api::types::*;
use crate::scrypto;
use radix_engine_interface::api::api::InvokableModel;
use radix_engine_interface::data::IndexedScryptoValue;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum RadixEngineInput {
    Invoke(SerializedInvocation),

    CreateNode(ScryptoRENode),
    GetVisibleNodeIds(),
    DropNode(RENodeId),

    LockSubstate(RENodeId, SubstateOffset, bool),
    DropLock(LockHandle),
    Read(LockHandle),
    Write(LockHandle, Vec<u8>),

    GetActor(),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum SerializedInvocation {
    Native(NativeFnInvocation),
    Scrypto(ScryptoInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum NativeFnInvocation {
    Method(NativeMethodInvocation),
    Function(NativeFunctionInvocation),
}

impl Into<SerializedInvocation> for NativeFnInvocation {
    fn into(self) -> SerializedInvocation {
        SerializedInvocation::Native(self)
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum NativeMethodInvocation {
    AccessRulesChain(AccessRulesChainMethodInvocation),
    Metadata(MetadataMethodInvocation),
    Package(PackageMethodInvocation),
    Component(ComponentMethodInvocation),
    EpochManager(EpochManagerMethodInvocation),
    Clock(ClockMethodInvocation),
    AuthZoneStack(AuthZoneStackMethodInvocation),
    ResourceManager(ResourceManagerMethodInvocation),
    Bucket(BucketMethodInvocation),
    Vault(VaultMethodInvocation),
    Proof(ProofMethodInvocation),
    Worktop(WorktopMethodInvocation),
    TransactionHash(TransactionHashMethodInvocation),
    Logger(LoggerInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum NativeFunctionInvocation {
    Component(ComponentFunctionInvocation),
    EpochManager(EpochManagerFunctionInvocation),
    Clock(ClockFunctionInvocation),
    ResourceManager(ResourceManagerFunctionInvocation),
    Package(PackageFunctionInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum LoggerInvocation {
    Log(LoggerLogInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum TransactionHashMethodInvocation {
    Get(TransactionRuntimeGetHashInvocation),
    GenerateUuid(TransactionRuntimeGenerateUuidInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum AccessRulesChainMethodInvocation {
    AddAccessCheck(AccessRulesAddAccessCheckInvocation),
    SetMethodAccessRule(AccessRulesSetMethodAccessRuleInvocation),
    SetMethodMutability(AccessRulesSetMethodMutabilityInvocation),
    SetGroupAccessRule(AccessRulesSetGroupAccessRuleInvocation),
    SetGroupMutability(AccessRulesSetGroupMutabilityInvocation),
    GetLength(AccessRulesGetLengthInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum MetadataMethodInvocation {
    Set(MetadataSetInvocation),
    Get(MetadataGetInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ComponentFunctionInvocation {
    Globalize(ComponentGlobalizeInvocation),
    GlobalizeWithOwner(ComponentGlobalizeWithOwnerInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum EpochManagerFunctionInvocation {
    Create(EpochManagerCreateInvocation),
}
#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ClockFunctionInvocation {
    Create(ClockCreateInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ClockMethodInvocation {
    GetCurrentTime(ClockGetCurrentTimeInvocation),
    CompareCurrentTime(ClockCompareCurrentTimeInvocation),
    SetCurrentTime(ClockSetCurrentTimeInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ComponentMethodInvocation {
    SetRoyaltyConfig(ComponentSetRoyaltyConfigInvocation),
    ClaimRoyalty(ComponentClaimRoyaltyInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum PackageMethodInvocation {
    SetRoyaltyConfig(PackageSetRoyaltyConfigInvocation),
    ClaimRoyalty(PackageClaimRoyaltyInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum EpochManagerMethodInvocation {
    GetCurrentEpoch(EpochManagerGetCurrentEpochInvocation),
    NextRound(EpochManagerNextRoundInvocation),
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum AuthZoneStackMethodInvocation {
    Pop(AuthZonePopInvocation),
    Push(AuthZonePushInvocation),
    CreateProof(AuthZoneCreateProofInvocation),
    CreateProofByAmount(AuthZoneCreateProofByAmountInvocation),
    CreateProofByIds(AuthZoneCreateProofByIdsInvocation),
    Clear(AuthZoneClearInvocation),
    Drain(AuthZoneDrainInvocation),
    AssertAuthRule(AuthZoneAssertAccessRuleInvocation),
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
    GetResourceType(ResourceManagerGetResourceTypeInvocation),
    Burn(ResourceManagerBurnInvocation),
    Mint(ResourceManagerMintInvocation),
    CreateBucket(ResourceManagerCreateBucketInvocation),
    CreateVault(ResourceManagerCreateVaultInvocation),
    UpdateVaultAuth(ResourceManagerUpdateVaultAuthInvocation),
    LockVaultAuth(ResourceManagerSetVaultAuthMutabilityInvocation),
    GetTotalSupply(ResourceManagerGetTotalSupplyInvocation),
    UpdateNonFungibleData(ResourceManagerUpdateNonFungibleDataInvocation),
    GetNonFungible(ResourceManagerGetNonFungibleInvocation),
    NonFungibleExists(ResourceManagerNonFungibleExistsInvocation),
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
    Put(VaultPutInvocation),
    TakeNonFungibles(VaultTakeNonFungiblesInvocation),
    GetAmount(VaultGetAmountInvocation),
    GetResourceAddress(VaultGetResourceAddressInvocation),
    GetNonFungibleIds(VaultGetNonFungibleIdsInvocation),
    CreateProof(VaultCreateProofInvocation),
    CreateProofByAmount(VaultCreateProofByAmountInvocation),
    CreateProofByIds(VaultCreateProofByIdsInvocation),
    Recall(VaultRecallInvocation),
    RecallNonFungibles(VaultRecallNonFungiblesInvocation),
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

impl NativeFnInvocation {
    pub fn invoke<Y, E>(self, api: &mut Y) -> Result<IndexedScryptoValue, E>
    where
        Y: InvokableModel<E>,
    {
        match self {
            NativeFnInvocation::Function(native_function) => match native_function {
                NativeFunctionInvocation::EpochManager(invocation) => match invocation {
                    EpochManagerFunctionInvocation::Create(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeFunctionInvocation::Clock(invocation) => match invocation {
                    ClockFunctionInvocation::Create(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeFunctionInvocation::ResourceManager(invocation) => match invocation {
                    ResourceManagerFunctionInvocation::Create(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerFunctionInvocation::BurnBucket(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeFunctionInvocation::Package(invocation) => match invocation {
                    PackageFunctionInvocation::Publish(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeFunctionInvocation::Component(invocation) => match invocation {
                    ComponentFunctionInvocation::Globalize(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ComponentFunctionInvocation::GlobalizeWithOwner(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
            },
            NativeFnInvocation::Method(native_method) => match native_method {
                NativeMethodInvocation::Component(component_method) => match component_method {
                    ComponentMethodInvocation::SetRoyaltyConfig(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ComponentMethodInvocation::ClaimRoyalty(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::Package(package_method) => match package_method {
                    PackageMethodInvocation::SetRoyaltyConfig(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    PackageMethodInvocation::ClaimRoyalty(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::Bucket(bucket_method) => match bucket_method {
                    BucketMethodInvocation::Take(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::CreateProof(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::TakeNonFungibles(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::GetNonFungibleIds(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::GetAmount(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::Put(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::GetResourceAddress(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::AuthZoneStack(auth_zone_method) => match auth_zone_method {
                    AuthZoneStackMethodInvocation::Pop(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneStackMethodInvocation::Push(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneStackMethodInvocation::CreateProof(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneStackMethodInvocation::CreateProofByAmount(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneStackMethodInvocation::CreateProofByIds(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneStackMethodInvocation::Clear(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneStackMethodInvocation::Drain(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneStackMethodInvocation::AssertAuthRule(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::Proof(proof_method) => match proof_method {
                    ProofMethodInvocation::GetAmount(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ProofMethodInvocation::GetNonFungibleIds(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ProofMethodInvocation::GetResourceAddress(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ProofMethodInvocation::Clone(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::Vault(vault_method) => match vault_method {
                    VaultMethodInvocation::Take(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::Put(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::LockFee(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::TakeNonFungibles(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::GetAmount(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::GetResourceAddress(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::GetNonFungibleIds(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::CreateProof(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::CreateProofByAmount(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::CreateProofByIds(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::Recall(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::RecallNonFungibles(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::AccessRulesChain(access_rules_method) => {
                    match access_rules_method {
                        AccessRulesChainMethodInvocation::AddAccessCheck(invocation) => api
                            .invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a)),
                        AccessRulesChainMethodInvocation::SetMethodAccessRule(invocation) => api
                            .invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a)),
                        AccessRulesChainMethodInvocation::SetMethodMutability(invocation) => api
                            .invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a)),
                        AccessRulesChainMethodInvocation::SetGroupAccessRule(invocation) => api
                            .invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a)),
                        AccessRulesChainMethodInvocation::SetGroupMutability(invocation) => api
                            .invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a)),
                        AccessRulesChainMethodInvocation::GetLength(invocation) => api
                            .invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a)),
                    }
                }
                NativeMethodInvocation::Metadata(metadata_method) => match metadata_method {
                    MetadataMethodInvocation::Set(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    MetadataMethodInvocation::Get(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::ResourceManager(resman_method) => match resman_method {
                    ResourceManagerMethodInvocation::Burn(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::UpdateVaultAuth(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::LockVaultAuth(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::CreateVault(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::CreateBucket(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::Mint(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::GetResourceType(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::GetTotalSupply(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::UpdateNonFungibleData(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::NonFungibleExists(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::GetNonFungible(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::EpochManager(epoch_manager_method) => {
                    match epoch_manager_method {
                        EpochManagerMethodInvocation::GetCurrentEpoch(invocation) => api
                            .invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a)),
                        EpochManagerMethodInvocation::NextRound(invocation) => api
                            .invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a)),
                    }
                }
                NativeMethodInvocation::Clock(clock_method) => match clock_method {
                    ClockMethodInvocation::SetCurrentTime(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ClockMethodInvocation::GetCurrentTime(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ClockMethodInvocation::CompareCurrentTime(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::Worktop(worktop_method) => match worktop_method {
                    WorktopMethodInvocation::TakeNonFungibles(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::Put(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::Drain(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::AssertContainsNonFungibles(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::AssertContains(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::AssertContainsAmount(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::TakeAll(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::TakeAmount(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::TransactionHash(method) => match method {
                    TransactionHashMethodInvocation::Get(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    TransactionHashMethodInvocation::GenerateUuid(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::Logger(method) => match method {
                    LoggerInvocation::Log(invocation) => api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
            },
        }
    }
}
