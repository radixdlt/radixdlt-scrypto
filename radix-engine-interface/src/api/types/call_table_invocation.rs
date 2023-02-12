use crate::api::component::*;
use crate::api::node_modules::auth::*;
use crate::api::node_modules::metadata::*;
use crate::api::package::PackageAddress;
use crate::api::package::*;
use crate::api::types::*;
use crate::blueprints::access_controller::*;
use crate::blueprints::clock::*;
use crate::blueprints::epoch_manager::*;
use crate::blueprints::logger::*;
use crate::blueprints::resource::*;
use crate::blueprints::transaction_runtime::TransactionRuntimeGenerateUuidInvocation;
use crate::blueprints::transaction_runtime::*;
use crate::data::scrypto_encode;
use crate::data::ScryptoValue;
use crate::*;
use sbor::rust::collections::HashSet;
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
    GetCurrentTime(ClockGetCurrentTimeInvocation),
    CompareCurrentTime(ClockCompareCurrentTimeInvocation),
    SetCurrentTime(ClockSetCurrentTimeInvocation),
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
    PublishNative(PackagePublishNativeInvocation),
    SetRoyaltyConfig(PackageSetRoyaltyConfigInvocation),
    ClaimRoyalty(PackageClaimRoyaltyInvocation),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum EpochManagerInvocation {
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
    UpdateKey(ValidatorUpdateKeyInvocation),
    UpdateAcceptDelegatedStake(ValidatorUpdateAcceptDelegatedStakeInvocation),
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
                PackageInvocation::PublishNative(..) => {}
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
                ValidatorInvocation::UpdateKey(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(
                        invocation.receiver,
                    )));
                }
                ValidatorInvocation::UpdateAcceptDelegatedStake(invocation) => {
                    refs.insert(RENodeId::Global(GlobalAddress::Component(
                        invocation.receiver,
                    )));
                }
            },
            NativeInvocation::Clock(clock_method) => match clock_method {
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
                AccessControllerInvocation::CancelPrimaryRoleRecoveryProposal(
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

fn get_native_fn<T: SerializableInvocation>(_: &T) -> NativeFn {
    T::native_fn()
}

impl NativeInvocation {
    pub fn flatten(&self) -> (NativeFn, Vec<u8>) {
        let (native_fn, encoding) = match self {
            NativeInvocation::AccessRulesChain(i) => match i {
                AccessRulesChainInvocation::AddAccessCheck(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::SetMethodAccessRule(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::SetMethodMutability(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::SetGroupAccessRule(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::SetGroupMutability(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessRulesChainInvocation::GetLength(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::Metadata(i) => match i {
                MetadataInvocation::Set(i) => (get_native_fn(i), scrypto_encode(i)),
                MetadataInvocation::Get(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::Package(i) => match i {
                PackageInvocation::Publish(i) => (get_native_fn(i), scrypto_encode(i)),
                PackageInvocation::PublishNative(i) => (get_native_fn(i), scrypto_encode(i)),
                PackageInvocation::SetRoyaltyConfig(i) => (get_native_fn(i), scrypto_encode(i)),
                PackageInvocation::ClaimRoyalty(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::Component(i) => match i {
                ComponentInvocation::Globalize(i) => (get_native_fn(i), scrypto_encode(i)),
                ComponentInvocation::GlobalizeWithOwner(i) => (get_native_fn(i), scrypto_encode(i)),
                ComponentInvocation::SetRoyaltyConfig(i) => (get_native_fn(i), scrypto_encode(i)),
                ComponentInvocation::ClaimRoyalty(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::EpochManager(i) => match i {
                EpochManagerInvocation::GetCurrentEpoch(i) => (get_native_fn(i), scrypto_encode(i)),
                EpochManagerInvocation::SetEpoch(i) => (get_native_fn(i), scrypto_encode(i)),
                EpochManagerInvocation::NextRound(i) => (get_native_fn(i), scrypto_encode(i)),
                EpochManagerInvocation::CreateValidator(i) => (get_native_fn(i), scrypto_encode(i)),
                EpochManagerInvocation::UpdateValidator(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::Validator(i) => match i {
                ValidatorInvocation::Register(i) => (get_native_fn(i), scrypto_encode(i)),
                ValidatorInvocation::Unregister(i) => (get_native_fn(i), scrypto_encode(i)),
                ValidatorInvocation::Stake(i) => (get_native_fn(i), scrypto_encode(i)),
                ValidatorInvocation::Unstake(i) => (get_native_fn(i), scrypto_encode(i)),
                ValidatorInvocation::ClaimXrd(i) => (get_native_fn(i), scrypto_encode(i)),
                ValidatorInvocation::UpdateKey(i) => (get_native_fn(i), scrypto_encode(i)),
                ValidatorInvocation::UpdateAcceptDelegatedStake(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
            },
            NativeInvocation::Clock(i) => match i {
                ClockInvocation::GetCurrentTime(i) => (get_native_fn(i), scrypto_encode(i)),
                ClockInvocation::CompareCurrentTime(i) => (get_native_fn(i), scrypto_encode(i)),
                ClockInvocation::SetCurrentTime(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::Logger(i) => match i {
                LoggerInvocation::Log(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::AuthZoneStack(i) => match i {
                AuthZoneStackInvocation::Pop(i) => (get_native_fn(i), scrypto_encode(i)),
                AuthZoneStackInvocation::Push(i) => (get_native_fn(i), scrypto_encode(i)),
                AuthZoneStackInvocation::CreateProof(i) => (get_native_fn(i), scrypto_encode(i)),
                AuthZoneStackInvocation::CreateProofByAmount(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AuthZoneStackInvocation::CreateProofByIds(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AuthZoneStackInvocation::Clear(i) => (get_native_fn(i), scrypto_encode(i)),
                AuthZoneStackInvocation::Drain(i) => (get_native_fn(i), scrypto_encode(i)),
                AuthZoneStackInvocation::AssertAuthRule(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::ResourceManager(i) => match i {
                ResourceInvocation::BurnBucket(i) => (get_native_fn(i), scrypto_encode(i)),
                ResourceInvocation::GetResourceType(i) => (get_native_fn(i), scrypto_encode(i)),
                ResourceInvocation::Burn(i) => (get_native_fn(i), scrypto_encode(i)),
                ResourceInvocation::MintNonFungible(i) => (get_native_fn(i), scrypto_encode(i)),
                ResourceInvocation::MintUuidNonFungible(i) => (get_native_fn(i), scrypto_encode(i)),
                ResourceInvocation::MintFungible(i) => (get_native_fn(i), scrypto_encode(i)),
                ResourceInvocation::CreateBucket(i) => (get_native_fn(i), scrypto_encode(i)),
                ResourceInvocation::CreateVault(i) => (get_native_fn(i), scrypto_encode(i)),
                ResourceInvocation::UpdateVaultAuth(i) => (get_native_fn(i), scrypto_encode(i)),
                ResourceInvocation::SetVaultAuthMutability(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                ResourceInvocation::GetTotalSupply(i) => (get_native_fn(i), scrypto_encode(i)),
                ResourceInvocation::UpdateNonFungibleData(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                ResourceInvocation::GetNonFungible(i) => (get_native_fn(i), scrypto_encode(i)),
                ResourceInvocation::NonFungibleExists(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::Bucket(i) => match i {
                BucketInvocation::Take(i) => (get_native_fn(i), scrypto_encode(i)),
                BucketInvocation::TakeNonFungibles(i) => (get_native_fn(i), scrypto_encode(i)),
                BucketInvocation::Put(i) => (get_native_fn(i), scrypto_encode(i)),
                BucketInvocation::GetNonFungibleLocalIds(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                BucketInvocation::GetAmount(i) => (get_native_fn(i), scrypto_encode(i)),
                BucketInvocation::GetResourceAddress(i) => (get_native_fn(i), scrypto_encode(i)),
                BucketInvocation::CreateProof(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::Vault(i) => match i {
                VaultInvocation::Take(i) => (get_native_fn(i), scrypto_encode(i)),
                VaultInvocation::LockFee(i) => (get_native_fn(i), scrypto_encode(i)),
                VaultInvocation::Put(i) => (get_native_fn(i), scrypto_encode(i)),
                VaultInvocation::TakeNonFungibles(i) => (get_native_fn(i), scrypto_encode(i)),
                VaultInvocation::GetAmount(i) => (get_native_fn(i), scrypto_encode(i)),
                VaultInvocation::GetResourceAddress(i) => (get_native_fn(i), scrypto_encode(i)),
                VaultInvocation::GetNonFungibleLocalIds(i) => (get_native_fn(i), scrypto_encode(i)),
                VaultInvocation::CreateProof(i) => (get_native_fn(i), scrypto_encode(i)),
                VaultInvocation::CreateProofByAmount(i) => (get_native_fn(i), scrypto_encode(i)),
                VaultInvocation::CreateProofByIds(i) => (get_native_fn(i), scrypto_encode(i)),
                VaultInvocation::Recall(i) => (get_native_fn(i), scrypto_encode(i)),
                VaultInvocation::RecallNonFungibles(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::Proof(i) => match i {
                ProofInvocation::Clone(i) => (get_native_fn(i), scrypto_encode(i)),
                ProofInvocation::GetAmount(i) => (get_native_fn(i), scrypto_encode(i)),
                ProofInvocation::GetNonFungibleLocalIds(i) => (get_native_fn(i), scrypto_encode(i)),
                ProofInvocation::GetResourceAddress(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::Worktop(i) => match i {
                WorktopInvocation::TakeAll(i) => (get_native_fn(i), scrypto_encode(i)),
                WorktopInvocation::TakeAmount(i) => (get_native_fn(i), scrypto_encode(i)),
                WorktopInvocation::TakeNonFungibles(i) => (get_native_fn(i), scrypto_encode(i)),
                WorktopInvocation::Put(i) => (get_native_fn(i), scrypto_encode(i)),
                WorktopInvocation::AssertContains(i) => (get_native_fn(i), scrypto_encode(i)),
                WorktopInvocation::AssertContainsAmount(i) => (get_native_fn(i), scrypto_encode(i)),
                WorktopInvocation::AssertContainsNonFungibles(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                WorktopInvocation::Drain(i) => (get_native_fn(i), scrypto_encode(i)),
            },
            NativeInvocation::TransactionRuntime(i) => match i {
                TransactionRuntimeInvocation::GetHash(i) => (get_native_fn(i), scrypto_encode(i)),
                TransactionRuntimeInvocation::GenerateUuid(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
            },
            NativeInvocation::AccessController(i) => match i {
                AccessControllerInvocation::CancelPrimaryRoleRecoveryProposal(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessControllerInvocation::CancelRecoveryRoleRecoveryProposal(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessControllerInvocation::LockPrimaryRole(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessControllerInvocation::UnlockPrimaryRole(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
                AccessControllerInvocation::StopTimedRecovery(i) => {
                    (get_native_fn(i), scrypto_encode(i))
                }
            },
        };

        (
            native_fn,
            encoding.expect("Failed to encode native invocation"),
        )
    }
}
