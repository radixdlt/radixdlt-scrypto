use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::actor::{ResolvedActor, ResolvedReceiver};
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::interpreters::deref_and_update;
use crate::kernel::kernel_api::{
    ExecutableInvocation, Executor, KernelNodeApi, KernelSubstateApi, LockFlags,
};
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
use native_sdk::resource::Vault;
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
            ACCOUNT_CREATE_GLOBAL_IDENT => Self::create_global(input, api),
            ACCOUNT_CREATE_LOCAL_IDENT => Self::create_local(input, api),
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::InvalidInvocation,
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
            let node_id = api.kernel_allocate_node_id(RENodeType::KeyValueStore)?;
            let node = RENodeInit::KeyValueStore;
            api.kernel_create_node(node_id, node, BTreeMap::new())?;
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

            let node_id = api.kernel_allocate_node_id(RENodeType::Account)?;
            let node = RENodeInit::Account(account_substate);
            api.kernel_create_node(node_id, node, node_modules)?;
            node_id
        };

        // Creating the account's global address
        let global_node_id = {
            let node = RENodeInit::Global(GlobalAddressSubstate::Account(node_id.into()));
            let node_id = api.kernel_allocate_node_id(RENodeType::GlobalAccount)?;
            api.kernel_create_node(node_id, node, BTreeMap::new())?;
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
            let node_id = api.kernel_allocate_node_id(RENodeType::KeyValueStore)?;
            let node = RENodeInit::KeyValueStore;
            api.kernel_create_node(node_id, node, BTreeMap::new())?;
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

            let node_id = api.kernel_allocate_node_id(RENodeType::Account)?;
            let node = RENodeInit::Account(account_substate);
            api.kernel_create_node(node_id, node, node_modules)?;
            node_id
        };

        // TODO: Verify this is correct
        let component_id: AccountId = node_id.into();
        Ok(IndexedScryptoValue::from_typed(&Own::Account(component_id)))
    }
}

//==================
// Account Lock Fee
//==================

pub struct AccountLockFeeExecutable {
    pub receiver: RENodeId,
    pub amount: Decimal,
}

impl ExecutableInvocation for AccountLockFeeInvocation {
    type Exec = AccountLockFeeExecutable;

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

        let actor = ResolvedActor::method(NativeFn::Account(AccountFn::LockFee), resolved_receiver);

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            amount: self.amount,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccountLockFeeExecutable {
    type Output = ();

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
        let resource_address = RADIX_TOKEN;
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let node_id = self.receiver;
        let offset = SubstateOffset::Account(AccountOffset::Account);
        let handle =
            api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?; // TODO: should this be an R or RW lock?

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.kernel_get_substate_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle = api.kernel_lock_substate(
                node_id,
                NodeModuleId::SELF,
                offset,
                LockFlags::read_only(),
            )?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let mut vault = {
            let substate = api.kernel_get_substate_ref(kv_store_entry_lock_handle)?;
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
        vault.sys_lock_fee(api, self.amount)?;

        // Drop locks (LIFO)
        api.kernel_drop_lock(kv_store_entry_lock_handle)?;
        api.kernel_drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//=============================
// Account Lock Contingent Fee
//=============================

pub struct AccountLockContingentFeeExecutable {
    pub receiver: RENodeId,
    pub amount: Decimal,
}

impl ExecutableInvocation for AccountLockContingentFeeInvocation {
    type Exec = AccountLockContingentFeeExecutable;

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
            NativeFn::Account(AccountFn::LockContingentFee),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            amount: self.amount,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccountLockContingentFeeExecutable {
    type Output = ();

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
        let resource_address = RADIX_TOKEN;
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let node_id = self.receiver;
        let offset = SubstateOffset::Account(AccountOffset::Account);
        let handle =
            api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?; // TODO: should this be an R or RW lock?

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.kernel_get_substate_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle = api.kernel_lock_substate(
                node_id,
                NodeModuleId::SELF,
                offset,
                LockFlags::read_only(),
            )?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let mut vault = {
            let substate = api.kernel_get_substate_ref(kv_store_entry_lock_handle)?;
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
        vault.sys_lock_contingent_fee(api, self.amount)?;

        // Drop locks (LIFO)
        api.kernel_drop_lock(kv_store_entry_lock_handle)?;
        api.kernel_drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//=================
// Account Deposit
//=================

pub struct AccountDepositExecutable {
    pub receiver: RENodeId,
    pub bucket: BucketId,
}

impl ExecutableInvocation for AccountDepositInvocation {
    type Exec = AccountDepositExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError> + KernelSubstateApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(self.bucket));
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Resource(
            bucket_resource_address(deref, self.bucket)?, // Required for vault creation
        )));

        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;
        let actor = ResolvedActor::method(NativeFn::Account(AccountFn::Deposit), resolved_receiver);

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            bucket: self.bucket,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccountDepositExecutable {
    type Output = ();

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
        let resource_address = bucket_resource_address(api, self.bucket)?;
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let node_id = self.receiver;
        let offset = SubstateOffset::Account(AccountOffset::Account);
        let handle =
            api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?; // TODO: should this be an R or RW lock?

        // Getting an RW lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.kernel_get_substate_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle =
                api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then create it and
        // insert it's entry into the KVStore
        let mut vault = {
            let substate = api.kernel_get_substate_ref(kv_store_entry_lock_handle)?;
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

                    let mut substate =
                        api.kernel_get_substate_ref_mut(kv_store_entry_lock_handle)?;
                    let entry = substate.kv_store_entry();
                    *entry =
                        KeyValueStoreEntrySubstate::Some(encoded_key.into(), encoded_value.into());

                    vault
                }
            }
        };

        // Put the bucket in the vault
        vault.sys_put(Bucket(self.bucket), api)?;

        // Drop locks (LIFO)
        api.kernel_drop_lock(kv_store_entry_lock_handle)?;
        api.kernel_drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//=======================
// Account Deposit Batch
//=======================

pub struct AccountDepositBatchExecutable {
    pub receiver: RENodeId,
    pub buckets: Vec<BucketId>,
}

impl ExecutableInvocation for AccountDepositBatchInvocation {
    type Exec = AccountDepositBatchExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError> + KernelSubstateApi>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate {
            nodes_to_move: self
                .buckets
                .iter()
                .map(|bucket_id| RENodeId::Bucket(*bucket_id))
                .collect(),

            node_refs_to_copy: self
                .buckets
                .iter()
                .map(|bucket_id| {
                    bucket_resource_address(deref, *bucket_id).map(|resource_address| {
                        RENodeId::Global(GlobalAddress::Resource(resource_address))
                        // Required for vault creation
                    })
                })
                .collect::<Result<_, _>>()?,
        };

        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;
        let actor = ResolvedActor::method(
            NativeFn::Account(AccountFn::DepositBatch),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            buckets: self.buckets,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccountDepositBatchExecutable {
    type Output = ();

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
        let node_id = self.receiver;
        let offset = SubstateOffset::Account(AccountOffset::Account);
        let handle =
            api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?; // TODO: should this be an R or RW lock?

        // TODO: We should optimize this a bit more so that we're not locking and unlocking the same
        // KV-store entries again and again because of buckets that have the same resource address.
        // Perhaps these should be grouped into a HashMap<ResourceAddress, Vec<Bucket>> when being
        // resolved.
        for bucket in self.buckets {
            let resource_address = bucket_resource_address(api, bucket)?;
            let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

            // Getting an RW lock handle on the KVStore ENTRY
            let kv_store_entry_lock_handle = {
                let substate = api.kernel_get_substate_ref(handle)?;
                let account = substate.account();
                let kv_store_id = account.vaults.key_value_store_id();

                let node_id = RENodeId::KeyValueStore(kv_store_id);
                let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
                let handle = api.kernel_lock_substate(
                    node_id,
                    NodeModuleId::SELF,
                    offset,
                    LockFlags::MUTABLE,
                )?;
                handle
            };

            // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then create it
            // and insert it's entry into the KVStore
            let mut vault = {
                let substate = api.kernel_get_substate_ref(kv_store_entry_lock_handle)?;
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

                        let mut substate =
                            api.kernel_get_substate_ref_mut(kv_store_entry_lock_handle)?;
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
            vault.sys_put(Bucket(bucket), api)?;

            api.kernel_drop_lock(kv_store_entry_lock_handle)?;
        }

        api.kernel_drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//==================
// Account Withdraw
//==================

pub struct AccountWithdrawAllExecutable {
    pub receiver: RENodeId,
    pub resource_address: ResourceAddress,
}

impl ExecutableInvocation for AccountWithdrawAllInvocation {
    type Exec = AccountWithdrawAllExecutable;

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
            ResolvedActor::method(NativeFn::Account(AccountFn::WithdrawAll), resolved_receiver);

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            resource_address: self.resource_address,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccountWithdrawAllExecutable {
    type Output = Bucket;

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
            api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?; // TODO: should this be an R or RW lock?

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.kernel_get_substate_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle = api.kernel_lock_substate(
                node_id,
                NodeModuleId::SELF,
                offset,
                LockFlags::read_only(),
            )?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let mut vault = {
            let substate = api.kernel_get_substate_ref(kv_store_entry_lock_handle)?;
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
        let bucket = vault.sys_take_all(api)?;

        // Drop locks (LIFO)
        api.kernel_drop_lock(kv_store_entry_lock_handle)?;
        api.kernel_drop_lock(handle)?;

        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(bucket.0));
        Ok((bucket, call_frame_update))
    }
}

//============================
// Account Withdraw By Amount
//============================

pub struct AccountWithdrawExecutable {
    pub receiver: RENodeId,
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

impl ExecutableInvocation for AccountWithdrawInvocation {
    type Exec = AccountWithdrawExecutable;

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
            ResolvedActor::method(NativeFn::Account(AccountFn::Withdraw), resolved_receiver);

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            resource_address: self.resource_address,
            amount: self.amount,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccountWithdrawExecutable {
    type Output = Bucket;

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
            api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?; // TODO: should this be an R or RW lock?

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.kernel_get_substate_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle = api.kernel_lock_substate(
                node_id,
                NodeModuleId::SELF,
                offset,
                LockFlags::read_only(),
            )?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let mut vault = {
            let substate = api.kernel_get_substate_ref(kv_store_entry_lock_handle)?;
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
        let bucket = vault.sys_take(self.amount, api)?;

        // Drop locks (LIFO)
        api.kernel_drop_lock(kv_store_entry_lock_handle)?;
        api.kernel_drop_lock(handle)?;

        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(bucket.0));
        Ok((bucket, call_frame_update))
    }
}

//=========================
// Account Withdraw By Ids
//=========================

pub struct AccountWithdrawNonFungiblesExecutable {
    pub receiver: RENodeId,
    pub resource_address: ResourceAddress,
    pub ids: BTreeSet<NonFungibleLocalId>,
}

impl ExecutableInvocation for AccountWithdrawNonFungiblesInvocation {
    type Exec = AccountWithdrawNonFungiblesExecutable;

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
            NativeFn::Account(AccountFn::WithdrawNonFungibles),
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

impl Executor for AccountWithdrawNonFungiblesExecutable {
    type Output = Bucket;

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
            api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?; // TODO: should this be an R or RW lock?

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.kernel_get_substate_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle = api.kernel_lock_substate(
                node_id,
                NodeModuleId::SELF,
                offset,
                LockFlags::read_only(),
            )?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let mut vault = {
            let substate = api.kernel_get_substate_ref(kv_store_entry_lock_handle)?;
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
        let bucket = vault.sys_take_ids(self.ids, api)?;

        // Drop locks (LIFO)
        api.kernel_drop_lock(kv_store_entry_lock_handle)?;
        api.kernel_drop_lock(handle)?;

        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(bucket.0));
        Ok((bucket, call_frame_update))
    }
}

//===========================
// Account Withdraw And Lock
//===========================

impl ExecutableInvocation for AccountLockFeeAndWithdrawAllInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let call_frame_update =
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Component(self.receiver)));
        let actor = ResolvedActor::method(
            NativeFn::Account(AccountFn::LockFeeAndWithdrawAll),
            ResolvedReceiver {
                derefed_from: None,
                receiver: RENodeId::Global(GlobalAddress::Component(self.receiver)),
            },
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccountLockFeeAndWithdrawAllInvocation {
    type Output = Bucket;

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
        api.call_native(AccountLockFeeInvocation {
            receiver: self.receiver,
            amount: self.amount_to_lock,
        })?;
        let bucket = api.call_native(AccountWithdrawAllInvocation {
            receiver: self.receiver,
            resource_address: self.resource_address,
        })?;

        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(bucket.0));
        Ok((bucket, call_frame_update))
    }
}

//=====================================
// Account Withdraw By Amount And Lock
//=====================================

impl ExecutableInvocation for AccountLockFeeAndWithdrawInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let call_frame_update =
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Component(self.receiver)));
        let actor = ResolvedActor::method(
            NativeFn::Account(AccountFn::LockFeeAndWithdraw),
            ResolvedReceiver {
                derefed_from: None,
                receiver: RENodeId::Global(GlobalAddress::Component(self.receiver)),
            },
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccountLockFeeAndWithdrawInvocation {
    type Output = Bucket;

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
        api.call_native(AccountLockFeeInvocation {
            receiver: self.receiver,
            amount: self.amount_to_lock,
        })?;
        let bucket = api.call_native(AccountWithdrawInvocation {
            receiver: self.receiver,
            resource_address: self.resource_address,
            amount: self.amount,
        })?;

        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(bucket.0));
        Ok((bucket, call_frame_update))
    }
}

//==================================
// Account Withdraw By Ids And Lock
//==================================

impl ExecutableInvocation for AccountLockFeeAndWithdrawNonFungiblesInvocation {
    type Exec = Self;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let call_frame_update =
            CallFrameUpdate::copy_ref(RENodeId::Global(GlobalAddress::Component(self.receiver)));
        let actor = ResolvedActor::method(
            NativeFn::Account(AccountFn::LockFeeAndWithdrawNonFungibles),
            ResolvedReceiver {
                derefed_from: None,
                receiver: RENodeId::Global(GlobalAddress::Component(self.receiver)),
            },
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccountLockFeeAndWithdrawNonFungiblesInvocation {
    type Output = Bucket;

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
        api.call_native(AccountLockFeeInvocation {
            receiver: self.receiver,
            amount: self.amount_to_lock,
        })?;
        let bucket = api.call_native(AccountWithdrawNonFungiblesInvocation {
            receiver: self.receiver,
            resource_address: self.resource_address,
            ids: self.ids,
        })?;

        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(bucket.0));
        Ok((bucket, call_frame_update))
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
        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            offset,
            LockFlags::read_only(), // TODO: should this be an R or RW lock?
        )?;

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.kernel_get_substate_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle = api.kernel_lock_substate(
                node_id,
                NodeModuleId::SELF,
                offset,
                LockFlags::read_only(),
            )?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let vault = {
            let substate = api.kernel_get_substate_ref(kv_store_entry_lock_handle)?;
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
        api.kernel_drop_lock(kv_store_entry_lock_handle)?;
        api.kernel_drop_lock(handle)?;

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
        let handle = api.kernel_lock_substate(
            node_id,
            NodeModuleId::SELF,
            offset,
            LockFlags::read_only(), // TODO: should this be an R or RW lock?
        )?;

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.kernel_get_substate_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle = api.kernel_lock_substate(
                node_id,
                NodeModuleId::SELF,
                offset,
                LockFlags::read_only(),
            )?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let vault = {
            let substate = api.kernel_get_substate_ref(kv_store_entry_lock_handle)?;
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
        api.kernel_drop_lock(kv_store_entry_lock_handle)?;
        api.kernel_drop_lock(handle)?;

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
            api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?; // TODO: should this be an R or RW lock?

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let substate = api.kernel_get_substate_ref(handle)?;
            let account = substate.account();
            let kv_store_id = account.vaults.key_value_store_id();

            let node_id = RENodeId::KeyValueStore(kv_store_id);
            let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(encoded_key));
            let handle = api.kernel_lock_substate(
                node_id,
                NodeModuleId::SELF,
                offset,
                LockFlags::read_only(),
            )?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let vault = {
            let substate = api.kernel_get_substate_ref(kv_store_entry_lock_handle)?;
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
        api.kernel_drop_lock(kv_store_entry_lock_handle)?;
        api.kernel_drop_lock(handle)?;

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
        AccessRuleKey::Native(NativeFn::Account(AccountFn::Deposit)),
        AccessRule::AllowAll,
        AccessRule::DenyAll,
    );
    access_rules.set_access_rule_and_mutability(
        AccessRuleKey::Native(NativeFn::Account(AccountFn::DepositBatch)),
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
    let handle =
        api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;
    let substate = api.kernel_get_substate_ref(handle)?;
    let bucket = substate.bucket();
    let resource_address = bucket.resource_address();
    api.kernel_drop_lock(handle)?;
    Ok(resource_address)
}
