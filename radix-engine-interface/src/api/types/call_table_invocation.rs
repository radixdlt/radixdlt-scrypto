use crate::api::component::*;
use crate::api::node_modules::auth::*;
use crate::api::node_modules::metadata::*;
use crate::api::package::PackageAddress;
use crate::api::package::*;
use crate::api::types::*;
use crate::blueprints::access_controller::*;
use crate::blueprints::clock::*;
use crate::blueprints::epoch_manager::*;
use crate::blueprints::identity::*;
use crate::blueprints::logger::*;
use crate::blueprints::resource::*;
use crate::blueprints::transaction_runtime::TransactionRuntimeGenerateUuidInvocation;
use crate::blueprints::transaction_runtime::*;
use crate::data::scrypto_encode;
use crate::data::types::{ManifestBucket, ManifestProof};
use crate::data::ScryptoValue;
use crate::*;
use radix_engine_interface::data::ReplaceManifestValuesError;
use sbor::rust::collections::{HashMap, HashSet};
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

// TODO: Remove enum
#[derive(Debug, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum CallTableInvocation {
    Native(NativeInvocation),
    Scrypto(ScryptoInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ScryptoInvocation {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub fn_name: String,
    pub receiver: Option<ScryptoReceiver>,
    pub args: Vec<u8>,
}

impl Invocation for ScryptoInvocation {
    type Output = ScryptoValue;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Scrypto(ScryptoFnIdentifier {
            package_address: self.package_address,
            blueprint_name: self.blueprint_name.clone(),
            ident: self.fn_name.clone(),
        })
    }
}

impl Into<CallTableInvocation> for ScryptoInvocation {
    fn into(self) -> CallTableInvocation {
        CallTableInvocation::Scrypto(self)
    }
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
    AccessController(AccessControllerInvocation),
}

impl NativeInvocation {
    pub fn replace_ids(
        &mut self,
        _proof_replacements: &mut HashMap<ManifestProof, ProofId>,
        bucket_replacements: &mut HashMap<ManifestBucket, BucketId>,
    ) -> Result<(), ReplaceManifestValuesError> {
        match self {
            NativeInvocation::EpochManager(EpochManagerInvocation::Create(invocation)) => {
                for (_, (bucket, _)) in &mut invocation.validator_set {
                    let next_id = bucket_replacements
                        .remove(&ManifestBucket(bucket.0))
                        .ok_or(ReplaceManifestValuesError::BucketNotFound(ManifestBucket(
                            bucket.0,
                        )))?;
                    bucket.0 = next_id;
                }
            }
            _ => {} // TODO: Expand this
        }
        Ok(())
    }
}

impl Into<CallTableInvocation> for NativeInvocation {
    fn into(self) -> CallTableInvocation {
        CallTableInvocation::Native(self)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TransactionRuntimeInvocation {
    GetHash(TransactionRuntimeGetHashInvocation),
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
    Stake(ValidatorStakeInvocation),
    Unstake(ValidatorUnstakeInvocation),
    ClaimXrd(ValidatorClaimXrdInvocation),
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
    BurnBucket(ResourceManagerBurnBucketInvocation),
    GetResourceType(ResourceManagerGetResourceTypeInvocation),
    Burn(ResourceManagerBurnInvocation),
    MintNonFungible(ResourceManagerMintNonFungibleInvocation),
    MintUuidNonFungible(ResourceManagerMintUuidNonFungibleInvocation),
    MintFungible(ResourceManagerMintFungibleInvocation),
    CreateBucket(ResourceManagerCreateBucketInvocation),
    CreateVault(ResourceManagerCreateVaultInvocation),
    UpdateVaultAuth(ResourceManagerUpdateVaultAuthInvocation),
    SetVaultAuthMutability(ResourceManagerSetVaultAuthMutabilityInvocation),
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum AccessControllerInvocation {
    CreateGlobal(AccessControllerCreateGlobalInvocation),

    CreateProof(AccessControllerCreateProofInvocation),

    InitiateRecoveryAsPrimary(AccessControllerInitiateRecoveryAsPrimaryInvocation),
    InitiateRecoveryAsRecovery(AccessControllerInitiateRecoveryAsRecoveryInvocation),

    QuickConfirmPrimaryRoleRecoveryProposal(
        AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInvocation,
    ),
    QuickConfirmRecoveryRoleRecoveryProposal(
        AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInvocation,
    ),

    TimedConfirmRecovery(AccessControllerTimedConfirmRecoveryInvocation),

    CancelPrimaryRoleRecoveryProposal(AccessControllerCancelPrimaryRoleRecoveryProposalInvocation),
    CancelRecoveryRoleRecoveryProposal(
        AccessControllerCancelRecoveryRoleRecoveryProposalInvocation,
    ),

    LockPrimaryRole(AccessControllerLockPrimaryRoleInvocation),
    UnlockPrimaryRole(AccessControllerUnlockPrimaryRoleInvocation),

    StopTimedRecovery(AccessControllerStopTimedRecoveryInvocation),
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
                ResourceInvocation::SetVaultAuthMutability(invocation) => {
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
                EpochManagerInvocation::Create(invocation) => {
                    for (_key, (_bucket, account_address)) in &invocation.validator_set {
                        refs.insert(RENodeId::Global(GlobalAddress::Component(*account_address)));
                    }
                }
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
                ValidatorInvocation::Stake(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(
                        invocation.receiver,
                    )));
                }
                ValidatorInvocation::Unstake(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(
                        invocation.receiver,
                    )));
                }
                ValidatorInvocation::ClaimXrd(invocation) => {
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
                TransactionRuntimeInvocation::GetHash(..) => {}
                TransactionRuntimeInvocation::GenerateUuid(..) => {}
            },
            NativeInvocation::AccessController(method) => match method {
                AccessControllerInvocation::CreateGlobal(..) => {}
                AccessControllerInvocation::CreateProof(
                    AccessControllerCreateProofInvocation { receiver, .. },
                )
                | AccessControllerInvocation::InitiateRecoveryAsPrimary(
                    AccessControllerInitiateRecoveryAsPrimaryInvocation { receiver, .. },
                )
                | AccessControllerInvocation::InitiateRecoveryAsRecovery(
                    AccessControllerInitiateRecoveryAsRecoveryInvocation { receiver, .. },
                )
                | AccessControllerInvocation::QuickConfirmPrimaryRoleRecoveryProposal(
                    AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInvocation {
                        receiver,
                        ..
                    },
                )
                | AccessControllerInvocation::QuickConfirmRecoveryRoleRecoveryProposal(
                    AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInvocation {
                        receiver,
                        ..
                    },
                )
                | AccessControllerInvocation::TimedConfirmRecovery(
                    AccessControllerTimedConfirmRecoveryInvocation { receiver, .. },
                )
                | AccessControllerInvocation::CancelPrimaryRoleRecoveryProposal(
                    AccessControllerCancelPrimaryRoleRecoveryProposalInvocation {
                        receiver, ..
                    },
                )
                | AccessControllerInvocation::CancelRecoveryRoleRecoveryProposal(
                    AccessControllerCancelRecoveryRoleRecoveryProposalInvocation {
                        receiver, ..
                    },
                )
                | AccessControllerInvocation::LockPrimaryRole(
                    AccessControllerLockPrimaryRoleInvocation { receiver, .. },
                )
                | AccessControllerInvocation::UnlockPrimaryRole(
                    AccessControllerUnlockPrimaryRoleInvocation { receiver, .. },
                )
                | AccessControllerInvocation::StopTimedRecovery(
                    AccessControllerStopTimedRecoveryInvocation { receiver, .. },
                ) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(*receiver)));
                }
            },
        }

        refs
    }
}

impl NativeInvocation {
    pub fn flatten(&self) -> (NativeFn, Vec<u8>) {
        let (fn_identifier, encoding) = match self {
            NativeInvocation::AccessRulesChain(i) => match i {
                AccessRulesChainInvocation::AddAccessCheck(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessRulesChainInvocation::SetMethodAccessRule(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessRulesChainInvocation::SetMethodMutability(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessRulesChainInvocation::SetGroupAccessRule(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessRulesChainInvocation::SetGroupMutability(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessRulesChainInvocation::GetLength(i) => (i.fn_identifier(), scrypto_encode(&i)),
            },
            NativeInvocation::Metadata(i) => match i {
                MetadataInvocation::Set(i) => (i.fn_identifier(), scrypto_encode(&i)),
                MetadataInvocation::Get(i) => (i.fn_identifier(), scrypto_encode(&i)),
            },
            NativeInvocation::Package(i) => match i {
                PackageInvocation::Publish(i) => (i.fn_identifier(), scrypto_encode(&i)),
                PackageInvocation::SetRoyaltyConfig(i) => (i.fn_identifier(), scrypto_encode(&i)),
                PackageInvocation::ClaimRoyalty(i) => (i.fn_identifier(), scrypto_encode(&i)),
            },
            NativeInvocation::Component(i) => match i {
                ComponentInvocation::Globalize(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ComponentInvocation::GlobalizeWithOwner(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                ComponentInvocation::SetRoyaltyConfig(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ComponentInvocation::ClaimRoyalty(i) => (i.fn_identifier(), scrypto_encode(&i)),
            },
            NativeInvocation::EpochManager(i) => match i {
                EpochManagerInvocation::Create(i) => (i.fn_identifier(), scrypto_encode(&i)),
                EpochManagerInvocation::GetCurrentEpoch(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                EpochManagerInvocation::SetEpoch(i) => (i.fn_identifier(), scrypto_encode(&i)),
                EpochManagerInvocation::NextRound(i) => (i.fn_identifier(), scrypto_encode(&i)),
                EpochManagerInvocation::CreateValidator(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                EpochManagerInvocation::UpdateValidator(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
            },
            NativeInvocation::Validator(i) => match i {
                ValidatorInvocation::Register(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ValidatorInvocation::Unregister(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ValidatorInvocation::Stake(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ValidatorInvocation::Unstake(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ValidatorInvocation::ClaimXrd(i) => (i.fn_identifier(), scrypto_encode(&i)),
            },
            NativeInvocation::Clock(i) => match i {
                ClockInvocation::Create(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ClockInvocation::GetCurrentTime(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ClockInvocation::CompareCurrentTime(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ClockInvocation::SetCurrentTime(i) => (i.fn_identifier(), scrypto_encode(&i)),
            },
            NativeInvocation::Identity(i) => match i {
                IdentityInvocation::Create(i) => (i.fn_identifier(), scrypto_encode(&i)),
            },
            NativeInvocation::Logger(i) => match i {
                LoggerInvocation::Log(i) => (i.fn_identifier(), scrypto_encode(&i)),
            },
            NativeInvocation::AuthZoneStack(i) => match i {
                AuthZoneStackInvocation::Pop(i) => (i.fn_identifier(), scrypto_encode(&i)),
                AuthZoneStackInvocation::Push(i) => (i.fn_identifier(), scrypto_encode(&i)),
                AuthZoneStackInvocation::CreateProof(i) => (i.fn_identifier(), scrypto_encode(&i)),
                AuthZoneStackInvocation::CreateProofByAmount(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AuthZoneStackInvocation::CreateProofByIds(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AuthZoneStackInvocation::Clear(i) => (i.fn_identifier(), scrypto_encode(&i)),
                AuthZoneStackInvocation::Drain(i) => (i.fn_identifier(), scrypto_encode(&i)),
                AuthZoneStackInvocation::AssertAuthRule(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
            },
            NativeInvocation::ResourceManager(i) => match i {
                ResourceInvocation::CreateNonFungible(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ResourceInvocation::CreateFungible(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ResourceInvocation::CreateNonFungibleWithInitialSupply(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                ResourceInvocation::CreateUuidNonFungibleWithInitialSupply(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                ResourceInvocation::CreateFungibleWithInitialSupply(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                ResourceInvocation::BurnBucket(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ResourceInvocation::GetResourceType(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ResourceInvocation::Burn(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ResourceInvocation::MintNonFungible(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ResourceInvocation::MintUuidNonFungible(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                ResourceInvocation::MintFungible(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ResourceInvocation::CreateBucket(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ResourceInvocation::CreateVault(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ResourceInvocation::UpdateVaultAuth(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ResourceInvocation::SetVaultAuthMutability(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                ResourceInvocation::GetTotalSupply(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ResourceInvocation::UpdateNonFungibleData(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                ResourceInvocation::GetNonFungible(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ResourceInvocation::NonFungibleExists(i) => (i.fn_identifier(), scrypto_encode(&i)),
            },
            NativeInvocation::Bucket(i) => match i {
                BucketInvocation::Take(i) => (i.fn_identifier(), scrypto_encode(&i)),
                BucketInvocation::TakeNonFungibles(i) => (i.fn_identifier(), scrypto_encode(&i)),
                BucketInvocation::Put(i) => (i.fn_identifier(), scrypto_encode(&i)),
                BucketInvocation::GetNonFungibleLocalIds(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                BucketInvocation::GetAmount(i) => (i.fn_identifier(), scrypto_encode(&i)),
                BucketInvocation::GetResourceAddress(i) => (i.fn_identifier(), scrypto_encode(&i)),
                BucketInvocation::CreateProof(i) => (i.fn_identifier(), scrypto_encode(&i)),
            },
            NativeInvocation::Vault(i) => match i {
                VaultInvocation::Take(i) => (i.fn_identifier(), scrypto_encode(&i)),
                VaultInvocation::LockFee(i) => (i.fn_identifier(), scrypto_encode(&i)),
                VaultInvocation::Put(i) => (i.fn_identifier(), scrypto_encode(&i)),
                VaultInvocation::TakeNonFungibles(i) => (i.fn_identifier(), scrypto_encode(&i)),
                VaultInvocation::GetAmount(i) => (i.fn_identifier(), scrypto_encode(&i)),
                VaultInvocation::GetResourceAddress(i) => (i.fn_identifier(), scrypto_encode(&i)),
                VaultInvocation::GetNonFungibleLocalIds(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                VaultInvocation::CreateProof(i) => (i.fn_identifier(), scrypto_encode(&i)),
                VaultInvocation::CreateProofByAmount(i) => (i.fn_identifier(), scrypto_encode(&i)),
                VaultInvocation::CreateProofByIds(i) => (i.fn_identifier(), scrypto_encode(&i)),
                VaultInvocation::Recall(i) => (i.fn_identifier(), scrypto_encode(&i)),
                VaultInvocation::RecallNonFungibles(i) => (i.fn_identifier(), scrypto_encode(&i)),
            },
            NativeInvocation::Proof(i) => match i {
                ProofInvocation::Clone(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ProofInvocation::GetAmount(i) => (i.fn_identifier(), scrypto_encode(&i)),
                ProofInvocation::GetNonFungibleLocalIds(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                ProofInvocation::GetResourceAddress(i) => (i.fn_identifier(), scrypto_encode(&i)),
            },
            NativeInvocation::Worktop(i) => match i {
                WorktopInvocation::TakeAll(i) => (i.fn_identifier(), scrypto_encode(&i)),
                WorktopInvocation::TakeAmount(i) => (i.fn_identifier(), scrypto_encode(&i)),
                WorktopInvocation::TakeNonFungibles(i) => (i.fn_identifier(), scrypto_encode(&i)),
                WorktopInvocation::Put(i) => (i.fn_identifier(), scrypto_encode(&i)),
                WorktopInvocation::AssertContains(i) => (i.fn_identifier(), scrypto_encode(&i)),
                WorktopInvocation::AssertContainsAmount(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                WorktopInvocation::AssertContainsNonFungibles(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                WorktopInvocation::Drain(i) => (i.fn_identifier(), scrypto_encode(&i)),
            },
            NativeInvocation::TransactionRuntime(i) => match i {
                TransactionRuntimeInvocation::GetHash(i) => (i.fn_identifier(), scrypto_encode(&i)),
                TransactionRuntimeInvocation::GenerateUuid(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
            },
            NativeInvocation::AccessController(i) => match i {
                AccessControllerInvocation::CreateGlobal(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessControllerInvocation::CreateProof(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessControllerInvocation::InitiateRecoveryAsPrimary(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessControllerInvocation::InitiateRecoveryAsRecovery(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessControllerInvocation::QuickConfirmPrimaryRoleRecoveryProposal(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessControllerInvocation::QuickConfirmRecoveryRoleRecoveryProposal(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessControllerInvocation::TimedConfirmRecovery(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessControllerInvocation::CancelPrimaryRoleRecoveryProposal(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessControllerInvocation::CancelRecoveryRoleRecoveryProposal(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessControllerInvocation::LockPrimaryRole(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessControllerInvocation::UnlockPrimaryRole(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
                AccessControllerInvocation::StopTimedRecovery(i) => {
                    (i.fn_identifier(), scrypto_encode(&i))
                }
            },
        };

        let native_fn = match fn_identifier {
            FnIdentifier::Scrypto(_) => panic!("TODO: refine the interface"),
            FnIdentifier::Native(f) => f,
        };
        let invocation = encoding.expect("Failed to encode native invocation");

        (native_fn, invocation)
    }
}
