use crate::api::types::*;
use crate::data::scrypto::model::*;
use crate::*;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, Sbor, PartialEq, Eq)]
pub struct ComponentStateSubstate {
    pub raw: Vec<u8>,
}

impl ComponentStateSubstate {
    pub fn new(raw: Vec<u8>) -> Self {
        ComponentStateSubstate { raw }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltyConfigSubstate {
    pub royalty_config: RoyaltyConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltyAccumulatorSubstate {
    pub royalty_vault: Option<Own>,
}
