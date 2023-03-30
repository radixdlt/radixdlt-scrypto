use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use native_sdk::resource::Vault;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::clock::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::time::TimeComparisonOperator;
use sbor::rust::boxed::Box;

use super::{
    AccessControllerError, AccessControllerSubstate, PrimaryOperationState, PrimaryRoleState,
    RecoveryOperationState, RecoveryRecoveryState,
};

/// A trait which defines the interface for an access controller transition for a given trigger or
/// input and the expected output.
pub(super) trait Transition<I> {
    type Output;

    fn transition<Y>(&self, api: &mut Y, input: I) -> Result<Self::Output, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>;
}

/// A trait which defines the interface for an access controller transition for a given trigger or
/// input and the expected output.
pub(super) trait TransitionMut<I> {
    type Output;

    fn transition_mut<Y>(&mut self, api: &mut Y, input: I) -> Result<Self::Output, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>;
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Proofs can only be created when the primary role is unlocked - regardless of whether the
        // controller is in recovery or normal operations.
        match self.state {
            (PrimaryRoleState::Unlocked, _, _) => {
                Vault(self.controlled_asset).sys_create_proof(api)
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match self.state {
            (_, ref mut primary_operations_state @ PrimaryOperationState::Normal, _) => {
                // Transition the primary operations state from normal to recovery
                *primary_operations_state = PrimaryOperationState::Recovery(input.proposal);
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match self.state {
            (_, _, ref mut recovery_operations_state @ RecoveryOperationState::Normal) => {
                match self.timed_recovery_delay_in_minutes {
                    Some(delay_in_minutes) => {
                        let current_time = Runtime::sys_current_time(api, TimePrecision::Minute)?;
                        let timed_recovery_allowed_after = current_time
                            .add_minutes(delay_in_minutes as i64)
                            .map_or(access_controller_runtime_error!(TimeOverflow), |instant| {
                                Ok(instant)
                            })?;

                        *recovery_operations_state =
                            RecoveryOperationState::Recovery(RecoveryRecoveryState::Timed {
                                proposal: input.proposal,
                                timed_recovery_allowed_after,
                            });
                        Ok(())
                    }
                    None => {
                        *recovery_operations_state = RecoveryOperationState::Recovery(
                            RecoveryRecoveryState::Untimed(input.proposal),
                        );
                        Ok(())
                    }
                }
            }
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match self.state {
            (_, PrimaryOperationState::Recovery(ref proposal), _) => {
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match self.state {
            (
                _,
                _,
                RecoveryOperationState::Recovery(
                    RecoveryRecoveryState::Untimed(ref proposal)
                    | RecoveryRecoveryState::Timed { ref proposal, .. },
                ),
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Timed confirm recovery can only be performed by the recovery role (this is checked
        // through access rules on the invocation itself) and can be performed in recovery mode
        // regardless of whether primary is locked or unlocked.
        match self.state {
            (
                _,
                _,
                RecoveryOperationState::Recovery(RecoveryRecoveryState::Timed {
                    ref proposal,
                    ref timed_recovery_allowed_after,
                }),
            ) => {
                let proposal = proposal.clone();

                // Ensure that the caller has passed in the expected proposal
                validate_recovery_proposal(&proposal, &input.proposal_to_confirm)?;

                let recovery_time_has_elapsed = Runtime::sys_compare_against_current_time(
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // A recovery attempt can only be canceled when we're in recovery mode regardless of whether
        // primary is locked or unlocked
        match self.state {
            (_, PrimaryOperationState::Recovery(..), _) => {
                // Transition from the recovery state to the normal operations state
                self.state.1 = PrimaryOperationState::Normal;
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // A recovery attempt can only be canceled when we're in recovery mode regardless of whether
        // primary is locked or unlocked
        match self.state {
            (_, _, RecoveryOperationState::Recovery(..)) => {
                // Transition from the recovery state to the normal operations state
                self.state.2 = RecoveryOperationState::Normal;
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

pub(super) struct AccessControllerLockPrimaryRoleStateMachineInput;

impl TransitionMut<AccessControllerLockPrimaryRoleStateMachineInput> for AccessControllerSubstate {
    type Output = ();

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        _input: AccessControllerLockPrimaryRoleStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Primary can only be locked when it's unlocked
        match self.state {
            (ref mut primary_state @ PrimaryRoleState::Unlocked, _, _) => {
                *primary_state = PrimaryRoleState::Locked;
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // Primary can only be unlocked when it's locked
        match self.state {
            (ref mut primary_state @ PrimaryRoleState::Locked, _, _) => {
                *primary_state = PrimaryRoleState::Unlocked;
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // We can only stop the timed recovery timer if we're in recovery mode. It doesn't matter
        // if primary is locked or unlocked
        match self.state {
            (
                _,
                _,
                RecoveryOperationState::Recovery(RecoveryRecoveryState::Timed {
                    ref proposal, ..
                }),
            ) => {
                // Ensure that the caller has passed in the expected proposal
                validate_recovery_proposal(&proposal, &input.proposal)?;

                // Transition from timed recovery to untimed recovery
                self.state.2 = RecoveryOperationState::Recovery(RecoveryRecoveryState::Untimed(
                    proposal.clone(),
                ));

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
