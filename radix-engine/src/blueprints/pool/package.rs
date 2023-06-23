use super::multi_resource_pool::*;
use super::one_resource_pool::*;
use super::two_resource_pool::*;
use crate::errors::*;
use crate::event_schema;
use crate::kernel::kernel_api::*;
use crate::roles_template;
use crate::system::system_callback::*;
use radix_engine_common::data::scrypto::*;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, FunctionAuth, MethodAuthTemplate,
    PackageDefinition,
};
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::blueprints::resource::MethodAccessibility;
use radix_engine_interface::schema::*;
use radix_engine_interface::types::*;
use sbor::rust::prelude::*;
use sbor::*;

pub const POOL_MANAGER_ROLE: &'static str = "pool_manager_role";

pub struct PoolNativePackage;
impl PoolNativePackage {
    pub fn definition() -> PackageDefinition {
        // One Resource Pool
        let one_resource_pool_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<OneResourcePoolSubstate>(),
            ));

            let collections = Vec::new();

            let mut functions = BTreeMap::new();

            functions.insert(
                ONE_RESOURCE_POOL_INSTANTIATE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<OneResourcePoolInstantiateInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<OneResourcePoolInstantiateOutput>(),
                    ),
                    export: ONE_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                ONE_RESOURCE_POOL_CONTRIBUTE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<OneResourcePoolContributeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<OneResourcePoolContributeOutput>(),
                    ),
                    export: ONE_RESOURCE_POOL_CONTRIBUTE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                ONE_RESOURCE_POOL_REDEEM_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<OneResourcePoolRedeemInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<OneResourcePoolRedeemOutput>(),
                    ),
                    export: ONE_RESOURCE_POOL_REDEEM_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                ONE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<OneResourcePoolProtectedDepositInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<OneResourcePoolProtectedDepositOutput>()),
                    export: ONE_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<OneResourcePoolProtectedWithdrawInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<OneResourcePoolProtectedWithdrawOutput>()),
                    export: ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<OneResourcePoolGetRedemptionValueInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<OneResourcePoolGetRedemptionValueOutput>(
                        )),
                    export: ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                ONE_RESOURCE_POOL_GET_VAULT_AMOUNT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<OneResourcePoolGetVaultAmountInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<OneResourcePoolGetVaultAmountOutput>(
                            ),
                    ),
                    export: ONE_RESOURCE_POOL_GET_VAULT_AMOUNT_EXPORT_NAME.to_string(),
                },
            );

            let event_schema = event_schema! {
                aggregator,
                [
                    super::one_resource_pool::ContributionEvent,
                    super::one_resource_pool::RedemptionEvent,
                    super::one_resource_pool::WithdrawEvent,
                    super::one_resource_pool::DepositEvent
                ]
            };

            let schema = generate_full_schema(aggregator);

            BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                dependencies: btreeset!(),
                feature_set: btreeset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections,
                    },
                    events: event_schema,
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::StaticRoles(roles_template! {
                        roles {
                            POOL_MANAGER_ROLE;
                        },
                        methods {
                            // Main Module rules
                            ONE_RESOURCE_POOL_REDEEM_IDENT => MethodAccessibility::Public;
                            ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT => MethodAccessibility::Public;
                            ONE_RESOURCE_POOL_GET_VAULT_AMOUNT_IDENT => MethodAccessibility::Public;
                            ONE_RESOURCE_POOL_CONTRIBUTE_IDENT => [POOL_MANAGER_ROLE];
                            ONE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT => [POOL_MANAGER_ROLE];
                            ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT => [POOL_MANAGER_ROLE];
                        }
                    }),
                },
            }
        };

        // Two Resource Pool
        let two_resource_pool_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<TwoResourcePoolSubstate>(),
            ));

            let collections = Vec::new();

            let mut functions = BTreeMap::new();

            functions.insert(
                TWO_RESOURCE_POOL_INSTANTIATE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<TwoResourcePoolInstantiateInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<TwoResourcePoolInstantiateOutput>(),
                    ),
                    export: TWO_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                TWO_RESOURCE_POOL_CONTRIBUTE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<TwoResourcePoolContributeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<TwoResourcePoolContributeOutput>(),
                    ),
                    export: TWO_RESOURCE_POOL_CONTRIBUTE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                TWO_RESOURCE_POOL_REDEEM_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<TwoResourcePoolRedeemInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<TwoResourcePoolRedeemOutput>(),
                    ),
                    export: TWO_RESOURCE_POOL_REDEEM_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolProtectedDepositInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolProtectedDepositOutput>()),
                    export: TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolProtectedWithdrawInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolProtectedWithdrawOutput>()),
                    export: TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolGetRedemptionValueInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolGetRedemptionValueOutput>(
                        )),
                    export: TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<TwoResourcePoolGetVaultAmountsInput>(
                            ),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<TwoResourcePoolGetVaultAmountsOutput>(
                            ),
                    ),
                    export: TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_EXPORT_NAME.to_string(),
                },
            );

            let event_schema = event_schema! {
                aggregator,
                [
                    super::two_resource_pool::ContributionEvent,
                    super::two_resource_pool::RedemptionEvent,
                    super::two_resource_pool::WithdrawEvent,
                    super::two_resource_pool::DepositEvent
                ]
            };

            let schema = generate_full_schema(aggregator);

            BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                dependencies: btreeset!(),
                feature_set: btreeset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections,
                    },
                    events: event_schema,
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                },

                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::StaticRoles(roles_template! {
                        roles {
                            POOL_MANAGER_ROLE;
                        },
                        methods {
                            // Main Module rules
                            TWO_RESOURCE_POOL_REDEEM_IDENT => MethodAccessibility::Public;
                            TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT => MethodAccessibility::Public;
                            TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT => MethodAccessibility::Public;
                            TWO_RESOURCE_POOL_CONTRIBUTE_IDENT => [POOL_MANAGER_ROLE];
                            TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT => [POOL_MANAGER_ROLE];
                            TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT => [POOL_MANAGER_ROLE];
                        }
                    }),
                },
            }
        };

        // Multi Resource Pool
        let multi_resource_pool_blueprint = {
            let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

            let mut fields = Vec::new();
            fields.push(FieldSchema::static_field(
                aggregator.add_child_type_and_descendents::<MultiResourcePoolSubstate>(),
            ));

            let collections = Vec::new();

            let mut functions = BTreeMap::new();

            functions.insert(
                MULTI_RESOURCE_POOL_INSTANTIATE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: None,
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<MultiResourcePoolInstantiateInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<MultiResourcePoolInstantiateOutput>(),
                    ),
                    export: MULTI_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<MultiResourcePoolContributeInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<MultiResourcePoolContributeOutput>(),
                    ),
                    export: MULTI_RESOURCE_POOL_CONTRIBUTE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                MULTI_RESOURCE_POOL_REDEEM_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(
                        aggregator.add_child_type_and_descendents::<MultiResourcePoolRedeemInput>(),
                    ),
                    output: TypeRef::Static(
                        aggregator
                            .add_child_type_and_descendents::<MultiResourcePoolRedeemOutput>(),
                    ),
                    export: MULTI_RESOURCE_POOL_REDEEM_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<MultiResourcePoolProtectedDepositInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<MultiResourcePoolProtectedDepositOutput>(
                        )),
                    export: MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref_mut()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<MultiResourcePoolProtectedWithdrawInput>(
                        )),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<MultiResourcePoolProtectedWithdrawOutput>(
                        )),
                    export: MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                MULTI_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<MultiResourcePoolGetRedemptionValueInput>(
                        )),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<MultiResourcePoolGetRedemptionValueOutput>(
                        )),
                    export: MULTI_RESOURCE_POOL_GET_REDEMPTION_VALUE_EXPORT_NAME.to_string(),
                },
            );

            functions.insert(
                MULTI_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT.to_string(),
                FunctionSchemaInit {
                    receiver: Some(ReceiverInfo::normal_ref()),
                    input: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<MultiResourcePoolGetVaultAmountsInput>()),
                    output: TypeRef::Static(aggregator
                        .add_child_type_and_descendents::<MultiResourcePoolGetVaultAmountsOutput>()),
                    export: MULTI_RESOURCE_POOL_GET_VAULT_AMOUNTS_EXPORT_NAME.to_string(),
                },
            );

            let event_schema = event_schema! {
                aggregator,
                [
                    super::multi_resource_pool::ContributionEvent,
                    super::multi_resource_pool::RedemptionEvent,
                    super::multi_resource_pool::WithdrawEvent,
                    super::multi_resource_pool::DepositEvent
                ]
            };

            let schema = generate_full_schema(aggregator);

            BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                dependencies: btreeset!(),
                feature_set: btreeset!(),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections,
                    },
                    events: event_schema,
                    functions: BlueprintFunctionsSchemaInit {
                        functions,
                        virtual_lazy_load_functions: btreemap!(),
                    },
                },
                royalty_config: PackageRoyaltyConfig::default(),
                auth_config: AuthConfig {
                    function_auth: FunctionAuth::AllowAll,
                    method_auth: MethodAuthTemplate::StaticRoles(roles_template! {
                        roles {
                            POOL_MANAGER_ROLE;
                        },
                        methods {
                            MULTI_RESOURCE_POOL_REDEEM_IDENT => MethodAccessibility::Public;
                            MULTI_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT => MethodAccessibility::Public;
                            MULTI_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT => MethodAccessibility::Public;
                            MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT => [POOL_MANAGER_ROLE];
                            MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT => [POOL_MANAGER_ROLE];
                            MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT => [POOL_MANAGER_ROLE];
                        }
                    }),
                },
            }
        };

        let blueprints = btreemap!(
            ONE_RESOURCE_POOL_BLUEPRINT_IDENT.to_string() => one_resource_pool_blueprint,
            TWO_RESOURCE_POOL_BLUEPRINT_IDENT.to_string() => two_resource_pool_blueprint,
            MULTI_RESOURCE_POOL_BLUEPRINT_IDENT.to_string() => multi_resource_pool_blueprint,
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
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = OneResourcePoolBlueprint::instantiate(
                    resource_address,
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
                let OneResourcePoolProtectedWithdrawInput { amount } =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = OneResourcePoolBlueprint::protected_withdraw(amount, api)?;
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
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = TwoResourcePoolBlueprint::instantiate(
                    resource_addresses,
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
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn =
                    TwoResourcePoolBlueprint::protected_withdraw(resource_address, amount, api)?;
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
                    pool_manager_rule,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = MultiResourcePoolBlueprint::instantiate(
                    resource_addresses,
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
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn =
                    MultiResourcePoolBlueprint::protected_withdraw(resource_address, amount, api)?;
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
