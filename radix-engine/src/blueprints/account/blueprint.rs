use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::resource::{AccessRulesConfig, Bucket, Proof};

use crate::blueprints::util::{MethodType, PresecurifiedAccessRules, SecurifiedAccessRules};
use native_sdk::resource::{SysBucket, Vault};
use radix_engine_interface::api::kernel_modules::virtualization::VirtualLazyLoadOutput;
use radix_engine_interface::schema::KeyValueStoreSchema;

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

struct SecurifiedAccount;

impl SecurifiedAccessRules for SecurifiedAccount {
    const SECURIFY_IDENT: Option<&'static str> = Some(ACCOUNT_SECURIFY_IDENT);
    const OWNER_GROUP_NAME: &'static str = "owner";
    const OWNER_TOKEN: ResourceAddress = ACCOUNT_OWNER_TOKEN;

    fn non_owner_methods() -> Vec<(&'static str, MethodType)> {
        vec![
            (ACCOUNT_DEPOSIT_IDENT, MethodType::Public),
            (ACCOUNT_DEPOSIT_BATCH_IDENT, MethodType::Public),
        ]
    }
}

impl PresecurifiedAccessRules for SecurifiedAccount {
    const PACKAGE: PackageAddress = ACCOUNT_PACKAGE;
}

pub struct AccountBlueprint;

impl AccountBlueprint {
    fn create_modules<Y>(
        access_rules: AccessRules,
        api: &mut Y,
    ) -> Result<BTreeMap<TypedModuleId, Own>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let metadata = Metadata::sys_create(api)?;
        let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

        let modules = btreemap!(
            TypedModuleId::AccessRules => access_rules.0,
            TypedModuleId::Metadata => metadata,
            TypedModuleId::Royalty => royalty,
        );

        Ok(modules)
    }

    pub fn create_virtual_ecdsa_256k1<Y>(
        id: [u8; 26],
        api: &mut Y,
    ) -> Result<VirtualLazyLoadOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let account = Self::create_local(api)?;
        let non_fungible_global_id = NonFungibleGlobalId::new(
            ECDSA_SECP256K1_TOKEN,
            NonFungibleLocalId::bytes(id.to_vec()).unwrap(),
        );
        let access_rules = SecurifiedAccount::create_presecurified(non_fungible_global_id, api)?;
        let modules = Self::create_modules(access_rules, api)?;

        Ok((account, modules))
    }

    pub fn create_virtual_eddsa_25519<Y>(
        id: [u8; 26],
        api: &mut Y,
    ) -> Result<VirtualLazyLoadOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let account = Self::create_local(api)?;
        let non_fungible_global_id = NonFungibleGlobalId::new(
            EDDSA_ED25519_TOKEN,
            NonFungibleLocalId::bytes(id.to_vec()).unwrap(),
        );
        let access_rules = SecurifiedAccount::create_presecurified(non_fungible_global_id, api)?;
        let modules = Self::create_modules(access_rules, api)?;

        Ok((account, modules))
    }

    pub fn securify<Y>(receiver: &NodeId, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        SecurifiedAccount::securify(receiver, api)
    }

    pub fn create_advanced<Y>(
        config: AccessRulesConfig,
        api: &mut Y,
    ) -> Result<GlobalAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let account = Self::create_local(api)?;
        let access_rules = SecurifiedAccount::create_advanced(config, api)?;
        let modules = Self::create_modules(access_rules, api)?;
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();

        let address = api.globalize(account.0, modules)?;

        Ok(address)
    }

    pub fn create<Y>(api: &mut Y) -> Result<(GlobalAddress, Bucket), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let account = Self::create_local(api)?;
        let (access_rules, bucket) = SecurifiedAccount::create_securified(api)?;
        let modules = Self::create_modules(access_rules, api)?;
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();

        let address = api.globalize(account.0, modules)?;

        Ok((address, bucket))
    }

    pub fn create_local<Y>(api: &mut Y) -> Result<Own, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let account_id = {
            // Creating the key-value-store where the vaults will be held. This is a KVStore of
            // [`ResourceAddress`] and [`Own`]ed vaults.
            let kv_store_id =
                api.new_key_value_store(KeyValueStoreSchema::new::<ResourceAddress, Own>(true))?;

            let account_substate = AccountSubstate {
                vaults: Own(kv_store_id),
            };
            api.new_object(
                ACCOUNT_BLUEPRINT,
                vec![scrypto_encode(&account_substate).unwrap()],
            )?
        };

        Ok(Own(account_id))
    }

    fn lock_fee_internal<Y>(
        receiver: &NodeId,
        amount: Decimal,
        contingent: bool,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resource_address = RADIX_TOKEN;
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");
        let substate_key = SubstateKey::from_vec(encoded_key).expect("Impossible Case!");

        let handle = api.sys_lock_substate(
            receiver,
            &AccountOffset::Account.into(),
            LockFlags::read_only(),
        )?; // TODO: should this be an R or RW lock?

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let account: AccountSubstate = api.sys_read_substate_typed(handle)?;
            let handle = api.sys_lock_substate(
                account.vaults.as_node_id(),
                &substate_key,
                LockFlags::read_only(),
            )?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let mut vault = {
            let entry: Option<ScryptoValue> =
                api.sys_read_substate_typed(kv_store_entry_lock_handle)?;

            match entry {
                Option::Some(value) => Ok(scrypto_decode::<Own>(&scrypto_encode(&value).unwrap())
                    .map(|own| Vault(own))
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

    pub fn lock_fee<Y>(receiver: &NodeId, amount: Decimal, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelSubstateApi + ClientApi<RuntimeError>,
    {
        Self::lock_fee_internal(receiver, amount, false, api)?;
        Ok(())
    }

    pub fn lock_contingent_fee<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelSubstateApi + ClientApi<RuntimeError>,
    {
        Self::lock_fee_internal(receiver, amount, true, api)?;
        Ok(())
    }

    pub fn deposit<Y>(receiver: &NodeId, bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resource_address = bucket.sys_resource_address(api)?;
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");
        let substate_key = SubstateKey::from_vec(encoded_key).expect("Impossible Case!");

        let handle = api.sys_lock_substate(
            receiver,
            &AccountOffset::Account.into(),
            LockFlags::read_only(),
        )?;

        // Getting an RW lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let account: AccountSubstate = api.sys_read_substate_typed(handle)?;
            let handle = api.sys_lock_substate(
                account.vaults.as_node_id(),
                &substate_key,
                LockFlags::MUTABLE,
            )?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then create it and
        // insert it's entry into the KVStore
        let mut vault = {
            let entry: Option<ScryptoValue> =
                api.sys_read_substate_typed(kv_store_entry_lock_handle)?;

            match entry {
                Option::Some(value) => scrypto_decode::<Own>(&scrypto_encode(&value).unwrap())
                    .map(|own| Vault(own))
                    .expect("Impossible Case!"),
                Option::None => {
                    let vault = Vault::sys_new(resource_address, api)?;
                    let encoded_value = IndexedScryptoValue::from_typed(&vault.0);

                    api.sys_write_substate_typed(
                        kv_store_entry_lock_handle,
                        &Some(encoded_value.to_scrypto_value()),
                    )?;
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

    pub fn deposit_batch<Y>(
        receiver: &NodeId,
        buckets: Vec<Bucket>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver,
            &AccountOffset::Account.into(),
            LockFlags::read_only(),
        )?; // TODO: should this be an R or RW lock?

        // TODO: We should optimize this a bit more so that we're not locking and unlocking the same
        // KV-store entries again and again because of buckets that have the same resource address.
        // Perhaps these should be grouped into a HashMap<ResourceAddress, Vec<Bucket>> when being
        // resolved.
        for bucket in buckets {
            let resource_address = bucket.sys_resource_address(api)?;
            let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");
            let substate_key = SubstateKey::from_vec(encoded_key).expect("Impossible Case!");

            // Getting an RW lock handle on the KVStore ENTRY
            let kv_store_entry_lock_handle = {
                let account: AccountSubstate = api.sys_read_substate_typed(handle)?;
                let handle = api.sys_lock_substate(
                    account.vaults.as_node_id(),
                    &substate_key,
                    LockFlags::MUTABLE,
                )?;
                handle
            };

            // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then create it
            // and insert it's entry into the KVStore
            let mut vault = {
                let entry: Option<ScryptoValue> =
                    api.sys_read_substate_typed(kv_store_entry_lock_handle)?;

                match entry {
                    Option::Some(value) => scrypto_decode::<Own>(&scrypto_encode(&value).unwrap())
                        .map(|own| Vault(own))
                        .expect("Impossible Case!"),
                    Option::None => {
                        let vault = Vault::sys_new(resource_address, api)?;
                        let encoded_value = IndexedScryptoValue::from_typed(&vault.0);

                        api.sys_write_substate_typed(
                            kv_store_entry_lock_handle,
                            &Some(encoded_value.to_scrypto_value()),
                        )?;
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
        receiver: &NodeId,
        resource_address: ResourceAddress,
        vault_fn: F,
        api: &mut Y,
    ) -> Result<R, RuntimeError>
    where
        Y: KernelSubstateApi + ClientApi<RuntimeError>,
        F: FnOnce(&mut Vault, &mut Y) -> Result<R, RuntimeError>,
    {
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");
        let substate_key = SubstateKey::from_vec(encoded_key).expect("Impossible Case!");

        let handle = api.sys_lock_substate(
            receiver,
            &AccountOffset::Account.into(),
            LockFlags::read_only(),
        )?; // TODO: should this be an R or RW lock?

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let account: AccountSubstate = api.sys_read_substate_typed(handle)?;
            let handle = api.sys_lock_substate(
                account.vaults.as_node_id(),
                &substate_key,
                LockFlags::read_only(),
            )?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let mut vault = {
            let entry: Option<ScryptoValue> =
                api.sys_read_substate_typed(kv_store_entry_lock_handle)?;

            match entry {
                Option::Some(value) => Ok(scrypto_decode::<Own>(&scrypto_encode(&value).unwrap())
                    .map(|own| Vault(own))
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

    pub fn withdraw<Y>(
        receiver: &NodeId,
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let bucket = Self::get_vault(
            receiver,
            resource_address,
            |vault, api| vault.sys_take(amount, api),
            api,
        )?;

        Ok(bucket)
    }

    pub fn withdraw_non_fungibles<Y>(
        receiver: &NodeId,
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let bucket = Self::get_vault(
            receiver,
            resource_address,
            |vault, api| vault.sys_take_non_fungibles(ids, api),
            api,
        )?;

        Ok(bucket)
    }

    pub fn lock_fee_and_withdraw<Y>(
        receiver: &NodeId,
        amount_to_lock: Decimal,
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelSubstateApi + ClientApi<RuntimeError>,
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

    pub fn lock_fee_and_withdraw_non_fungibles<Y>(
        receiver: &NodeId,
        amount_to_lock: Decimal,
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelSubstateApi + ClientApi<RuntimeError>,
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

    pub fn create_proof<Y>(
        receiver: &NodeId,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let proof = Self::get_vault(
            receiver,
            resource_address,
            |vault, api| vault.sys_create_proof(api),
            api,
        )?;

        Ok(proof)
    }

    pub fn create_proof_by_amount<Y>(
        receiver: &NodeId,
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let proof = Self::get_vault(
            receiver,
            resource_address,
            |vault, api| vault.sys_create_proof_by_amount(amount, api),
            api,
        )?;

        Ok(proof)
    }

    pub fn create_proof_by_ids<Y>(
        receiver: &NodeId,
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelSubstateApi + ClientApi<RuntimeError>,
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
