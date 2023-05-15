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
    /// A sum of staked XRD amounts of the entire validator set of the *concluded* epoch.
    /// Note: calculating `EpochManagerConfigSubstate.total_emission_xrd_per_epoch / validator_set_stake_xrd`
    /// gives a good estimate on an expected gains per 1 XRD staked per epoch (ignoring validator
    /// reliability penalties and validator fees).
    pub validator_set_stake_xrd: Decimal,
}
