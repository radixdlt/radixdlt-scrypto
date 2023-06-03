use super::single_resource_pool::*;
use super::two_resource_pool::*;
use crate::errors::*;
use crate::event_schema;
use crate::kernel::kernel_api::*;
use crate::method_auth_template;
use crate::system::system_callback::*;
use crate::system::system_modules::costing::*;
use radix_engine_common::data::scrypto::*;
use radix_engine_common::types::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::node_modules::royalty::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::package::{BlueprintSetup, PackageSetup};
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::rule;
use radix_engine_interface::schema::*;
use radix_engine_interface::types::*;
use resources_tracker_macro::*;
use sbor::rust::prelude::*;
use sbor::*;

pub const POOL_MANAGER_ROLE: &'static str = "pool_manager_role";

pub struct PoolNativePackage;
impl PoolNativePackage {
    pub fn definition() -> PackageSetup {
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
                    super::single_resource_pool::ContributionEvent,
                    super::single_resource_pool::RedemptionEvent,
                    super::single_resource_pool::WithdrawEvent,
                    super::single_resource_pool::DepositEvent
                ]
            };

            let method_auth_template = method_auth_template! {
                // Metadata Module rules
                SchemaMethodKey::metadata(METADATA_REMOVE_IDENT) => [POOL_MANAGER_ROLE];
                SchemaMethodKey::metadata(METADATA_SET_IDENT) => [POOL_MANAGER_ROLE];
                SchemaMethodKey::metadata(METADATA_GET_IDENT) => SchemaMethodPermission::Public;

                // Royalty Module rules
                SchemaMethodKey::royalty(COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT) => [];
                SchemaMethodKey::royalty(COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT) => [];

                // Main Module rules
                SchemaMethodKey::main(SINGLE_RESOURCE_POOL_REDEEM_IDENT) => SchemaMethodPermission::Public;
                SchemaMethodKey::main(SINGLE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT) => SchemaMethodPermission::Public;
                SchemaMethodKey::main(SINGLE_RESOURCE_POOL_GET_VAULT_AMOUNT_IDENT) => SchemaMethodPermission::Public;
                SchemaMethodKey::main(SINGLE_RESOURCE_POOL_CONTRIBUTE_IDENT) => [POOL_MANAGER_ROLE];
                SchemaMethodKey::main(SINGLE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT) => [POOL_MANAGER_ROLE];
                SchemaMethodKey::main(SINGLE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT) => [POOL_MANAGER_ROLE];
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
                method_auth_template,
                outer_method_auth_template: btreemap!(),
                dependencies: btreeset!(),
            }
        };

        // Two Resource Pool
        let two_resource_pool_blueprint_schema = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(aggregator.add_child_type_and_descendents::<TwoResourcePoolSubstate>());

            let collections = Vec::new();

            let mut functions = BTreeMap::new();

            functions.insert(
                TWO_RESOURCE_POOL_INSTANTIATE_IDENT.to_string(),
                FunctionSchema {
                    receiver: None,
                    input: aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolInstantiateInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolInstantiateOutput>(),
                    export_name: TWO_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                TWO_RESOURCE_POOL_CONTRIBUTE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolContributeInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolContributeOutput>(),
                    export_name: TWO_RESOURCE_POOL_CONTRIBUTE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                TWO_RESOURCE_POOL_REDEEM_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolRedeemInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolRedeemOutput>(),
                    export_name: TWO_RESOURCE_POOL_REDEEM_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolProtectedDepositInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolProtectedDepositOutput>(),
                    export_name: TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolProtectedWithdrawInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolProtectedWithdrawOutput>(),
                    export_name: TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolGetRedemptionValueInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolGetRedemptionValueOutput>(
                        ),
                    export_name: TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT.to_string(),
                FunctionSchema {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolGetVaultAmountsInput>(),
                    output: aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolGetVaultAmountsOutput>(),
                    export_name: TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_EXPORT_NAME.to_string(),
                },
            );

            let virtual_lazy_load_functions = BTreeMap::new();

            let event_schema = event_schema! {
                aggregator,
                [
                    super::two_resource_pool::ContributionEvent,
                    super::two_resource_pool::RedemptionEvent,
                    super::two_resource_pool::WithdrawEvent,
                    super::two_resource_pool::DepositEvent
                ]
            };

            let method_auth_template = method_auth_template! {
                // Metadata Module rules
                SchemaMethodKey::metadata(METADATA_REMOVE_IDENT) => [POOL_MANAGER_ROLE];
                SchemaMethodKey::metadata(METADATA_SET_IDENT) => [POOL_MANAGER_ROLE];
                SchemaMethodKey::metadata(METADATA_GET_IDENT) => SchemaMethodPermission::Public;

                // Royalty Module rules
                SchemaMethodKey::royalty(COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT) => [];
                SchemaMethodKey::royalty(COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT) => [];

                // Main Module rules
                SchemaMethodKey::main(TWO_RESOURCE_POOL_REDEEM_IDENT) => SchemaMethodPermission::Public;
                SchemaMethodKey::main(TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT) => SchemaMethodPermission::Public;
                SchemaMethodKey::main(TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT) => SchemaMethodPermission::Public;
                SchemaMethodKey::main(TWO_RESOURCE_POOL_CONTRIBUTE_IDENT) => [POOL_MANAGER_ROLE];
                SchemaMethodKey::main(TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT) => [POOL_MANAGER_ROLE];
                SchemaMethodKey::main(TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT) => [POOL_MANAGER_ROLE];
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
                method_auth_template,
                outer_method_auth_template: btreemap!(),
                dependencies: btreeset!(),
            }
        };

        let blueprints = btreemap!(
                SINGLE_RESOURCE_POOL_BLUEPRINT_IDENT.to_string() => BlueprintSetup {
                    schema: single_resource_pool_blueprint_schema
                },
                TWO_RESOURCE_POOL_BLUEPRINT_IDENT.to_string() => BlueprintSetup {
                    schema: two_resource_pool_blueprint_schema
                }
            );

        let function_access_rules = btreemap!(
            SINGLE_RESOURCE_POOL_BLUEPRINT_IDENT.to_string() => btreemap!(
                SINGLE_RESOURCE_POOL_INSTANTIATE_IDENT.to_string() => rule!(allow_all),
            ),
            TWO_RESOURCE_POOL_BLUEPRINT_IDENT.to_string() => btreemap!(
                TWO_RESOURCE_POOL_INSTANTIATE_IDENT.to_string() => rule!(allow_all),
            )
        );

        PackageSetup {
            blueprints,
            function_access_rules,
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

            TWO_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let TwoResourcePoolInstantiateInput {
                    resource_addresses,
                    pool_manager_rule,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = TwoResourcePoolBlueprint::instantiate(
                    resource_addresses,
                    pool_manager_rule,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_CONTRIBUTE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let TwoResourcePoolContributeInput { buckets } = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = TwoResourcePoolBlueprint::contribute(buckets, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_REDEEM_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let TwoResourcePoolRedeemInput { bucket } = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = TwoResourcePoolBlueprint::redeem(bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let TwoResourcePoolProtectedDepositInput { bucket } =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;
                let rtn = TwoResourcePoolBlueprint::protected_deposit(bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let TwoResourcePoolProtectedWithdrawInput {
                    amount,
                    resource_address,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn =
                    TwoResourcePoolBlueprint::protected_withdraw(resource_address, amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let TwoResourcePoolGetRedemptionValueInput {
                    amount_of_pool_units,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn =
                    TwoResourcePoolBlueprint::get_redemption_value(amount_of_pool_units, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let TwoResourcePoolGetVaultAmountsInput {} = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = TwoResourcePoolBlueprint::get_vault_amounts(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            _ => Err(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
