use crate::api::types::*;
use crate::*;
use sbor::rust::collections::HashSet;
use sbor::rust::fmt::Debug;

// TODO: Remove enum
#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum CallTableInvocation {
    Native(NativeInvocation),
    Scrypto(ScryptoInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum NativeInvocation {
    AccessRulesChain(AccessRulesChainInvocation),
    Metadata(MetadataInvocation),
    Package(PackageInvocation),
    Component(ComponentInvocation),
    EpochManager(EpochManagerInvocation),
    Validator(ValidatorInvocation),
    Clock(ClockInvocation),
    Identity(IdentityInvocation),
    Logger(LoggerInvocation),
    AuthZoneStack(AuthZoneStackInvocation),
    ResourceManager(ResourceInvocation),
    Bucket(BucketInvocation),
    Vault(VaultInvocation),
    Proof(ProofInvocation),
    Worktop(WorktopInvocation),
    TransactionRuntime(TransactionRuntimeInvocation),
}

impl Into<CallTableInvocation> for NativeInvocation {
    fn into(self) -> CallTableInvocation {
        CallTableInvocation::Native(self)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TransactionRuntimeInvocation {
    Get(TransactionRuntimeGetHashInvocation),
    GenerateUuid(TransactionRuntimeGenerateUuidInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum AccessRulesChainInvocation {
    AddAccessCheck(AccessRulesAddAccessCheckInvocation),
    SetMethodAccessRule(AccessRulesSetMethodAccessRuleInvocation),
    SetMethodMutability(AccessRulesSetMethodMutabilityInvocation),
    SetGroupAccessRule(AccessRulesSetGroupAccessRuleInvocation),
    SetGroupMutability(AccessRulesSetGroupMutabilityInvocation),
    GetLength(AccessRulesGetLengthInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum MetadataInvocation {
    Set(MetadataSetInvocation),
    Get(MetadataGetInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ClockInvocation {
    Create(ClockCreateInvocation),
    GetCurrentTime(ClockGetCurrentTimeInvocation),
    CompareCurrentTime(ClockCompareCurrentTimeInvocation),
    SetCurrentTime(ClockSetCurrentTimeInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum IdentityInvocation {
    Create(IdentityCreateInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum LoggerInvocation {
    Log(LoggerLogInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ComponentInvocation {
    Globalize(ComponentGlobalizeInvocation),
    GlobalizeWithOwner(ComponentGlobalizeWithOwnerInvocation),
    SetRoyaltyConfig(ComponentSetRoyaltyConfigInvocation),
    ClaimRoyalty(ComponentClaimRoyaltyInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum PackageInvocation {
    Publish(PackagePublishInvocation),
    SetRoyaltyConfig(PackageSetRoyaltyConfigInvocation),
    ClaimRoyalty(PackageClaimRoyaltyInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum EpochManagerInvocation {
    Create(EpochManagerCreateInvocation),
    GetCurrentEpoch(EpochManagerGetCurrentEpochInvocation),
    SetEpoch(EpochManagerSetEpochInvocation),
    NextRound(EpochManagerNextRoundInvocation),
    CreateValidator(EpochManagerCreateValidatorInvocation),
    UpdateValidator(EpochManagerUpdateValidatorInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ValidatorInvocation {
    Register(ValidatorRegisterInvocation),
    Unregister(ValidatorUnregisterInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum AuthZoneStackInvocation {
    Pop(AuthZonePopInvocation),
    Push(AuthZonePushInvocation),
    CreateProof(AuthZoneCreateProofInvocation),
    CreateProofByAmount(AuthZoneCreateProofByAmountInvocation),
    CreateProofByIds(AuthZoneCreateProofByIdsInvocation),
    Clear(AuthZoneClearInvocation),
    Drain(AuthZoneDrainInvocation),
    AssertAuthRule(AuthZoneAssertAccessRuleInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ResourceInvocation {
    CreateNonFungible(ResourceManagerCreateNonFungibleInvocation),
    CreateFungible(ResourceManagerCreateFungibleInvocation),
    CreateNonFungibleWithInitialSupply(ResourceManagerCreateNonFungibleWithInitialSupplyInvocation),
    CreateUuidNonFungibleWithInitialSupply(
        ResourceManagerCreateUuidNonFungibleWithInitialSupplyInvocation,
    ),
    CreateFungibleWithInitialSupply(ResourceManagerCreateFungibleWithInitialSupplyInvocation),
    BurnBucket(ResourceManagerBucketBurnInvocation),
    GetResourceType(ResourceManagerGetResourceTypeInvocation),
    Burn(ResourceManagerBurnInvocation),
    MintNonFungible(ResourceManagerMintNonFungibleInvocation),
    MintUuidNonFungible(ResourceManagerMintUuidNonFungibleInvocation),
    MintFungible(ResourceManagerMintFungibleInvocation),
    CreateBucket(ResourceManagerCreateBucketInvocation),
    CreateVault(ResourceManagerCreateVaultInvocation),
    UpdateVaultAuth(ResourceManagerUpdateVaultAuthInvocation),
    LockVaultAuth(ResourceManagerSetVaultAuthMutabilityInvocation),
    GetTotalSupply(ResourceManagerGetTotalSupplyInvocation),
    UpdateNonFungibleData(ResourceManagerUpdateNonFungibleDataInvocation),
    GetNonFungible(ResourceManagerGetNonFungibleInvocation),
    NonFungibleExists(ResourceManagerNonFungibleExistsInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum BucketInvocation {
    Take(BucketTakeInvocation),
    TakeNonFungibles(BucketTakeNonFungiblesInvocation),
    Put(BucketPutInvocation),
    GetNonFungibleLocalIds(BucketGetNonFungibleLocalIdsInvocation),
    GetAmount(BucketGetAmountInvocation),
    GetResourceAddress(BucketGetResourceAddressInvocation),
    CreateProof(BucketCreateProofInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum VaultInvocation {
    Take(VaultTakeInvocation),
    LockFee(VaultLockFeeInvocation),
    Put(VaultPutInvocation),
    TakeNonFungibles(VaultTakeNonFungiblesInvocation),
    GetAmount(VaultGetAmountInvocation),
    GetResourceAddress(VaultGetResourceAddressInvocation),
    GetNonFungibleLocalIds(VaultGetNonFungibleLocalIdsInvocation),
    CreateProof(VaultCreateProofInvocation),
    CreateProofByAmount(VaultCreateProofByAmountInvocation),
    CreateProofByIds(VaultCreateProofByIdsInvocation),
    Recall(VaultRecallInvocation),
    RecallNonFungibles(VaultRecallNonFungiblesInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ProofInvocation {
    Clone(ProofCloneInvocation),
    GetAmount(ProofGetAmountInvocation),
    GetNonFungibleLocalIds(ProofGetNonFungibleLocalIdsInvocation),
    GetResourceAddress(ProofGetResourceAddressInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum WorktopInvocation {
    TakeAll(WorktopTakeAllInvocation),
    TakeAmount(WorktopTakeAmountInvocation),
    TakeNonFungibles(WorktopTakeNonFungiblesInvocation),
    Put(WorktopPutInvocation),
    AssertContains(WorktopAssertContainsInvocation),
    AssertContainsAmount(WorktopAssertContainsAmountInvocation),
    AssertContainsNonFungibles(WorktopAssertContainsNonFungiblesInvocation),
    Drain(WorktopDrainInvocation),
}

impl NativeInvocation {
    pub fn refs(&self) -> HashSet<RENodeId> {
        let mut refs = HashSet::new();
        match self {
            NativeInvocation::Component(invocation) => match invocation {
                ComponentInvocation::Globalize(..) => {}
                ComponentInvocation::GlobalizeWithOwner(..) => {}
                ComponentInvocation::SetRoyaltyConfig(invocation) => {
                    refs.insert(invocation.receiver);
                }
                ComponentInvocation::ClaimRoyalty(invocation) => {
                    refs.insert(invocation.receiver);
                }
            },
            NativeInvocation::Package(package_method) => match package_method {
                PackageInvocation::Publish(..) => {}
                PackageInvocation::SetRoyaltyConfig(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Package(
                        invocation.receiver,
                    )));
                }
                PackageInvocation::ClaimRoyalty(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Package(
                        invocation.receiver,
                    )));
                }
            },
            NativeInvocation::Bucket(bucket_method) => match bucket_method {
                BucketInvocation::Take(..) => {}
                BucketInvocation::CreateProof(..) => {}
                BucketInvocation::TakeNonFungibles(..) => {}
                BucketInvocation::GetNonFungibleLocalIds(..) => {}
                BucketInvocation::GetAmount(..) => {}
                BucketInvocation::Put(..) => {}
                BucketInvocation::GetResourceAddress(..) => {}
            },
            NativeInvocation::AuthZoneStack(auth_zone_method) => match auth_zone_method {
                AuthZoneStackInvocation::Pop(..) => {}
                AuthZoneStackInvocation::Push(..) => {}
                AuthZoneStackInvocation::CreateProof(..) => {}
                AuthZoneStackInvocation::CreateProofByAmount(..) => {}
                AuthZoneStackInvocation::CreateProofByIds(..) => {}
                AuthZoneStackInvocation::Clear(..) => {}
                AuthZoneStackInvocation::Drain(..) => {}
                AuthZoneStackInvocation::AssertAuthRule(..) => {}
            },
            NativeInvocation::Proof(proof_method) => match proof_method {
                ProofInvocation::GetAmount(..) => {}
                ProofInvocation::GetNonFungibleLocalIds(..) => {}
                ProofInvocation::GetResourceAddress(..) => {}
                ProofInvocation::Clone(..) => {}
            },
            NativeInvocation::Vault(vault_method) => match vault_method {
                VaultInvocation::Take(..) => {}
                VaultInvocation::Put(..) => {}
                VaultInvocation::LockFee(..) => {}
                VaultInvocation::TakeNonFungibles(..) => {}
                VaultInvocation::GetAmount(..) => {}
                VaultInvocation::GetResourceAddress(..) => {}
                VaultInvocation::GetNonFungibleLocalIds(..) => {}
                VaultInvocation::CreateProof(..) => {}
                VaultInvocation::CreateProofByAmount(..) => {}
                VaultInvocation::CreateProofByIds(..) => {}
                VaultInvocation::Recall(..) => {}
                VaultInvocation::RecallNonFungibles(..) => {}
            },
            NativeInvocation::AccessRulesChain(access_rules_method) => match access_rules_method {
                AccessRulesChainInvocation::AddAccessCheck(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::SetMethodAccessRule(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::SetMethodMutability(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::SetGroupAccessRule(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::SetGroupMutability(invocation) => {
                    refs.insert(invocation.receiver);
                }
                AccessRulesChainInvocation::GetLength(invocation) => {
                    refs.insert(invocation.receiver);
                }
            },
            NativeInvocation::Metadata(metadata_method) => match metadata_method {
                MetadataInvocation::Set(invocation) => {
                    refs.insert(invocation.receiver);
                }
                MetadataInvocation::Get(invocation) => {
                    refs.insert(invocation.receiver);
                }
            },
            NativeInvocation::ResourceManager(resman_method) => match resman_method {
                ResourceInvocation::CreateNonFungible(..) => {}
                ResourceInvocation::CreateFungible(..) => {}
                ResourceInvocation::CreateNonFungibleWithInitialSupply(..) => {}
                ResourceInvocation::CreateUuidNonFungibleWithInitialSupply(..) => {}
                ResourceInvocation::CreateFungibleWithInitialSupply(..) => {}
                ResourceInvocation::BurnBucket(..) => {}
                ResourceInvocation::Burn(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Resource(
                        invocation.receiver,
                    )));
                }
                ResourceInvocation::UpdateVaultAuth(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Resource(
                        invocation.receiver,
                    )));
                }
                ResourceInvocation::LockVaultAuth(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Resource(
                        invocation.receiver,
                    )));
                }
                ResourceInvocation::CreateVault(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Resource(
                        invocation.receiver,
                    )));
                }
                ResourceInvocation::CreateBucket(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Resource(
                        invocation.receiver,
                    )));
                }
                ResourceInvocation::MintNonFungible(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Resource(
                        invocation.receiver,
                    )));
                }
                ResourceInvocation::MintUuidNonFungible(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Resource(
                        invocation.receiver,
                    )));
                }
                ResourceInvocation::MintFungible(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Resource(
                        invocation.receiver,
                    )));
                }
                ResourceInvocation::GetResourceType(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Resource(
                        invocation.receiver,
                    )));
                }
                ResourceInvocation::GetTotalSupply(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Resource(
                        invocation.receiver,
                    )));
                }
                ResourceInvocation::UpdateNonFungibleData(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Resource(
                        invocation.receiver,
                    )));
                }
                ResourceInvocation::NonFungibleExists(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Resource(
                        invocation.receiver,
                    )));
                }
                ResourceInvocation::GetNonFungible(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Resource(
                        invocation.receiver,
                    )));
                }
            },
            NativeInvocation::EpochManager(epoch_manager_method) => match epoch_manager_method {
                EpochManagerInvocation::Create(..) => {}
                EpochManagerInvocation::GetCurrentEpoch(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(
                        invocation.receiver,
                    )));
                }
                EpochManagerInvocation::NextRound(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(
                        invocation.receiver,
                    )));
                }
                EpochManagerInvocation::SetEpoch(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(
                        invocation.receiver,
                    )));
                }
                EpochManagerInvocation::CreateValidator(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(
                        invocation.receiver,
                    )));
                }
                EpochManagerInvocation::UpdateValidator(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(
                        invocation.receiver,
                    )));
                }
            },
            NativeInvocation::Validator(method) => match method {
                ValidatorInvocation::Register(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(
                        invocation.receiver,
                    )));
                }
                ValidatorInvocation::Unregister(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(
                        invocation.receiver,
                    )));
                }
            },
            NativeInvocation::Clock(clock_method) => match clock_method {
                ClockInvocation::Create(..) => {}
                ClockInvocation::SetCurrentTime(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(
                        invocation.receiver,
                    )));
                }
                ClockInvocation::GetCurrentTime(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(
                        invocation.receiver,
                    )));
                }
                ClockInvocation::CompareCurrentTime(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(
                        invocation.receiver,
                    )));
                }
            },
            NativeInvocation::Identity(invocation) => match invocation {
                IdentityInvocation::Create(..) => {}
            },
            NativeInvocation::Logger(method) => match method {
                LoggerInvocation::Log(..) => {
                    refs.insert(RENodeId::Logger);
                }
            },
            NativeInvocation::Worktop(worktop_method) => match worktop_method {
                WorktopInvocation::TakeNonFungibles(..) => {}
                WorktopInvocation::Put(..) => {}
                WorktopInvocation::Drain(..) => {}
                WorktopInvocation::AssertContainsNonFungibles(..) => {}
                WorktopInvocation::AssertContains(..) => {}
                WorktopInvocation::AssertContainsAmount(..) => {}
                WorktopInvocation::TakeAll(..) => {}
                WorktopInvocation::TakeAmount(..) => {}
            },
            NativeInvocation::TransactionRuntime(method) => match method {
                TransactionRuntimeInvocation::Get(..) => {}
                TransactionRuntimeInvocation::GenerateUuid(..) => {}
            },
        }

        refs
    }
}
