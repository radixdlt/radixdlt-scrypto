use crate::blueprints::epoch_manager::Validator;
use native_sdk::{LegacyDescribe, ScryptoSbor};
use radix_engine_interface::api::types::{rust::collections::BTreeMap, ComponentAddress};

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct RoundChangeEvent {
    pub round: u64,
}

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct EpochChangeEvent {
    pub epoch: u64,
    pub validators: BTreeMap<ComponentAddress, Validator>,
}
