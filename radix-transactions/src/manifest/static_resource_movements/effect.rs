use super::*;
use radix_common::prelude::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::identity::*;
use radix_engine_interface::blueprints::locker::*;
use radix_engine_interface::blueprints::package::*;

pub trait StaticInvocationResourcesOutput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        let _ = details;
        Ok(TrackedResources::new_empty())
    }
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

// region:Typed Invocation
impl StaticInvocationResourcesOutput for TypedNativeInvocation {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        match self {
            TypedNativeInvocation::AccessControllerPackage(access_controller_invocations) => {
                match access_controller_invocations {
                    AccessControllerInvocations::AccessControllerBlueprint(
                        access_controller_blueprint_invocations,
                    ) => match access_controller_blueprint_invocations {
                        AccessControllerBlueprintInvocations::Function(access_controller_function) => {
                            match access_controller_function {
                                AccessControllerFunction::Create(input) => {
                                    input.output(details)
                                }
                            }
                        }
                        AccessControllerBlueprintInvocations::Method(access_controller_method) => {
                            match access_controller_method {
                                AccessControllerMethod::CreateProof(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::InitiateRecoveryAsPrimary(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::InitiateRecoveryAsRecovery(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::QuickConfirmPrimaryRoleRecoveryProposal(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::QuickConfirmRecoveryRoleRecoveryProposal(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::TimedConfirmRecovery(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::StopTimedRecovery(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::MintRecoveryBadges(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::LockRecoveryFee(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::WithdrawRecoveryFee(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::ContributeRecoveryFee(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::InitiateBadgeWithdrawAttemptAsPrimary(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::InitiateBadgeWithdrawAttemptAsRecovery(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::QuickConfirmPrimaryRoleBadgeWithdrawAttempt(
                                    input,
                                ) => input.output(details),
                                AccessControllerMethod::QuickConfirmRecoveryRoleBadgeWithdrawAttempt(
                                    input,
                                ) => input.output(details),
                                AccessControllerMethod::CancelPrimaryRoleRecoveryProposal(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::CancelRecoveryRoleRecoveryProposal(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::CancelPrimaryRoleBadgeWithdrawAttempt(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::CancelRecoveryRoleBadgeWithdrawAttempt(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::LockPrimaryRole(input) => {
                                    input.output(details)
                                }
                                AccessControllerMethod::UnlockPrimaryRole(input) => {
                                    input.output(details)
                                }
                            }
                        }
                    },
                }
            }
            TypedNativeInvocation::AccountPackage(account_invocations) => match account_invocations {
                AccountInvocations::AccountBlueprint(account_blueprint_invocations) => {
                    match account_blueprint_invocations {
                        AccountBlueprintInvocations::Function(account_function) => match account_function {
                            AccountFunction::Create(input) => {
                                input.output(details)
                            }
                            AccountFunction::CreateAdvanced(input) => {
                                input.output(details)
                            }
                        },
                        AccountBlueprintInvocations::Method(account_method) => match account_method {
                            AccountMethod::Securify(input) => {
                                input.output(details)
                            }
                            AccountMethod::LockFee(input) => input.output(details),
                            AccountMethod::LockContingentFee(input) => {
                                input.output(details)
                            }
                            AccountMethod::Deposit(input) => input.output(details),
                            AccountMethod::Withdraw(input) => {
                                input.output(details)
                            }
                            AccountMethod::WithdrawNonFungibles(input) => {
                                input.output(details)
                            }
                            AccountMethod::LockFeeAndWithdraw(input) => {
                                input.output(details)
                            }
                            AccountMethod::LockFeeAndWithdrawNonFungibles(input) => {
                                input.output(details)
                            }
                            AccountMethod::CreateProofOfAmount(input) => {
                                input.output(details)
                            }
                            AccountMethod::CreateProofOfNonFungibles(input) => {
                                input.output(details)
                            }
                            AccountMethod::SetDefaultDepositRule(input) => {
                                input.output(details)
                            }
                            AccountMethod::SetResourcePreference(input) => {
                                input.output(details)
                            }
                            AccountMethod::RemoveResourcePreference(input) => {
                                input.output(details)
                            }
                            AccountMethod::TryDepositOrAbort(input) => {
                                input.output(details)
                            }
                            AccountMethod::Burn(input) => input.output(details),
                            AccountMethod::BurnNonFungibles(input) => {
                                input.output(details)
                            }
                            AccountMethod::AddAuthorizedDepositor(input) => {
                                input.output(details)
                            }
                            AccountMethod::RemoveAuthorizedDepositor(input) => {
                                input.output(details)
                            }
                            AccountMethod::TryDepositOrRefund(input) => {
                                input.output(details)
                            }
                            AccountMethod::DepositBatch(input) => input.output(details),
                            AccountMethod::TryDepositBatchOrRefund(input) => input.output(details),
                            AccountMethod::TryDepositBatchOrAbort(input) => input.output(details),
                        },
                    }
                }
            },
            TypedNativeInvocation::ConsensusManagerPackage(consensus_manager_invocations) => {
                match consensus_manager_invocations {
                    ConsensusManagerInvocations::ValidatorBlueprint(validator_blueprint_invocations) => {
                        match validator_blueprint_invocations {
                            ValidatorBlueprintInvocations::Function(validator_function) => {
                                match *validator_function {}
                            }
                            ValidatorBlueprintInvocations::Method(validator_method) => {
                                match validator_method {
                                    ValidatorMethod::Register(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::Unregister(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::StakeAsOwner(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::Stake(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::Unstake(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::ClaimXrd(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::UpdateKey(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::UpdateFee(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::UpdateAcceptDelegatedStake(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::AcceptsDelegatedStake(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::TotalStakeXrdAmount(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::TotalStakeUnitSupply(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::GetRedemptionValue(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::SignalProtocolUpdateReadiness(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::GetProtocolUpdateReadiness(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::LockOwnerStakeUnits(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::StartUnlockOwnerStakeUnits(input) => {
                                        input.output(details)
                                    }
                                    ValidatorMethod::FinishUnlockOwnerStakeUnits(input) => {
                                        input.output(details)
                                    }
                                }
                            }
                        }
                    }
                    ConsensusManagerInvocations::ConsensusManagerBlueprint(
                        consensus_manager_blueprint_invocations,
                    ) => match consensus_manager_blueprint_invocations {
                        ConsensusManagerBlueprintInvocations::Function(consensus_manager_function) => {
                            match consensus_manager_function {
                                ConsensusManagerFunction::Create(input) => {
                                    input.output(details)
                                }
                            }
                        }
                        ConsensusManagerBlueprintInvocations::Method(consensus_manager_method) => {
                            match consensus_manager_method {
                                ConsensusManagerMethod::GetCurrentEpoch(input) => {
                                    input.output(details)
                                }
                                ConsensusManagerMethod::Start(input) => {
                                    input.output(details)
                                }
                                ConsensusManagerMethod::GetCurrentTime(input) => {
                                    input.output(details)
                                }
                                ConsensusManagerMethod::NextRound(input) => {
                                    input.output(details)
                                }
                                ConsensusManagerMethod::CreateValidator(input) => {
                                    input.output(details)
                                }
                            }
                        }
                    },
                }
            }
            TypedNativeInvocation::IdentityPackage(identity_invocations) => match identity_invocations {
                IdentityInvocations::IdentityBlueprint(identity_blueprint_invocations) => {
                    match identity_blueprint_invocations {
                        IdentityBlueprintInvocations::Function(identity_function) => {
                            match identity_function {
                                IdentityFunction::Create(input) => {
                                    input.output(details)
                                }
                                IdentityFunction::CreateAdvanced(input) => {
                                    input.output(details)
                                }
                            }
                        }
                        IdentityBlueprintInvocations::Method(identity_method) => match identity_method {
                            IdentityMethod::Securify(input) => {
                                input.output(details)
                            }
                        },
                    }
                }
            },
            TypedNativeInvocation::LockerPackage(locker_invocations) => match locker_invocations {
                LockerInvocations::AccountLockerBlueprint(account_locker_blueprint_invocations) => {
                    match account_locker_blueprint_invocations {
                        AccountLockerBlueprintInvocations::Function(account_locker_function) => {
                            match account_locker_function {
                                AccountLockerFunction::Instantiate(input) => {
                                    input.output(details)
                                }
                                AccountLockerFunction::InstantiateSimple(input) => {
                                    input.output(details)
                                }
                            }
                        }
                        AccountLockerBlueprintInvocations::Method(account_locker_method) => {
                            match account_locker_method {
                                AccountLockerMethod::Store(input) => {
                                    input.output(details)
                                }
                                AccountLockerMethod::Airdrop(input) => {
                                    input.output(details)
                                }
                                AccountLockerMethod::Recover(input) => {
                                    input.output(details)
                                }
                                AccountLockerMethod::RecoverNonFungibles(input) => {
                                    input.output(details)
                                }
                                AccountLockerMethod::Claim(input) => {
                                    input.output(details)
                                }
                                AccountLockerMethod::ClaimNonFungibles(input) => {
                                    input.output(details)
                                }
                                AccountLockerMethod::GetAmount(input) => {
                                    input.output(details)
                                }
                                AccountLockerMethod::GetNonFungibleLocalIds(input) => {
                                    input.output(details)
                                }
                            }
                        }
                    }
                }
            },
        }
    }
}
// endregion:Typed Invocation

// region:Account
impl StaticInvocationResourcesOutput for AccountCreateAdvancedManifestInput {}

impl StaticInvocationResourcesOutput for AccountCreateInput {
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

impl StaticInvocationResourcesOutput for AccountSecurifyInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        match details.receiver {
            InvocationReceiver::GlobalMethod(global_address) => {
                let local_id = NonFungibleLocalId::bytes(global_address.as_bytes()).unwrap();
                TrackedResources::new_empty().add_resource(
                    ACCOUNT_OWNER_BADGE,
                    TrackedResource::non_fungibles([local_id], [details.source]),
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

impl StaticInvocationResourcesOutput for AccountLockFeeInput {}

impl StaticInvocationResourcesOutput for AccountLockContingentFeeInput {}

impl StaticInvocationResourcesOutput for AccountDepositManifestInput {}

impl StaticInvocationResourcesOutput for AccountDepositBatchManifestInput {}

impl StaticInvocationResourcesOutput for AccountWithdrawInput {
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

impl StaticInvocationResourcesOutput for AccountWithdrawNonFungiblesInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            self.resource_address,
            TrackedResource::non_fungibles(self.ids.clone(), [details.source]),
        )
    }
}

impl StaticInvocationResourcesOutput for AccountLockFeeAndWithdrawInput {
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

impl StaticInvocationResourcesOutput for AccountLockFeeAndWithdrawNonFungiblesInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty().add_resource(
            self.resource_address,
            TrackedResource::non_fungibles(self.ids.clone(), [details.source]),
        )
    }
}

impl StaticInvocationResourcesOutput for AccountCreateProofOfAmountInput {}

impl StaticInvocationResourcesOutput for AccountCreateProofOfNonFungiblesInput {}

impl StaticInvocationResourcesOutput for AccountSetDefaultDepositRuleInput {}

impl StaticInvocationResourcesOutput for AccountSetResourcePreferenceInput {}

impl StaticInvocationResourcesOutput for AccountRemoveResourcePreferenceInput {}

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
        let (_lower_bound, upper_bound) = attempted_deposit.inclusive_bounds();
        let refunded_amount =
            ResourceBounds::general_no_id_allowlist(Decimal::ZERO, upper_bound, [])?;
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

impl StaticInvocationResourcesOutput for AccountTryDepositOrAbortManifestInput {}

impl StaticInvocationResourcesOutput for AccountTryDepositBatchOrAbortManifestInput {}

impl StaticInvocationResourcesOutput for AccountBurnInput {}

impl StaticInvocationResourcesOutput for AccountBurnNonFungiblesInput {}

impl StaticInvocationResourcesOutput for AccountAddAuthorizedDepositorInput {}

impl StaticInvocationResourcesOutput for AccountRemoveAuthorizedDepositorInput {}
// endregion:Account

// region:Access Controller
impl StaticInvocationResourcesOutput for AccessControllerCreateManifestInput {}

impl StaticInvocationResourcesOutput for AccessControllerCreateProofInput {}

impl StaticInvocationResourcesOutput for AccessControllerInitiateRecoveryAsPrimaryInput {}

impl StaticInvocationResourcesOutput for AccessControllerInitiateRecoveryAsRecoveryInput {}

impl StaticInvocationResourcesOutput
    for AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput
{
}

impl StaticInvocationResourcesOutput
    for AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryInput
{
}

impl StaticInvocationResourcesOutput
    for AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput
{
}

impl StaticInvocationResourcesOutput
    for AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput
{
}

impl StaticInvocationResourcesOutput
    for AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptInput
{
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // The withdrawn badge is of an unknown resource
        Ok(TrackedResources::new_with_possible_balance_of_unspecified_resources([details.source]))
    }
}

impl StaticInvocationResourcesOutput
    for AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptInput
{
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // The withdrawn badge is of an unknown resource
        Ok(TrackedResources::new_with_possible_balance_of_unspecified_resources([details.source]))
    }
}

impl StaticInvocationResourcesOutput for AccessControllerTimedConfirmRecoveryInput {}

impl StaticInvocationResourcesOutput for AccessControllerCancelPrimaryRoleRecoveryProposalInput {}

impl StaticInvocationResourcesOutput for AccessControllerCancelRecoveryRoleRecoveryProposalInput {}

impl StaticInvocationResourcesOutput
    for AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptInput
{
}

impl StaticInvocationResourcesOutput
    for AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptInput
{
}

impl StaticInvocationResourcesOutput for AccessControllerLockPrimaryRoleInput {}

impl StaticInvocationResourcesOutput for AccessControllerUnlockPrimaryRoleInput {}

impl StaticInvocationResourcesOutput for AccessControllerStopTimedRecoveryInput {}

impl StaticInvocationResourcesOutput for AccessControllerMintRecoveryBadgesInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // The minted badge is of a new / unknown resource
        Ok(TrackedResources::new_with_possible_balance_of_unspecified_resources([details.source]))
    }
}

impl StaticInvocationResourcesOutput for AccessControllerLockRecoveryFeeInput {}

impl StaticInvocationResourcesOutput for AccessControllerWithdrawRecoveryFeeInput {
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

impl StaticInvocationResourcesOutput for AccessControllerContributeRecoveryFeeManifestInput {}
// endregion:Access Controller

// region:Consensus Manager
impl StaticInvocationResourcesOutput for ConsensusManagerCreateManifestInput {}

impl StaticInvocationResourcesOutput for ConsensusManagerGetCurrentEpochInput {}

impl StaticInvocationResourcesOutput for ConsensusManagerStartInput {}

impl StaticInvocationResourcesOutput for ConsensusManagerGetCurrentTimeInputV2 {}

impl StaticInvocationResourcesOutput for ConsensusManagerCompareCurrentTimeInputV2 {}

impl StaticInvocationResourcesOutput for ConsensusManagerNextRoundInput {}

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

impl StaticInvocationResourcesOutput for ValidatorRegisterInput {}

impl StaticInvocationResourcesOutput for ValidatorUnregisterInput {}

impl StaticInvocationResourcesOutput for ValidatorStakeAsOwnerManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // The validator stake unit resource is unknown at static validation time
        Ok(TrackedResources::new_with_possible_balance_of_unspecified_resources([details.source]))
    }
}

impl StaticInvocationResourcesOutput for ValidatorStakeManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // The validator stake unit resource is unknown at static validation time
        Ok(TrackedResources::new_with_possible_balance_of_unspecified_resources([details.source]))
    }
}

impl StaticInvocationResourcesOutput for ValidatorUnstakeManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // The validator unstake receipt is unknown at static validation time
        Ok(TrackedResources::new_with_possible_balance_of_unspecified_resources([details.source]))
    }
}

impl StaticInvocationResourcesOutput for ValidatorClaimXrdManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty()
            .add_resource(XRD, TrackedResource::zero_or_more([details.source]))
    }
}

impl StaticInvocationResourcesOutput for ValidatorUpdateKeyInput {}

impl StaticInvocationResourcesOutput for ValidatorUpdateFeeInput {}

impl StaticInvocationResourcesOutput for ValidatorUpdateAcceptDelegatedStakeInput {}

impl StaticInvocationResourcesOutput for ValidatorAcceptsDelegatedStakeInput {}

impl StaticInvocationResourcesOutput for ValidatorTotalStakeXrdAmountInput {}

impl StaticInvocationResourcesOutput for ValidatorTotalStakeUnitSupplyInput {}

impl StaticInvocationResourcesOutput for ValidatorGetRedemptionValueInput {}

impl StaticInvocationResourcesOutput for ValidatorSignalProtocolUpdateReadinessInput {}

impl StaticInvocationResourcesOutput for ValidatorGetProtocolUpdateReadinessInput {}

impl StaticInvocationResourcesOutput for ValidatorApplyEmissionInput {}

impl StaticInvocationResourcesOutput for ValidatorApplyRewardInput {}

impl StaticInvocationResourcesOutput for ValidatorLockOwnerStakeUnitsManifestInput {}

impl StaticInvocationResourcesOutput for ValidatorStartUnlockOwnerStakeUnitsInput {}

impl StaticInvocationResourcesOutput for ValidatorFinishUnlockOwnerStakeUnitsInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // This can return validator stake units which are an unknown resource at static validation time
        Ok(TrackedResources::new_with_possible_balance_of_unspecified_resources([details.source]))
    }
}
// endregion:Consensus Manager

// region:Identity
impl StaticInvocationResourcesOutput for IdentityCreateAdvancedInput {}

impl StaticInvocationResourcesOutput for IdentityCreateInput {
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

impl StaticInvocationResourcesOutput for IdentitySecurifyToSingleBadgeInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        Ok(match details.receiver {
            InvocationReceiver::GlobalMethod(global_address) => {
                let local_id = NonFungibleLocalId::bytes(global_address.as_bytes()).unwrap();
                TrackedResources::new_empty().add_resource(
                    IDENTITY_OWNER_BADGE,
                    TrackedResource::non_fungibles([local_id], [details.source]),
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

// region:Account Locker
impl StaticInvocationResourcesOutput for AccountLockerInstantiateManifestInput {}

impl StaticInvocationResourcesOutput for AccountLockerInstantiateSimpleManifestInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        // This generates and returns a new badge resource, which is unknowable at static time
        Ok(TrackedResources::new_with_possible_balance_of_unspecified_resources([details.source]))
    }
}

impl StaticInvocationResourcesOutput for AccountLockerStoreManifestInput {}

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
            TrackedResource::non_fungibles(self.ids.clone(), [details.source]),
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
            TrackedResource::non_fungibles(self.ids.clone(), [details.source]),
        )
    }
}

impl StaticInvocationResourcesOutput for AccountLockerGetAmountManifestInput {}

impl StaticInvocationResourcesOutput for AccountLockerGetNonFungibleLocalIdsManifestInput {}
// endregion:Account Locker

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

impl StaticInvocationResourcesOutput for PackagePublishWasmAdvancedManifestInput {}

impl StaticInvocationResourcesOutput for PackagePublishNativeManifestInput {}

impl StaticInvocationResourcesOutput for PackageClaimRoyaltiesInput {
    fn output(
        &self,
        details: InvocationDetails,
    ) -> Result<TrackedResources, StaticResourceMovementsError> {
        TrackedResources::new_empty()
            .add_resource(XRD, TrackedResource::zero_or_more([details.source]))
    }
}
// endregion:Package
