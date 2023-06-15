use crate::blueprints::resource::Vault;
use crate::*;
use sbor::rust::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltyAccumulatorSubstate {
    pub royalty_vault: Vault,
}

impl Clone for ComponentRoyaltyAccumulatorSubstate {
    fn clone(&self) -> Self {
        Self {
            royalty_vault: Vault(self.royalty_vault.0.clone()),
        }
    }
}
