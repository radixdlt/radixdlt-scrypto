use crate::blueprints::resource::Vault;
use crate::*;
use sbor::rust::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltySubstate {
    pub royalty_vault: Vault,
}
