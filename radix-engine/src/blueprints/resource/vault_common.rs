use crate::blueprints::resource::*;
use crate::errors::{ApplicationError, RuntimeError};
use crate::internal_prelude::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum VaultError {
    ResourceError(ResourceError),
    ProofError(ProofError),
    InvalidAmount(Decimal),
    NotFreezable,
    NotRecallable,
    VaultIsFrozen,
    LockFeeNotRadixToken,
    LockFeeInsufficientBalance { requested: Decimal, actual: Decimal },
    DecimalOverflow,
}

impl From<VaultError> for RuntimeError {
    fn from(error: VaultError) -> Self {
        RuntimeError::ApplicationError(ApplicationError::VaultError(error))
    }
}

pub struct VaultUtil;

impl VaultUtil {
    pub fn is_vault_blueprint(blueprint: &BlueprintId) -> bool {
        blueprint.package_address.eq(&RESOURCE_PACKAGE)
            && (blueprint.blueprint_name.eq(NON_FUNGIBLE_VAULT_BLUEPRINT)
                || blueprint.blueprint_name.eq(FUNGIBLE_VAULT_BLUEPRINT))
    }
}
