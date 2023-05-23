use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use native_sdk::resource::NativeVault;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::consensus_manager::TimePrecision;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::time::TimeComparisonOperator;
use sbor::rust::boxed::Box;

use super::PrimaryRoleBadgeWithdrawAttemptState;
use super::RecoveryRoleBadgeWithdrawAttemptState;
use super::{
    AccessControllerError, AccessControllerSubstate, PrimaryRoleLockingState,
    PrimaryRoleRecoveryAttemptState, RecoveryRoleRecoveryAttemptState, RecoveryRoleRecoveryState,
};

/// A trait which defines the interface for an access controller transition for a given trigger or
/// input and the expected output.
pub(super) trait Transition<I> {
    type Output;

    fn transition<Y>(&self, api: &mut Y, input: I) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>;
}

/// A trait which defines the interface for an access controller transition for a given trigger or
/// input and the expected output.
pub(super) trait TransitionMut<I> {
    type Output;

    fn transition_mut<Y>(&mut self, api: &mut Y, input: I) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>;
}

//=================================================
// State Machine Input & Transition Implementation
//=================================================

macro_rules! access_controller_runtime_error {
    ($variant: ident) => {
        Err(RuntimeError::ApplicationError(
            ApplicationError::AccessControllerError(AccessControllerError::$variant),
        ))
    };
}

pub(super) struct AccessControllerCreateProofStateMachineInput;

impl Transition<AccessControllerCreateProofStateMachineInput> for AccessControllerSubstate {
    type Output = Proof;

    fn transition<Y>(
        &self,
        api: &mut Y,
        _input: AccessControllerCreateProofStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // Proofs can only be created when the primary role is unlocked - regardless of any pending
        // recovery or withdraw attempts.
        match self.state {
            (PrimaryRoleLockingState::Unlocked, _, _, _, _) => {
                Vault(self.controlled_asset).create_proof(api)
            }
            _ => access_controller_runtime_error!(OperationRequiresUnlockedPrimaryRole),
        }
    }
}

pub(super) struct AccessControllerInitiateRecoveryAsPrimaryStateMachineInput {
    pub proposal: RecoveryProposal,
}

impl TransitionMut<AccessControllerInitiateRecoveryAsPrimaryStateMachineInput>
    for AccessControllerSubstate
{
    type Output = ();

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        input: AccessControllerInitiateRecoveryAsPrimaryStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match self.state {
            (
                _,
                ref mut
                primary_role_recovery_attempt_state @ PrimaryRoleRecoveryAttemptState::NoRecoveryAttempt,
                _,
                _,
                _,
            ) => {
                // Transition the primary recovery attempt state from normal to recovery
                *primary_role_recovery_attempt_state =
                    PrimaryRoleRecoveryAttemptState::RecoveryAttempt(input.proposal);
                Ok(())
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::AccessControllerError(
                    AccessControllerError::RecoveryAlreadyExistsForProposer {
                        proposer: Proposer::Primary,
                    },
                ),
            )),
        }
    }
}

pub(super) struct AccessControllerInitiateRecoveryAsRecoveryStateMachineInput {
    pub proposal: RecoveryProposal,
}

impl TransitionMut<AccessControllerInitiateRecoveryAsRecoveryStateMachineInput>
    for AccessControllerSubstate
{
    type Output = ();

    fn transition_mut<Y>(
        &mut self,
        api: &mut Y,
        input: AccessControllerInitiateRecoveryAsRecoveryStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match self.state {
            (
                _,
                _,
                _,
                ref mut recovery_role_recovery_attempt_state @ RecoveryRoleRecoveryAttemptState::NoRecoveryAttempt,
                _,
            ) => match self.timed_recovery_delay_in_minutes {
                Some(delay_in_minutes) => {
                    let current_time = Runtime::current_time(api, TimePrecision::Minute)?;
                    let timed_recovery_allowed_after = current_time
                        .add_minutes(delay_in_minutes as i64)
                        .map_or(access_controller_runtime_error!(TimeOverflow), |instant| {
                            Ok(instant)
                        })?;

                    *recovery_role_recovery_attempt_state = RecoveryRoleRecoveryAttemptState::RecoveryAttempt(
                        RecoveryRoleRecoveryState::TimedRecovery {
                            proposal: input.proposal,
                            timed_recovery_allowed_after,
                        },
                    );
                    Ok(())
                }
                None => {
                    *recovery_role_recovery_attempt_state = RecoveryRoleRecoveryAttemptState::RecoveryAttempt(
                        RecoveryRoleRecoveryState::UntimedRecovery(input.proposal),
                    );
                    Ok(())
                }
            },
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::AccessControllerError(
                    AccessControllerError::RecoveryAlreadyExistsForProposer {
                        proposer: Proposer::Recovery,
                    },
                ),
            )),
        }
    }
}

pub(super) struct AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryStateMachineInput;

impl TransitionMut<AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryStateMachineInput>
    for AccessControllerSubstate
{
    type Output = ();

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        _input: AccessControllerInitiateBadgeWithdrawAttemptAsPrimaryStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match self.state {
            (
                _,
                _,
                ref mut
                primary_role_withdraw_badge_attempt_state @ PrimaryRoleBadgeWithdrawAttemptState::NoBadgeWithdrawAttempt,
                _,
                _,
            ) => {
                // Transition the primary role withdraw attempt state to withdraw attempt
                *primary_role_withdraw_badge_attempt_state = PrimaryRoleBadgeWithdrawAttemptState::BadgeWithdrawAttempt;
                Ok(())
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::AccessControllerError(
                    AccessControllerError::BadgeWithdrawAttemptAlreadyExistsForProposer {
                        proposer: Proposer::Primary,
                    },
                ),
            )),
        }
    }
}

pub(super) struct AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryStateMachineInput;

impl TransitionMut<AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryStateMachineInput>
    for AccessControllerSubstate
{
    type Output = ();

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        _input: AccessControllerInitiateBadgeWithdrawAttemptAsRecoveryStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match self.state {
            (
                _,
                _,
                _,
                _,
                ref mut recovery_role_badge_withdraw_attempt_state @ RecoveryRoleBadgeWithdrawAttemptState::NoBadgeWithdrawAttempt,
            ) => {
                *recovery_role_badge_withdraw_attempt_state = RecoveryRoleBadgeWithdrawAttemptState::BadgeWithdrawAttempt;
                Ok(())
            },
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::AccessControllerError(
                    AccessControllerError::RecoveryAlreadyExistsForProposer {
                        proposer: Proposer::Recovery,
                    },
                ),
            )),
        }
    }
}

pub(super) struct AccessControllerQuickConfirmPrimaryRoleRecoveryProposalStateMachineInput {
    pub proposal_to_confirm: RecoveryProposal,
}

impl TransitionMut<AccessControllerQuickConfirmPrimaryRoleRecoveryProposalStateMachineInput>
    for AccessControllerSubstate
{
    type Output = RecoveryProposal;

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        input: AccessControllerQuickConfirmPrimaryRoleRecoveryProposalStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match self.state {
            (_, PrimaryRoleRecoveryAttemptState::RecoveryAttempt(ref proposal), _, _, _) => {
                let proposal = proposal.clone();

                // Ensure that the caller has passed in the expected proposal
                validate_recovery_proposal(&proposal, &input.proposal_to_confirm)?;

                // Transition back to the initial state of the state machine
                self.state = Default::default();
                Ok(proposal)
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::AccessControllerError(
                    AccessControllerError::NoRecoveryExistsForProposer {
                        proposer: Proposer::Primary,
                    },
                ),
            )),
        }
    }
}

pub(super) struct AccessControllerQuickConfirmRecoveryRoleRecoveryProposalStateMachineInput {
    pub proposal_to_confirm: RecoveryProposal,
}

impl TransitionMut<AccessControllerQuickConfirmRecoveryRoleRecoveryProposalStateMachineInput>
    for AccessControllerSubstate
{
    type Output = RecoveryProposal;

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        input: AccessControllerQuickConfirmRecoveryRoleRecoveryProposalStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match self.state {
            (
                _,
                _,
                _,
                RecoveryRoleRecoveryAttemptState::RecoveryAttempt(
                    RecoveryRoleRecoveryState::UntimedRecovery(ref proposal)
                    | RecoveryRoleRecoveryState::TimedRecovery { ref proposal, .. },
                ),
                _,
            ) => {
                let proposal = proposal.clone();

                // Ensure that the caller has passed in the expected proposal
                validate_recovery_proposal(&proposal, &input.proposal_to_confirm)?;

                // Transition back to the initial state of the state machine
                self.state = Default::default();
                Ok(proposal)
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::AccessControllerError(
                    AccessControllerError::NoRecoveryExistsForProposer {
                        proposer: Proposer::Recovery,
                    },
                ),
            )),
        }
    }
}

pub(super) struct AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptStateMachineInput;

impl TransitionMut<AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptStateMachineInput>
    for AccessControllerSubstate
{
    type Output = Bucket;

    fn transition_mut<Y>(
        &mut self,
        api: &mut Y,
        _input: AccessControllerQuickConfirmPrimaryRoleBadgeWithdrawAttemptStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match self.state {
            (_, _, PrimaryRoleBadgeWithdrawAttemptState::BadgeWithdrawAttempt, _, _) => {
                // Transition back to the initial state of the state machine
                self.state = Default::default();
                Vault(self.controlled_asset).take_all(api)
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::AccessControllerError(
                    AccessControllerError::NoBadgeWithdrawAttemptExistsForProposer {
                        proposer: Proposer::Primary,
                    },
                ),
            )),
        }
    }
}

pub(super) struct AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptStateMachineInput;

impl TransitionMut<AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptStateMachineInput>
    for AccessControllerSubstate
{
    type Output = Bucket;

    fn transition_mut<Y>(
        &mut self,
        api: &mut Y,
        _input: AccessControllerQuickConfirmRecoveryRoleBadgeWithdrawAttemptStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match self.state {
            (_, _, _, _, RecoveryRoleBadgeWithdrawAttemptState::BadgeWithdrawAttempt) => {
                // Transition back to the initial state of the state machine
                self.state = Default::default();
                Vault(self.controlled_asset).take_all(api)
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::AccessControllerError(
                    AccessControllerError::NoBadgeWithdrawAttemptExistsForProposer {
                        proposer: Proposer::Recovery,
                    },
                ),
            )),
        }
    }
}

pub(super) struct AccessControllerTimedConfirmRecoveryStateMachineInput {
    pub proposal_to_confirm: RecoveryProposal,
}

impl TransitionMut<AccessControllerTimedConfirmRecoveryStateMachineInput>
    for AccessControllerSubstate
{
    type Output = RecoveryProposal;

    fn transition_mut<Y>(
        &mut self,
        api: &mut Y,
        input: AccessControllerTimedConfirmRecoveryStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // Timed confirm recovery can only be performed by the recovery role (this is checked
        // through access rules on the invocation itself) and can be performed in recovery mode
        // regardless of whether primary is locked or unlocked.
        match self.state {
            (
                _,
                _,
                _,
                RecoveryRoleRecoveryAttemptState::RecoveryAttempt(
                    RecoveryRoleRecoveryState::TimedRecovery {
                        ref proposal,
                        ref timed_recovery_allowed_after,
                    },
                ),
                _,
            ) => {
                let proposal = proposal.clone();

                // Ensure that the caller has passed in the expected proposal
                validate_recovery_proposal(&proposal, &input.proposal_to_confirm)?;

                let recovery_time_has_elapsed = Runtime::compare_against_current_time(
                    api,
                    timed_recovery_allowed_after.clone(),
                    TimePrecision::Minute,
                    TimeComparisonOperator::Gte,
                )?;

                // If the timed recovery delay has elapsed, then we transition into normal
                // operations mode with primary unlocked and return the ruleset that was found.
                if !recovery_time_has_elapsed {
                    access_controller_runtime_error!(TimedRecoveryDelayHasNotElapsed)
                } else {
                    self.state = Default::default();

                    Ok(proposal)
                }
            }
            _ => access_controller_runtime_error!(NoTimedRecoveriesFound),
        }
    }
}

pub(super) struct AccessControllerCancelPrimaryRoleRecoveryProposalStateMachineInput;

impl TransitionMut<AccessControllerCancelPrimaryRoleRecoveryProposalStateMachineInput>
    for AccessControllerSubstate
{
    type Output = ();

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        _input: AccessControllerCancelPrimaryRoleRecoveryProposalStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // A recovery attempt can only be canceled when we're in recovery mode regardless of whether
        // primary is locked or unlocked
        match self.state {
            (_, PrimaryRoleRecoveryAttemptState::RecoveryAttempt(..), _, _, _) => {
                // Transition from the recovery state to the normal operations state
                self.state.1 = PrimaryRoleRecoveryAttemptState::NoRecoveryAttempt;
                Ok(())
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::AccessControllerError(
                    AccessControllerError::NoRecoveryExistsForProposer {
                        proposer: Proposer::Primary,
                    },
                ),
            )),
        }
    }
}

pub(super) struct AccessControllerCancelRecoveryRoleRecoveryProposalStateMachineInput;

impl TransitionMut<AccessControllerCancelRecoveryRoleRecoveryProposalStateMachineInput>
    for AccessControllerSubstate
{
    type Output = ();

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        _input: AccessControllerCancelRecoveryRoleRecoveryProposalStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // A recovery attempt can only be canceled when we're in recovery mode regardless of whether
        // primary is locked or unlocked
        match self.state {
            (_, _, _, RecoveryRoleRecoveryAttemptState::RecoveryAttempt(..), _) => {
                // Transition from the recovery state to the normal operations state
                self.state.3 = RecoveryRoleRecoveryAttemptState::NoRecoveryAttempt;
                Ok(())
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::AccessControllerError(
                    AccessControllerError::NoRecoveryExistsForProposer {
                        proposer: Proposer::Recovery,
                    },
                ),
            )),
        }
    }
}

pub(super) struct AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptStateMachineInput;

impl TransitionMut<AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptStateMachineInput>
    for AccessControllerSubstate
{
    type Output = ();

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        _input: AccessControllerCancelPrimaryRoleBadgeWithdrawAttemptStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // A badge withdraw attempt can only be canceled when it exists regardless of whether
        // primary is locked or unlocked
        match self.state {
            (_, _, PrimaryRoleBadgeWithdrawAttemptState::BadgeWithdrawAttempt, _, _) => {
                // Transition from the recovery state to the normal operations state
                self.state.2 = PrimaryRoleBadgeWithdrawAttemptState::NoBadgeWithdrawAttempt;
                Ok(())
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::AccessControllerError(
                    AccessControllerError::NoBadgeWithdrawAttemptExistsForProposer {
                        proposer: Proposer::Primary,
                    },
                ),
            )),
        }
    }
}

pub(super) struct AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptStateMachineInput;

impl TransitionMut<AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptStateMachineInput>
    for AccessControllerSubstate
{
    type Output = ();

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        _input: AccessControllerCancelRecoveryRoleBadgeWithdrawAttemptStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // A badge withdraw attempt can only be canceled when it exists regardless of whether
        // primary is locked or unlocked
        match self.state {
            (_, _, _, _, RecoveryRoleBadgeWithdrawAttemptState::BadgeWithdrawAttempt) => {
                // Transition from the recovery state to the normal operations state
                self.state.4 = RecoveryRoleBadgeWithdrawAttemptState::NoBadgeWithdrawAttempt;
                Ok(())
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::AccessControllerError(
                    AccessControllerError::NoBadgeWithdrawAttemptExistsForProposer {
                        proposer: Proposer::Recovery,
                    },
                ),
            )),
        }
    }
}

pub(super) struct AccessControllerLockPrimaryRoleStateMachineInput;

impl TransitionMut<AccessControllerLockPrimaryRoleStateMachineInput> for AccessControllerSubstate {
    type Output = ();

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        _input: AccessControllerLockPrimaryRoleStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // Primary can only be locked when it's unlocked
        match self.state {
            (ref mut primary_role_locking_state @ PrimaryRoleLockingState::Unlocked, ..) => {
                *primary_role_locking_state = PrimaryRoleLockingState::Locked;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

pub(super) struct AccessControllerUnlockPrimaryRoleStateMachineInput;

impl TransitionMut<AccessControllerUnlockPrimaryRoleStateMachineInput>
    for AccessControllerSubstate
{
    type Output = ();

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        _input: AccessControllerUnlockPrimaryRoleStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // Primary can only be unlocked when it's locked
        match self.state {
            (ref mut primary_role_locking_state @ PrimaryRoleLockingState::Locked, ..) => {
                *primary_role_locking_state = PrimaryRoleLockingState::Unlocked;
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

pub(super) struct AccessControllerStopTimedRecoveryStateMachineInput {
    pub proposal: RecoveryProposal,
}

impl TransitionMut<AccessControllerStopTimedRecoveryStateMachineInput>
    for AccessControllerSubstate
{
    type Output = ();

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        input: AccessControllerStopTimedRecoveryStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // We can only stop the timed recovery timer if we're in recovery mode. It doesn't matter
        // if primary is locked or unlocked
        match self.state {
            (
                _,
                _,
                _,
                RecoveryRoleRecoveryAttemptState::RecoveryAttempt(
                    RecoveryRoleRecoveryState::TimedRecovery { ref proposal, .. },
                ),
                _,
            ) => {
                // Ensure that the caller has passed in the expected proposal
                validate_recovery_proposal(&proposal, &input.proposal)?;

                // Transition from timed recovery to untimed recovery
                self.state.3 = RecoveryRoleRecoveryAttemptState::RecoveryAttempt(
                    RecoveryRoleRecoveryState::UntimedRecovery(proposal.clone()),
                );

                Ok(())
            }
            // TODO: A more descriptive error is needed here.
            _ => access_controller_runtime_error!(NoTimedRecoveriesFound),
        }
    }
}

fn validate_recovery_proposal(
    expected: &RecoveryProposal,
    actual: &RecoveryProposal,
) -> Result<(), AccessControllerError> {
    if expected == actual {
        Ok(())
    } else {
        Err(AccessControllerError::RecoveryProposalMismatch {
            expected: Box::new(expected.clone()),
            found: Box::new(actual.clone()),
        })
    }
}
