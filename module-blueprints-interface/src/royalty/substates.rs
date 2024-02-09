use native_blueprints_interface::resource::Vault;
use radix_engine_common::prelude::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub struct ComponentRoyaltySubstate {
    pub royalty_vault: Vault,
}
