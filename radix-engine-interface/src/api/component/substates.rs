use crate::data::scrypto::model::*;
use crate::types::*;
use crate::*;
use radix_engine_common::data::scrypto::ScryptoValue;
use radix_engine_derive::ScryptoDescribe;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, ScryptoEncode, ScryptoDecode, ScryptoDescribe, PartialEq, Eq)]
#[sbor(transparent)]
pub struct ComponentStateSubstate(pub ScryptoValue);

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltyConfigSubstate {
    pub royalty_config: RoyaltyConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltyAccumulatorSubstate {
    pub royalty_vault: Option<Own>,
}
