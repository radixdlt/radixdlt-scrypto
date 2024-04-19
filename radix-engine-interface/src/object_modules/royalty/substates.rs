use crate::blueprints::resource::Vault;
use crate::internal_prelude::*;
use radix_rust::rust::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltySubstate {
    pub royalty_vault: Vault,
}
