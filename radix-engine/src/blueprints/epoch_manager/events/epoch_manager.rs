use crate::blueprints::epoch_manager::Validator;
use radix_engine_interface::api::types::{rust::collections::BTreeMap, ComponentAddress};
use radix_engine_interface::*;

#[derive(ScryptoSbor)]
pub struct RoundChangeEvent {
    pub round: u64,
}

#[derive(ScryptoSbor)]
pub struct EpochChangeEvent {
    pub epoch: u64,
    pub validators: BTreeMap<ComponentAddress, Validator>,
}
