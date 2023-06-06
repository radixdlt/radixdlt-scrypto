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
    pub royalty_vault: Vault,
}

impl Clone for ComponentRoyaltyAccumulatorSubstate {
    fn clone(&self) -> Self {
        Self {
            royalty_vault: Vault(self.royalty_vault.0.clone())
        }
    }
}
