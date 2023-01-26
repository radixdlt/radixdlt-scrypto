use crate::engine::{ApplicationError, RuntimeError, SystemApi};
use crate::types::*;
use native_sdk::{resource::Vault, runtime::Runtime};
use radix_engine_interface::{
    api::{EngineApi, InvokableModel},
    model::{Proof, Proposer, Role, RuleSet, TimePrecision},
    time::TimeComparisonOperator,
};

use super::{
    AccessControllerError, AccessControllerSubstate, OperationState, PrimaryRoleState,
    RecoveryProposal,
};

/// A trait which defines the interface for an access controller transition for a given trigger or
/// input and the expected output.
pub(super) trait Transition<I> {
    type Output;

    fn transition<Y>(&self, api: &mut Y, input: I) -> Result<Self::Output, RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>;
}

/// A trait which defines the interface for an access controller transition for a given trigger or
/// input and the expected output.
pub(super) trait TransitionMut<I> {
    type Output;

    fn transition_mut<Y>(&mut self, api: &mut Y, input: I) -> Result<Self::Output, RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>;
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
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Proofs can only be created when the primary role is unlocked - regardless of whether the
        // controller is in recovery or normal operations.
        match self.state {
            (PrimaryRoleState::Unlocked, _) => Vault(self.controlled_asset).sys_create_proof(api),
            _ => access_controller_runtime_error!(OperationRequiresUnlockedPrimaryRole),
        }
    }
}

pub(super) struct AccessControllerInitiateRecoveryStateMachineInput {
    pub rule_set: RuleSet,
    pub proposer: Proposer,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

impl TransitionMut<AccessControllerInitiateRecoveryStateMachineInput> for AccessControllerSubstate {
    type Output = ();

    fn transition_mut<Y>(
        &mut self,
        api: &mut Y,
        input: AccessControllerInitiateRecoveryStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Calculate the time when this proposal can be confirmed through a timed recovery
        // confirmation
        let timed_recovery_allowed_after = {
            // Only the recovery role is allowed to perform timed recoveries. If the proposer is not
            // Recovery, then return `None`.
            match input.proposer {
                Proposer::Primary => None,
                Proposer::Recovery => match self.timed_recovery_delay_in_minutes {
                    Some(delay_in_minutes) => {
                        let current_time = Runtime::sys_current_time(api, TimePrecision::Minute)?;
                        let timed_recovery_allowed_after = current_time
                            .add_minutes(delay_in_minutes as i64)
                            .map_or(access_controller_runtime_error!(TimeOverflow), |instant| {
                                Ok(instant)
                            })?;
                        Some(timed_recovery_allowed_after)
                    }
                    None => None,
                },
            }
        };
        let recovery_proposal = RecoveryProposal {
            rule_set: input.rule_set.clone(),
            timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
            timed_recovery_allowed_after,
        };

        // Initiate recovery can be performed regardless whether we're in recovery mode already or
        // outside of recovery mode.
        match self.state {
            (_, ref mut mode @ OperationState::Normal) => {
                // No recoveries are happening at the current moment, so transition to recovery mode
                // and add a new entry
                let mut ongoing_recoveries = HashMap::new();
                ongoing_recoveries.insert(input.proposer, recovery_proposal);
                *mode = OperationState::Recovery { ongoing_recoveries };
                Ok(())
            }
            (
                _,
                OperationState::Recovery {
                    ref mut ongoing_recoveries,
                },
            ) => {
                // Only insert after checking that this proposer doesn't already have something
                // proposed - so, don't just silently override the recovery proposal.
                if !ongoing_recoveries.contains_key(&input.proposer) {
                    ongoing_recoveries.insert(input.proposer, recovery_proposal);
                    Ok(())
                } else {
                    Err(RuntimeError::ApplicationError(
                        ApplicationError::AccessControllerError(
                            AccessControllerError::RecoveryForThisProposerAlreadyExists {
                                proposer: input.proposer,
                            },
                        ),
                    ))
                }
            }
        }
    }
}

pub(super) struct AccessControllerQuickConfirmRecoveryStateMachineInput {
    pub rule_set: RuleSet,
    pub proposer: Proposer,
    pub confirmor: Role,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

impl TransitionMut<AccessControllerQuickConfirmRecoveryStateMachineInput>
    for AccessControllerSubstate
{
    type Output = RecoveryProposal;

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        input: AccessControllerQuickConfirmRecoveryStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // This transition can not be performed when the confirmor and the proposer are the same.
        // As in, Role X can't perform a quick confirm on a recovery initiate by Role X, it has to
        // be different.
        if input.confirmor == input.proposer.into() {
            return access_controller_runtime_error!(ProposerAndConfirmorAreTheSame);
        }

        // This can be performed regardless if primary is locked or unlocked and only when in
        // recovery mode.
        match self.state {
            (
                _,
                OperationState::Recovery {
                    ref mut ongoing_recoveries,
                },
            ) => {
                // Attempt to find a recovery proposal that matches the given input
                let recovery_proposal = find_recovery_proposal(
                    &ongoing_recoveries,
                    &input.proposer,
                    &input.rule_set,
                    &input.timed_recovery_delay_in_minutes,
                )?
                .clone();

                // If we have successfully found the recovery proposal, then we transition into
                // normal operations mode with primary unlocked and return the ruleset that was
                // found.
                self.state = (PrimaryRoleState::Unlocked, OperationState::Normal);

                Ok(recovery_proposal)
            }
            _ => access_controller_runtime_error!(OperationRequiresRecoveryMode),
        }
    }
}

pub(super) struct AccessControllerTimedConfirmRecoveryStateMachineInput {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
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
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Timed confirm recovery can only be performed by the recovery role (this is checked
        // through access rules on the invocation itself) and can be performed in recovery mode
        // regardless of whether primary is locked or unlocked.
        match self.state {
            (
                _,
                OperationState::Recovery {
                    ref mut ongoing_recoveries,
                },
            ) => {
                // Attempt to find the recovery proposal
                let recovery_proposal = find_recovery_proposal(
                    &ongoing_recoveries,
                    &Proposer::Recovery,
                    &input.rule_set,
                    &input.timed_recovery_delay_in_minutes,
                )?
                .clone();

                // Check if the timed recovery delay has elapsed or not (if it's defined for this
                // proposal)
                let recovery_time_has_elapsed = match recovery_proposal.timed_recovery_allowed_after
                {
                    Some(instant) => Runtime::sys_compare_against_current_time(
                        api,
                        instant,
                        TimePrecision::Minute,
                        TimeComparisonOperator::Gte,
                    ),
                    None => access_controller_runtime_error!(
                        TimedRecoveryDelayIsNotEnabledForThisProposal
                    ),
                }?;

                // If the timed recovery delay has elapsed, then we transition into normal
                // operations mode with primary unlocked and return the ruleset that was found.
                if !recovery_time_has_elapsed {
                    access_controller_runtime_error!(TimedRecoveryDelayHasNotElapsed)
                } else {
                    self.state = (PrimaryRoleState::Unlocked, OperationState::Normal);

                    Ok(recovery_proposal)
                }
            }
            _ => access_controller_runtime_error!(OperationRequiresRecoveryMode),
        }
    }
}

pub(super) struct AccessControllerCancelRecoveryAttemptStateMachineInput {
    pub rule_set: RuleSet,
    pub proposer: Proposer,
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

impl TransitionMut<AccessControllerCancelRecoveryAttemptStateMachineInput>
    for AccessControllerSubstate
{
    type Output = ();

    fn transition_mut<Y>(
        &mut self,
        _api: &mut Y,
        input: AccessControllerCancelRecoveryAttemptStateMachineInput,
    ) -> Result<Self::Output, RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // A recovery attempt can only be canceled when we're in recovery mode regardless of whether
        // primary is locked or unlocked
        match self.state {
            (
                _,
                OperationState::Recovery {
                    ref mut ongoing_recoveries,
                },
            ) => {
                // Check that the proposal information passed as input matches one of the proposals
                find_recovery_proposal(
                    &ongoing_recoveries,
                    &input.proposer,
                    &input.rule_set,
                    &input.timed_recovery_delay_in_minutes,
                )?;

                ongoing_recoveries.remove_entry(&input.proposer).expect(
                    "Impossible Case! The proposal was found above and should be found again",
                );

                // If no more recoveries remain, transition to the Regular operations mode
                if ongoing_recoveries.is_empty() {
                    self.state.1 = OperationState::Normal;
                }

                Ok(())
            }
            _ => access_controller_runtime_error!(OperationRequiresRecoveryMode),
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
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Primary can only be locked when it's unlocked
        match self.state {
            (ref mut primary_state @ PrimaryRoleState::Unlocked, _) => {
                *primary_state = PrimaryRoleState::Locked;
                Ok(())
            }
            _ => access_controller_runtime_error!(OperationRequiresUnlockedPrimaryRole),
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
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Primary can only be unlocked when it's locked
        match self.state {
            (ref mut primary_state @ PrimaryRoleState::Locked, _) => {
                *primary_state = PrimaryRoleState::Unlocked;
                Ok(())
            }
            _ => access_controller_runtime_error!(OperationRequiresLockedPrimaryRole),
        }
    }
}

pub(super) struct AccessControllerStopTimedRecoveryStateMachineInput {
    pub rule_set: RuleSet,
    pub timed_recovery_delay_in_minutes: Option<u32>,
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
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // We can only stop the timed recovery timer if we're in recovery mode. It doesn't matter
        // if primary is locked or unlocked
        match self.state {
            (
                _,
                OperationState::Recovery {
                    ref mut ongoing_recoveries,
                },
            ) => {
                let recovery_proposal = find_recovery_proposal_mut(
                    ongoing_recoveries,
                    &Proposer::Recovery,
                    &input.rule_set,
                    &input.timed_recovery_delay_in_minutes,
                )?;

                // If the recovery proposal has a end time defined, then switch it to none,
                // otherwise error out since this was not a correct OP.
                if recovery_proposal.timed_recovery_allowed_after.is_some() {
                    recovery_proposal.timed_recovery_allowed_after = None;
                    Ok(())
                } else {
                    access_controller_runtime_error!(TimedRecoveryDelayIsNotEnabledForThisProposal)
                }
            }
            _ => access_controller_runtime_error!(OperationRequiresRecoveryMode),
        }
    }
}

fn find_recovery_proposal<'a>(
    proposals: &'a HashMap<Proposer, RecoveryProposal>,
    proposer: &Proposer,
    rule_set: &RuleSet,
    timed_recovery_delay_in_minutes: &Option<u32>,
) -> Result<&'a RecoveryProposal, AccessControllerError> {
    proposals
        .iter()
        .find(
            |(
                item_proposer,
                RecoveryProposal {
                    rule_set: item_rule_set,
                    timed_recovery_delay_in_minutes: item_timed_recovery_delay_in_minutes,
                    ..
                },
            )| {
                proposer == *item_proposer
                    && rule_set == item_rule_set
                    && timed_recovery_delay_in_minutes == item_timed_recovery_delay_in_minutes
            },
        )
        .map_or(
            Err(AccessControllerError::NoValidRecoveryProposalExists),
            |(_, proposal)| Ok(proposal),
        )
}

fn find_recovery_proposal_mut<'a>(
    proposals: &'a mut HashMap<Proposer, RecoveryProposal>,
    proposer: &Proposer,
    rule_set: &RuleSet,
    timed_recovery_delay_in_minutes: &Option<u32>,
) -> Result<&'a mut RecoveryProposal, AccessControllerError> {
    proposals
        .iter_mut()
        .find(
            |(
                item_proposer,
                RecoveryProposal {
                    rule_set: item_rule_set,
                    timed_recovery_delay_in_minutes: item_timed_recovery_delay_in_minutes,
                    ..
                },
            )| {
                proposer == *item_proposer
                    && rule_set == item_rule_set
                    && timed_recovery_delay_in_minutes == item_timed_recovery_delay_in_minutes
            },
        )
        .map_or(
            Err(AccessControllerError::NoValidRecoveryProposalExists),
            |(_, proposal)| Ok(proposal),
        )
}
