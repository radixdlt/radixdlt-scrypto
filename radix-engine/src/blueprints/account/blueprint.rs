use crate::blueprints::util::{PresecurifiedAccessRules, SecurifiedAccessRules};
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::resource::NativeBucket;
use native_sdk::resource::NativeFungibleVault;
use native_sdk::resource::NativeNonFungibleVault;
use native_sdk::resource::NativeVault;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::kernel_modules::virtualization::VirtualLazyLoadInput;
use radix_engine_interface::api::kernel_modules::virtualization::VirtualLazyLoadOutput;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::CollectionIndex;
use radix_engine_interface::api::{ClientApi, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::resource::{require, Bucket, Proof};

#[derive(Debug, PartialEq, Eq, ScryptoSbor, Clone)]
pub struct AccountSubstate {
    deposits_mode: AccountDepositsMode,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AccountError {
    VaultDoesNotExist { resource_address: ResourceAddress },
    AccountIsNotInAllowListDepositsMode { deposits_mode: AccountDepositsMode },
    AccountIsNotInDisallowListDepositsMode { deposits_mode: AccountDepositsMode },
}

impl From<AccountError> for RuntimeError {
    fn from(value: AccountError) -> Self {
        Self::ApplicationError(ApplicationError::AccountError(value))
    }
}

const ACCOUNT_DEPOSITS_AUTHORITY: &str = "deposits_authority";

struct SecurifiedAccount;

impl SecurifiedAccessRules for SecurifiedAccount {
    const OWNER_BADGE: ResourceAddress = ACCOUNT_OWNER_BADGE;
    const SECURIFY_AUTHORITY: Option<&'static str> = Some(ACCOUNT_SECURIFY_IDENT);

    fn authority_rules() -> AuthorityRules {
        /*
        FIXME: The following is temporary until we implement the ability to map methods to roles or
        authorities. Once that's done, we would like the methods to be grouped as follows:

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
            "safe_deposit",
            "deposit_batch",
            "safe_deposit_batch",
        ]
         */
        let mut authority_rules = AuthorityRules::new();
        authority_rules.set_main_authority_rule(
            ACCOUNT_SECURIFY_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_LOCK_FEE_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_LOCK_CONTINGENT_FEE_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_WITHDRAW_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_CREATE_PROOF_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_CHANGE_ALLOWED_DEPOSITS_MODE_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_ADD_RESOURCE_TO_ALLOWED_DEPOSITS_LIST_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_REMOVE_RESOURCE_FROM_ALLOWED_DEPOSITS_LIST_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_ADD_RESOURCE_TO_DISALLOWED_DEPOSITS_LIST_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_REMOVE_RESOURCE_FROM_DISALLOWED_DEPOSITS_LIST_IDENT,
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            ACCOUNT_DEPOSITS_AUTHORITY,
            rule!(require_owner()),
            rule!(deny_all),
        );

        authority_rules
    }
}

impl PresecurifiedAccessRules for SecurifiedAccount {
    const PACKAGE: PackageAddress = ACCOUNT_PACKAGE;
}

pub const ACCOUNT_VAULT_INDEX: CollectionIndex = 0u8;

pub type AccountVaultIndexEntry = Option<Own>;

pub struct AccountBlueprint;

impl AccountBlueprint {
    fn create_modules<Y>(
        access_rules: AccessRules,
        api: &mut Y,
    ) -> Result<BTreeMap<ObjectModuleId, Own>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let metadata = Metadata::create(api)?;
        let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

        let modules = btreemap!(
            ObjectModuleId::AccessRules => access_rules.0,
            ObjectModuleId::Metadata => metadata,
            ObjectModuleId::Royalty => royalty,
        );

        Ok(modules)
    }

    pub fn create_virtual_secp256k1<Y>(
        input: VirtualLazyLoadInput,
        api: &mut Y,
    ) -> Result<VirtualLazyLoadOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let public_key_hash = PublicKeyHash::EcdsaSecp256k1(EcdsaSecp256k1PublicKeyHash(input.id));
        Self::create_virtual(public_key_hash, api)
    }

    pub fn create_virtual_ed25519<Y>(
        input: VirtualLazyLoadInput,
        api: &mut Y,
    ) -> Result<VirtualLazyLoadOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let public_key_hash = PublicKeyHash::EddsaEd25519(EddsaEd25519PublicKeyHash(input.id));
        Self::create_virtual(public_key_hash, api)
    }

    fn create_virtual<Y>(
        public_key_hash: PublicKeyHash,
        api: &mut Y,
    ) -> Result<VirtualLazyLoadOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let account = Self::create_local(api)?;
        let owner_id = NonFungibleGlobalId::from_public_key_hash(public_key_hash);
        let access_rules = SecurifiedAccount::create_presecurified(owner_id, api)?;
        let mut modules = Self::create_modules(access_rules, api)?;

        {
            // Set up metadata
            // TODO: Improve this when the Metadata module API is nicer
            let metadata = modules.get(&ObjectModuleId::Metadata).unwrap();
            // NOTE:
            // This is the owner key for ROLA.
            // We choose to set this explicitly to simplify the security-critical logic off-ledger.
            // In particular, we want an owner to be able to explicitly delete the owner keys.
            // If we went with a "no metadata = assume default public key hash", then this could cause unexpeted
            // security-critical behaviour if a user expected that deleting the metadata removed the owner keys.
            api.call_method(
                &metadata.0,
                METADATA_SET_IDENT,
                scrypto_encode(&MetadataSetInput {
                    key: "owner_keys".to_string(),
                    value: MetadataValue::PublicKeyHashArray(vec![public_key_hash]),
                })
                .unwrap(),
            )?;
        }

        modules.insert(ObjectModuleId::Main, account);

        Ok(modules)
    }

    pub fn securify<Y>(receiver: &NodeId, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        SecurifiedAccount::securify(receiver, api)
    }

    pub fn create_advanced<Y>(
        authority_rules: AuthorityRules,
        api: &mut Y,
    ) -> Result<GlobalAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let account = Self::create_local(api)?;
        let access_rules = SecurifiedAccount::create_advanced(authority_rules, api)?;
        let mut modules = Self::create_modules(access_rules, api)?;
        modules.insert(ObjectModuleId::Main, account);
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
        modules.insert(ObjectModuleId::Main, account);
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();

        let address = api.globalize(modules)?;

        Ok((address, bucket))
    }

    pub fn create_local<Y>(api: &mut Y) -> Result<Own, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let account_id = api.new_object(
            ACCOUNT_BLUEPRINT,
            None,
            vec![scrypto_encode(&AccountSubstate {
                deposits_mode: AccountDepositsMode::AllowAll,
            })
            .unwrap()],
            btreemap!(),
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

        Self::get_vault(
            resource_address,
            |vault, api| {
                if contingent {
                    vault.lock_contingent_fee(api, amount)
                } else {
                    vault.lock_fee(api, amount)
                }
            },
            false,
            api,
        )?;

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
        let resource_address = bucket.resource_address(api)?;
        let deposits_mode = Self::get_current_deposits_mode(api)?;

        let is_deposit_allowed = Self::is_deposit_allowed(&deposits_mode, &resource_address, api)?;
        if !is_deposit_allowed {
            Runtime::assert_access_rule(rule!(require(ACCOUNT_DEPOSITS_AUTHORITY)), api)?;
        }

        Self::get_vault(
            resource_address,
            |vault, api| vault.put(bucket, api),
            true,
            api,
        )?;

        Ok(())
    }

    pub fn deposit_batch<Y>(buckets: Vec<Bucket>, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        for bucket in buckets {
            Self::deposit(bucket, api)?;
        }

        Ok(())
    }

    pub fn safe_deposit<Y>(bucket: Bucket, api: &mut Y) -> Result<Option<Bucket>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = bucket.resource_address(api)?;
        let deposits_mode = Self::get_current_deposits_mode(api)?;

        let is_deposit_allowed = Self::is_deposit_allowed(&deposits_mode, &resource_address, api)?;
        if !is_deposit_allowed {
            return Ok(Some(bucket));
        }

        Self::get_vault(
            resource_address,
            |vault, api| vault.put(bucket, api),
            true,
            api,
        )?;

        Ok(None)
    }

    pub fn safe_deposit_batch<Y>(
        buckets: Vec<Bucket>,
        api: &mut Y,
    ) -> Result<Vec<Bucket>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let mut undeposited_buckets = vec![];
        for bucket in buckets {
            let rtn = Self::safe_deposit(bucket, api)?;
            if let Some(bucket) = rtn {
                undeposited_buckets.push(bucket)
            }
        }

        Ok(undeposited_buckets)
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
            |vault, api| vault.take(amount, api),
            false,
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
            |vault, api| vault.take_non_fungibles(ids, api),
            false,
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
            |vault, api| vault.take(amount, api),
            false,
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
            |vault, api| vault.take_non_fungibles(ids, api),
            false,
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
            |vault, api| vault.create_proof(api),
            false,
            api,
        )?;

        Ok(proof)
    }

    pub fn create_proof_of_amount<Y>(
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let proof = Self::get_vault(
            resource_address,
            |vault, api| vault.create_proof_of_amount(amount, api),
            false,
            api,
        )?;

        Ok(proof)
    }

    pub fn create_proof_of_non_fungibles<Y>(
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let proof = Self::get_vault(
            resource_address,
            |vault, api| vault.create_proof_of_non_fungibles(ids, api),
            false,
            api,
        )?;

        Ok(proof)
    }

    pub fn change_allowed_deposits_mode<Y>(
        deposits_mode: AccountDepositsMode,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let substate_key = AccountField::Account.into();
        let handle = api.actor_lock_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::MUTABLE)?;
        let mut account = api.field_lock_read_typed::<AccountSubstate>(handle)?;

        account.deposits_mode = deposits_mode;

        api.field_lock_write_typed(handle, account)?;
        api.field_lock_release(handle)?;

        Ok(())
    }

    pub fn add_resource_to_allowed_deposits_list<Y>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let substate_key = AccountField::Account.into();
        let handle = api.actor_lock_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::MUTABLE)?;
        let mut account = api.field_lock_read_typed::<AccountSubstate>(handle)?;

        let rtn = match &mut account.deposits_mode {
            AccountDepositsMode::AllowList(allow_list) => Ok(allow_list.insert(resource_address)),
            _ => Err(AccountError::AccountIsNotInAllowListDepositsMode {
                deposits_mode: account.deposits_mode.clone(),
            }
            .into()),
        };

        if rtn.is_ok() {
            api.field_lock_write_typed(handle, account)?;
            api.field_lock_release(handle)?;
        }

        rtn
    }

    pub fn remove_resource_from_allowed_deposits_list<Y>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let substate_key = AccountField::Account.into();
        let handle = api.actor_lock_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::MUTABLE)?;
        let mut account = api.field_lock_read_typed::<AccountSubstate>(handle)?;

        let rtn = match &mut account.deposits_mode {
            AccountDepositsMode::AllowList(allow_list) => Ok(allow_list.remove(&resource_address)),
            _ => Err(AccountError::AccountIsNotInAllowListDepositsMode {
                deposits_mode: account.deposits_mode.clone(),
            }
            .into()),
        };

        if rtn.is_ok() {
            api.field_lock_write_typed(handle, account)?;
            api.field_lock_release(handle)?;
        }

        rtn
    }

    pub fn add_resource_to_disallowed_deposits_list<Y>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let substate_key = AccountField::Account.into();
        let handle = api.actor_lock_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::MUTABLE)?;
        let mut account = api.field_lock_read_typed::<AccountSubstate>(handle)?;

        let rtn = match &mut account.deposits_mode {
            AccountDepositsMode::DisallowList(disallow_list) => {
                Ok(disallow_list.insert(resource_address))
            }
            _ => Err(AccountError::AccountIsNotInDisallowListDepositsMode {
                deposits_mode: account.deposits_mode.clone(),
            }
            .into()),
        };

        if rtn.is_ok() {
            api.field_lock_write_typed(handle, account)?;
            api.field_lock_release(handle)?;
        }

        rtn
    }

    pub fn remove_resource_from_disallowed_deposits_list<Y>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let substate_key = AccountField::Account.into();
        let handle = api.actor_lock_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::MUTABLE)?;
        let mut account = api.field_lock_read_typed::<AccountSubstate>(handle)?;

        let rtn = match &mut account.deposits_mode {
            AccountDepositsMode::DisallowList(disallow_list) => {
                Ok(disallow_list.remove(&resource_address))
            }
            _ => Err(AccountError::AccountIsNotInDisallowListDepositsMode {
                deposits_mode: account.deposits_mode.clone(),
            }
            .into()),
        };

        if rtn.is_ok() {
            api.field_lock_write_typed(handle, account)?;
            api.field_lock_release(handle)?;
        }

        rtn
    }

    fn get_current_deposits_mode<Y>(api: &mut Y) -> Result<AccountDepositsMode, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let substate_key = AccountField::Account.into();
        let handle =
            api.actor_lock_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::read_only())?;
        let account = api.field_lock_read_typed::<AccountSubstate>(handle)?;
        let deposits_mode = account.deposits_mode;
        api.field_lock_release(handle)?;

        Ok(deposits_mode)
    }

    fn get_vault<F, Y, R>(
        resource_address: ResourceAddress,
        vault_fn: F,
        create: bool,
        api: &mut Y,
    ) -> Result<R, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
        F: FnOnce(&mut Vault, &mut Y) -> Result<R, RuntimeError>,
    {
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let kv_store_entry_lock_handle = api.actor_lock_key_value_entry(
            OBJECT_HANDLE_SELF,
            ACCOUNT_VAULT_INDEX,
            &encoded_key,
            if create {
                LockFlags::MUTABLE
            } else {
                LockFlags::read_only()
            },
        )?;

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then create it if
        // instructed to.
        let vault = {
            let entry: AccountVaultIndexEntry =
                api.key_value_entry_get_typed(kv_store_entry_lock_handle)?;

            match entry {
                Option::Some(own) => Ok(Vault(own)),
                Option::None => {
                    if create {
                        let vault = Vault::create(resource_address, api)?;
                        let encoded_value = IndexedScryptoValue::from_typed(&vault.0);

                        api.key_value_entry_set_typed(
                            kv_store_entry_lock_handle,
                            &encoded_value.to_scrypto_value(),
                        )?;
                        Ok(vault)
                    } else {
                        Err(AccountError::VaultDoesNotExist { resource_address })
                    }
                }
            }
        };

        if let Ok(mut vault) = vault {
            match vault_fn(&mut vault, api) {
                Ok(rtn) => {
                    api.key_value_entry_release(kv_store_entry_lock_handle)?;
                    Ok(rtn)
                }
                Err(error) => Err(error),
            }
        } else {
            api.key_value_entry_release(kv_store_entry_lock_handle)?;
            Err(vault.unwrap_err().into())
        }
    }

    fn is_deposit_allowed<Y>(
        deposits_mode: &AccountDepositsMode,
        resource_address: &ResourceAddress,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        match deposits_mode {
            AccountDepositsMode::AllowAll => Ok(true),
            AccountDepositsMode::AllowList(ref allow_list)
                if allow_list.contains(resource_address) =>
            {
                Ok(true)
            }
            AccountDepositsMode::DisallowList(ref disallow_list)
                if !disallow_list.contains(resource_address) =>
            {
                Ok(true)
            }
            // Case: Only if the resource exists (not just that we have a vault for it). So,
            // we need to check how much of it we have. If it's more than zero then we allow
            // it.
            AccountDepositsMode::AllowExisting => {
                // Case: Deposit of XRD is always allowed in `AllowExisting` mode.
                if *resource_address == RADIX_TOKEN {
                    Ok(true)
                } else {
                    let amount_lookup_result = Self::get_vault(
                        *resource_address,
                        |vault, api| vault.amount(api),
                        false,
                        api,
                    );
                    if let Ok(amount_lookup_result) = amount_lookup_result {
                        Ok(amount_lookup_result > Decimal::zero())
                    } else if let Err(RuntimeError::ApplicationError(
                        ApplicationError::AccountError(AccountError::VaultDoesNotExist { .. }),
                    )) = amount_lookup_result
                    {
                        Ok(false)
                    } else {
                        Err(amount_lookup_result.unwrap_err())
                    }
                }
            }
            _ => Ok(false),
        }
    }
}
