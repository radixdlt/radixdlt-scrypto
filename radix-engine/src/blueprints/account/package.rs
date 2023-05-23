use crate::errors::RuntimeError;
use crate::errors::SystemUpstreamError;
use crate::types::*;
use radix_engine_interface::api::kernel_modules::virtualization::VirtualLazyLoadInput;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::schema::{
    BlueprintCollectionSchema, BlueprintKeyValueStoreSchema, BlueprintSchema, FunctionSchema,
    PackageSchema, ReceiverInfo, TypeSchema, VirtualLazyLoadSchema,
};

use crate::blueprints::account::AccountBlueprint;
use crate::system::system_modules::costing::FIXED_LOW_FEE;
use radix_engine_interface::types::ClientCostingReason;
use resources_tracker_macro::trace_resources;

use super::AccountSubstate;

const ACCOUNT_CREATE_VIRTUAL_ECDSA_SECP256K1_EXPORT_NAME: &str = "create_virtual_ecdsa_secp256k1";
const ACCOUNT_CREATE_VIRTUAL_EDDSA_ED25519_EXPORT_NAME: &str = "create_virtual_ecdsa_ed25519";

pub struct AccountNativePackage;

impl AccountNativePackage {
    pub fn schema() -> PackageSchema {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut fields = Vec::new();
        fields.push(aggregator.add_child_type_and_descendents::<AccountSubstate>());

        let mut collections = Vec::new();
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeSchema::Blueprint(
                    aggregator.add_child_type_and_descendents::<ResourceAddress>(),
                ),
                value: TypeSchema::Blueprint(aggregator.add_child_type_and_descendents::<Own>()),
                can_own: true,
            },
        ));
        collections.push(BlueprintCollectionSchema::KeyValueStore(
            BlueprintKeyValueStoreSchema {
                key: TypeSchema::Blueprint(
                    aggregator.add_child_type_and_descendents::<ResourceAddress>(),
                ),
                value: TypeSchema::Blueprint(
                    aggregator.add_child_type_and_descendents::<ResourceDepositRule>(),
                ),
                can_own: false,
            },
        ));

        let mut functions = BTreeMap::new();

        functions.insert(
            ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<AccountCreateAdvancedInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountCreateAdvancedOutput>(),
                export_name: ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<AccountCreateInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountCreateOutput>(),
                export_name: ACCOUNT_CREATE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_LOCAL_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<AccountCreateLocalInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountCreateLocalOutput>(),
                export_name: ACCOUNT_CREATE_LOCAL_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_SECURIFY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AccountSecurifyInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountSecurifyOutput>(),
                export_name: ACCOUNT_SECURIFY_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_FEE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AccountLockFeeInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountLockFeeOutput>(),
                export_name: ACCOUNT_LOCK_FEE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AccountLockContingentFeeInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountLockContingentFeeOutput>(),
                export_name: ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_DEPOSIT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AccountDepositInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountDepositOutput>(),
                export_name: ACCOUNT_DEPOSIT_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_DEPOSIT_BATCH_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AccountDepositBatchInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountDepositBatchOutput>(),
                export_name: ACCOUNT_DEPOSIT_BATCH_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_WITHDRAW_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator.add_child_type_and_descendents::<AccountWithdrawInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountWithdrawOutput>(),
                export_name: ACCOUNT_WITHDRAW_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountWithdrawNonFungiblesInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountWithdrawNonFungiblesOutput>(),
                export_name: ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountLockFeeAndWithdrawInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountLockFeeAndWithdrawOutput>(),
                export_name: ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountLockFeeAndWithdrawNonFungiblesInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountLockFeeAndWithdrawNonFungiblesOutput>(
                    ),
                export_name: ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_PROOF_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: aggregator.add_child_type_and_descendents::<AccountCreateProofInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountCreateProofOutput>(),
                export_name: ACCOUNT_CREATE_PROOF_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountCreateProofOfAmountInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountCreateProofOfAmountOutput>(),
                export_name: ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountCreateProofOfNonFungiblesInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountCreateProofOfNonFungiblesOutput>(),
                export_name: ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CHANGE_DEFAULT_DEPOSIT_RULE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountChangeDefaultDepositRuleInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountChangeDefaultDepositRuleOutput>(),
                export_name: ACCOUNT_CHANGE_DEFAULT_DEPOSIT_RULE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CONFIGURE_RESOURCE_DEPOSIT_RULE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountConfigureResourceDepositRuleInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountConfigureResourceDepositRuleOutput>(),
                export_name: ACCOUNT_CONFIGURE_RESOURCE_DEPOSIT_RULE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_RETURN_ON_FAILURE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositReturnOnFailureInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositReturnOnFailureOutput>(),
                export_name: ACCOUNT_TRY_DEPOSIT_RETURN_ON_FAILURE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_BATCH_RETURN_ON_FAILURE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositBatchReturnOnFailureInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositBatchReturnOnFailureOutput>(
                    ),
                export_name: ACCOUNT_TRY_DEPOSIT_BATCH_RETURN_ON_FAILURE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_ABORT_ON_FAILURE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositAbortOnFailureInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositAbortOnFailureOutput>(),
                export_name: ACCOUNT_TRY_DEPOSIT_ABORT_ON_FAILURE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_TRY_DEPOSIT_BATCH_ABORT_ON_FAILURE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(ReceiverInfo::normal_ref_mut()),
                input: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositBatchAbortOnFailureInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountTryDepositBatchAbortOnFailureOutput>(),
                export_name: ACCOUNT_TRY_DEPOSIT_BATCH_ABORT_ON_FAILURE_IDENT.to_string(),
            },
        );

        let virtual_lazy_load_functions = btreemap!(
            ACCOUNT_CREATE_VIRTUAL_ECDSA_SECP256K1_ID => VirtualLazyLoadSchema {
                export_name: ACCOUNT_CREATE_VIRTUAL_ECDSA_SECP256K1_EXPORT_NAME.to_string(),
            },
            ACCOUNT_CREATE_VIRTUAL_EDDSA_ED25519_ID => VirtualLazyLoadSchema {
                export_name: ACCOUNT_CREATE_VIRTUAL_EDDSA_ED25519_EXPORT_NAME.to_string(),
            }
        );

        let schema = generate_full_schema(aggregator);
        PackageSchema {
            blueprints: btreemap!(
                ACCOUNT_BLUEPRINT.to_string() => BlueprintSchema {
                    outer_blueprint: None,
                    schema,
                    fields,
                    collections,
                    functions,
                    virtual_lazy_load_functions,
                    event_schema: [].into()
                }
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
        Y: ClientApi<RuntimeError>,
    {
        match export_name {
            ACCOUNT_CREATE_VIRTUAL_ECDSA_SECP256K1_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let input: VirtualLazyLoadInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::create_virtual_secp256k1(input, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_VIRTUAL_EDDSA_ED25519_EXPORT_NAME => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let input: VirtualLazyLoadInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::create_virtual_ed25519(input, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_ADVANCED_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let input: AccountCreateAdvancedInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::create_advanced(input.authority_rules, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let _input: AccountCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::create(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_LOCAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let _input: AccountCreateLocalInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::create_local(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_SECURIFY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::SystemUpstreamError(
                    SystemUpstreamError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let _input: AccountSecurifyInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::securify(receiver, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_FEE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountLockFeeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::lock_fee(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_CONTINGENT_FEE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountLockContingentFeeInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::lock_contingent_fee(input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_DEPOSIT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountDepositInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::deposit(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_DEPOSIT_BATCH_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountDepositBatchInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::deposit_batch(input.buckets, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_TRY_DEPOSIT_RETURN_ON_FAILURE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountTryDepositReturnOnFailureInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;

                let rtn = AccountBlueprint::try_deposit_return_on_failure(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_TRY_DEPOSIT_BATCH_RETURN_ON_FAILURE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountTryDepositBatchReturnOnFailureInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;

                let rtn =
                    AccountBlueprint::try_deposit_batch_return_on_failure(input.buckets, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_TRY_DEPOSIT_ABORT_ON_FAILURE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountTryDepositAbortOnFailureInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;

                let rtn = AccountBlueprint::try_deposit_abort_on_failure(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_TRY_DEPOSIT_BATCH_ABORT_ON_FAILURE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountTryDepositBatchAbortOnFailureInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                    })?;

                let rtn = AccountBlueprint::try_deposit_batch_abort_on_failure(input.buckets, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_WITHDRAW_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountWithdrawInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;

                let rtn = AccountBlueprint::withdraw(input.resource_address, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountWithdrawNonFungiblesInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::withdraw_non_fungibles(
                    input.resource_address,
                    input.ids,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountLockFeeAndWithdrawInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
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
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
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
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::create_proof(input.resource_address, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let input: AccountCreateProofOfAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
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
                        RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
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
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
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
                    RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
                })?;
                let rtn = AccountBlueprint::configure_resource_deposit_rule(
                    resource_address,
                    resource_deposit_configuration,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
