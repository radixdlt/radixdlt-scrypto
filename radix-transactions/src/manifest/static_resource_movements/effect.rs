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
        node_id: &NodeId,
        inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo>;
}

// region:Typed Invocation
impl StaticInvocationResourcesOutput for TypedNativeInvocation {
    fn output(
        &self,
        node_id: &NodeId,
        inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        match self {
            TypedNativeInvocation::AccessControllerPackage(access_controller_invocations) => {
                match access_controller_invocations {
                    AccessControllerInvocations::AccessControllerBlueprint(
                        access_controller_blueprint_invocations,
                    ) => match access_controller_blueprint_invocations {
                        AccessControllerBlueprintInvocations::Function(access_controller_function) => {
                            match access_controller_function {
                                AccessControllerFunction::Create(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                            }
                        }
                        AccessControllerBlueprintInvocations::Method(_, access_controller_method) => {
                            match access_controller_method {
                                AccessControllerMethod::CreateProof(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::InitiateRecoveryAsPrimary(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::InitiateRecoveryAsRecovery(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::QuickConfirmPrimaryRoleRecoveryProposal(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::QuickConfirmRecoveryRoleRecoveryProposal(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::TimedConfirmRecovery(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::StopTimedRecovery(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::MintRecoveryBadges(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::LockRecoveryFee(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::WithdrawRecoveryFee(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::ContributeRecoveryFee(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::InitiateBadgeWithdrawAttemptAsPrimary(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::InitiateBadgeWithdrawAttemptAsRecovery(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::QuickConfirmPrimaryRoleBadgeWithdrawAttempt(
                                    input,
                                ) => input.output(node_id, inputs, instruction_index),
                                AccessControllerMethod::QuickConfirmRecoveryRoleBadgeWithdrawAttempt(
                                    input,
                                ) => input.output(node_id, inputs, instruction_index),
                                AccessControllerMethod::CancelPrimaryRoleRecoveryProposal(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::CancelRecoveryRoleRecoveryProposal(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::CancelPrimaryRoleBadgeWithdrawAttempt(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::CancelRecoveryRoleBadgeWithdrawAttempt(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::LockPrimaryRole(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccessControllerMethod::UnlockPrimaryRole(input) => {
                                    input.output(node_id, inputs, instruction_index)
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
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountFunction::CreateAdvanced(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                        },
                        AccountBlueprintInvocations::Method(_, account_method) => match account_method {
                            AccountMethod::Securify(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::LockFee(input) => input.output(node_id, inputs, instruction_index),
                            AccountMethod::LockContingentFee(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::Deposit(input) => input.output(node_id, inputs, instruction_index),
                            AccountMethod::Withdraw(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::WithdrawNonFungibles(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::LockFeeAndWithdraw(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::LockFeeAndWithdrawNonFungibles(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::CreateProofOfAmount(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::CreateProofOfNonFungibles(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::SetDefaultDepositRule(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::SetResourcePreference(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::RemoveResourcePreference(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::TryDepositOrAbort(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::Burn(input) => input.output(node_id, inputs, instruction_index),
                            AccountMethod::BurnNonFungibles(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::AddAuthorizedDepositor(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::RemoveAuthorizedDepositor(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::TryDepositOrRefund(input) => {
                                input.output(node_id, inputs, instruction_index)
                            }
                            // TODO: A bit of a hack to support expressions. We need a better way to
                            // do this in the future, but certainly not one for this PR.
                            AccountMethod::DepositBatch(..) => AccountDepositBatchManifestInput {
                                buckets: Default::default(),
                            }
                            .output(node_id, inputs, instruction_index),
                            AccountMethod::TryDepositBatchOrRefund(..) => {
                                AccountTryDepositBatchOrRefundManifestInput {
                                    buckets: Default::default(),
                                    authorized_depositor_badge: None,
                                }
                                .output(node_id, inputs, instruction_index)
                            }
                            AccountMethod::TryDepositBatchOrAbort(..) => {
                                AccountTryDepositBatchOrAbortManifestInput {
                                    buckets: Default::default(),
                                    authorized_depositor_badge: None,
                                }
                                .output(node_id, inputs, instruction_index)
                            }
                        },
                    }
                }
            },
            TypedNativeInvocation::ConsensusManagerPackage(consensus_manager_invocations) => {
                match consensus_manager_invocations {
                    ConsensusManagerInvocations::ValidatorBlueprint(validator_blueprint_invocations) => {
                        match validator_blueprint_invocations {
                            ValidatorBlueprintInvocations::Function(..) => {
                                unreachable!()
                            }
                            ValidatorBlueprintInvocations::Method(_, validator_method) => {
                                match validator_method {
                                    ValidatorMethod::Register(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::Unregister(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::StakeAsOwner(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::Stake(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::Unstake(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::ClaimXrd(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::UpdateKey(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::UpdateFee(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::UpdateAcceptDelegatedStake(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::AcceptsDelegatedStake(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::TotalStakeXrdAmount(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::TotalStakeUnitSupply(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::GetRedemptionValue(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::SignalProtocolUpdateReadiness(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::GetProtocolUpdateReadiness(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::LockOwnerStakeUnits(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::StartUnlockOwnerStakeUnits(input) => {
                                        input.output(node_id, inputs, instruction_index)
                                    }
                                    ValidatorMethod::FinishUnlockOwnerStakeUnits(input) => {
                                        input.output(node_id, inputs, instruction_index)
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
                                    input.output(node_id, inputs, instruction_index)
                                }
                            }
                        }
                        ConsensusManagerBlueprintInvocations::Method(_, consensus_manager_method) => {
                            match consensus_manager_method {
                                ConsensusManagerMethod::GetCurrentEpoch(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                ConsensusManagerMethod::Start(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                ConsensusManagerMethod::GetCurrentTime(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                ConsensusManagerMethod::NextRound(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                ConsensusManagerMethod::CreateValidator(input) => {
                                    input.output(node_id, inputs, instruction_index)
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
                                    input.output(node_id, inputs, instruction_index)
                                }
                                IdentityFunction::CreateAdvanced(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                            }
                        }
                        IdentityBlueprintInvocations::Method(_, identity_method) => match identity_method {
                            IdentityMethod::Securify(input) => {
                                input.output(node_id, inputs, instruction_index)
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
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccountLockerFunction::InstantiateSimple(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                            }
                        }
                        AccountLockerBlueprintInvocations::Method(_, account_locker_method) => {
                            match account_locker_method {
                                AccountLockerMethod::Store(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccountLockerMethod::Airdrop(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccountLockerMethod::Recover(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccountLockerMethod::RecoverNonFungibles(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccountLockerMethod::Claim(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccountLockerMethod::ClaimNonFungibles(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccountLockerMethod::GetAmount(input) => {
                                    input.output(node_id, inputs, instruction_index)
                                }
                                AccountLockerMethod::GetNonFungibleLocalIds(input) => {
                                    input.output(node_id, inputs, instruction_index)
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
impl StaticInvocationResourcesOutput for AccountCreateAdvancedManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountCreateInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::Unknown(
            WorktopUncertaintySource::Invocation { instruction_index },
        )]
    }
}

impl StaticInvocationResourcesOutput for AccountSecurifyInput {
    fn output(
        &self,
        node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::KnownNonFungible(
            NonFungibleResourceAddress(ACCOUNT_OWNER_BADGE),
            NonFungibleBounds::new_exact(indexset![
                NonFungibleLocalId::bytes(node_id.as_bytes()).unwrap()
            ]),
        )]
    }
}

impl StaticInvocationResourcesOutput for AccountLockFeeInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountLockContingentFeeInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountDepositManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountDepositBatchManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountWithdrawInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        match CompositeResourceAddress::from(self.resource_address) {
            CompositeResourceAddress::Fungible(fungible_resource_address) => {
                vec![InvocationIo::KnownFungible(
                    fungible_resource_address,
                    FungibleBounds::new_exact(self.amount),
                )]
            }
            CompositeResourceAddress::NonFungible(non_fungible_resource_address) => {
                vec![InvocationIo::KnownNonFungible(
                    non_fungible_resource_address,
                    NonFungibleBounds::new_with_amount(self.amount),
                )]
            }
        }
    }
}

impl StaticInvocationResourcesOutput for AccountWithdrawNonFungiblesInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::KnownNonFungible(
            NonFungibleResourceAddress(self.resource_address),
            NonFungibleBounds::new_exact(self.ids.clone()),
        )]
    }
}

impl StaticInvocationResourcesOutput for AccountLockFeeAndWithdrawInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        match CompositeResourceAddress::from(self.resource_address) {
            CompositeResourceAddress::Fungible(fungible_resource_address) => {
                vec![InvocationIo::KnownFungible(
                    fungible_resource_address,
                    FungibleBounds::new_exact(self.amount),
                )]
            }
            CompositeResourceAddress::NonFungible(non_fungible_resource_address) => {
                vec![InvocationIo::KnownNonFungible(
                    non_fungible_resource_address,
                    NonFungibleBounds::new_with_amount(self.amount),
                )]
            }
        }
    }
}

impl StaticInvocationResourcesOutput for AccountLockFeeAndWithdrawNonFungiblesInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::KnownNonFungible(
            NonFungibleResourceAddress(self.resource_address),
            NonFungibleBounds::new_exact(self.ids.clone()),
        )]
    }
}

impl StaticInvocationResourcesOutput for AccountCreateProofOfAmountInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountCreateProofOfNonFungiblesInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountSetDefaultDepositRuleInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountSetResourcePreferenceInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountRemoveResourcePreferenceInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountTryDepositOrRefundManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        inputs
            .iter()
            .cloned()
            .map(|mut item| {
                match item {
                    InvocationIo::KnownFungible(_, ref mut fungible_bounds) => {
                        fungible_bounds.lower = LowerFungibleBound::Amount(Decimal::ZERO);
                    }
                    InvocationIo::KnownNonFungible(_, ref mut non_fungible_bounds) => {
                        non_fungible_bounds.amount_bounds.lower =
                            LowerFungibleBound::Amount(Decimal::ZERO);
                        non_fungible_bounds.id_bounds = match non_fungible_bounds.id_bounds.clone()
                        {
                            NonFungibleIdBounds::FullyKnown(index_set)
                            | NonFungibleIdBounds::PartiallyKnown(index_set) => {
                                NonFungibleIdBounds::PartiallyKnown(index_set)
                            }
                            NonFungibleIdBounds::Unknown => NonFungibleIdBounds::Unknown,
                        }
                    }
                    InvocationIo::Unknown(..) => {}
                };
                item
            })
            .collect()
    }
}

impl StaticInvocationResourcesOutput for AccountTryDepositBatchOrRefundManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        inputs
            .iter()
            .cloned()
            .map(|mut item| {
                match item {
                    InvocationIo::KnownFungible(_, ref mut fungible_bounds) => {
                        fungible_bounds.lower = LowerFungibleBound::Amount(Decimal::ZERO);
                    }
                    InvocationIo::KnownNonFungible(_, ref mut non_fungible_bounds) => {
                        non_fungible_bounds.amount_bounds.lower =
                            LowerFungibleBound::Amount(Decimal::ZERO);
                        non_fungible_bounds.id_bounds = match non_fungible_bounds.id_bounds.clone()
                        {
                            NonFungibleIdBounds::FullyKnown(index_set)
                            | NonFungibleIdBounds::PartiallyKnown(index_set) => {
                                NonFungibleIdBounds::PartiallyKnown(index_set)
                            }
                            NonFungibleIdBounds::Unknown => NonFungibleIdBounds::Unknown,
                        }
                    }
                    InvocationIo::Unknown(..) => {}
                };
                item
            })
            .collect()
    }
}

impl StaticInvocationResourcesOutput for AccountTryDepositOrAbortManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountTryDepositBatchOrAbortManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountBurnInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountBurnNonFungiblesInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountAddAuthorizedDepositorInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountRemoveAuthorizedDepositorInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}
// endregion:Account

// region:Access Controller
impl StaticInvocationResourcesOutput for AccessControllerCreateManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccessControllerCreateProofInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccessControllerInitiateRecoveryAsPrimaryInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccessControllerInitiateRecoveryAsRecoveryInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput
    for AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryInput
{
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput
    for AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryInput
{
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput
    for AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput
{
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput
    for AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput
{
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput
    for AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptInput
{
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::Unknown(
            WorktopUncertaintySource::Invocation { instruction_index },
        )]
    }
}

impl StaticInvocationResourcesOutput
    for AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptInput
{
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::Unknown(
            WorktopUncertaintySource::Invocation { instruction_index },
        )]
    }
}

impl StaticInvocationResourcesOutput for AccessControllerTimedConfirmRecoveryInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccessControllerCancelPrimaryRoleRecoveryProposalInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccessControllerCancelRecoveryRoleRecoveryProposalInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput
    for AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptInput
{
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput
    for AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptInput
{
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccessControllerLockPrimaryRoleInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccessControllerUnlockPrimaryRoleInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccessControllerStopTimedRecoveryInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccessControllerMintRecoveryBadgesInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::Unknown(
            WorktopUncertaintySource::Invocation { instruction_index },
        )]
    }
}

impl StaticInvocationResourcesOutput for AccessControllerLockRecoveryFeeInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccessControllerWithdrawRecoveryFeeInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::KnownFungible(
            FungibleResourceAddress(XRD),
            FungibleBounds::new_exact(self.amount),
        )]
    }
}

impl StaticInvocationResourcesOutput for AccessControllerContributeRecoveryFeeManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}
// endregion:Access Controller

// region:Consensus Manager
impl StaticInvocationResourcesOutput for ConsensusManagerCreateManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ConsensusManagerGetCurrentEpochInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ConsensusManagerStartInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ConsensusManagerGetCurrentTimeInputV2 {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ConsensusManagerCompareCurrentTimeInputV2 {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ConsensusManagerNextRoundInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ConsensusManagerCreateValidatorManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::Unknown(
            WorktopUncertaintySource::Invocation { instruction_index },
        )]
    }
}

impl StaticInvocationResourcesOutput for ValidatorRegisterInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorUnregisterInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorStakeAsOwnerManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::Unknown(
            WorktopUncertaintySource::Invocation { instruction_index },
        )]
    }
}

impl StaticInvocationResourcesOutput for ValidatorStakeManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::Unknown(
            WorktopUncertaintySource::Invocation { instruction_index },
        )]
    }
}

impl StaticInvocationResourcesOutput for ValidatorUnstakeManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::Unknown(
            WorktopUncertaintySource::Invocation { instruction_index },
        )]
    }
}

impl StaticInvocationResourcesOutput for ValidatorClaimXrdManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::Unknown(
            WorktopUncertaintySource::Invocation { instruction_index },
        )]
    }
}

impl StaticInvocationResourcesOutput for ValidatorUpdateKeyInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorUpdateFeeInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorUpdateAcceptDelegatedStakeInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorAcceptsDelegatedStakeInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorTotalStakeXrdAmountInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorTotalStakeUnitSupplyInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorGetRedemptionValueInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorSignalProtocolUpdateReadinessInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorGetProtocolUpdateReadinessInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorApplyEmissionInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorApplyRewardInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorLockOwnerStakeUnitsManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorStartUnlockOwnerStakeUnitsInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for ValidatorFinishUnlockOwnerStakeUnitsInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::Unknown(
            WorktopUncertaintySource::Invocation { instruction_index },
        )]
    }
}
// endregion:Consensus Manager

// region:Identity
impl StaticInvocationResourcesOutput for IdentityCreateAdvancedInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for IdentityCreateInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::Unknown(
            WorktopUncertaintySource::Invocation { instruction_index },
        )]
    }
}

impl StaticInvocationResourcesOutput for IdentitySecurifyToSingleBadgeInput {
    fn output(
        &self,
        node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::KnownNonFungible(
            NonFungibleResourceAddress(IDENTITY_OWNER_BADGE),
            NonFungibleBounds::new_exact(indexset![
                NonFungibleLocalId::bytes(node_id.as_bytes()).unwrap()
            ]),
        )]
    }
}
// endregion:Identity

// region:Account Locker
impl StaticInvocationResourcesOutput for AccountLockerInstantiateManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountLockerInstantiateSimpleManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::Unknown(
            WorktopUncertaintySource::Invocation { instruction_index },
        )]
    }
}

impl StaticInvocationResourcesOutput for AccountLockerStoreManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountLockerAirdropManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::Unknown(
            WorktopUncertaintySource::Invocation { instruction_index },
        )]
    }
}

impl StaticInvocationResourcesOutput for AccountLockerRecoverManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        match CompositeResourceAddress::from(self.resource_address) {
            CompositeResourceAddress::Fungible(fungible_resource_address) => {
                vec![InvocationIo::KnownFungible(
                    fungible_resource_address,
                    FungibleBounds::new_exact(self.amount),
                )]
            }
            CompositeResourceAddress::NonFungible(non_fungible_resource_address) => {
                vec![InvocationIo::KnownNonFungible(
                    non_fungible_resource_address,
                    NonFungibleBounds::new_with_amount(self.amount),
                )]
            }
        }
    }
}

impl StaticInvocationResourcesOutput for AccountLockerRecoverNonFungiblesManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::KnownNonFungible(
            NonFungibleResourceAddress(self.resource_address),
            NonFungibleBounds::new_exact(self.ids.clone()),
        )]
    }
}

impl StaticInvocationResourcesOutput for AccountLockerClaimManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        match CompositeResourceAddress::from(self.resource_address) {
            CompositeResourceAddress::Fungible(fungible_resource_address) => {
                vec![InvocationIo::KnownFungible(
                    fungible_resource_address,
                    FungibleBounds::new_exact(self.amount),
                )]
            }
            CompositeResourceAddress::NonFungible(non_fungible_resource_address) => {
                vec![InvocationIo::KnownNonFungible(
                    non_fungible_resource_address,
                    NonFungibleBounds::new_with_amount(self.amount),
                )]
            }
        }
    }
}

impl StaticInvocationResourcesOutput for AccountLockerClaimNonFungiblesManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::KnownNonFungible(
            NonFungibleResourceAddress(self.resource_address),
            NonFungibleBounds::new_exact(self.ids.clone()),
        )]
    }
}

impl StaticInvocationResourcesOutput for AccountLockerGetAmountManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for AccountLockerGetNonFungibleLocalIdsManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}
// endregion:Account Locker

// region:Package
impl StaticInvocationResourcesOutput for PackagePublishWasmManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::Unknown(
            WorktopUncertaintySource::Invocation { instruction_index },
        )]
    }
}

impl StaticInvocationResourcesOutput for PackagePublishWasmAdvancedManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for PackagePublishNativeManifestInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![]
    }
}

impl StaticInvocationResourcesOutput for PackageClaimRoyaltiesInput {
    fn output(
        &self,
        _node_id: &NodeId,
        _inputs: &[InvocationIo],
        _instruction_index: usize,
    ) -> Vec<InvocationIo> {
        vec![InvocationIo::KnownFungible(
            FungibleResourceAddress(XRD),
            FungibleBounds {
                lower: LowerFungibleBound::Amount(0.into()),
                upper: UpperFungibleBound::Unbounded,
            },
        )]
    }
}
// endregion:Package
