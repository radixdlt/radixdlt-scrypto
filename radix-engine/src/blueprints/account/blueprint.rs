use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::substate_lock_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::resource::{AccessRulesConfig, Bucket, Proof};

use crate::blueprints::util::{MethodType, PresecurifiedAccessRules, SecurifiedAccessRules};
use native_sdk::resource::{SysBucket, Vault};
use radix_engine_interface::api::kernel_modules::virtualization::VirtualLazyLoadOutput;
use radix_engine_interface::api::object_api::ObjectModuleId;

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
    ) -> Result<BTreeMap<ObjectModuleId, Own>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let metadata = Metadata::sys_create(api)?;
        let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

        let modules = btreemap!(
            ObjectModuleId::AccessRules => access_rules.0,
            ObjectModuleId::Metadata => metadata,
            ObjectModuleId::Royalty => royalty,
        );

        Ok(modules)
    }

    pub fn create_virtual_ecdsa_256k1<Y>(
        id: [u8; NodeId::UUID_LENGTH],
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
        let mut modules = Self::create_modules(access_rules, api)?;
        modules.insert(ObjectModuleId::SELF, account);

        Ok(modules)
    }

    pub fn create_virtual_eddsa_25519<Y>(
        id: [u8; NodeId::UUID_LENGTH],
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
        let mut modules = Self::create_modules(access_rules, api)?;
        modules.insert(ObjectModuleId::SELF, account);

        Ok(modules)
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
        let mut modules = Self::create_modules(access_rules, api)?;
        modules.insert(ObjectModuleId::SELF, account);
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();

        let address = api.globalize(modules)?;

        Ok(address)
    }

    pub fn create<Y>(api: &mut Y) -> Result<(GlobalAddress, Bucket), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let account = Self::create_local(api)?;
        let (access_rules, bucket) = SecurifiedAccount::create_securified(api)?;
        let mut modules = Self::create_modules(access_rules, api)?;
        modules.insert(ObjectModuleId::SELF, account);
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();

        let address = api.globalize(modules)?;

        Ok((address, bucket))
    }

    pub fn create_local<Y>(api: &mut Y) -> Result<Own, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let account_id = api.new_object_with_schemas(
            ACCOUNT_BLUEPRINT,
            vec![],
            None,
            vec![vec![]],
        )?;

        Ok(Own(account_id))
    }

    fn lock_fee_internal<Y>(
        amount: Decimal,
        contingent: bool,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = RADIX_TOKEN;
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let handle = api.actor_lock_key_value_entry(&encoded_key, LockFlags::read_only())?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let mut vault = {
            let entry: Option<ScryptoValue> =
                api.key_value_entry_get_typed(kv_store_entry_lock_handle)?;

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
        api.key_value_entry_lock_release(kv_store_entry_lock_handle)?;

        Ok(())
    }

    pub fn lock_fee<Y>(amount: Decimal, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::lock_fee_internal(amount, false, api)?;
        Ok(())
    }

    pub fn lock_contingent_fee<Y>(amount: Decimal, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::lock_fee_internal(amount, true, api)?;
        Ok(())
    }

    pub fn deposit<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = bucket.sys_resource_address(api)?;
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        // Getting an RW lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let handle = api.actor_lock_key_value_entry(&encoded_key, LockFlags::MUTABLE)?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then create it and
        // insert it's entry into the KVStore
        let mut vault = {
            let entry: Option<ScryptoValue> =
                api.key_value_entry_get_typed(kv_store_entry_lock_handle)?;

            match entry {
                Option::Some(value) => scrypto_decode::<Own>(&scrypto_encode(&value).unwrap())
                    .map(|own| Vault(own))
                    .expect("Impossible Case!"),
                Option::None => {
                    let vault = Vault::sys_new(resource_address, api)?;
                    let encoded_value = IndexedScryptoValue::from_typed(&vault.0);

                    api.key_value_entry_set_typed(
                        kv_store_entry_lock_handle,
                        &encoded_value.to_scrypto_value(),
                    )?;
                    vault
                }
            }
        };

        // Put the bucket in the vault
        vault.sys_put(bucket, api)?;

        // Drop locks (LIFO)
        api.key_value_entry_lock_release(kv_store_entry_lock_handle)?;

        Ok(())
    }

    pub fn deposit_batch<Y>(buckets: Vec<Bucket>, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // TODO: We should optimize this a bit more so that we're not locking and unlocking the same
        // KV-store entries again and again because of buckets that have the same resource address.
        // Perhaps these should be grouped into a HashMap<ResourceAddress, Vec<Bucket>> when being
        // resolved.
        for bucket in buckets {
            let resource_address = bucket.sys_resource_address(api)?;
            let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

            // Getting an RW lock handle on the KVStore ENTRY
            let kv_store_entry_lock_handle = {
                let handle = api.actor_lock_key_value_entry(&encoded_key, LockFlags::MUTABLE)?;
                handle
            };

            // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then create it
            // and insert it's entry into the KVStore
            let mut vault = {
                let entry: Option<ScryptoValue> =
                    api.key_value_entry_get_typed(kv_store_entry_lock_handle)?;

                match entry {
                    Option::Some(value) => scrypto_decode::<Own>(&scrypto_encode(&value).unwrap())
                        .map(|own| Vault(own))
                        .expect("Impossible Case!"),
                    Option::None => {
                        let vault = Vault::sys_new(resource_address, api)?;
                        let encoded_value = IndexedScryptoValue::from_typed(&vault.0);

                        api.key_value_entry_set_typed(
                            kv_store_entry_lock_handle,
                            &encoded_value.to_scrypto_value(),
                        )?;
                        vault
                    }
                }
            };

            // Put the bucket in the vault
            vault.sys_put(bucket, api)?;

            api.key_value_entry_lock_release(kv_store_entry_lock_handle)?;
        }

        Ok(())
    }

    fn get_vault<F, Y, R>(
        resource_address: ResourceAddress,
        vault_fn: F,
        api: &mut Y,
    ) -> Result<R, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
        F: FnOnce(&mut Vault, &mut Y) -> Result<R, RuntimeError>,
    {
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let handle = api.actor_lock_key_value_entry(&encoded_key, LockFlags::read_only())?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then error out.
        let mut vault = {
            let entry: Option<ScryptoValue> =
                api.key_value_entry_get_typed(kv_store_entry_lock_handle)?;

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
        api.key_value_entry_lock_release(kv_store_entry_lock_handle)?;

        Ok(rtn)
    }

    pub fn withdraw<Y>(
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let bucket = Self::get_vault(
            resource_address,
            |vault, api| vault.sys_take(amount, api),
            api,
        )?;

        Ok(bucket)
    }

    pub fn withdraw_non_fungibles<Y>(
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let bucket = Self::get_vault(
            resource_address,
            |vault, api| vault.sys_take_non_fungibles(ids, api),
            api,
        )?;

        Ok(bucket)
    }

    pub fn lock_fee_and_withdraw<Y>(
        amount_to_lock: Decimal,
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::lock_fee_internal(amount_to_lock, false, api)?;

        let bucket = Self::get_vault(
            resource_address,
            |vault, api| vault.sys_take(amount, api),
            api,
        )?;

        Ok(bucket)
    }

    pub fn lock_fee_and_withdraw_non_fungibles<Y>(
        amount_to_lock: Decimal,
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::lock_fee_internal(amount_to_lock, false, api)?;

        let bucket = Self::get_vault(
            resource_address,
            |vault, api| vault.sys_take_non_fungibles(ids, api),
            api,
        )?;

        Ok(bucket)
    }

    pub fn create_proof<Y>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let proof = Self::get_vault(
            resource_address,
            |vault, api| vault.sys_create_proof(api),
            api,
        )?;

        Ok(proof)
    }

    pub fn create_proof_by_amount<Y>(
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let proof = Self::get_vault(
            resource_address,
            |vault, api| vault.sys_create_proof_by_amount(amount, api),
            api,
        )?;

        Ok(proof)
    }

    pub fn create_proof_by_ids<Y>(
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let proof = Self::get_vault(
            resource_address,
            |vault, api| vault.sys_create_proof_by_ids(ids, api),
            api,
        )?;

        Ok(proof)
    }
}
