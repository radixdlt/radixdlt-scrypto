use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::CostingError;
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::{types::*, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum VaultError {
    ResourceError(ResourceError),
    ProofError(ProofError),
    MismatchingResource,
    InvalidAmount,

    LockFeeNotRadixToken,
    LockFeeInsufficientBalance,
}

pub struct VaultUtil;

impl VaultUtil {
    pub fn is_vault_blueprint(blueprint: &Blueprint) -> bool {
        blueprint.package_address.eq(&RESOURCE_MANAGER_PACKAGE)
            && (blueprint.blueprint_name.eq(NON_FUNGIBLE_VAULT_BLUEPRINT)
                || blueprint.blueprint_name.eq(FUNGIBLE_VAULT_BLUEPRINT))
    }
}
