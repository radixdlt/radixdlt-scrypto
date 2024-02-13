use crate::blueprints::resource::Vault;
use crate::internal_prelude::*;
use sbor::rust::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltySubstate {
    pub royalty_vault: Vault,
}
