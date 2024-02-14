use crate::blueprints::consensus_manager::ActiveValidatorSet;
use crate::internal_prelude::*;

#[derive(Debug, Clone, ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct RoundChangeEvent {
    pub round: Round,
}

#[derive(Debug, Clone, ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct EpochChangeEvent {
    /// The *new* epoch's number.
    pub epoch: Epoch,
    /// The *new* epoch's validator set.
    pub validator_set: ActiveValidatorSet,
    /// A mapping of protocol version name to a total stake (using the *new* epoch's validator set)
    /// that has signalled the readiness for the given protocol update.
    /// The mapping only contains entries with associated stake of at least 10%
    /// of the total stake (in the *new* epoch's validator set).
    pub significant_protocol_update_readiness: IndexMap<String, Decimal>,
}
