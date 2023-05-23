use super::single_resource_pool::*;
use crate::errors::*;
use crate::event_schema;
use crate::kernel::kernel_api::*;
use crate::system::system_callback::*;
use crate::system::system_modules::costing::*;
use radix_engine_common::data::scrypto::*;
use radix_engine_common::types::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::schema::*;
use radix_engine_interface::types::*;
use resources_tracker_macro::*;
use sbor::rust::prelude::*;
use sbor::*;

pub struct PoolNativePackage;
impl PoolNativePackage {
    pub fn schema() -> PackageSchema {
        // Single Resource Pool
        let single_resource_pool_blueprint_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(aggregator.add_child_type_and_descendents::<SingleResourcePoolSubstate>());

            let collections = Vec::new();

            let mut functions = BTreeMap::new();

            functions.insert(
                SINGLE_RESOURCE_POOL_INSTANTIATE_IDENT.to_string(),
                FunctionSchema {
                    receiver: None,
                    input: aggregator
                        .add_child_type_and_descendents::<SingleResourcePoolInstantiateInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<SingleResourcePoolInstantiateOutput>(),
                    export_name: SINGLE_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                SINGLE_RESOURCE_POOL_CONTRIBUTE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<SingleResourcePoolContributeInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<SingleResourcePoolContributeOutput>(),
                    export_name: SINGLE_RESOURCE_POOL_CONTRIBUTE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                SINGLE_RESOURCE_POOL_REDEEM_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<SingleResourcePoolRedeemInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<SingleResourcePoolRedeemOutput>(),
                    export_name: SINGLE_RESOURCE_POOL_REDEEM_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                SINGLE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<SingleResourcePoolProtectedDepositInput>(
                        ),
                    output: aggregator
                        .add_child_type_and_descendents::<SingleResourcePoolProtectedDepositOutput>(
                        ),
                    export_name: SINGLE_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                SINGLE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator.add_child_type_and_descendents::<SingleResourcePoolProtectedWithdrawInput>(),
                    output: aggregator.add_child_type_and_descendents::<SingleResourcePoolProtectedWithdrawOutput>(),
                    export_name: SINGLE_RESOURCE_POOL_PROTECTED_WITHDRAW_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                SINGLE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator.add_child_type_and_descendents::<SingleResourcePoolGetRedemptionValueInput>(),
                    output: aggregator.add_child_type_and_descendents::<SingleResourcePoolGetRedemptionValueOutput>(),
                    export_name: SINGLE_RESOURCE_POOL_GET_REDEMPTION_VALUE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                SINGLE_RESOURCE_POOL_GET_VAULT_AMOUNT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<SingleResourcePoolGetVaultAmountInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<SingleResourcePoolGetVaultAmountOutput>(),
                    export_name: SINGLE_RESOURCE_POOL_GET_VAULT_AMOUNT_EXPORT_NAME.to_string(),
                },
            );

            let virtual_lazy_load_functions = BTreeMap::new();

            let event_schema = event_schema! {
                aggregator,
                [
                    SingleResourcePoolContributionEvent,
                    SingleResourcePoolRedemptionEvent,
                    SingleResourceProtectedWithdrawEvent,
                    SingleResourceProtectedDepositEvent
                ]
            };

            let schema = generate_full_schema(aggregator);
            BlueprintSchema {
                outer_blueprint: None,
                schema,
                fields,
                collections,
                functions,
                virtual_lazy_load_functions,
                event_schema,
            }
        };

        PackageSchema {
            blueprints: btreemap!(
                SINGLE_RESOURCE_POOL_BLUEPRINT_IDENT.to_string() => single_resource_pool_blueprint_schema
            ),
        }
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<&NodeId>,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>,
    {
        match export_name {
            SINGLE_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let SingleResourcePoolInstantiateInput {
                    resource_address,
                    pool_manager_rule,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = SingleResourcePoolBlueprint::instantiate(
                    resource_address,
                    pool_manager_rule,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            SINGLE_RESOURCE_POOL_CONTRIBUTE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let SingleResourcePoolContributeInput { bucket } =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = SingleResourcePoolBlueprint::contribute(bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            SINGLE_RESOURCE_POOL_REDEEM_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let SingleResourcePoolRedeemInput { bucket } = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = SingleResourcePoolBlueprint::redeem(bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            SINGLE_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let SingleResourcePoolProtectedDepositInput { bucket } =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = SingleResourcePoolBlueprint::protected_deposit(bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            SINGLE_RESOURCE_POOL_PROTECTED_WITHDRAW_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let SingleResourcePoolProtectedWithdrawInput { amount } =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = SingleResourcePoolBlueprint::protected_withdraw(amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            SINGLE_RESOURCE_POOL_GET_REDEMPTION_VALUE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let SingleResourcePoolGetRedemptionValueInput {
                    amount_of_pool_units,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn =
                    SingleResourcePoolBlueprint::get_redemption_value(amount_of_pool_units, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            SINGLE_RESOURCE_POOL_GET_VAULT_AMOUNT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let SingleResourcePoolGetVaultAmountInput {} = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = SingleResourcePoolBlueprint::get_vault_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
