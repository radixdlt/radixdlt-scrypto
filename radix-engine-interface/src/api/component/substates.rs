use crate::data::scrypto::model::*;
use crate::types::*;
use crate::*;
use sbor::rust::prelude::*;
use crate::blueprints::resource::Vault;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltyConfigSubstate {
    pub royalty_config: RoyaltyConfig,
}

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltyAccumulatorSubstate {
    pub royalty_vault: Option<Vault>,
}

impl Clone for ComponentRoyaltyAccumulatorSubstate {
    fn clone(&self) -> Self {
        Self {
            royalty_vault: self.royalty_vault.as_ref().map(|v| Vault(v.0.clone()))
        }
    }
}
