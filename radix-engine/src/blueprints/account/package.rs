use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::system_modules::virtualization::VirtualLazyLoadInput;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::package::{
    AuthConfig, BlueprintDefinitionInit, BlueprintType, MethodAuthTemplate, PackageDefinition,
};
use radix_engine_interface::schema::{
    BlueprintCollectionSchema, BlueprintEventSchemaInit, BlueprintFunctionsSchemaInit,
    BlueprintKeyValueStoreSchema, BlueprintSchemaInit, BlueprintStateSchemaInit, FieldSchema,
    FunctionSchemaInit, ReceiverInfo, TypeRef,
};

use crate::blueprints::account::{AccountBlueprint, SECURIFY_ROLE};
use crate::method_auth_template;
use crate::system::system_modules::costing::FIXED_LOW_FEE;
use radix_engine_interface::types::ClientCostingReason;
use resources_tracker_macro::trace_resources;

use super::AccountSubstate;

const ACCOUNT_CREATE_VIRTUAL_SECP256K1_EXPORT_NAME: &str = "create_virtual_secp256k1";
const ACCOUNT_CREATE_VIRTUAL_ED25519_EXPORT_NAME: &str = "create_virtual_ed25519";

pub struct AccountNativePackage;

impl AccountNativePackage {
    pub fn definition() -> PackageDefinition {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(FieldSchema::static_field(
            aggregator.add_child_type_and_descendents::<AccountSubstate>(),
        ));

        let mut collections = Vec::new();
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ResourceAddress>(),
                ),
                value: TypeRef::Static(aggregator.add_child_type_and_descendents::<Own>()),
                can_own: true,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ResourceAddress>(),
                ),
                value: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<ResourceDepositRule>(),
                ),
                can_own: false,
            },
        ));

        let mut functions = BTreeMap::new();

        functions.insert(
            ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateAdvancedInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateAdvancedOutput>(),
                ),
                export: ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateOutput>(),
                ),
                export: ACCOUNT_CREATE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_LOCAL_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateLocalInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateLocalOutput>(),
                ),
                export: ACCOUNT_CREATE_LOCAL_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_SECURIFY_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountSecurifyInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountSecurifyOutput>(),
                ),
                export: ACCOUNT_SECURIFY_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_FEE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountLockFeeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountLockFeeOutput>(),
                ),
                export: ACCOUNT_LOCK_FEE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountLockContingentFeeInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountLockContingentFeeOutput>(),
                ),
                export: ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_DEPOSIT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountDepositInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountDepositOutput>(),
                ),
                export: ACCOUNT_DEPOSIT_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_DEPOSIT_BATCH_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountDepositBatchInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountDepositBatchOutput>(),
                ),
                export: ACCOUNT_DEPOSIT_BATCH_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_WITHDRAW_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountWithdrawInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountWithdrawOutput>(),
                ),
                export: ACCOUNT_WITHDRAW_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountWithdrawNonFungiblesInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountWithdrawNonFungiblesOutput>(),
                ),
                export: ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_BURN_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountBurnInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountBurnOutput>(),
                ),
                export: ACCOUNT_BURN_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_BURN_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountBurnNonFungiblesInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountBurnNonFungiblesOutput>(),
                ),
                export: ACCOUNT_BURN_NON_FUNGIBLES_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountLockFeeAndWithdrawInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountLockFeeAndWithdrawOutput>(),
                ),
                export: ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccountLockFeeAndWithdrawNonFungiblesInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccountLockFeeAndWithdrawNonFungiblesOutput>(
                    )),
                export: ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_PROOF_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateProofInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateProofOutput>(),
                ),
                export: ACCOUNT_CREATE_PROOF_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateProofOfAmountInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountCreateProofOfAmountOutput>(),
                ),
                export: ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountCreateProofOfNonFungiblesInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountCreateProofOfNonFungiblesOutput>(),
                ),
                export: ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CHANGE_DEFAULT_DEPOSIT_RULE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountChangeDefaultDepositRuleInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountChangeDefaultDepositRuleOutput>(),
                ),
                export: ACCOUNT_CHANGE_DEFAULT_DEPOSIT_RULE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CONFIGURE_RESOURCE_DEPOSIT_RULE_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccountConfigureResourceDepositRuleInput>()),
                output: TypeRef::Static(aggregator
                    .add_child_type_and_descendents::<AccountConfigureResourceDepositRuleOutput>()),
                export: ACCOUNT_CONFIGURE_RESOURCE_DEPOSIT_RULE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountTryDepositOrRefundInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountTryDepositOrRefundOutput>(),
                ),
                export: ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountTryDepositBatchOrRefundInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountTryDepositBatchOrRefundOutput>(),
                ),
                export: ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountTryDepositOrAbortInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<AccountTryDepositOrAbortOutput>(),
                ),
                export: ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT.to_string(),
            FunctionSchemaInit {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountTryDepositBatchOrAbortInput>(),
                ),
                output: TypeRef::Static(
                    aggregator
                        .add_child_type_and_descendents::<AccountTryDepositBatchOrAbortOutput>(),
                ),
                export: ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT.to_string(),
            },
        );

        let virtual_lazy_load_functions = btreemap!(
            ACCOUNT_CREATE_VIRTUAL_SECP256K1_ID => ACCOUNT_CREATE_VIRTUAL_SECP256K1_EXPORT_NAME.to_string(),
            ACCOUNT_CREATE_VIRTUAL_ED25519_ID => ACCOUNT_CREATE_VIRTUAL_ED25519_EXPORT_NAME.to_string(),
        );

        /* TODO: If we decide to add back groupings, we'd want the following:
            lock_fee: [
                "lock_fee",
                "lock_contingent_fee"
            ]
            withdraw: [
                "withdraw",
                "withdraw_non_fungibles"
            ]
            withdraw & lock fee: [
                "lock_fee_and_withdraw",
                "lock_fee_and_withdraw_non_fungibles"
            ]
            create_proof: [
                "create_proof",
                "create_proof_of_amount",
                "create_proof_of_ids",
            ]
            deposit: [
                "deposit",
                "deposit_batch",
            ]
        */

        let method_auth = method_auth_template!(
            ACCOUNT_CHANGE_DEFAULT_DEPOSIT_RULE_IDENT => [OWNER_ROLE];
            ACCOUNT_CONFIGURE_RESOURCE_DEPOSIT_RULE_IDENT => [OWNER_ROLE];
            ACCOUNT_WITHDRAW_IDENT => [OWNER_ROLE];
            ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT => [OWNER_ROLE];
            ACCOUNT_LOCK_FEE_IDENT => [OWNER_ROLE];
            ACCOUNT_LOCK_CONTINGENT_FEE_IDENT => [OWNER_ROLE];
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT => [OWNER_ROLE];
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT => [OWNER_ROLE];
            ACCOUNT_CREATE_PROOF_IDENT => [OWNER_ROLE];
            ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT => [OWNER_ROLE];
            ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT => [OWNER_ROLE];
            ACCOUNT_SECURIFY_IDENT => [SECURIFY_ROLE];
            ACCOUNT_DEPOSIT_IDENT => [OWNER_ROLE];
            ACCOUNT_DEPOSIT_BATCH_IDENT => [OWNER_ROLE];
            ACCOUNT_BURN_IDENT => [OWNER_ROLE];
            ACCOUNT_BURN_NON_FUNGIBLES_IDENT => [OWNER_ROLE];

            ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT => MethodAccessibility::Public;
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT => MethodAccessibility::Public;
            ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT => MethodAccessibility::Public;
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT => MethodAccessibility::Public;
        );

        let schema = generate_full_schema(aggregator);
        let blueprints = btreemap!(
            ACCOUNT_BLUEPRINT.to_string() => BlueprintDefinitionInit {
                blueprint_type: BlueprintType::default(),
                feature_set: btreeset!(),
                dependencies: btreeset!(
                    SECP256K1_SIGNATURE_VIRTUAL_BADGE.into(),
                    ED25519_SIGNATURE_VIRTUAL_BADGE.into(),
                    ACCOUNT_OWNER_BADGE.into(),
                    PACKAGE_OF_DIRECT_CALLER_VIRTUAL_BADGE.into(),
                ),

                schema: BlueprintSchemaInit {
                    generics: vec![],
                    schema,
                    state: BlueprintStateSchemaInit {
                        fields,
                        collections,
                    },
                    events: BlueprintEventSchemaInit::default(),
                    functions: BlueprintFunctionsSchemaInit {
                        virtual_lazy_load_functions,
                        functions,
                    },
                },

                royalty_config: RoyaltyConfig::default(),
                auth_config: AuthConfig {
                     function_auth: btreemap!(
                        ACCOUNT_CREATE_IDENT.to_string() => rule!(allow_all),
                        ACCOUNT_CREATE_LOCAL_IDENT.to_string() => rule!(allow_all),
                        ACCOUNT_CREATE_ADVANCED_IDENT.to_string() => rule!(allow_all),
                    ),
                    method_auth: MethodAuthTemplate::Static(
                        method_auth,
                    ),
                },
            }
        );

        PackageDefinition { blueprints }
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match export_name {
            ACCOUNT_CREATE_VIRTUAL_SECP256K1_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: VirtualLazyLoadInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::create_virtual_secp256k1(input, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_VIRTUAL_ED25519_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: VirtualLazyLoadInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::create_virtual_ed25519(input, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_ADVANCED_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountCreateAdvancedInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::create_advanced(input.owner_role, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: AccountCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::create(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_LOCAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let _input: AccountCreateLocalInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::create_local(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_SECURIFY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = Runtime::get_node_id(api)?;
                let _input: AccountSecurifyInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::securify(&receiver, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_FEE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountLockFeeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::lock_fee(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_CONTINGENT_FEE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountLockContingentFeeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::lock_contingent_fee(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_DEPOSIT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountDepositInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::deposit(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_DEPOSIT_BATCH_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountDepositBatchInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::deposit_batch(input.buckets, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_TRY_DEPOSIT_OR_REFUND_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountTryDepositOrRefundInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::try_deposit_or_refund(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountTryDepositBatchOrRefundInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::try_deposit_batch_or_refund(input.buckets, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_TRY_DEPOSIT_OR_ABORT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountTryDepositOrAbortInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::try_deposit_or_abort(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountTryDepositBatchOrAbortInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::try_deposit_batch_or_abort(input.buckets, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_WITHDRAW_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountWithdrawInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::withdraw(input.resource_address, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountWithdrawNonFungiblesInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::withdraw_non_fungibles(
                    input.resource_address,
                    input.ids,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_BURN_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountBurnInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::burn(input.resource_address, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_BURN_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountBurnNonFungiblesInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn =
                    AccountBlueprint::burn_non_fungibles(input.resource_address, input.ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountLockFeeAndWithdrawInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::lock_fee_and_withdraw(
                    input.amount_to_lock,
                    input.resource_address,
                    input.amount,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountLockFeeAndWithdrawNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = AccountBlueprint::lock_fee_and_withdraw_non_fungibles(
                    input.amount_to_lock,
                    input.resource_address,
                    input.ids,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_PROOF_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountCreateProofInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::create_proof(input.resource_address, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::create_proof_of_amount(
                    input.resource_address,
                    input.amount,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountCreateProofOfNonFungiblesInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = AccountBlueprint::create_proof_of_non_fungibles(
                    input.resource_address,
                    input.ids,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CHANGE_DEFAULT_DEPOSIT_RULE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let AccountChangeDefaultDepositRuleInput {
                    default_deposit_rule,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::change_account_default_deposit_rule(
                    default_deposit_rule,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CONFIGURE_RESOURCE_DEPOSIT_RULE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let AccountConfigureResourceDepositRuleInput {
                    resource_address,
                    resource_deposit_configuration,
                } = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::configure_resource_deposit_rule(
                    resource_address,
                    resource_deposit_configuration,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
