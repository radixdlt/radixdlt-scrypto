#![allow(clippy::let_unit_value)]

use super::constants::*;
use super::substates::multi_resource_pool::*;
use super::substates::one_resource_pool::*;
use super::substates::two_resource_pool::*;
use crate::internal_prelude::*;
use crate::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::prelude::*;

/// The minor version of the Pool V1 package
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Sbor)]
pub enum PoolV1MinorVersion {
    Zero,
    One,
}

pub struct PoolNativePackage;
impl PoolNativePackage {
    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        minor_version: PoolV1MinorVersion,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match export_name {
            ONE_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME => {
                let OneResourcePoolInstantiateInput {
                    resource_address,
                    pool_manager_rule,
                    owner_role,
                    address_reservation,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => super::v1_0::OneResourcePoolBlueprint::instantiate(
                        resource_address,
                        owner_role,
                        pool_manager_rule,
                        address_reservation,
                        api,
                    )?,
                    PoolV1MinorVersion::One => super::v1_1::OneResourcePoolBlueprint::instantiate(
                        resource_address,
                        owner_role,
                        pool_manager_rule,
                        address_reservation,
                        api,
                    )?,
                };

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            ONE_RESOURCE_POOL_CONTRIBUTE_EXPORT_NAME => {
                let OneResourcePoolContributeInput { bucket } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::OneResourcePoolBlueprint::contribute(bucket, api)?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::OneResourcePoolBlueprint::contribute(bucket, api)?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            ONE_RESOURCE_POOL_REDEEM_EXPORT_NAME => {
                let OneResourcePoolRedeemInput { bucket } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::OneResourcePoolBlueprint::redeem(bucket, api)?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::OneResourcePoolBlueprint::redeem(bucket, api)?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            ONE_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME => {
                let OneResourcePoolProtectedDepositInput { bucket } =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::OneResourcePoolBlueprint::protected_deposit(bucket, api)?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::OneResourcePoolBlueprint::protected_deposit(bucket, api)?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_EXPORT_NAME => {
                let OneResourcePoolProtectedWithdrawInput {
                    amount,
                    withdraw_strategy,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::OneResourcePoolBlueprint::protected_withdraw(
                            amount,
                            withdraw_strategy,
                            api,
                        )?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::OneResourcePoolBlueprint::protected_withdraw(
                            amount,
                            withdraw_strategy,
                            api,
                        )?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_EXPORT_NAME => {
                let OneResourcePoolGetRedemptionValueInput {
                    amount_of_pool_units,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::OneResourcePoolBlueprint::get_redemption_value(
                            amount_of_pool_units,
                            api,
                        )?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::OneResourcePoolBlueprint::get_redemption_value(
                            amount_of_pool_units,
                            api,
                        )?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            ONE_RESOURCE_POOL_GET_VAULT_AMOUNT_EXPORT_NAME => {
                let OneResourcePoolGetVaultAmountInput {} = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::OneResourcePoolBlueprint::get_vault_amount(api)?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::OneResourcePoolBlueprint::get_vault_amount(api)?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME => {
                let TwoResourcePoolInstantiateInput {
                    resource_addresses,
                    pool_manager_rule,
                    owner_role,
                    address_reservation,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => super::v1_0::TwoResourcePoolBlueprint::instantiate(
                        resource_addresses,
                        owner_role,
                        pool_manager_rule,
                        address_reservation,
                        api,
                    )?,
                    PoolV1MinorVersion::One => super::v1_1::TwoResourcePoolBlueprint::instantiate(
                        resource_addresses,
                        owner_role,
                        pool_manager_rule,
                        address_reservation,
                        api,
                    )?,
                };

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_CONTRIBUTE_EXPORT_NAME => {
                let TwoResourcePoolContributeInput { buckets } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::TwoResourcePoolBlueprint::contribute(buckets, api)?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::TwoResourcePoolBlueprint::contribute(buckets, api)?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_REDEEM_EXPORT_NAME => {
                let TwoResourcePoolRedeemInput { bucket } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::TwoResourcePoolBlueprint::redeem(bucket, api)?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::TwoResourcePoolBlueprint::redeem(bucket, api)?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME => {
                let TwoResourcePoolProtectedDepositInput { bucket } =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::TwoResourcePoolBlueprint::protected_deposit(bucket, api)?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::TwoResourcePoolBlueprint::protected_deposit(bucket, api)?
                    }
                };
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
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::TwoResourcePoolBlueprint::protected_withdraw(
                            resource_address,
                            amount,
                            withdraw_strategy,
                            api,
                        )?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::TwoResourcePoolBlueprint::protected_withdraw(
                            resource_address,
                            amount,
                            withdraw_strategy,
                            api,
                        )?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_EXPORT_NAME => {
                let TwoResourcePoolGetRedemptionValueInput {
                    amount_of_pool_units,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::TwoResourcePoolBlueprint::get_redemption_value(
                            amount_of_pool_units,
                            api,
                        )?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::TwoResourcePoolBlueprint::get_redemption_value(
                            amount_of_pool_units,
                            api,
                        )?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_EXPORT_NAME => {
                let TwoResourcePoolGetVaultAmountsInput {} = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::TwoResourcePoolBlueprint::get_vault_amounts(api)?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::TwoResourcePoolBlueprint::get_vault_amounts(api)?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            MULTI_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME => {
                let MultiResourcePoolInstantiateInput {
                    resource_addresses,
                    owner_role,
                    pool_manager_rule,
                    address_reservation,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::MultiResourcePoolBlueprint::instantiate(
                            resource_addresses,
                            owner_role,
                            pool_manager_rule,
                            address_reservation,
                            api,
                        )?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::MultiResourcePoolBlueprint::instantiate(
                            resource_addresses,
                            owner_role,
                            pool_manager_rule,
                            address_reservation,
                            api,
                        )?
                    }
                };

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            MULTI_RESOURCE_POOL_CONTRIBUTE_EXPORT_NAME => {
                let MultiResourcePoolContributeInput { buckets } =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::MultiResourcePoolBlueprint::contribute(buckets, api)?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::MultiResourcePoolBlueprint::contribute(buckets, api)?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            MULTI_RESOURCE_POOL_REDEEM_EXPORT_NAME => {
                let MultiResourcePoolRedeemInput { bucket } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::MultiResourcePoolBlueprint::redeem(bucket, api)?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::MultiResourcePoolBlueprint::redeem(bucket, api)?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME => {
                let MultiResourcePoolProtectedDepositInput { bucket } =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::MultiResourcePoolBlueprint::protected_deposit(bucket, api)?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::MultiResourcePoolBlueprint::protected_deposit(bucket, api)?
                    }
                };
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
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::MultiResourcePoolBlueprint::protected_withdraw(
                            resource_address,
                            amount,
                            withdraw_strategy,
                            api,
                        )?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::MultiResourcePoolBlueprint::protected_withdraw(
                            resource_address,
                            amount,
                            withdraw_strategy,
                            api,
                        )?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            MULTI_RESOURCE_POOL_GET_REDEMPTION_VALUE_EXPORT_NAME => {
                let MultiResourcePoolGetRedemptionValueInput {
                    amount_of_pool_units,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::MultiResourcePoolBlueprint::get_redemption_value(
                            amount_of_pool_units,
                            api,
                        )?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::MultiResourcePoolBlueprint::get_redemption_value(
                            amount_of_pool_units,
                            api,
                        )?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            MULTI_RESOURCE_POOL_GET_VAULT_AMOUNTS_EXPORT_NAME => {
                let MultiResourcePoolGetVaultAmountsInput {} = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = match minor_version {
                    PoolV1MinorVersion::Zero => {
                        super::v1_0::MultiResourcePoolBlueprint::get_vault_amounts(api)?
                    }
                    PoolV1MinorVersion::One => {
                        super::v1_1::MultiResourcePoolBlueprint::get_vault_amounts(api)?
                    }
                };
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }

            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    pub fn definition(minor_version: PoolV1MinorVersion) -> PackageDefinition {
        let blueprints = indexmap!(
            ONE_RESOURCE_POOL_BLUEPRINT_IDENT.to_string()
                => Self::one_resource_pool_blueprint_definition(minor_version),
            TWO_RESOURCE_POOL_BLUEPRINT_IDENT.to_string()
                => Self::two_resource_pool_blueprint_definition(minor_version),
            MULTI_RESOURCE_POOL_BLUEPRINT_IDENT.to_string()
                => Self::multi_resource_pool_blueprint_definition(minor_version),
        );

        PackageDefinition { blueprints }
    }

    pub fn one_resource_pool_blueprint_definition(
        _minor_version: PoolV1MinorVersion,
    ) -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let feature_set = OneResourcePoolFeatureSet::all_features();
        let state = OneResourcePoolStateSchemaInit::create_schema_init(&mut aggregator);
        let mut functions = index_map_new();

        functions.insert(
            ONE_RESOURCE_POOL_INSTANTIATE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<OneResourcePoolInstantiateInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<OneResourcePoolInstantiateOutput>(),
                ),
                export: ONE_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME.to_string(),
            },
        );

        functions.insert(
            ONE_RESOURCE_POOL_CONTRIBUTE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<OneResourcePoolContributeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<OneResourcePoolContributeOutput>(),
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
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<OneResourcePoolProtectedDepositInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<OneResourcePoolProtectedDepositOutput>(),
                ),
                export: ONE_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME.to_string(),
            },
        );

        functions.insert(
            ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<OneResourcePoolProtectedWithdrawInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<OneResourcePoolProtectedWithdrawOutput>(),
                ),
                export: ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_EXPORT_NAME.to_string(),
            },
        );

        functions.insert(
            ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<OneResourcePoolGetRedemptionValueInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<OneResourcePoolGetRedemptionValueOutput>(
                        ),
                ),
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
                        .add_child_type_and_descendents::<OneResourcePoolGetVaultAmountOutput>(),
                ),
                export: ONE_RESOURCE_POOL_GET_VAULT_AMOUNT_EXPORT_NAME.to_string(),
            },
        );

        let event_schema = event_schema! {
            aggregator,
            [
                super::events::one_resource_pool::ContributionEvent,
                super::events::one_resource_pool::RedemptionEvent,
                super::events::one_resource_pool::WithdrawEvent,
                super::events::one_resource_pool::DepositEvent
            ]
        };

        let schema = generate_full_schema(aggregator);

        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::default(),
            is_transient: false,
            dependencies: indexset!(),
            feature_set,

            schema: BlueprintSchemaInit {
                generics: vec![],
                schema,
                state,
                events: event_schema,
                types: BlueprintTypeSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit { functions },
                hooks: BlueprintHooksInit::default(),
            },

            royalty_config: PackageRoyaltyConfig::default(),
            auth_config: AuthConfig {
                function_auth: FunctionAuth::AllowAll,
                method_auth: MethodAuthTemplate::StaticRoleDefinition(roles_template! {
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
    }

    pub fn two_resource_pool_blueprint_definition(
        _minor_version: PoolV1MinorVersion,
    ) -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();
        let feature_set = TwoResourcePoolFeatureSet::all_features();
        let state = TwoResourcePoolStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = index_map_new();

        functions.insert(
            TWO_RESOURCE_POOL_INSTANTIATE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<TwoResourcePoolInstantiateInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<TwoResourcePoolInstantiateOutput>(),
                ),
                export: TWO_RESOURCE_POOL_INSTANTIATE_EXPORT_NAME.to_string(),
            },
        );

        functions.insert(
            TWO_RESOURCE_POOL_CONTRIBUTE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<TwoResourcePoolContributeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<TwoResourcePoolContributeOutput>(),
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
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolProtectedDepositInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolProtectedDepositOutput>(),
                ),
                export: TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME.to_string(),
            },
        );

        functions.insert(
            TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolProtectedWithdrawInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolProtectedWithdrawOutput>(),
                ),
                export: TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_EXPORT_NAME.to_string(),
            },
        );

        functions.insert(
            TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolGetRedemptionValueInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolGetRedemptionValueOutput>(
                        ),
                ),
                export: TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_EXPORT_NAME.to_string(),
            },
        );

        functions.insert(
            TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolGetVaultAmountsInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<TwoResourcePoolGetVaultAmountsOutput>(),
                ),
                export: TWO_RESOURCE_POOL_GET_VAULT_AMOUNTS_EXPORT_NAME.to_string(),
            },
        );

        let event_schema = event_schema! {
            aggregator,
            [
                super::events::two_resource_pool::ContributionEvent,
                super::events::two_resource_pool::RedemptionEvent,
                super::events::two_resource_pool::WithdrawEvent,
                super::events::two_resource_pool::DepositEvent
            ]
        };

        let schema = generate_full_schema(aggregator);

        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::default(),
            is_transient: false,
            dependencies: indexset!(),
            feature_set,

            schema: BlueprintSchemaInit {
                generics: vec![],
                schema,
                state,
                events: event_schema,
                types: BlueprintTypeSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit { functions },
                hooks: BlueprintHooksInit::default(),
            },

            royalty_config: PackageRoyaltyConfig::default(),
            auth_config: AuthConfig {
                function_auth: FunctionAuth::AllowAll,
                method_auth: MethodAuthTemplate::StaticRoleDefinition(roles_template! {
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
    }

    pub fn multi_resource_pool_blueprint_definition(
        _minor_version: PoolV1MinorVersion,
    ) -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let feature_set = MultiResourcePoolFeatureSet::all_features();
        let state = MultiResourcePoolStateSchemaInit::create_schema_init(&mut aggregator);

        let mut functions = index_map_new();

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
                    aggregator.add_child_type_and_descendents::<MultiResourcePoolContributeInput>(),
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
                    aggregator.add_child_type_and_descendents::<MultiResourcePoolRedeemOutput>(),
                ),
                export: MULTI_RESOURCE_POOL_REDEEM_EXPORT_NAME.to_string(),
            },
        );

        functions.insert(
            MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<MultiResourcePoolProtectedDepositInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<MultiResourcePoolProtectedDepositOutput>(
                        ),
                ),
                export: MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_EXPORT_NAME.to_string(),
            },
        );

        functions.insert(
            MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<MultiResourcePoolProtectedWithdrawInput>(
                        ),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<MultiResourcePoolProtectedWithdrawOutput>(
                        ),
                ),
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
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<MultiResourcePoolGetVaultAmountsInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<MultiResourcePoolGetVaultAmountsOutput>(),
                ),
                export: MULTI_RESOURCE_POOL_GET_VAULT_AMOUNTS_EXPORT_NAME.to_string(),
            },
        );

        let event_schema = event_schema! {
            aggregator,
            [
                super::events::multi_resource_pool::ContributionEvent,
                super::events::multi_resource_pool::RedemptionEvent,
                super::events::multi_resource_pool::WithdrawEvent,
                super::events::multi_resource_pool::DepositEvent
            ]
        };

        let schema = generate_full_schema(aggregator);

        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::default(),
            is_transient: false,
            dependencies: indexset!(),
            feature_set,

            schema: BlueprintSchemaInit {
                generics: vec![],
                schema,
                state,
                events: event_schema,
                types: BlueprintTypeSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit { functions },
                hooks: BlueprintHooksInit::default(),
            },
            royalty_config: PackageRoyaltyConfig::default(),
            auth_config: AuthConfig {
                function_auth: FunctionAuth::AllowAll,
                method_auth: MethodAuthTemplate::StaticRoleDefinition(roles_template! {
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
    }
}
