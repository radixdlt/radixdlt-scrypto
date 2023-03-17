use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::node::RENodeInit;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRulesObject;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::resource::{AccessRule, Bucket, Proof};
use radix_engine_interface::blueprints::resource::AccessRulesConfig;
use radix_engine_interface::blueprints::resource::MethodKey;
use radix_engine_interface::schema::{BlueprintSchema, FunctionSchema, PackageSchema, Receiver};

use crate::system::kernel_modules::costing::FIXED_LOW_FEE;
use native_sdk::resource::{SysBucket, Vault};
use radix_engine_interface::api::types::ClientCostingReason;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct AccountSubstate {
    /// An owned [`KeyValueStore`] which maps the [`ResourceAddress`] to an [`Own`] of the vault
    /// containing that resource.
    pub vaults: Own,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AccountError {
    VaultDoesNotExist { resource_address: ResourceAddress },
}

impl From<AccountError> for RuntimeError {
    fn from(value: AccountError) -> Self {
        Self::ApplicationError(ApplicationError::AccountError(value))
    }
}

//================
// Account Create
//================

pub struct AccountNativePackage;

impl AccountNativePackage {
    pub fn schema() -> PackageSchema {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut substates = Vec::new();
        substates.push(aggregator.add_child_type_and_descendents::<AccountSubstate>());

        let mut functions = BTreeMap::new();
        functions.insert(
            ACCOUNT_CREATE_GLOBAL_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<AccountCreateGlobalInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountCreateGlobalOutput>(),
                export_name: ACCOUNT_CREATE_GLOBAL_IDENT.to_string(),
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
            ACCOUNT_LOCK_FEE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRef),
                input: aggregator.add_child_type_and_descendents::<AccountLockFeeInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountLockFeeOutput>(),
                export_name: ACCOUNT_LOCK_FEE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRef),
                input: aggregator.add_child_type_and_descendents::<AccountLockContingentFeeInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountLockContingentFeeOutput>(),
                export_name: ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_DEPOSIT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRef),
                input: aggregator.add_child_type_and_descendents::<AccountDepositInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountDepositOutput>(),
                export_name: ACCOUNT_DEPOSIT_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_DEPOSIT_BATCH_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRef),
                input: aggregator.add_child_type_and_descendents::<AccountDepositBatchInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountDepositBatchOutput>(),
                export_name: ACCOUNT_DEPOSIT_BATCH_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_WITHDRAW_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRef),
                input: aggregator.add_child_type_and_descendents::<AccountWithdrawInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountWithdrawOutput>(),
                export_name: ACCOUNT_WITHDRAW_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRef),
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
                receiver: Some(Receiver::SelfRef),
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
                receiver: Some(Receiver::SelfRef),
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
                receiver: Some(Receiver::SelfRef),
                input: aggregator.add_child_type_and_descendents::<AccountCreateProofInput>(),
                output: aggregator.add_child_type_and_descendents::<AccountCreateProofOutput>(),
                export_name: ACCOUNT_CREATE_PROOF_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_PROOF_BY_AMOUNT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRef),
                input: aggregator
                    .add_child_type_and_descendents::<AccountCreateProofByAmountInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountCreateProofByAmountOutput>(),
                export_name: ACCOUNT_CREATE_PROOF_BY_AMOUNT_IDENT.to_string(),
            },
        );

        functions.insert(
            ACCOUNT_CREATE_PROOF_BY_IDS_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRef),
                input: aggregator.add_child_type_and_descendents::<AccountCreateProofByIdsInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccountCreateProofByIdsOutput>(),
                export_name: ACCOUNT_CREATE_PROOF_BY_IDS_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        PackageSchema {
            blueprints: btreemap!(
                ACCOUNT_BLUEPRINT.to_string() => BlueprintSchema {
                    schema,
                    substates,
                    functions
                }
            ),
        }
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match export_name {
            ACCOUNT_CREATE_GLOBAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let input: AccountCreateGlobalInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;
                let rtn = Self::create_global(input.withdraw_rule, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_LOCAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }

                let _input: AccountCreateLocalInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;

                let rtn = Self::create_local(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_FEE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: AccountLockFeeInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;
                let rtn = Self::lock_fee(receiver, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_CONTINGENT_FEE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: AccountLockContingentFeeInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;

                let rtn = Self::lock_contingent_fee(receiver, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_DEPOSIT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: AccountDepositInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;

                let rtn = Self::deposit(receiver, input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_DEPOSIT_BATCH_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: AccountDepositBatchInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;

                let rtn = Self::deposit_batch(receiver, input.buckets, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_WITHDRAW_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: AccountWithdrawInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;

                let rtn = Self::withdraw(receiver, input.resource_address, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;

                let input: AccountWithdrawNonFungiblesInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;
                let rtn = Self::withdraw_non_fungibles(receiver, input.resource_address, input.ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: AccountLockFeeAndWithdrawInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;
                let rtn = Self::lock_fee_and_withdraw(receiver, input.amount_to_lock, input.resource_address, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: AccountLockFeeAndWithdrawNonFungiblesInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;
                let rtn = Self::lock_fee_and_withdraw_non_fungibles(receiver, input.amount_to_lock, input.resource_address, input.ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_PROOF_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: AccountCreateProofInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;
                let rtn = Self::create_proof(receiver, input.resource_address, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_PROOF_BY_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: AccountCreateProofByAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;
                let rtn = Self::create_proof_by_amount(receiver, input.resource_address, input.amount, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            ACCOUNT_CREATE_PROOF_BY_IDS_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                let input: AccountCreateProofByIdsInput = input.as_typed().map_err(|e| {
                    RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
                })?;
                let rtn = Self::create_proof_by_ids(receiver, input.resource_address, input.ids, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    fn create_global<Y>(withdraw_rule: AccessRule, api: &mut Y) -> Result<Address, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let account = Self::create_local(api)?;
        let access_rules =
            AccessRulesObject::sys_new(access_rules_from_withdraw_rule(withdraw_rule), api)?;
        let metadata = Metadata::sys_create(api)?;
        let royalty = ComponentRoyalty::sys_create(api, RoyaltyConfig::default())?;

        let address = api.globalize(
            RENodeId::Object(account.id()),
            btreemap!(
                NodeModuleId::AccessRules => access_rules.id(),
                NodeModuleId::Metadata => metadata.id(),
                NodeModuleId::ComponentRoyalty => royalty.id(),
            ),
        )?;

        Ok(address)
    }

    fn create_local<Y>(api: &mut Y) -> Result<Own, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let account_id = {
            // Creating the key-value-store where the vaults will be held. This is a KVStore of
            // [`ResourceAddress`] and [`Own`]ed vaults.
            let kv_store_id = {
                let node_id = api.kernel_allocate_node_id(RENodeType::KeyValueStore)?;
                let node = RENodeInit::KeyValueStore;
                api.kernel_create_node(node_id, node, BTreeMap::new())?;
                node_id
            };

            let account_substate = AccountSubstate {
                vaults: Own::KeyValueStore(kv_store_id.into()),
            };
            api.new_object(
                ACCOUNT_BLUEPRINT,
                vec![scrypto_encode(&account_substate).unwrap()],
            )?
        };

        Ok(Own::Object(account_id))
    }

    fn lock_fee_internal<Y>(
        receiver: RENodeId,
        amount: Decimal,
        contingent: bool,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resource_address = RADIX_TOKEN;
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::Account(AccountOffset::Account),
            LockFlags::read_only(),
        )?; // TODO: should this be an R or RW lock?

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let account: &AccountSubstate = api.kernel_get_substate_ref(handle)?;
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle = api.sys_lock_substate(node_id, offset, LockFlags::read_only())?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let mut vault = {
            let entry: &Option<ScryptoValue> =
                api.kernel_get_substate_ref(kv_store_entry_lock_handle)?;

            match entry {
                Option::Some(value) => Ok(scrypto_decode::<Own>(&scrypto_encode(value).unwrap())
                    .map(|own| Vault(own.vault_id()))
                    .expect("Impossible Case!")),
                Option::None => Err(AccountError::VaultDoesNotExist { resource_address }),
            }
        }?;

        // Lock fee against the vault
        if !contingent {
            vault.sys_lock_fee(api, amount)?;
        } else {
            vault.sys_lock_contingent_fee(api, amount)?;
        }

        // Drop locks (LIFO)
        api.sys_drop_lock(kv_store_entry_lock_handle)?;
        api.sys_drop_lock(handle)?;

        Ok(())
    }

    fn lock_fee<Y>(
        receiver: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        Self::lock_fee_internal(receiver, amount, false, api)?;
        Ok(())
    }

    fn lock_contingent_fee<Y>(
        receiver: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        Self::lock_fee_internal(receiver, amount, true, api)?;
        Ok(())
    }

    fn deposit<Y>(
        receiver: RENodeId,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resource_address = bucket.sys_resource_address(api)?;
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::Account(AccountOffset::Account),
            LockFlags::read_only(),
        )?;

        // Getting an RW lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let account: &AccountSubstate = api.kernel_get_substate_ref(handle)?;
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle = api.sys_lock_substate(node_id, offset, LockFlags::MUTABLE)?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then create it and
        // insert it's entry into the KVStore
        let mut vault = {
            let entry: &Option<ScryptoValue> =
                api.kernel_get_substate_ref(kv_store_entry_lock_handle)?;

            match entry {
                Option::Some(value) => scrypto_decode::<Own>(&scrypto_encode(value).unwrap())
                    .map(|own| Vault(own.vault_id()))
                    .expect("Impossible Case!"),
                Option::None => {
                    let vault = Vault::sys_new(resource_address, api)?;
                    let encoded_value = IndexedScryptoValue::from_typed(&Own::Vault(vault.0));

                    let entry: &mut Option<ScryptoValue> =
                        api.kernel_get_substate_ref_mut(kv_store_entry_lock_handle)?;
                    *entry = Option::Some(encoded_value.to_scrypto_value());
                    vault
                }
            }
        };

        // Put the bucket in the vault
        vault.sys_put(bucket, api)?;

        // Drop locks (LIFO)
        api.sys_drop_lock(kv_store_entry_lock_handle)?;
        api.sys_drop_lock(handle)?;

        Ok(())
    }

    fn deposit_batch<Y>(
        receiver: RENodeId,
        buckets: Vec<Bucket>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::Account(AccountOffset::Account),
            LockFlags::read_only(),
        )?; // TODO: should this be an R or RW lock?

        // TODO: We should optimize this a bit more so that we're not locking and unlocking the same
        // KV-store entries again and again because of buckets that have the same resource address.
        // Perhaps these should be grouped into a HashMap<ResourceAddress, Vec<Bucket>> when being
        // resolved.
        for bucket in buckets {
            let resource_address = bucket.sys_resource_address(api)?;
            let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

            // Getting an RW lock handle on the KVStore ENTRY
            let kv_store_entry_lock_handle = {
                let account: &AccountSubstate = api.kernel_get_substate_ref(handle)?;
                let kv_store_id = account.vaults.key_value_store_id();

                let node_id = RENodeId::KeyValueStore(kv_store_id);
                let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
                let handle = api.sys_lock_substate(node_id, offset, LockFlags::MUTABLE)?;
                handle
            };

            // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then create it
            // and insert it's entry into the KVStore
            let mut vault = {
                let entry: &Option<ScryptoValue> =
                    api.kernel_get_substate_ref(kv_store_entry_lock_handle)?;

                match entry {
                    Option::Some(value) => scrypto_decode::<Own>(&scrypto_encode(value).unwrap())
                        .map(|own| Vault(own.vault_id()))
                        .expect("Impossible Case!"),
                    Option::None => {
                        let vault = Vault::sys_new(resource_address, api)?;
                        let encoded_value = IndexedScryptoValue::from_typed(&Own::Vault(vault.0));

                        let entry: &mut Option<ScryptoValue> =
                            api.kernel_get_substate_ref_mut(kv_store_entry_lock_handle)?;
                        *entry = Option::Some(encoded_value.to_scrypto_value());
                        vault
                    }
                }
            };

            // Put the bucket in the vault
            vault.sys_put(bucket, api)?;

            api.sys_drop_lock(kv_store_entry_lock_handle)?;
        }

        api.sys_drop_lock(handle)?;

        Ok(())
    }

    fn get_vault<F, Y, R>(
        receiver: RENodeId,
        resource_address: ResourceAddress,
        vault_fn: F,
        api: &mut Y,
    ) -> Result<R, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
        F: FnOnce(&mut Vault, &mut Y) -> Result<R, RuntimeError>,
    {
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::Account(AccountOffset::Account),
            LockFlags::read_only(),
        )?; // TODO: should this be an R or RW lock?

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let account: &AccountSubstate = api.kernel_get_substate_ref(handle)?;
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle = api.sys_lock_substate(node_id, offset, LockFlags::read_only())?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let mut vault = {
            let entry: &Option<ScryptoValue> =
                api.kernel_get_substate_ref(kv_store_entry_lock_handle)?;

            match entry {
                Option::Some(value) => Ok(scrypto_decode::<Own>(&scrypto_encode(value).unwrap())
                    .map(|own| Vault(own.vault_id()))
                    .expect("Impossible Case!")),
                Option::None => Err(AccountError::VaultDoesNotExist { resource_address }),
            }
        }?;

        // Withdraw to bucket
        let rtn = vault_fn(&mut vault, api)?;

        // Drop locks (LIFO)
        api.sys_drop_lock(kv_store_entry_lock_handle)?;
        api.sys_drop_lock(handle)?;

        Ok(rtn)
    }

    fn withdraw<Y>(
        receiver: RENodeId,
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let bucket = Self::get_vault(
            receiver,
            resource_address,
            |vault, api| vault.sys_take(amount, api),
            api,
        )?;

        Ok(bucket)
    }

    fn withdraw_non_fungibles<Y>(
        receiver: RENodeId,
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let bucket = Self::get_vault(
            receiver,
            resource_address,
            |vault, api| vault.sys_take_non_fungibles(ids, api),
            api,
        )?;

        Ok(bucket)
    }

    fn lock_fee_and_withdraw<Y>(
        receiver: RENodeId,
        amount_to_lock: Decimal,
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        Self::lock_fee_internal(receiver, amount_to_lock, false, api)?;

        let bucket = Self::get_vault(
            receiver,
            resource_address,
            |vault, api| vault.sys_take(amount, api),
            api,
        )?;

        Ok(bucket)
    }

    fn lock_fee_and_withdraw_non_fungibles<Y>(
        receiver: RENodeId,
        amount_to_lock: Decimal,
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        Self::lock_fee_internal(receiver, amount_to_lock, false, api)?;

        let bucket = Self::get_vault(
            receiver,
            resource_address,
            |vault, api| vault.sys_take_non_fungibles(ids, api),
            api,
        )?;

        Ok(bucket)
    }

    fn create_proof<Y>(
        receiver: RENodeId,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let proof = Self::get_vault(
            receiver,
            resource_address,
            |vault, api| vault.sys_create_proof(api),
            api,
        )?;

        Ok(proof)
    }

    fn create_proof_by_amount<Y>(
        receiver: RENodeId,
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let proof = Self::get_vault(
            receiver,
            resource_address,
            |vault, api| vault.sys_create_proof_by_amount(amount, api),
            api,
        )?;

        Ok(proof)
    }

    fn create_proof_by_ids<Y>(
        receiver: RENodeId,
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {

        let proof = Self::get_vault(
            receiver,
            resource_address,
            |vault, api| vault.sys_create_proof_by_ids(ids, api),
            api,
        )?;

        Ok(proof)
    }
}

//=========
// Helpers
//=========

fn access_rules_from_withdraw_rule(withdraw_rule: AccessRule) -> AccessRulesConfig {
    let mut access_rules = AccessRulesConfig::new();
    access_rules.set_access_rule_and_mutability(
        MethodKey::new(NodeModuleId::SELF, ACCOUNT_DEPOSIT_IDENT.to_string()),
        AccessRule::AllowAll,
        AccessRule::DenyAll,
    );
    access_rules.set_access_rule_and_mutability(
        MethodKey::new(NodeModuleId::SELF, ACCOUNT_DEPOSIT_BATCH_IDENT.to_string()),
        AccessRule::AllowAll,
        AccessRule::DenyAll,
    );
    access_rules.default(withdraw_rule.clone(), withdraw_rule)
}
