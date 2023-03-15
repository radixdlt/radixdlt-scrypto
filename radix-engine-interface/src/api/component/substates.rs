use crate::api::types::*;
use crate::data::scrypto::model::*;
use crate::*;
use radix_engine_common::data::scrypto::ScryptoValue;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct ComponentStateSubstate(pub ScryptoValue);

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltyConfigSubstate {
    pub royalty_config: RoyaltyConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltyAccumulatorSubstate {
    pub royalty_vault: Option<Own>,
}
