use crate::api::types::*;
use crate::scrypto;
use radix_engine_interface::api::api::InvokableNative;
use radix_engine_interface::data::IndexedScryptoValue;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum RadixEngineInput {
    InvokeScrypto(ScryptoInvocation),
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
    SetEpoch(EpochManagerSetEpochInvocation),
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
    CreateWithOwner(ResourceManagerCreateWithOwnerInvocation),
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
    PublishWithOwner(PackagePublishWithOwnerInvocation),
}

impl NativeFnInvocation {
    pub fn invoke<Y, E>(self, system_api: &mut Y) -> Result<IndexedScryptoValue, E>
    where
        Y: InvokableNative<E>,
    {
        match self {
            NativeFnInvocation::Function(native_function) => match native_function {
                NativeFunctionInvocation::EpochManager(invocation) => match invocation {
                    EpochManagerFunctionInvocation::Create(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeFunctionInvocation::Clock(invocation) => match invocation {
                    ClockFunctionInvocation::Create(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeFunctionInvocation::ResourceManager(invocation) => match invocation {
                    ResourceManagerFunctionInvocation::Create(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerFunctionInvocation::CreateWithOwner(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerFunctionInvocation::BurnBucket(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeFunctionInvocation::Package(invocation) => match invocation {
                    PackageFunctionInvocation::Publish(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    PackageFunctionInvocation::PublishWithOwner(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeFunctionInvocation::Component(invocation) => match invocation {
                    ComponentFunctionInvocation::Globalize(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ComponentFunctionInvocation::GlobalizeWithOwner(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
            },
            NativeFnInvocation::Method(native_method) => match native_method {
                NativeMethodInvocation::Component(component_method) => match component_method {
                    ComponentMethodInvocation::SetRoyaltyConfig(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ComponentMethodInvocation::ClaimRoyalty(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::Package(package_method) => match package_method {
                    PackageMethodInvocation::SetRoyaltyConfig(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    PackageMethodInvocation::ClaimRoyalty(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::Bucket(bucket_method) => match bucket_method {
                    BucketMethodInvocation::Take(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::CreateProof(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::TakeNonFungibles(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::GetNonFungibleIds(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::GetAmount(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::Put(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    BucketMethodInvocation::GetResourceAddress(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::AuthZoneStack(auth_zone_method) => match auth_zone_method {
                    AuthZoneStackMethodInvocation::Pop(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneStackMethodInvocation::Push(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneStackMethodInvocation::CreateProof(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneStackMethodInvocation::CreateProofByAmount(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneStackMethodInvocation::CreateProofByIds(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneStackMethodInvocation::Clear(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneStackMethodInvocation::Drain(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    AuthZoneStackMethodInvocation::AssertAuthRule(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::Proof(proof_method) => match proof_method {
                    ProofMethodInvocation::GetAmount(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ProofMethodInvocation::GetNonFungibleIds(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ProofMethodInvocation::GetResourceAddress(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ProofMethodInvocation::Clone(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::Vault(vault_method) => match vault_method {
                    VaultMethodInvocation::Take(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::Put(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::LockFee(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::TakeNonFungibles(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::GetAmount(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::GetResourceAddress(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::GetNonFungibleIds(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::CreateProof(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::CreateProofByAmount(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    VaultMethodInvocation::CreateProofByIds(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::AccessRulesChain(access_rules_method) => {
                    match access_rules_method {
                        AccessRulesChainMethodInvocation::AddAccessCheck(invocation) => system_api
                            .invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a)),
                        AccessRulesChainMethodInvocation::SetMethodAccessRule(invocation) => {
                            system_api
                                .invoke(invocation)
                                .map(|a| IndexedScryptoValue::from_typed(&a))
                        }
                        AccessRulesChainMethodInvocation::SetMethodMutability(invocation) => {
                            system_api
                                .invoke(invocation)
                                .map(|a| IndexedScryptoValue::from_typed(&a))
                        }
                        AccessRulesChainMethodInvocation::SetGroupAccessRule(invocation) => {
                            system_api
                                .invoke(invocation)
                                .map(|a| IndexedScryptoValue::from_typed(&a))
                        }
                        AccessRulesChainMethodInvocation::SetGroupMutability(invocation) => {
                            system_api
                                .invoke(invocation)
                                .map(|a| IndexedScryptoValue::from_typed(&a))
                        }
                        AccessRulesChainMethodInvocation::GetLength(invocation) => system_api
                            .invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a)),
                    }
                }
                NativeMethodInvocation::Metadata(metadata_method) => match metadata_method {
                    MetadataMethodInvocation::Set(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    MetadataMethodInvocation::Get(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::ResourceManager(resman_method) => match resman_method {
                    ResourceManagerMethodInvocation::Burn(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::UpdateVaultAuth(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::LockVaultAuth(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::CreateVault(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::CreateBucket(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::Mint(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::GetResourceType(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::GetTotalSupply(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::UpdateNonFungibleData(invocation) => {
                        system_api
                            .invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a))
                    }
                    ResourceManagerMethodInvocation::NonFungibleExists(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ResourceManagerMethodInvocation::GetNonFungible(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::EpochManager(epoch_manager_method) => {
                    match epoch_manager_method {
                        EpochManagerMethodInvocation::GetCurrentEpoch(invocation) => system_api
                            .invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a)),
                        EpochManagerMethodInvocation::SetEpoch(invocation) => system_api
                            .invoke(invocation)
                            .map(|a| IndexedScryptoValue::from_typed(&a)),
                    }
                }
                NativeMethodInvocation::Clock(clock_method) => match clock_method {
                    ClockMethodInvocation::SetCurrentTime(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ClockMethodInvocation::GetCurrentTime(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    ClockMethodInvocation::CompareCurrentTime(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
                NativeMethodInvocation::Worktop(worktop_method) => match worktop_method {
                    WorktopMethodInvocation::TakeNonFungibles(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::Put(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::Drain(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::AssertContainsNonFungibles(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::AssertContains(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::AssertContainsAmount(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::TakeAll(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                    WorktopMethodInvocation::TakeAmount(invocation) => system_api
                        .invoke(invocation)
                        .map(|a| IndexedScryptoValue::from_typed(&a)),
                },
            },
        }
    }
}
