use super::multi_resource_pool::*;
use super::one_resource_pool::*;
use super::two_resource_pool::*;
use crate::errors::*;
use crate::kernel::kernel_api::*;
use crate::system::system_callback::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::types::*;
use sbor::rust::prelude::*;

pub const POOL_MANAGER_ROLE: &'static str = "pool_manager_role";

pub struct PoolNativePackage;
impl PoolNativePackage {
    pub fn definition() -> PackageDefinition {
        let blueprints = btreemap!(
            ONE_RESOURCE_POOL_BLUEPRINT_IDENT.to_string() => OneResourcePoolBlueprint::definition(),
            TWO_RESOURCE_POOL_BLUEPRINT_IDENT.to_string() => TwoResourcePoolBlueprint::definition(),
            MULTI_RESOURCE_POOL_BLUEPRINT_IDENT.to_string() => MultiResourcePoolBlueprint::definition(),
        );

        PackageDefinition { blueprints }
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>,
    {
        match export_name {
            ONE_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME => {
                let OneResourcePoolInstantiateInput {
                    resource_address,
                    pool_manager_rule,
                    owner_role,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = OneResourcePoolBlueprint::instantiate(
                    resource_address,
                    owner_role,
                    pool_manager_rule,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            ONE_RESOURCE_POOL_CONTRIBUTE_EXPORT_NAME => {
                let OneResourcePoolContributeInput { bucket } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = OneResourcePoolBlueprint::contribute(bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            ONE_RESOURCE_POOL_REDEEM_EXPORT_NAME => {
                let OneResourcePoolRedeemInput { bucket } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = OneResourcePoolBlueprint::redeem(bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            ONE_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME => {
                let OneResourcePoolProtectedDepositInput { bucket } =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = OneResourcePoolBlueprint::protected_deposit(bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_EXPORT_NAME => {
                let OneResourcePoolProtectedWithdrawInput {
                    amount,
                    withdraw_strategy,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn =
                    OneResourcePoolBlueprint::protected_withdraw(amount, withdraw_strategy, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_EXPORT_NAME => {
                let OneResourcePoolGetRedemptionValueInput {
                    amount_of_pool_units,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn =
                    OneResourcePoolBlueprint::get_redemption_value(amount_of_pool_units, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            ONE_RESOURCE_POOL_GET_VAULT_AMOUNT_EXPORT_NAME => {
                let OneResourcePoolGetVaultAmountInput {} = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = OneResourcePoolBlueprint::get_vault_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME => {
                let TwoResourcePoolInstantiateInput {
                    resource_addresses,
                    pool_manager_rule,
                    owner_role,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = TwoResourcePoolBlueprint::instantiate(
                    resource_addresses,
                    owner_role,
                    pool_manager_rule,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_CONTRIBUTE_EXPORT_NAME => {
                let TwoResourcePoolContributeInput { buckets } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = TwoResourcePoolBlueprint::contribute(buckets, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_REDEEM_EXPORT_NAME => {
                let TwoResourcePoolRedeemInput { bucket } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = TwoResourcePoolBlueprint::redeem(bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME => {
                let TwoResourcePoolProtectedDepositInput { bucket } =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = TwoResourcePoolBlueprint::protected_deposit(bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_EXPORT_NAME => {
                let TwoResourcePoolProtectedWithdrawInput {
                    amount,
                    resource_address,
                    withdraw_strategy,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = TwoResourcePoolBlueprint::protected_withdraw(
                    resource_address,
                    amount,
                    withdraw_strategy,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_EXPORT_NAME => {
                let TwoResourcePoolGetRedemptionValueInput {
                    amount_of_pool_units,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn =
                    TwoResourcePoolBlueprint::get_redemption_value(amount_of_pool_units, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_EXPORT_NAME => {
                let TwoResourcePoolGetVaultAmountsInput {} = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = TwoResourcePoolBlueprint::get_vault_amounts(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            MULTI_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME => {
                let MultiResourcePoolInstantiateInput {
                    resource_addresses,
                    owner_role,
                    pool_manager_rule,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = MultiResourcePoolBlueprint::instantiate(
                    resource_addresses,
                    owner_role,
                    pool_manager_rule,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            MULTI_RESOURCE_POOL_CONTRIBUTE_EXPORT_NAME => {
                let MultiResourcePoolContributeInput { buckets } =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = MultiResourcePoolBlueprint::contribute(buckets, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            MULTI_RESOURCE_POOL_REDEEM_EXPORT_NAME => {
                let MultiResourcePoolRedeemInput { bucket } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = MultiResourcePoolBlueprint::redeem(bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME => {
                let MultiResourcePoolProtectedDepositInput { bucket } =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = MultiResourcePoolBlueprint::protected_deposit(bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_EXPORT_NAME => {
                let MultiResourcePoolProtectedWithdrawInput {
                    amount,
                    resource_address,
                    withdraw_strategy,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = MultiResourcePoolBlueprint::protected_withdraw(
                    resource_address,
                    amount,
                    withdraw_strategy,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            MULTI_RESOURCE_POOL_GET_REDEMPTION_VALUE_EXPORT_NAME => {
                let MultiResourcePoolGetRedemptionValueInput {
                    amount_of_pool_units,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn =
                    MultiResourcePoolBlueprint::get_redemption_value(amount_of_pool_units, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            MULTI_RESOURCE_POOL_GET_VAULT_AMOUNTS_EXPORT_NAME => {
                let MultiResourcePoolGetVaultAmountsInput {} = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = MultiResourcePoolBlueprint::get_vault_amounts(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
