use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::kernel_api::LockFlags;
use crate::kernel::*;
use crate::system::global::GlobalAddressSubstate;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::auth::AccessRulesChainSubstate;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::component::KeyValueStoreEntrySubstate;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{GlobalAddress, NativeFn, RENodeId, SubstateOffset};
use radix_engine_interface::api::ClientNativeInvokeApi;
use radix_engine_interface::api::ClientNodeApi;
use radix_engine_interface::api::ClientSubstateApi;
use radix_engine_interface::api::{ClientApi, ClientDerefApi};
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::resource::AccessRule;
use radix_engine_interface::blueprints::resource::AccessRuleKey;
use radix_engine_interface::blueprints::resource::AccessRules;
use radix_engine_interface::blueprints::resource::Bucket;
use radix_engine_interface::blueprints::resource::Proof;

use super::AccountSubstate;
use crate::system::node_modules::metadata::MetadataSubstate;
use native_sdk::resource::{SysBucket, Vault};
use radix_engine_interface::data::ScryptoValue;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<ComponentId>,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        match export_name {
            ACCOUNT_CREATE_GLOBAL_IDENT => {
                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                Self::create_global(input, api)
            }
            ACCOUNT_CREATE_LOCAL_IDENT => {
                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                Self::create_local(input, api)
            }
            ACCOUNT_LOCK_FEE_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::lock_fee(receiver, input, api)
            }
            ACCOUNT_LOCK_CONTINGENT_FEE_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::lock_contingent_fee(receiver, input, api)
            }
            ACCOUNT_DEPOSIT_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::deposit(receiver, input, api)
            }
            ACCOUNT_DEPOSIT_BATCH_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::deposit_batch(receiver, input, api)
            }
            ACCOUNT_WITHDRAW_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::withdraw(receiver, input, api)
            }
            ACCOUNT_WITHDRAW_ALL_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::withdraw_all(receiver, input, api)
            }
            ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::withdraw_non_fungibles(receiver, input, api)
            }
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::lock_fee_and_withdraw(receiver, input, api)
            }
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_ALL_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::lock_fee_and_withdraw_all(receiver, input, api)
            }
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::lock_fee_and_withdraw_non_fungibles(receiver, input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    fn create_global<Y>(
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: AccountCreateGlobalInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        // Creating the key-value-store where the vaults will be held. This is a KVStore of
        // [`ResourceAddress`] and [`Own`]ed vaults.
        let kv_store_id = {
            let node_id = api.allocate_node_id(RENodeType::KeyValueStore)?;
            let node = RENodeInit::KeyValueStore;
            api.create_node(node_id, node, BTreeMap::new())?;
            node_id
        };

        // Creating [`AccessRules`] from the passed withdraw access rule.
        let access_rules = access_rules_from_withdraw_rule(input.withdraw_rule);

        // Creating the Account substates and RENode
        let node_id = {
            let mut node_modules = BTreeMap::new();
            node_modules.insert(
                NodeModuleId::Metadata,
                RENodeModuleInit::Metadata(MetadataSubstate {
                    metadata: BTreeMap::new(),
                }),
            );
            let access_rules_substate = AccessRulesChainSubstate {
                access_rules_chain: [access_rules].into(),
            };
            node_modules.insert(
                NodeModuleId::AccessRules,
                RENodeModuleInit::AccessRulesChain(access_rules_substate),
            );

            let account_substate = AccountSubstate {
                vaults: Own::KeyValueStore(kv_store_id.into()),
            };

            let node_id = api.allocate_node_id(RENodeType::Account)?;
            let node = RENodeInit::Account(account_substate);
            api.create_node(node_id, node, node_modules)?;
            node_id
        };

        // Creating the account's global address
        let global_node_id = {
            let node = RENodeInit::Global(GlobalAddressSubstate::Account(node_id.into()));
            let node_id = api.allocate_node_id(RENodeType::GlobalAccount)?;
            api.create_node(node_id, node, BTreeMap::new())?;
            node_id
        };

        let address: ComponentAddress = global_node_id.into();
        Ok(IndexedScryptoValue::from_typed(&address))
    }

    fn create_local<Y>(
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: AccountCreateLocalInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        // Creating the key-value-store where the vaults will be held. This is a KVStore of
        // [`ResourceAddress`] and [`Own`]ed vaults.
        let kv_store_id = {
            let node_id = api.allocate_node_id(RENodeType::KeyValueStore)?;
            let node = RENodeInit::KeyValueStore;
            api.create_node(node_id, node, BTreeMap::new())?;
            node_id
        };

        // Creating [`AccessRules`] from the passed withdraw access rule.
        let access_rules = access_rules_from_withdraw_rule(input.withdraw_rule);

        // Creating the Account substates and RENode
        let node_id = {
            let mut node_modules = BTreeMap::new();
            node_modules.insert(
                NodeModuleId::Metadata,
                RENodeModuleInit::Metadata(MetadataSubstate {
                    metadata: BTreeMap::new(),
                }),
            );
            let access_rules_substate = AccessRulesChainSubstate {
                access_rules_chain: [access_rules].into(),
            };
            node_modules.insert(
                NodeModuleId::AccessRules,
                RENodeModuleInit::AccessRulesChain(access_rules_substate),
            );
            let account_substate = AccountSubstate {
                vaults: Own::KeyValueStore(kv_store_id.into()),
            };

            let node_id = api.allocate_node_id(RENodeType::Account)?;
            let node = RENodeInit::Account(account_substate);
            api.create_node(node_id, node, node_modules)?;
            node_id
        };

        // TODO: Verify this is correct
        let component_id: AccountId = node_id.into();
        Ok(IndexedScryptoValue::from_typed(&Own::Account(component_id)))
    }

    fn lock_fee_internal<Y>(
        receiver: ComponentId,
        amount: Decimal,
        contingent: bool,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        let resource_address = RADIX_TOKEN;
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let handle = api.lock_substate(
            RENodeId::Account(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Account(AccountOffset::Account),
            LockFlags::read_only(),
        )?; // TODO: should this be an R or RW lock?

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.get_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle =
                api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let mut vault = {
            let substate = api.get_ref(kv_store_entry_lock_handle)?;
            let entry = substate.kv_store_entry();

            match entry {
                KeyValueStoreEntrySubstate::Some(_, value) => {
                    Ok(scrypto_decode::<Own>(&scrypto_encode(value).unwrap())
                        .map(|own| Vault(own.vault_id()))
                        .expect("Impossible Case!"))
                }
                KeyValueStoreEntrySubstate::None => {
                    Err(AccountError::VaultDoesNotExist { resource_address })
                }
            }
        }?;

        // Lock fee against the vault
        if !contingent {
            vault.sys_lock_fee(api, amount)?;
        } else {
            vault.sys_lock_contingent_fee(api, amount)?;
        }

        // Drop locks (LIFO)
        api.drop_lock(kv_store_entry_lock_handle)?;
        api.drop_lock(handle)?;

        Ok(())
    }

    fn lock_fee<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: AccountLockFeeInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        Self::lock_fee_internal(receiver, input.amount, false, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn lock_contingent_fee<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: AccountLockContingentFeeInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        Self::lock_fee_internal(receiver, input.amount, true, api)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn deposit<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: AccountDepositInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let resource_address = input.bucket.sys_resource_address(api)?;
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let handle = api.lock_substate(
            RENodeId::Account(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Account(AccountOffset::Account),
            LockFlags::read_only(),
        )?;

        // Getting an RW lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.get_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle =
                api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then create it and
        // insert it's entry into the KVStore
        let mut vault = {
            let substate = api.get_ref(kv_store_entry_lock_handle)?;
            let entry = substate.kv_store_entry();

            match entry {
                KeyValueStoreEntrySubstate::Some(_, value) => {
                    scrypto_decode::<Own>(&scrypto_encode(value).unwrap())
                        .map(|own| Vault(own.vault_id()))
                        .expect("Impossible Case!")
                }
                KeyValueStoreEntrySubstate::None => {
                    let vault = Vault::sys_new(resource_address, api)?;
                    let encoded_key = IndexedScryptoValue::from_typed(&resource_address);
                    let encoded_value = IndexedScryptoValue::from_typed(&Own::Vault(vault.0));

                    let mut substate = api.get_ref_mut(kv_store_entry_lock_handle)?;
                    let entry = substate.kv_store_entry();
                    *entry =
                        KeyValueStoreEntrySubstate::Some(encoded_key.into(), encoded_value.into());

                    vault
                }
            }
        };

        // Put the bucket in the vault
        vault.sys_put(input.bucket, api)?;

        // Drop locks (LIFO)
        api.drop_lock(kv_store_entry_lock_handle)?;
        api.drop_lock(handle)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn deposit_batch<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: AccountDepositBatchInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.lock_substate(
            RENodeId::Account(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Account(AccountOffset::Account),
            LockFlags::read_only(),
        )?; // TODO: should this be an R or RW lock?

        // TODO: We should optimize this a bit more so that we're not locking and unlocking the same
        // KV-store entries again and again because of buckets that have the same resource address.
        // Perhaps these should be grouped into a HashMap<ResourceAddress, Vec<Bucket>> when being
        // resolved.
        for bucket in input.buckets {
            let resource_address = bucket.sys_resource_address(api)?;
            let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

            // Getting an RW lock handle on the KVStore ENTRY
            let kv_store_entry_lock_handle = {
                let substate = api.get_ref(handle)?;
                let account = substate.account();
                let kv_store_id = account.vaults.key_value_store_id();

                let node_id = RENodeId::KeyValueStore(kv_store_id);
                let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
                let handle =
                    api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;
                handle
            };

            // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then create it
            // and insert it's entry into the KVStore
            let mut vault = {
                let substate = api.get_ref(kv_store_entry_lock_handle)?;
                let entry = substate.kv_store_entry();

                match entry {
                    KeyValueStoreEntrySubstate::Some(_, value) => {
                        scrypto_decode::<Own>(&scrypto_encode(value).unwrap())
                            .map(|own| Vault(own.vault_id()))
                            .expect("Impossible Case!")
                    }
                    KeyValueStoreEntrySubstate::None => {
                        let vault = Vault::sys_new(resource_address, api)?;
                        let encoded_key = IndexedScryptoValue::from_typed(&resource_address);
                        let encoded_value = IndexedScryptoValue::from_typed(&Own::Vault(vault.0));

                        let mut substate = api.get_ref_mut(kv_store_entry_lock_handle)?;
                        let entry = substate.kv_store_entry();
                        *entry = KeyValueStoreEntrySubstate::Some(
                            encoded_key.into(),
                            encoded_value.into(),
                        );

                        vault
                    }
                }
            };

            // Put the bucket in the vault
            vault.sys_put(bucket, api)?;

            api.drop_lock(kv_store_entry_lock_handle)?;
        }

        api.drop_lock(handle)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn withdraw_internal<Y, F>(
        receiver: ComponentId,
        resource_address: ResourceAddress,
        vault_fn: F,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
        F: FnOnce(&mut Vault, &mut Y) -> Result<Bucket, RuntimeError>,
    {
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let handle = api.lock_substate(
            RENodeId::Account(receiver),
            NodeModuleId::SELF,
            SubstateOffset::Account(AccountOffset::Account),
            LockFlags::read_only(),
        )?; // TODO: should this be an R or RW lock?

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.get_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle =
                api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let mut vault = {
            let substate = api.get_ref(kv_store_entry_lock_handle)?;
            let entry = substate.kv_store_entry();

            match entry {
                KeyValueStoreEntrySubstate::Some(_, value) => {
                    Ok(scrypto_decode::<Own>(&scrypto_encode(value).unwrap())
                        .map(|own| Vault(own.vault_id()))
                        .expect("Impossible Case!"))
                }
                KeyValueStoreEntrySubstate::None => {
                    Err(AccountError::VaultDoesNotExist { resource_address })
                }
            }
        }?;

        // Withdraw to bucket
        let bucket = vault_fn(&mut vault, api)?;

        // Drop locks (LIFO)
        api.drop_lock(kv_store_entry_lock_handle)?;
        api.drop_lock(handle)?;

        Ok(IndexedScryptoValue::from_typed(&bucket))
    }

    fn withdraw<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: AccountWithdrawInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        Self::withdraw_internal(
            receiver,
            input.resource_address,
            |vault, api| vault.sys_take(input.amount, api),
            api,
        )
    }

    fn withdraw_all<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: AccountWithdrawAllInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        Self::withdraw_internal(
            receiver,
            input.resource_address,
            |vault, api| vault.sys_take_all(api),
            api,
        )
    }

    fn withdraw_non_fungibles<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: AccountWithdrawNonFungiblesInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        Self::withdraw_internal(
            receiver,
            input.resource_address,
            |vault, api| vault.sys_take_non_fungibles(input.ids, api),
            api,
        )
    }

    fn lock_fee_and_withdraw<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: AccountLockFeeAndWithdrawInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        Self::lock_fee_internal(receiver, input.amount_to_lock, false, api)?;

        Self::withdraw_internal(
            receiver,
            input.resource_address,
            |vault, api| vault.sys_take(input.amount, api),
            api,
        )
    }

    fn lock_fee_and_withdraw_all<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: AccountLockFeeAndWithdrawAllInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        Self::lock_fee_internal(receiver, input.amount_to_lock, false, api)?;

        Self::withdraw_internal(
            receiver,
            input.resource_address,
            |vault, api| vault.sys_take_all(api),
            api,
        )
    }

    fn lock_fee_and_withdraw_non_fungibles<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: AccountLockFeeAndWithdrawNonFungiblesInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        Self::lock_fee_internal(receiver, input.amount_to_lock, false, api)?;

        Self::withdraw_internal(
            receiver,
            input.resource_address,
            |vault, api| vault.sys_take_non_fungibles(input.ids, api),
            api,
        )
    }
}

//======================
// Account Create Proof
//======================

pub struct AccountCreateProofExecutable {
    pub receiver: RENodeId,
    pub resource_address: ResourceAddress,
}

impl ExecutableInvocation for AccountCreateProofInvocation {
    type Exec = AccountCreateProofExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();

        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;
        let actor =
            ResolvedActor::method(NativeFn::Account(AccountFn::CreateProof), resolved_receiver);

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            resource_address: self.resource_address,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccountCreateProofExecutable {
    type Output = Proof;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        let resource_address = self.resource_address;
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let node_id = self.receiver;
        let offset = SubstateOffset::Account(AccountOffset::Account);
        let handle = api.lock_substate(
            node_id,
            NodeModuleId::SELF,
            offset,
            LockFlags::read_only(), // TODO: should this be an R or RW lock?
        )?;

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.get_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle =
                api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let vault = {
            let substate = api.get_ref(kv_store_entry_lock_handle)?;
            let entry = substate.kv_store_entry();

            match entry {
                KeyValueStoreEntrySubstate::Some(_, value) => {
                    Ok(scrypto_decode::<Own>(&scrypto_encode(value).unwrap())
                        .map(|own| Vault(own.vault_id()))
                        .expect("Impossible Case!"))
                }
                KeyValueStoreEntrySubstate::None => {
                    Err(AccountError::VaultDoesNotExist { resource_address })
                }
            }
        }?;

        // Create Proof
        let proof = vault.sys_create_proof(api)?;

        // Drop locks (LIFO)
        api.drop_lock(kv_store_entry_lock_handle)?;
        api.drop_lock(handle)?;

        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Proof(proof.0));
        Ok((proof, call_frame_update))
    }
}

//================================
// Account Create Proof By Amount
//================================

pub struct AccountCreateProofByAmountExecutable {
    pub receiver: RENodeId,
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl ExecutableInvocation for AccountCreateProofByAmountInvocation {
    type Exec = AccountCreateProofByAmountExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();

        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;
        let actor = ResolvedActor::method(
            NativeFn::Account(AccountFn::CreateProofByAmount),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            resource_address: self.resource_address,
            amount: self.amount,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccountCreateProofByAmountExecutable {
    type Output = Proof;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        let resource_address = self.resource_address;
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let node_id = self.receiver;
        let offset = SubstateOffset::Account(AccountOffset::Account);
        let handle = api.lock_substate(
            node_id,
            NodeModuleId::SELF,
            offset,
            LockFlags::read_only(), // TODO: should this be an R or RW lock?
        )?;

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.get_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle =
                api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let vault = {
            let substate = api.get_ref(kv_store_entry_lock_handle)?;
            let entry = substate.kv_store_entry();

            match entry {
                KeyValueStoreEntrySubstate::Some(_, value) => {
                    Ok(scrypto_decode::<Own>(&scrypto_encode(value).unwrap())
                        .map(|own| Vault(own.vault_id()))
                        .expect("Impossible Case!"))
                }
                KeyValueStoreEntrySubstate::None => {
                    Err(AccountError::VaultDoesNotExist { resource_address })
                }
            }
        }?;

        // Create Proof
        let proof = vault.sys_create_proof_by_amount(api, self.amount)?;

        // Drop locks (LIFO)
        api.drop_lock(kv_store_entry_lock_handle)?;
        api.drop_lock(handle)?;

        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Proof(proof.0));
        Ok((proof, call_frame_update))
    }
}

//=============================
// Account Create Proof By Ids
//=============================

pub struct AccountCreateProofByIdsExecutable {
    pub receiver: RENodeId,
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

impl ExecutableInvocation for AccountCreateProofByIdsInvocation {
    type Exec = AccountCreateProofByIdsExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();

        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;
        let actor = ResolvedActor::method(
            NativeFn::Account(AccountFn::CreateProofByIds),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            resource_address: self.resource_address,
            ids: self.ids,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccountCreateProofByIdsExecutable {
    type Output = Proof;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>
            + ClientNodeApi<RuntimeError>,
    {
        let resource_address = self.resource_address;
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let node_id = self.receiver;
        let offset = SubstateOffset::Account(AccountOffset::Account);
        let handle =
            api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?; // TODO: should this be an R or RW lock?

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.get_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle =
                api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let vault = {
            let substate = api.get_ref(kv_store_entry_lock_handle)?;
            let entry = substate.kv_store_entry();

            match entry {
                KeyValueStoreEntrySubstate::Some(_, value) => {
                    Ok(scrypto_decode::<Own>(&scrypto_encode(value).unwrap())
                        .map(|own| Vault(own.vault_id()))
                        .expect("Impossible Case!"))
                }
                KeyValueStoreEntrySubstate::None => {
                    Err(AccountError::VaultDoesNotExist { resource_address })
                }
            }
        }?;

        // Create Proof
        let proof = vault.sys_create_proof_by_ids(api, self.ids)?;

        // Drop locks (LIFO)
        api.drop_lock(kv_store_entry_lock_handle)?;
        api.drop_lock(handle)?;

        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Proof(proof.0));
        Ok((proof, call_frame_update))
    }
}

//=========
// Helpers
//=========

fn access_rules_from_withdraw_rule(withdraw_rule: AccessRule) -> AccessRules {
    let mut access_rules = AccessRules::new();
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::ScryptoMethod(ACCOUNT_DEPOSIT_IDENT.to_string()),
        AccessRule::AllowAll,
        AccessRule::DenyAll,
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::ScryptoMethod(ACCOUNT_DEPOSIT_BATCH_IDENT.to_string()),
        AccessRule::AllowAll,
        AccessRule::DenyAll,
    );
    access_rules.default(withdraw_rule.clone(), withdraw_rule)
}

/// Gets the resource address for a bucket with the given id. Typically used when resolving
/// invocations to get the global address of the resource manager in case we need to create a vault
/// for this resource.
pub fn bucket_resource_address<Y>(
    api: &mut Y,
    bucket: BucketId,
) -> Result<ResourceAddress, RuntimeError>
where
    Y: KernelSubstateApi,
{
    let node_id = RENodeId::Bucket(bucket);
    let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
    let handle = api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;
    let substate = api.get_ref(handle)?;
    let bucket = substate.bucket();
    let resource_address = bucket.resource_address();
    api.drop_lock(handle)?;
    Ok(resource_address)
}
