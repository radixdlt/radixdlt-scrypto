use crate::blueprints::resource::*;
use crate::types::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum VaultError {
    ResourceError(ResourceError),
    ProofError(ProofError),
    MismatchingResource,
    InvalidAmount,
    NotFreezable,
    NotRecallable,
    VaultIsFrozen,

    LockFeeNotRadixToken,
    LockFeeInsufficientBalance,
}

pub struct VaultUtil;

impl VaultUtil {
    pub fn is_vault_blueprint(blueprint: &BlueprintId) -> bool {
        blueprint.package_address.eq(&RESOURCE_PACKAGE)
            && (blueprint.blueprint_name.eq(NON_FUNGIBLE_VAULT_BLUEPRINT)
                || blueprint.blueprint_name.eq(FUNGIBLE_VAULT_BLUEPRINT))
    }
}
