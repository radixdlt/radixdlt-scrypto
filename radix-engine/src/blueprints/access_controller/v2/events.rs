use crate::internal_prelude::*;
use radix_engine_interface::blueprints::access_controller::{Proposer, RecoveryProposal};

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub struct InitiateRecoveryEvent {
    pub proposer: Proposer,
    pub proposal: RecoveryProposal,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub struct InitiateBadgeWithdrawAttemptEvent {
    pub proposer: Proposer,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub struct RuleSetUpdateEvent {
    pub proposer: Proposer,
    pub proposal: RecoveryProposal,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub struct BadgeWithdrawEvent {
    pub proposer: Proposer,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub struct CancelRecoveryProposalEvent {
    pub proposer: Proposer,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub struct CancelBadgeWithdrawAttemptEvent {
    pub proposer: Proposer,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub struct LockPrimaryRoleEvent;

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub struct UnlockPrimaryRoleEvent;

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub struct StopTimedRecoveryEvent;

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub struct DepositRecoveryXrdEvent {
    pub amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, PartialEq, Eq)]
pub struct WithdrawRecoveryXrdEvent {
    pub amount: Decimal,
}
