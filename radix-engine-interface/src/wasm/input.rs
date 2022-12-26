use crate::api::types::*;
use crate::scrypto;
use sbor::rust::collections::HashSet;
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
    TransactionRuntime(TransactionRuntimeMethodInvocation),
    Logger(LoggerInvocation),
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum NativeFunctionInvocation {
    Component(ComponentFunctionInvocation),
    EpochManager(EpochManagerFunctionInvocation),
    Clock(ClockFunctionInvocation),
    ResourceManager(ResourceManagerFunctionInvocation),
    Package(PackageFunctionInvocation),
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum LoggerInvocation {
    Log(LoggerLogInvocation),
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum TransactionRuntimeMethodInvocation {
    Get(TransactionRuntimeGetHashInvocation),
    GenerateUuid(TransactionRuntimeGenerateUuidInvocation),
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum AccessRulesChainMethodInvocation {
    AddAccessCheck(AccessRulesAddAccessCheckInvocation),
    SetMethodAccessRule(AccessRulesSetMethodAccessRuleInvocation),
    SetMethodMutability(AccessRulesSetMethodMutabilityInvocation),
    SetGroupAccessRule(AccessRulesSetGroupAccessRuleInvocation),
    SetGroupMutability(AccessRulesSetGroupMutabilityInvocation),
    GetLength(AccessRulesGetLengthInvocation),
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum MetadataMethodInvocation {
    Set(MetadataSetInvocation),
    Get(MetadataGetInvocation),
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ComponentFunctionInvocation {
    Globalize(ComponentGlobalizeInvocation),
    GlobalizeWithOwner(ComponentGlobalizeWithOwnerInvocation),
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum EpochManagerFunctionInvocation {
    Create(EpochManagerCreateInvocation),
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ClockFunctionInvocation {
    Create(ClockCreateInvocation),
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ClockMethodInvocation {
    GetCurrentTime(ClockGetCurrentTimeInvocation),
    CompareCurrentTime(ClockCompareCurrentTimeInvocation),
    SetCurrentTime(ClockSetCurrentTimeInvocation),
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ComponentMethodInvocation {
    SetRoyaltyConfig(ComponentSetRoyaltyConfigInvocation),
    ClaimRoyalty(ComponentClaimRoyaltyInvocation),
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum PackageMethodInvocation {
    SetRoyaltyConfig(PackageSetRoyaltyConfigInvocation),
    ClaimRoyalty(PackageClaimRoyaltyInvocation),
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum EpochManagerMethodInvocation {
    GetCurrentEpoch(EpochManagerGetCurrentEpochInvocation),
    NextRound(EpochManagerNextRoundInvocation),
    SetEpoch(EpochManagerSetEpochInvocation),
    RegisterValidator(EpochManagerRegisterValidatorInvocation),
    UnregisterValidator(EpochManagerUnregisterValidatorInvocation),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ResourceManagerFunctionInvocation {
    Create(ResourceManagerCreateInvocation),
    BurnBucket(ResourceManagerBucketBurnInvocation),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ProofMethodInvocation {
    Clone(ProofCloneInvocation),
    GetAmount(ProofGetAmountInvocation),
    GetNonFungibleIds(ProofGetNonFungibleIdsInvocation),
    GetResourceAddress(ProofGetResourceAddressInvocation),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum PackageFunctionInvocation {
    Publish(PackagePublishInvocation),
}

impl NativeFnInvocation {
    pub fn refs(&self) -> HashSet<RENodeId> {
        let mut refs = HashSet::new();
        match self {
            NativeFnInvocation::Function(native_function) => match native_function {
                NativeFunctionInvocation::EpochManager(invocation) => match invocation {
                    EpochManagerFunctionInvocation::Create(..) => {}
                },
                NativeFunctionInvocation::Clock(invocation) => match invocation {
                    ClockFunctionInvocation::Create(..) => {}
                },
                NativeFunctionInvocation::ResourceManager(invocation) => match invocation {
                    ResourceManagerFunctionInvocation::Create(..) => {}
                    ResourceManagerFunctionInvocation::BurnBucket(..) => {}
                },
                NativeFunctionInvocation::Package(invocation) => match invocation {
                    PackageFunctionInvocation::Publish(..) => {}
                },
                NativeFunctionInvocation::Component(invocation) => match invocation {
                    ComponentFunctionInvocation::Globalize(..) => {}
                    ComponentFunctionInvocation::GlobalizeWithOwner(..) => {}
                },
            },
            NativeFnInvocation::Method(native_method) => match native_method {
                NativeMethodInvocation::Component(component_method) => match component_method {
                    ComponentMethodInvocation::SetRoyaltyConfig(invocation) => {
                        refs.insert(invocation.receiver);
                    }
                    ComponentMethodInvocation::ClaimRoyalty(invocation) => {
                        refs.insert(invocation.receiver);
                    }
                },
                NativeMethodInvocation::Package(package_method) => match package_method {
                    PackageMethodInvocation::SetRoyaltyConfig(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::Package(
                            invocation.receiver,
                        )));
                    }
                    PackageMethodInvocation::ClaimRoyalty(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::Package(
                            invocation.receiver,
                        )));
                    }
                },
                NativeMethodInvocation::Bucket(bucket_method) => match bucket_method {
                    BucketMethodInvocation::Take(..) => {}
                    BucketMethodInvocation::CreateProof(..) => {}
                    BucketMethodInvocation::TakeNonFungibles(..) => {}
                    BucketMethodInvocation::GetNonFungibleIds(..) => {}
                    BucketMethodInvocation::GetAmount(..) => {}
                    BucketMethodInvocation::Put(..) => {}
                    BucketMethodInvocation::GetResourceAddress(..) => {}
                },
                NativeMethodInvocation::AuthZoneStack(auth_zone_method) => match auth_zone_method {
                    AuthZoneStackMethodInvocation::Pop(..) => {}
                    AuthZoneStackMethodInvocation::Push(..) => {}
                    AuthZoneStackMethodInvocation::CreateProof(..) => {}
                    AuthZoneStackMethodInvocation::CreateProofByAmount(..) => {}
                    AuthZoneStackMethodInvocation::CreateProofByIds(..) => {}
                    AuthZoneStackMethodInvocation::Clear(..) => {}
                    AuthZoneStackMethodInvocation::Drain(..) => {}
                    AuthZoneStackMethodInvocation::AssertAuthRule(..) => {}
                },
                NativeMethodInvocation::Proof(proof_method) => match proof_method {
                    ProofMethodInvocation::GetAmount(..) => {}
                    ProofMethodInvocation::GetNonFungibleIds(..) => {}
                    ProofMethodInvocation::GetResourceAddress(..) => {}
                    ProofMethodInvocation::Clone(..) => {}
                },
                NativeMethodInvocation::Vault(vault_method) => match vault_method {
                    VaultMethodInvocation::Take(..) => {}
                    VaultMethodInvocation::Put(..) => {}
                    VaultMethodInvocation::LockFee(..) => {}
                    VaultMethodInvocation::TakeNonFungibles(..) => {}
                    VaultMethodInvocation::GetAmount(..) => {}
                    VaultMethodInvocation::GetResourceAddress(..) => {}
                    VaultMethodInvocation::GetNonFungibleIds(..) => {}
                    VaultMethodInvocation::CreateProof(..) => {}
                    VaultMethodInvocation::CreateProofByAmount(..) => {}
                    VaultMethodInvocation::CreateProofByIds(..) => {}
                    VaultMethodInvocation::Recall(..) => {}
                    VaultMethodInvocation::RecallNonFungibles(..) => {}
                },
                NativeMethodInvocation::AccessRulesChain(access_rules_method) => {
                    match access_rules_method {
                        AccessRulesChainMethodInvocation::AddAccessCheck(invocation) => {
                            refs.insert(invocation.receiver);
                        }
                        AccessRulesChainMethodInvocation::SetMethodAccessRule(invocation) => {
                            refs.insert(invocation.receiver);
                        }
                        AccessRulesChainMethodInvocation::SetMethodMutability(invocation) => {
                            refs.insert(invocation.receiver);
                        }
                        AccessRulesChainMethodInvocation::SetGroupAccessRule(invocation) => {
                            refs.insert(invocation.receiver);
                        }
                        AccessRulesChainMethodInvocation::SetGroupMutability(invocation) => {
                            refs.insert(invocation.receiver);
                        }
                        AccessRulesChainMethodInvocation::GetLength(invocation) => {
                            refs.insert(invocation.receiver);
                        }
                    }
                }
                NativeMethodInvocation::Metadata(metadata_method) => match metadata_method {
                    MetadataMethodInvocation::Set(invocation) => {
                        refs.insert(invocation.receiver);
                    }
                    MetadataMethodInvocation::Get(invocation) => {
                        refs.insert(invocation.receiver);
                    }
                },
                NativeMethodInvocation::ResourceManager(resman_method) => match resman_method {
                    ResourceManagerMethodInvocation::Burn(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::Resource(
                            invocation.receiver,
                        )));
                    }
                    ResourceManagerMethodInvocation::UpdateVaultAuth(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::Resource(
                            invocation.receiver,
                        )));
                    }
                    ResourceManagerMethodInvocation::LockVaultAuth(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::Resource(
                            invocation.receiver,
                        )));
                    }
                    ResourceManagerMethodInvocation::CreateVault(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::Resource(
                            invocation.receiver,
                        )));
                    }
                    ResourceManagerMethodInvocation::CreateBucket(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::Resource(
                            invocation.receiver,
                        )));
                    }
                    ResourceManagerMethodInvocation::Mint(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::Resource(
                            invocation.receiver,
                        )));
                    }
                    ResourceManagerMethodInvocation::GetResourceType(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::Resource(
                            invocation.receiver,
                        )));
                    }
                    ResourceManagerMethodInvocation::GetTotalSupply(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::Resource(
                            invocation.receiver,
                        )));
                    }
                    ResourceManagerMethodInvocation::UpdateNonFungibleData(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::Resource(
                            invocation.receiver,
                        )));
                    }
                    ResourceManagerMethodInvocation::NonFungibleExists(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::Resource(
                            invocation.receiver,
                        )));
                    }
                    ResourceManagerMethodInvocation::GetNonFungible(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::Resource(
                            invocation.receiver,
                        )));
                    }
                },
                NativeMethodInvocation::EpochManager(epoch_manager_method) => {
                    match epoch_manager_method {
                        EpochManagerMethodInvocation::GetCurrentEpoch(invocation) => {
                            refs.insert(RENodeId::Global(GlobalAddress::System(
                                invocation.receiver,
                            )));
                        }
                        EpochManagerMethodInvocation::NextRound(invocation) => {
                            refs.insert(RENodeId::Global(GlobalAddress::System(
                                invocation.receiver,
                            )));
                        }
                        EpochManagerMethodInvocation::SetEpoch(invocation) => {
                            refs.insert(RENodeId::Global(GlobalAddress::System(
                                invocation.receiver,
                            )));
                        }
                        EpochManagerMethodInvocation::RegisterValidator(invocation) => {
                            refs.insert(RENodeId::Global(GlobalAddress::System(
                                invocation.receiver,
                            )));
                        }
                        EpochManagerMethodInvocation::UnregisterValidator(invocation) => {
                            refs.insert(RENodeId::Global(GlobalAddress::System(
                                invocation.receiver,
                            )));
                        }
                    }
                }
                NativeMethodInvocation::Clock(clock_method) => match clock_method {
                    ClockMethodInvocation::SetCurrentTime(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::System(invocation.receiver)));
                    }
                    ClockMethodInvocation::GetCurrentTime(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::System(invocation.receiver)));
                    }
                    ClockMethodInvocation::CompareCurrentTime(invocation) => {
                        refs.insert(RENodeId::Global(GlobalAddress::System(invocation.receiver)));
                    }
                },
                NativeMethodInvocation::Worktop(worktop_method) => match worktop_method {
                    WorktopMethodInvocation::TakeNonFungibles(..) => {}
                    WorktopMethodInvocation::Put(..) => {}
                    WorktopMethodInvocation::Drain(..) => {}
                    WorktopMethodInvocation::AssertContainsNonFungibles(..) => {}
                    WorktopMethodInvocation::AssertContains(..) => {}
                    WorktopMethodInvocation::AssertContainsAmount(..) => {}
                    WorktopMethodInvocation::TakeAll(..) => {}
                    WorktopMethodInvocation::TakeAmount(..) => {}
                },
                NativeMethodInvocation::TransactionRuntime(method) => match method {
                    TransactionRuntimeMethodInvocation::Get(..) => {}
                    TransactionRuntimeMethodInvocation::GenerateUuid(..) => {}
                },
                NativeMethodInvocation::Logger(method) => match method {
                    LoggerInvocation::Log(..) => {}
                },
            },
        }

        refs
    }
}
