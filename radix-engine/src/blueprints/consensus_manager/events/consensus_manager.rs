use crate::blueprints::consensus_manager::ActiveValidatorSet;
use crate::types::*;

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct RoundChangeEvent {
    pub round: Round,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct EpochChangeEvent {
    /// The *new* epoch's number.
    pub epoch: Epoch,
    /// The *new* epoch's validator set.
    pub validator_set: ActiveValidatorSet,
}
