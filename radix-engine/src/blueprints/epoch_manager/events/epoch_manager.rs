use crate::blueprints::epoch_manager::Validator;
use crate::types::*;

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct RoundChangeEvent {
    pub round: u64,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct EpochChangeEvent {
    /// The *new* epoch's number.
    pub epoch: u64,
    /// The *new* epoch's validator set.
    pub validators: BTreeMap<ComponentAddress, Validator>,
}
