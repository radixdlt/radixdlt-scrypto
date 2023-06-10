use crate::data::scrypto::model::*;
use crate::types::*;
use crate::*;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltyConfigSubstate {
    pub royalty_config: RoyaltyConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltyAccumulatorSubstate {
    pub royalty_vault: Option<Own>,
}
