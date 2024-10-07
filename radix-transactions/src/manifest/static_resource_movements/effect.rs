use super::*;
use radix_common::prelude::*;

use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::locker::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::transaction_tracker::*;
use radix_engine_interface::object_modules::metadata::*;
use radix_engine_interface::object_modules::role_assignment::*;
use radix_engine_interface::object_modules::royalty::*;

pub trait StaticInvocationResourcesOutput
where
    Self: ManifestEncode + ManifestDecode,
{
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError>;
}

pub struct InvocationDetails<'a> {
    pub receiver: InvocationReceiver,
    pub sent_resources: &'a TrackedResources,
    pub source: ChangeSource,
}

#[derive(Debug)]
pub enum InvocationReceiver {
    GlobalMethod(GlobalAddress),
    GlobalMethodOnReservedAddress,
    DirectAccess(InternalAddress),
    BlueprintFunction,
}

macro_rules! no_output_static_invocation_resources_output_impl {
    (
        $(
            $output_ident: ident
        ),* $(,)?
    ) => {
        $(
            impl StaticInvocationResourcesOutput for $output_ident {
                fn output(
                    &self,
                    _details: InvocationDetails
                ) -> Result<TrackedResources, StaticResourceMovementsError> {
                    Ok(TrackedResources::new_empty())
                }
            }
        )*
    };
}

macro_rules! unknown_output_static_invocation_resources_output_impl {
    (
        $(
            $output_ident: ident
        ),* $(,)?
    ) => {
        $(
            impl StaticInvocationResourcesOutput for $output_ident {
                fn output(
                    &self,
                    details: InvocationDetails
                ) -> Result<TrackedResources, StaticResourceMovementsError> {
                    Ok(TrackedResources::new_with_possible_balance_of_unspecified_resources([
                        details.source
                    ]))
                }
            }
        )*
    };
}

no_output_static_invocation_resources_output_impl![
    // AccessController
    AccessControllerCreateManifestInput,
    AccessControllerCreateProofManifestInput,
    AccessControllerInitiateRecoveryAsPrimaryManifestInput,
    AccessControllerInitiateRecoveryAsRecoveryManifestInput,
    AccessControllerQuickConfirmPrimaryRoleRecoveryProposalManifestInput,
    AccessControllerQuickConfirmRecoveryRoleRecoveryProposalManifestInput,
    AccessControllerTimedConfirmRecoveryManifestInput,
    AccessControllerCancelPrimaryRoleRecoveryProposalManifestInput,
    AccessControllerCancelRecoveryRoleRecoveryProposalManifestInput,
    AccessControllerLockPrimaryRoleManifestInput,
    AccessControllerUnlockPrimaryRoleManifestInput,
    AccessControllerStopTimedRecoveryManifestInput,
    AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryManifestInput,
    AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryManifestInput,
    AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptManifestInput,
    AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptManifestInput,
    AccessControllerLockRecoveryFeeManifestInput,
    AccessControllerContributeRecoveryFeeManifestInput,
    // Account
    AccountCreateAdvancedManifestInput,
    AccountLockFeeManifestInput,
    AccountLockContingentFeeManifestInput,
    AccountDepositManifestInput,
    AccountDepositBatchManifestInput,
    AccountBurnManifestInput,
    AccountBurnNonFungiblesManifestInput,
    AccountCreateProofOfAmountManifestInput,
    AccountCreateProofOfNonFungiblesManifestInput,
    AccountSetDefaultDepositRuleManifestInput,
    AccountSetResourcePreferenceManifestInput,
    AccountRemoveResourcePreferenceManifestInput,
    AccountTryDepositOrAbortManifestInput,
    AccountTryDepositBatchOrAbortManifestInput,
    AccountAddAuthorizedDepositorManifestInput,
    AccountRemoveAuthorizedDepositorManifestInput,
    // ConsensusManager
    ConsensusManagerCreateManifestInput,
    ConsensusManagerGetCurrentEpochManifestInput,
    ConsensusManagerStartManifestInput,
    ConsensusManagerGetCurrentTimeManifestInputV1,
    ConsensusManagerGetCurrentTimeManifestInputV2,
    ConsensusManagerCompareCurrentTimeManifestInputV1,
    ConsensusManagerCompareCurrentTimeManifestInputV2,
    ConsensusManagerNextRoundManifestInput,
    // Validator
    ValidatorRegisterManifestInput,
    ValidatorUnregisterManifestInput,
    ValidatorUpdateKeyManifestInput,
    ValidatorUpdateFeeManifestInput,
    ValidatorUpdateAcceptDelegatedStakeManifestInput,
    ValidatorAcceptsDelegatedStakeManifestInput,
    ValidatorTotalStakeXrdAmountManifestInput,
    ValidatorTotalStakeUnitSupplyManifestInput,
    ValidatorGetRedemptionValueManifestInput,
    ValidatorSignalProtocolUpdateReadinessManifestInput,
    ValidatorGetProtocolUpdateReadinessManifestInput,
    ValidatorLockOwnerStakeUnitsManifestInput,
    ValidatorStartUnlockOwnerStakeUnitsManifestInput,
    ValidatorApplyEmissionManifestInput,
    ValidatorApplyRewardManifestInput,
    // Identity
    IdentityCreateAdvancedManifestInput,
    // AccountLocker
    AccountLockerInstantiateManifestInput,
    AccountLockerStoreManifestInput,
    AccountLockerGetAmountManifestInput,
    AccountLockerGetNonFungibleLocalIdsManifestInput,
    // Package
    PackagePublishWasmAdvancedManifestInput,
    PackagePublishNativeManifestInput,
    // OneResourcePool
    OneResourcePoolInstantiateManifestInput,
    OneResourcePoolProtectedDepositManifestInput,
    OneResourcePoolGetRedemptionValueManifestInput,
    OneResourcePoolGetVaultAmountManifestInput,
    // TwoResourcePool
    TwoResourcePoolInstantiateManifestInput,
    TwoResourcePoolProtectedDepositManifestInput,
    TwoResourcePoolGetRedemptionValueManifestInput,
    TwoResourcePoolGetVaultAmountsManifestInput,
    // MultiResourcePool
    MultiResourcePoolInstantiateManifestInput,
    MultiResourcePoolProtectedDepositManifestInput,
    MultiResourcePoolGetRedemptionValueManifestInput,
    MultiResourcePoolGetVaultAmountsManifestInput,
    // ResourceManager
    ResourceManagerBurnManifestInput,
    ResourceManagerPackageBurnManifestInput,
    ResourceManagerGetTotalSupplyManifestInput,
    ResourceManagerGetResourceTypeManifestInput,
    ResourceManagerCreateEmptyVaultManifestInput,
    ResourceManagerGetAmountForWithdrawalManifestInput,
    ResourceManagerDropEmptyBucketManifestInput,
    // FungibleResourceManager
    FungibleResourceManagerCreateManifestInput,
    // NonFungibleResourceManager
    NonFungibleResourceManagerCreateManifestInput,
    NonFungibleResourceManagerGetNonFungibleManifestInput,
    NonFungibleResourceManagerUpdateDataManifestInput,
    NonFungibleResourceManagerExistsManifestInput,
    // Vault
    VaultGetAmountManifestInput,
    VaultFreezeManifestInput,
    VaultUnfreezeManifestInput,
    VaultBurnManifestInput,
    VaultPutManifestInput,
    // FungibleVault
    FungibleVaultLockFeeManifestInput,
    FungibleVaultCreateProofOfAmountManifestInput,
    FungibleVaultLockFungibleAmountManifestInput,
    FungibleVaultUnlockFungibleAmountManifestInput,
    // NonFungibleVault
    NonFungibleVaultGetNonFungibleLocalIdsManifestInput,
    NonFungibleVaultContainsNonFungibleManifestInput,
    NonFungibleVaultCreateProofOfNonFungiblesManifestInput,
    NonFungibleVaultLockNonFungiblesManifestInput,
    NonFungibleVaultUnlockNonFungiblesManifestInput,
    NonFungibleVaultBurnNonFungiblesManifestInput,
    // TransactionTracker
    TransactionTrackerCreateManifestInput,
    // Metadata
    MetadataCreateManifestInput,
    MetadataCreateWithDataManifestInput,
    MetadataSetManifestInput,
    MetadataLockManifestInput,
    MetadataGetManifestInput,
    MetadataRemoveManifestInput,
    // RoleAssignment
    RoleAssignmentCreateManifestInput,
    RoleAssignmentSetOwnerManifestInput,
    RoleAssignmentLockOwnerManifestInput,
    RoleAssignmentSetManifestInput,
    RoleAssignmentGetManifestInput,
    // ComponentRoyalty
    ComponentRoyaltyCreateManifestInput,
    ComponentRoyaltySetManifestInput,
    ComponentRoyaltyLockManifestInput,
];

unknown_output_static_invocation_resources_output_impl![
    /* AccessController */
    // The withdrawn badge is of an unknown resource
    AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptManifestInput,
    // The withdrawn badge is of an unknown resource
    AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptManifestInput,
    // The minted badge is of a new / unknown resource
    AccessControllerMintRecoveryBadgesManifestInput,
    // The validator stake unit resource is unknown at static validation time
    /* Validator */
    ValidatorStakeAsOwnerManifestInput,
    // The validator stake unit resource is unknown at static validation time
    ValidatorStakeManifestInput,
    // The validator unstake receipt is unknown at static validation time
    ValidatorUnstakeManifestInput,
    // This can return validator stake units which are an unknown resource at static validation time
    ValidatorFinishUnlockOwnerStakeUnitsManifestInput,
    // This generates and returns a new badge resource, which is unknowable at static time
    /* AccountLocker */
    AccountLockerInstantiateSimpleManifestInput,
    /* OneResourcePool */
    // This returns pool units of an unknown resource address and an unknown amount.
    OneResourcePoolContributeManifestInput,
    // This returns unknown resources of an unknown amount from the redemption.
    OneResourcePoolRedeemManifestInput,
    // This returns an unknown resource but a known amount which we can't do much with.
    OneResourcePoolProtectedWithdrawManifestInput,
    /* TwoResourcePool */
    // This returns pool units of an unknown resource address and an unknown amount.
    TwoResourcePoolContributeManifestInput,
    // This returns unknown resources of an unknown amount from the redemption.
    TwoResourcePoolRedeemManifestInput,
    /* MultiResourcePool */
    // This returns pool units of an unknown resource address and an unknown amount.
    MultiResourcePoolContributeManifestInput,
    // This returns unknown resources of an unknown amount from the redemption.
    MultiResourcePoolRedeemManifestInput,
    /* FungibleResourceManager */
    // This returns this resource so we know the amount but we don't know the resource address
    // so we can't do much with that.
    FungibleResourceManagerCreateWithInitialSupplyManifestInput,
    /* NonFungibleResourceManager */
    // This returns this resource so we know the ids but we don't know the resource address
    // so we can't do much with that.
    NonFungibleResourceManagerCreateWithInitialSupplyManifestInput,
    // This returns this resource so we know the ids but we don't know the resource address
    // so we can't do much with that.
    NonFungibleResourceManagerCreateRuidWithInitialSupplyManifestInput,
    /* Vault */
    // We don't know what resource is in the vault. We know the amount/ids returned but not the
    // resource address.
    VaultTakeManifestInput,
    // We don't know what resource is in the vault. We know the amount/ids returned but not the
    // resource address.
    VaultTakeAdvancedManifestInput,
    // We don't know what resource is in the vault. We know the amount/ids returned but not the
    // resource address.
    VaultRecallManifestInput,
    /* NonFungibleVault */
    // We don't know what resource is in the vault. We know the amount/ids returned but not the
    // resource address.
    NonFungibleVaultTakeNonFungiblesManifestInput,
    // We don't know what resource is in the vault. We know the amount/ids returned but not the
    // resource address.
    NonFungibleVaultRecallNonFungiblesManifestInput,
];

// region:Typed Invocation
impl StaticInvocationResourcesOutput for TypedNativeInvocation {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        match self {
            TypedNativeInvocation::AccessControllerPackage(invocation) => match invocation {
                AccessControllerInvocations::AccessControllerBlueprint(invocation) => match invocation {
                    AccessControllerBlueprintInvocations::Function(invocation) => match invocation {
                        AccessControllerFunction::Create(input) => input.output(details),
                    },
                    AccessControllerBlueprintInvocations::Method(invocation) => match invocation {
                        AccessControllerMethod::CreateProof(input) => input.output(details),
                        AccessControllerMethod::InitiateRecoveryAsPrimary(input) => input.output(details),
                        AccessControllerMethod::InitiateRecoveryAsRecovery(input) => input.output(details),
                        AccessControllerMethod::QuickConfirmPrimaryRoleRecoveryProposal(input) => input.output(details),
                        AccessControllerMethod::QuickConfirmRecoveryRoleRecoveryProposal(input) => {
                            input.output(details)
                        }
                        AccessControllerMethod::TimedConfirmRecovery(input) => input.output(details),
                        AccessControllerMethod::CancelPrimaryRoleRecoveryProposal(input) => input.output(details),
                        AccessControllerMethod::CancelRecoveryRoleRecoveryProposal(input) => input.output(details),
                        AccessControllerMethod::LockPrimaryRole(input) => input.output(details),
                        AccessControllerMethod::UnlockPrimaryRole(input) => input.output(details),
                        AccessControllerMethod::StopTimedRecovery(input) => input.output(details),
                        AccessControllerMethod::InitiateBadgeWithdrawAttemptAsPrimary(input) => input.output(details),
                        AccessControllerMethod::InitiateBadgeWithdrawAttemptAsRecovery(input) => input.output(details),
                        AccessControllerMethod::QuickConfirmPrimaryRoleBadgeWithdrawAttempt(input) => {
                            input.output(details)
                        }
                        AccessControllerMethod::QuickConfirmRecoveryRoleBadgeWithdrawAttempt(input) => {
                            input.output(details)
                        }
                        AccessControllerMethod::CancelPrimaryRoleBadgeWithdrawAttempt(input) => input.output(details),
                        AccessControllerMethod::CancelRecoveryRoleBadgeWithdrawAttempt(input) => input.output(details),
                        AccessControllerMethod::MintRecoveryBadges(input) => input.output(details),
                        AccessControllerMethod::LockRecoveryFee(input) => input.output(details),
                        AccessControllerMethod::WithdrawRecoveryFee(input) => input.output(details),
                        AccessControllerMethod::ContributeRecoveryFee(input) => input.output(details),
                    },
                },
            },
            TypedNativeInvocation::AccountPackage(invocation) => match invocation {
                AccountInvocations::AccountBlueprint(invocation) => match invocation {
                    AccountBlueprintInvocations::Function(invocation) => match invocation {
                        AccountFunction::CreateAdvanced(input) => input.output(details),
                        AccountFunction::Create(input) => input.output(details),
                    },
                    AccountBlueprintInvocations::Method(invocation) => match invocation {
                        AccountMethod::Securify(input) => input.output(details),
                        AccountMethod::LockFee(input) => input.output(details),
                        AccountMethod::LockContingentFee(input) => input.output(details),
                        AccountMethod::Deposit(input) => input.output(details),
                        AccountMethod::DepositBatch(input) => input.output(details),
                        AccountMethod::Withdraw(input) => input.output(details),
                        AccountMethod::WithdrawNonFungibles(input) => input.output(details),
                        AccountMethod::Burn(input) => input.output(details),
                        AccountMethod::BurnNonFungibles(input) => input.output(details),
                        AccountMethod::LockFeeAndWithdraw(input) => input.output(details),
                        AccountMethod::LockFeeAndWithdrawNonFungibles(input) => input.output(details),
                        AccountMethod::CreateProofOfAmount(input) => input.output(details),
                        AccountMethod::CreateProofOfNonFungibles(input) => input.output(details),
                        AccountMethod::SetDefaultDepositRule(input) => input.output(details),
                        AccountMethod::SetResourcePreference(input) => input.output(details),
                        AccountMethod::RemoveResourcePreference(input) => input.output(details),
                        AccountMethod::TryDepositOrRefund(input) => input.output(details),
                        AccountMethod::TryDepositBatchOrRefund(input) => input.output(details),
                        AccountMethod::TryDepositOrAbort(input) => input.output(details),
                        AccountMethod::TryDepositBatchOrAbort(input) => input.output(details),
                        AccountMethod::AddAuthorizedDepositor(input) => input.output(details),
                        AccountMethod::RemoveAuthorizedDepositor(input) => input.output(details),
                    },
                },
            },
            TypedNativeInvocation::ConsensusManagerPackage(invocation) => match invocation {
                ConsensusManagerInvocations::ConsensusManagerBlueprint(invocation) => match invocation {
                    ConsensusManagerBlueprintInvocations::Function(invocation) => match invocation {
                        ConsensusManagerFunction::Create(input) => input.output(details),
                    },
                    ConsensusManagerBlueprintInvocations::Method(invocation) => match invocation {
                        ConsensusManagerMethod::GetCurrentEpoch(input) => input.output(details),
                        ConsensusManagerMethod::Start(input) => input.output(details),
                        ConsensusManagerMethod::GetCurrentTime(input) => input.output(details),
                        ConsensusManagerMethod::CompareCurrentTime(input) => input.output(details),
                        ConsensusManagerMethod::NextRound(input) => input.output(details),
                        ConsensusManagerMethod::CreateValidator(input) => input.output(details),
                    },
                },
                ConsensusManagerInvocations::ValidatorBlueprint(invocation) => match invocation {
                    ValidatorBlueprintInvocations::Function(invocation) => match *invocation {},
                    ValidatorBlueprintInvocations::Method(invocation) => match invocation {
                        ValidatorMethod::Register(input) => input.output(details),
                        ValidatorMethod::Unregister(input) => input.output(details),
                        ValidatorMethod::StakeAsOwner(input) => input.output(details),
                        ValidatorMethod::Stake(input) => input.output(details),
                        ValidatorMethod::Unstake(input) => input.output(details),
                        ValidatorMethod::ClaimXrd(input) => input.output(details),
                        ValidatorMethod::UpdateKey(input) => input.output(details),
                        ValidatorMethod::UpdateFee(input) => input.output(details),
                        ValidatorMethod::UpdateAcceptDelegatedStake(input) => input.output(details),
                        ValidatorMethod::AcceptsDelegatedStake(input) => input.output(details),
                        ValidatorMethod::TotalStakeXrdAmount(input) => input.output(details),
                        ValidatorMethod::TotalStakeUnitSupply(input) => input.output(details),
                        ValidatorMethod::GetRedemptionValue(input) => input.output(details),
                        ValidatorMethod::SignalProtocolUpdateReadiness(input) => input.output(details),
                        ValidatorMethod::GetProtocolUpdateReadiness(input) => input.output(details),
                        ValidatorMethod::LockOwnerStakeUnits(input) => input.output(details),
                        ValidatorMethod::StartUnlockOwnerStakeUnits(input) => input.output(details),
                        ValidatorMethod::FinishUnlockOwnerStakeUnits(input) => input.output(details),
                        ValidatorMethod::ApplyEmission(input) => input.output(details),
                        ValidatorMethod::ApplyReward(input) => input.output(details),
                    },
                },
            },
            TypedNativeInvocation::IdentityPackage(invocation) => match invocation {
                IdentityInvocations::IdentityBlueprint(invocation) => match invocation {
                    IdentityBlueprintInvocations::Function(invocation) => match invocation {
                        IdentityFunction::CreateAdvanced(input) => input.output(details),
                        IdentityFunction::Create(input) => input.output(details),
                    },
                    IdentityBlueprintInvocations::Method(invocation) => match invocation {
                        IdentityMethod::Securify(input) => input.output(details),
                    },
                },
            },
            TypedNativeInvocation::LockerPackage(invocation) => match invocation {
                LockerInvocations::AccountLockerBlueprint(invocation) => match invocation {
                    AccountLockerBlueprintInvocations::Function(invocation) => match invocation {
                        AccountLockerFunction::Instantiate(input) => input.output(details),
                        AccountLockerFunction::InstantiateSimple(input) => input.output(details),
                    },
                    AccountLockerBlueprintInvocations::Method(invocation) => match invocation {
                        AccountLockerMethod::Store(input) => input.output(details),
                        AccountLockerMethod::Airdrop(input) => input.output(details),
                        AccountLockerMethod::Recover(input) => input.output(details),
                        AccountLockerMethod::RecoverNonFungibles(input) => input.output(details),
                        AccountLockerMethod::Claim(input) => input.output(details),
                        AccountLockerMethod::ClaimNonFungibles(input) => input.output(details),
                        AccountLockerMethod::GetAmount(input) => input.output(details),
                        AccountLockerMethod::GetNonFungibleLocalIds(input) => input.output(details),
                    },
                },
            },
            TypedNativeInvocation::PackagePackage(invocation) => match invocation {
                PackageInvocations::PackageBlueprint(invocation) => match invocation {
                    PackageBlueprintInvocations::Function(invocation) => match invocation {
                        PackageFunction::PublishWasm(input) => input.output(details),
                        PackageFunction::PublishWasmAdvanced(input) => input.output(details),
                        PackageFunction::PublishNative(input) => input.output(details),
                    },
                    PackageBlueprintInvocations::Method(invocation) => match invocation {
                        PackageMethod::PackageRoyaltyClaimRoyalties(input) => input.output(details),
                    },
                },
            },
            TypedNativeInvocation::PoolPackage(invocation) => match invocation {
                PoolInvocations::OneResourcePoolBlueprint(invocation) => match invocation {
                    OneResourcePoolBlueprintInvocations::Function(invocation) => match invocation {
                        OneResourcePoolFunction::Instantiate(input) => input.output(details),
                    },
                    OneResourcePoolBlueprintInvocations::Method(invocation) => match invocation {
                        OneResourcePoolMethod::Contribute(input) => input.output(details),
                        OneResourcePoolMethod::Redeem(input) => input.output(details),
                        OneResourcePoolMethod::ProtectedDeposit(input) => input.output(details),
                        OneResourcePoolMethod::ProtectedWithdraw(input) => input.output(details),
                        OneResourcePoolMethod::GetRedemptionValue(input) => input.output(details),
                        OneResourcePoolMethod::GetVaultAmount(input) => input.output(details),
                    },
                },
                PoolInvocations::TwoResourcePoolBlueprint(invocation) => match invocation {
                    TwoResourcePoolBlueprintInvocations::Function(invocation) => match invocation {
                        TwoResourcePoolFunction::Instantiate(input) => input.output(details),
                    },
                    TwoResourcePoolBlueprintInvocations::Method(invocation) => match invocation {
                        TwoResourcePoolMethod::Contribute(input) => input.output(details),
                        TwoResourcePoolMethod::Redeem(input) => input.output(details),
                        TwoResourcePoolMethod::ProtectedDeposit(input) => input.output(details),
                        TwoResourcePoolMethod::ProtectedWithdraw(input) => input.output(details),
                        TwoResourcePoolMethod::GetRedemptionValue(input) => input.output(details),
                        TwoResourcePoolMethod::GetVaultAmounts(input) => input.output(details),
                    },
                },
                PoolInvocations::MultiResourcePoolBlueprint(invocation) => match invocation {
                    MultiResourcePoolBlueprintInvocations::Function(invocation) => match invocation {
                        MultiResourcePoolFunction::Instantiate(input) => input.output(details),
                    },
                    MultiResourcePoolBlueprintInvocations::Method(invocation) => match invocation {
                        MultiResourcePoolMethod::Contribute(input) => input.output(details),
                        MultiResourcePoolMethod::Redeem(input) => input.output(details),
                        MultiResourcePoolMethod::ProtectedDeposit(input) => input.output(details),
                        MultiResourcePoolMethod::ProtectedWithdraw(input) => input.output(details),
                        MultiResourcePoolMethod::GetRedemptionValue(input) => input.output(details),
                        MultiResourcePoolMethod::GetVaultAmounts(input) => input.output(details),
                    },
                },
            },
            TypedNativeInvocation::ResourcePackage(invocation) => match invocation {
                ResourceInvocations::FungibleResourceManagerBlueprint(invocation) => match invocation {
                    FungibleResourceManagerBlueprintInvocations::Function(invocation) => match invocation {
                        FungibleResourceManagerFunction::Create(input) => input.output(details),
                        FungibleResourceManagerFunction::CreateWithInitialSupply(input) => input.output(details),
                    },
                    FungibleResourceManagerBlueprintInvocations::Method(invocation) => match invocation {
                        FungibleResourceManagerMethod::Mint(input) => input.output(details),
                        FungibleResourceManagerMethod::Burn(input) => input.output(details),
                        FungibleResourceManagerMethod::PackageBurn(input) => input.output(details),
                        FungibleResourceManagerMethod::CreateEmptyVault(input) => input.output(details),
                        FungibleResourceManagerMethod::CreateEmptyBucket(input) => input.output(details),
                        FungibleResourceManagerMethod::GetResourceType(input) => input.output(details),
                        FungibleResourceManagerMethod::GetTotalSupply(input) => input.output(details),
                        FungibleResourceManagerMethod::AmountForWithdrawal(input) => input.output(details),
                        FungibleResourceManagerMethod::DropEmptyBucket(input) => input.output(details),
                    },
                },
                ResourceInvocations::NonFungibleResourceManagerBlueprint(invocation) => match invocation
                {
                    NonFungibleResourceManagerBlueprintInvocations::Function(invocation) => match invocation {
                        NonFungibleResourceManagerFunction::Create(input) => input.output(details),
                        NonFungibleResourceManagerFunction::CreateWithInitialSupply(input) => input.output(details),
                        NonFungibleResourceManagerFunction::CreateRuidNonFungibleWithInitialSupply(input) => {
                            input.output(details)
                        }
                    },
                    NonFungibleResourceManagerBlueprintInvocations::Method(invocation) => match invocation {
                        NonFungibleResourceManagerMethod::Mint(input) => input.output(details),
                        NonFungibleResourceManagerMethod::GetNonFungible(input) => input.output(details),
                        NonFungibleResourceManagerMethod::UpdateNonFungibleData(input) => input.output(details),
                        NonFungibleResourceManagerMethod::NonFungibleExists(input) => input.output(details),
                        NonFungibleResourceManagerMethod::MintRuid(input) => input.output(details),
                        NonFungibleResourceManagerMethod::MintSingleRuid(input) => input.output(details),
                        NonFungibleResourceManagerMethod::PackageBurn(input) => input.output(details),
                        NonFungibleResourceManagerMethod::Burn(input) => input.output(details),
                        NonFungibleResourceManagerMethod::CreateEmptyVault(input) => input.output(details),
                        NonFungibleResourceManagerMethod::CreateEmptyBucket(input) => input.output(details),
                        NonFungibleResourceManagerMethod::GetResourceType(input) => input.output(details),
                        NonFungibleResourceManagerMethod::GetTotalSupply(input) => input.output(details),
                        NonFungibleResourceManagerMethod::AmountForWithdrawal(input) => input.output(details),
                        NonFungibleResourceManagerMethod::DropEmptyBucket(input) => input.output(details),
                    },
                },
                ResourceInvocations::FungibleVaultBlueprint(invocation) => match invocation {
                    FungibleVaultBlueprintInvocations::Function(invocation) => match *invocation {},
                    FungibleVaultBlueprintInvocations::Method(invocation) => match invocation {
                        FungibleVaultMethod::Take(input) => input.output(details),
                        FungibleVaultMethod::TakeAdvanced(input) => input.output(details),
                        FungibleVaultMethod::Put(input) => input.output(details),
                        FungibleVaultMethod::GetAmount(input) => input.output(details),
                        FungibleVaultMethod::LockFee(input) => input.output(details),
                        FungibleVaultMethod::Recall(input) => input.output(details),
                        FungibleVaultMethod::Freeze(input) => input.output(details),
                        FungibleVaultMethod::Unfreeze(input) => input.output(details),
                        FungibleVaultMethod::CreateProofOfAmount(input) => input.output(details),
                        FungibleVaultMethod::LockAmount(input) => input.output(details),
                        FungibleVaultMethod::UnlockAmount(input) => input.output(details),
                        FungibleVaultMethod::Burn(input) => input.output(details),
                    },
                },
                ResourceInvocations::NonFungibleVaultBlueprint(invocation) => match invocation {
                    NonFungibleVaultBlueprintInvocations::Function(invocation) => match *invocation {},
                    NonFungibleVaultBlueprintInvocations::Method(invocation) => match invocation {
                        NonFungibleVaultMethod::Take(input) => input.output(details),
                        NonFungibleVaultMethod::TakeAdvanced(input) => input.output(details),
                        NonFungibleVaultMethod::TakeNonFungibles(input) => input.output(details),
                        NonFungibleVaultMethod::Recall(input) => input.output(details),
                        NonFungibleVaultMethod::Freeze(input) => input.output(details),
                        NonFungibleVaultMethod::Unfreeze(input) => input.output(details),
                        NonFungibleVaultMethod::RecallNonFungibles(input) => input.output(details),
                        NonFungibleVaultMethod::Put(input) => input.output(details),
                        NonFungibleVaultMethod::GetAmount(input) => input.output(details),
                        NonFungibleVaultMethod::GetNonFungibleLocalIds(input) => input.output(details),
                        NonFungibleVaultMethod::ContainsNonFungible(input) => input.output(details),
                        NonFungibleVaultMethod::CreateProofOfNonFungibles(input) => input.output(details),
                        NonFungibleVaultMethod::LockNonFungibles(input) => input.output(details),
                        NonFungibleVaultMethod::UnlockNonFungibles(input) => input.output(details),
                        NonFungibleVaultMethod::Burn(input) => input.output(details),
                        NonFungibleVaultMethod::BurnNonFungibles(input) => input.output(details),
                    },
                },
            },
            TypedNativeInvocation::TransactionTrackerPackage(invocation) => match invocation {
                TransactionTrackerInvocations::TransactionTrackerBlueprint(invocation) => match invocation {
                    TransactionTrackerBlueprintInvocations::Function(invocation) => match invocation {
                        TransactionTrackerFunction::Create(input) => input.output(details),
                    },
                    TransactionTrackerBlueprintInvocations::Method(invocation) => match *invocation {},
                },
            },
            TypedNativeInvocation::MetadataPackage(invocation) => match invocation {
                MetadataInvocations::MetadataBlueprint(invocation) => match invocation {
                    MetadataBlueprintInvocations::Function(invocation) => match invocation {
                        MetadataFunction::Create(input) => input.output(details),
                        MetadataFunction::CreateWithData(input) => input.output(details),
                    },
                    MetadataBlueprintInvocations::Method(invocation) => match invocation {
                        MetadataMethod::Set(input) => input.output(details),
                        MetadataMethod::Lock(input) => input.output(details),
                        MetadataMethod::Get(input) => input.output(details),
                        MetadataMethod::Remove(input) => input.output(details),
                    },
                },
            },
            TypedNativeInvocation::RoleAssignmentPackage(invocation) => match invocation {
                RoleAssignmentInvocations::RoleAssignmentBlueprint(invocation) => match invocation {
                    RoleAssignmentBlueprintInvocations::Function(invocation) => match invocation {
                        RoleAssignmentFunction::Create(input) => input.output(details),
                    },
                    RoleAssignmentBlueprintInvocations::Method(invocation) => match invocation {
                        RoleAssignmentMethod::SetOwner(input) => input.output(details),
                        RoleAssignmentMethod::LockOwner(input) => input.output(details),
                        RoleAssignmentMethod::Set(input) => input.output(details),
                        RoleAssignmentMethod::Get(input) => input.output(details),
                    },
                },
            },
            TypedNativeInvocation::ComponentRoyaltyPackage(invocation) => match invocation {
                ComponentRoyaltyInvocations::ComponentRoyaltyBlueprint(invocation) => match invocation {
                    ComponentRoyaltyBlueprintInvocations::Function(invocation) => match invocation {
                        ComponentRoyaltyFunction::Create(input) => input.output(details),
                    },
                    ComponentRoyaltyBlueprintInvocations::Method(invocation) => match invocation {
                        ComponentRoyaltyMethod::SetRoyalty(input) => input.output(details),
                        ComponentRoyaltyMethod::LockRoyalty(input) => input.output(details),
                        ComponentRoyaltyMethod::ClaimRoyalties(input) => input.output(details),
                    },
                },
            },
        }
    }
}
// endregion:Typed Invocation

// region:AccessController
impl StaticInvocationResourcesOutput for AccessControllerWithdrawRecoveryFeeManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            XRD,
            TrackedResource::exact_amount(self.amount, [details.source])?,
        )
    }
}
// endregion:AccessController

// region:Account
impl StaticInvocationResourcesOutput for AccountCreateManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            ACCOUNT_OWNER_BADGE,
            TrackedResource::exact_amount(1, [details.source])?,
        )
    }
}

impl StaticInvocationResourcesOutput for AccountSecurifyManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        match details.receiver {
            InvocationReceiver::GlobalMethod(global_address) => {
                let local_id = NonFungibleLocalId::bytes(global_address.as_bytes()).unwrap();
                TrackedResources::new_empty().add_resource(
                    ACCOUNT_OWNER_BADGE,
                    TrackedResource::exact_non_fungibles([local_id], [details.source]),
                )
            }
            InvocationReceiver::GlobalMethodOnReservedAddress => TrackedResources::new_empty()
                .add_resource(
                    ACCOUNT_OWNER_BADGE,
                    TrackedResource::exact_amount(1, [details.source])?,
                ),
            InvocationReceiver::DirectAccess(_) | InvocationReceiver::BlueprintFunction => {
                unreachable!()
            }
        }
    }
}

impl StaticInvocationResourcesOutput for AccountWithdrawManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            self.resource_address,
            TrackedResource::exact_amount(self.amount, [details.source])?,
        )
    }
}

impl StaticInvocationResourcesOutput for AccountWithdrawNonFungiblesManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            self.resource_address,
            TrackedResource::exact_non_fungibles(self.ids.clone(), [details.source]),
        )
    }
}

impl StaticInvocationResourcesOutput for AccountLockFeeAndWithdrawManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            self.resource_address,
            TrackedResource::exact_amount(self.amount, [details.source])?,
        )
    }
}

impl StaticInvocationResourcesOutput for AccountLockFeeAndWithdrawNonFungiblesManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            self.resource_address,
            TrackedResource::exact_non_fungibles(self.ids.clone(), [details.source]),
        )
    }
}

impl StaticInvocationResourcesOutput for AccountTryDepositOrRefundManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        handle_possible_refund(details)
    }
}

impl StaticInvocationResourcesOutput for AccountTryDepositBatchOrRefundManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        handle_possible_refund(details)
    }
}

fn handle_possible_refund(
    details: InvocationDetails,
) -> Result<TrackedResources, StaticResourceMovementsError> {
    let mut sent_resources = details.sent_resources.clone();
    let mut refunded_resources = TrackedResources::new_empty();

    // Handle the specified resources. First dump the resource keys to work around the borrow checker...
    let known_resources = sent_resources
        .specified_resources()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    for known_resource in known_resources {
        let attempted_deposit = sent_resources.take_resource(
            known_resource,
            ResourceTakeAmount::All,
            details.source,
        )?;
        let (bounds, _history) = attempted_deposit.deconstruct();
        // Either nothing or everything is returned, but we can't currently model a fork in
        // the timeline, so instead we handle it as a return of some amount between 0 and the sent amount.
        let refunded_amount = bounds.replace_lower_bounds_with_zero();
        refunded_resources.mut_add_resource(
            known_resource,
            TrackedResource::general(refunded_amount, [details.source]),
        )?;
    }
    // Handle the possible refund of the remaining unspecified resources
    if sent_resources.unspecified_resources().may_be_present() {
        refunded_resources.mut_add_unspecified_resources([details.source]);
    }

    Ok(refunded_resources)
}
// endregion:Account

// region:ConsensusManager
impl StaticInvocationResourcesOutput for ConsensusManagerCreateValidatorManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            VALIDATOR_OWNER_BADGE,
            TrackedResource::exact_amount(1, [details.source])?,
        )
    }
}
// endregion:ConsensusManager

// region:Validator
impl StaticInvocationResourcesOutput for ValidatorClaimXrdManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty()
            .add_resource(XRD, TrackedResource::zero_or_more([details.source]))
    }
}

// endregion:Validator

// region:Identity
impl StaticInvocationResourcesOutput for IdentityCreateManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            IDENTITY_OWNER_BADGE,
            TrackedResource::exact_amount(1, [details.source])?,
        )
    }
}

impl StaticInvocationResourcesOutput for IdentitySecurifyToSingleBadgeManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        Ok(match details.receiver {
            InvocationReceiver::GlobalMethod(global_address) => {
                let local_id = NonFungibleLocalId::bytes(global_address.as_bytes()).unwrap();
                TrackedResources::new_empty().add_resource(
                    IDENTITY_OWNER_BADGE,
                    TrackedResource::exact_non_fungibles([local_id], [details.source]),
                )?
            }
            InvocationReceiver::GlobalMethodOnReservedAddress => TrackedResources::new_empty()
                .add_resource(
                    IDENTITY_OWNER_BADGE,
                    TrackedResource::exact_amount(1, [details.source])?,
                )?,
            InvocationReceiver::DirectAccess(_) | InvocationReceiver::BlueprintFunction => {
                unreachable!()
            }
        })
    }
}
// endregion:Identity

// region:AccountLocker
impl StaticInvocationResourcesOutput for AccountLockerAirdropManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // This behaves roughly like a possible refund...
        // We could be even more exact... We can subtract the claimants from the bucket to calculate what gets returned.
        // But this is good enough for now.
        handle_possible_refund(details)
    }
}

impl StaticInvocationResourcesOutput for AccountLockerRecoverManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            self.resource_address,
            TrackedResource::exact_amount(self.amount, [details.source])?,
        )
    }
}

impl StaticInvocationResourcesOutput for AccountLockerRecoverNonFungiblesManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            self.resource_address,
            TrackedResource::exact_non_fungibles(self.ids.clone(), [details.source]),
        )
    }
}

impl StaticInvocationResourcesOutput for AccountLockerClaimManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            self.resource_address,
            TrackedResource::exact_amount(self.amount, [details.source])?,
        )
    }
}

impl StaticInvocationResourcesOutput for AccountLockerClaimNonFungiblesManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            self.resource_address,
            TrackedResource::exact_non_fungibles(self.ids.clone(), [details.source]),
        )
    }
}
// endregion:AccountLocker

// region:Package
impl StaticInvocationResourcesOutput for PackagePublishWasmManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            PACKAGE_OWNER_BADGE,
            TrackedResource::exact_amount(1, [details.source])?,
        )
    }
}

impl StaticInvocationResourcesOutput for PackageClaimRoyaltiesManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty()
            .add_resource(XRD, TrackedResource::zero_or_more([details.source]))
    }
}
// endregion:Package

// region:TwoResourcePool
impl StaticInvocationResourcesOutput for TwoResourcePoolProtectedWithdrawManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            self.resource_address,
            TrackedResource::exact_amount(self.amount, [details.source])?,
        )
    }
}
// endregion:TwoResourcePool

// region:MultiResourcePool
impl StaticInvocationResourcesOutput for MultiResourcePoolProtectedWithdrawManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            self.resource_address,
            TrackedResource::exact_amount(self.amount, [details.source])?,
        )
    }
}
// endregion:MultiResourcePool

// region:FungibleResourceManager
impl StaticInvocationResourcesOutput for FungibleResourceManagerMintManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // If the receiver is a global address then we can return something useful. Otherwise it
        // is a known amount and an unknown resource address.
        match details.receiver {
            InvocationReceiver::GlobalMethod(global_address) => {
                // Attempt to convert the global address to a resource address. Error if that fails.
                if let Ok(resource_address) = ResourceAddress::try_from(global_address) {
                    TrackedResources::new_empty().add_resource(
                        resource_address,
                        TrackedResource::exact_amount(self.amount, [details.source])?,
                    )
                } else {
                    Err(StaticResourceMovementsError::NotAResourceAddress(
                        global_address,
                    ))
                }
            }
            InvocationReceiver::GlobalMethodOnReservedAddress
            | InvocationReceiver::DirectAccess(_)
            | InvocationReceiver::BlueprintFunction => Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            ),
        }
    }
}

impl StaticInvocationResourcesOutput for FungibleResourceManagerCreateEmptyBucketManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // If the receiver is a global address then we can return something useful. Otherwise it
        // is a known amount and an unknown resource address.
        match details.receiver {
            InvocationReceiver::GlobalMethod(global_address) => {
                // Attempt to convert the global address to a resource address. Error if that fails.
                if let Ok(resource_address) = ResourceAddress::try_from(global_address) {
                    TrackedResources::new_empty().add_resource(
                        resource_address,
                        TrackedResource::exact_amount(Decimal::ZERO, [details.source])?,
                    )
                } else {
                    Err(StaticResourceMovementsError::NotAResourceAddress(
                        global_address,
                    ))
                }
            }
            InvocationReceiver::GlobalMethodOnReservedAddress
            | InvocationReceiver::DirectAccess(_)
            | InvocationReceiver::BlueprintFunction => Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            ),
        }
    }
}
// endregion:FungibleResourceManager

// region:NonFungibleResourceManager
impl StaticInvocationResourcesOutput for NonFungibleResourceManagerMintManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // If the receiver is a global address then we can return something useful. Otherwise it
        // is a known ids and an unknown resource address.
        match details.receiver {
            InvocationReceiver::GlobalMethod(global_address) => {
                // Attempt to convert the global address to a resource address. Error if that fails.
                if let Ok(resource_address) = ResourceAddress::try_from(global_address) {
                    TrackedResources::new_empty().add_resource(
                        resource_address,
                        TrackedResource::exact_non_fungibles(
                            self.entries.keys().cloned(),
                            [details.source],
                        ),
                    )
                } else {
                    Err(StaticResourceMovementsError::NotAResourceAddress(
                        global_address,
                    ))
                }
            }
            InvocationReceiver::GlobalMethodOnReservedAddress
            | InvocationReceiver::DirectAccess(_)
            | InvocationReceiver::BlueprintFunction => Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            ),
        }
    }
}

impl StaticInvocationResourcesOutput for NonFungibleResourceManagerMintRuidManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // If the receiver is a global address then we can return something useful. Otherwise it
        // is a known amount and an unknown resource address.
        match details.receiver {
            InvocationReceiver::GlobalMethod(global_address) => {
                // Attempt to convert the global address to a resource address. Error if that fails.
                if let Ok(resource_address) = ResourceAddress::try_from(global_address) {
                    TrackedResources::new_empty().add_resource(
                        resource_address,
                        TrackedResource::exact_amount(self.entries.len(), [details.source])?,
                    )
                } else {
                    Err(StaticResourceMovementsError::NotAResourceAddress(
                        global_address,
                    ))
                }
            }
            InvocationReceiver::GlobalMethodOnReservedAddress
            | InvocationReceiver::DirectAccess(_)
            | InvocationReceiver::BlueprintFunction => Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            ),
        }
    }
}

impl StaticInvocationResourcesOutput for NonFungibleResourceManagerMintSingleRuidManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // If the receiver is a global address then we can return something useful. Otherwise it
        // is a known amount and an unknown resource address.
        match details.receiver {
            InvocationReceiver::GlobalMethod(global_address) => {
                // Attempt to convert the global address to a resource address. Error if that fails.
                if let Ok(resource_address) = ResourceAddress::try_from(global_address) {
                    TrackedResources::new_empty().add_resource(
                        resource_address,
                        TrackedResource::exact_amount(Decimal::ONE, [details.source])?,
                    )
                } else {
                    Err(StaticResourceMovementsError::NotAResourceAddress(
                        global_address,
                    ))
                }
            }
            InvocationReceiver::GlobalMethodOnReservedAddress
            | InvocationReceiver::DirectAccess(_)
            | InvocationReceiver::BlueprintFunction => Ok(
                TrackedResources::new_with_possible_balance_of_unspecified_resources([
                    details.source
                ]),
            ),
        }
    }
}
// endregion:NonFungibleResourceManager

// region:ComponentRoyalty
impl StaticInvocationResourcesOutput for ComponentClaimRoyaltiesManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty()
            .add_resource(XRD, TrackedResource::zero_or_more([details.source]))
    }
}
// endregion:ComponentRoyalty
