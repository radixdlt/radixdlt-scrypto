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
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::system_modules::virtualization::VirtualLazyLoadInput;
use radix_engine_interface::api::system_modules::virtualization::VirtualLazyLoadOutput;
use radix_engine_interface::api::CollectionIndex;
use radix_engine_interface::api::{ClientApi, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::resource::{Bucket, Proof};
use radix_engine_interface::metadata_init;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, Clone)]
pub struct AccountSubstate {
    pub default_deposit_rule: AccountDefaultDepositRule,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AccountError {
    VaultDoesNotExist {
        resource_address: ResourceAddress,
    },
    AccountIsNotInAllowListDepositsMode {
        default_deposit_rule: AccountDefaultDepositRule,
    },
    AccountIsNotInDisallowListDepositsMode {
        default_deposit_rule: AccountDefaultDepositRule,
    },
    DepositIsDisallowed {
        resource_address: ResourceAddress,
    },
    NotAllBucketsCouldBeDeposited,
}

impl From<AccountError> for RuntimeError {
    fn from(value: AccountError) -> Self {
        Self::ApplicationError(ApplicationError::AccountError(value))
    }
}

pub const SECURIFY_ROLE: &'static str = "securify";

struct SecurifiedAccount;

impl SecurifiedAccessRules for SecurifiedAccount {
    type OwnerBadgeNonFungibleData = AccountOwnerBadgeData;
    const OWNER_BADGE: ResourceAddress = ACCOUNT_OWNER_BADGE;
    const SECURIFY_ROLE: Option<&'static str> = Some(SECURIFY_ROLE);
}

impl PresecurifiedAccessRules for SecurifiedAccount {}

pub const ACCOUNT_VAULT_INDEX: CollectionIndex = 0u8;
pub type AccountVaultIndexEntry = Option<Own>;

pub const ACCOUNT_RESOURCE_DEPOSIT_CONFIGURATION_INDEX: CollectionIndex = 1u8;
pub type AccountResourceDepositRuleEntry = Option<ResourceDepositRule>;

pub struct AccountBlueprint;

impl AccountBlueprint {
    fn create_modules<Y>(
        access_rules: AccessRules,
        metadata_init: MetadataInit,
        api: &mut Y,
    ) -> Result<BTreeMap<ObjectModuleId, Own>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let metadata = Metadata::create_with_data(metadata_init, api)?;
        let royalty = ComponentRoyalty::create(ComponentRoyaltyConfig::default(), api)?;

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
        let public_key_hash = PublicKeyHash::Secp256k1(Secp256k1PublicKeyHash(input.id));
        Self::create_virtual(public_key_hash, api)
    }

    pub fn create_virtual_ed25519<Y>(
        input: VirtualLazyLoadInput,
        api: &mut Y,
    ) -> Result<VirtualLazyLoadOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let public_key_hash = PublicKeyHash::Ed25519(Ed25519PublicKeyHash(input.id));
        Self::create_virtual(public_key_hash, api)
    }

    fn create_virtual<Y>(
        public_key_hash: PublicKeyHash,
        api: &mut Y,
    ) -> Result<VirtualLazyLoadOutput, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let owner_badge = {
            let bytes = public_key_hash.get_hash_bytes();
            let entity_type = match public_key_hash {
                PublicKeyHash::Ed25519(..) => EntityType::GlobalVirtualEd25519Account,
                PublicKeyHash::Secp256k1(..) => EntityType::GlobalVirtualSecp256k1Account,
            };

            let mut id_bytes = vec![entity_type as u8];
            id_bytes.extend(bytes);

            NonFungibleLocalId::bytes(id_bytes).unwrap()
        };

        let account = Self::create_local(api)?;
        let owner_id = NonFungibleGlobalId::from_public_key_hash(public_key_hash);
        let access_rules = SecurifiedAccount::create_presecurified(owner_id, api)?;
        let mut modules = Self::create_modules(
            access_rules,
            metadata_init!(
                // NOTE:
                // This is the owner key for ROLA. We choose to set this explicitly to simplify the
                // security-critical logic off-ledger. In particular, we want an owner to be able to
                // explicitly delete the owner keys. If we went with a "no metadata = assume default
                // public key hash", then this could cause unexpected security-critical behavior if
                // a user expected that deleting the metadata removed the owner keys.
                "owner_keys" => vec![public_key_hash], updatable;
                "owner_badge" => owner_badge, locked;
            ),
            api,
        )?;

        modules.insert(ObjectModuleId::Main, account);

        Ok(modules)
    }

    pub fn securify<Y>(receiver: &NodeId, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let owner_badge_data = AccountOwnerBadgeData {
            name: "Account Owner Badge".into(),
            account: ComponentAddress::new_or_panic(receiver.0),
        };
        SecurifiedAccount::securify(
            receiver,
            owner_badge_data,
            Some(NonFungibleLocalId::bytes(receiver.0).unwrap()),
            api,
        )
    }

    pub fn create_advanced<Y>(
        owner_rule: OwnerRole,
        api: &mut Y,
    ) -> Result<GlobalAddress, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let account = Self::create_local(api)?;
        let access_rules = SecurifiedAccount::create_advanced(owner_rule, api)?;
        let mut modules = Self::create_modules(
            access_rules,
            metadata_init!(
                "owner_badge" => EMPTY, locked;
            ),
            api,
        )?;
        modules.insert(ObjectModuleId::Main, account);
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();

        let address = api.globalize(modules, None)?;

        Ok(address)
    }

    pub fn create<Y>(api: &mut Y) -> Result<(GlobalAddress, Bucket), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let (address_reservation, address) = api.allocate_global_address(BlueprintId {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
        })?;

        let account = Self::create_local(api)?;
        let (access_rules, bucket) = SecurifiedAccount::create_securified(
            AccountOwnerBadgeData {
                name: "Account Owner Badge".into(),
                account: address.try_into().expect("Impossible Case"),
            },
            Some(NonFungibleLocalId::bytes(address.as_node_id().0).unwrap()),
            api,
        )?;
        let mut modules = Self::create_modules(
            access_rules,
            metadata_init! {
                "owner_badge" => NonFungibleLocalId::bytes(address.as_node_id().0).unwrap(), locked;
            },
            api,
        )?;
        modules.insert(ObjectModuleId::Main, account);
        let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();

        let address = api.globalize(modules, Some(address_reservation))?;

        Ok((address, bucket))
    }

    fn create_local<Y>(api: &mut Y) -> Result<Own, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let account_id = api.new_object(
            ACCOUNT_BLUEPRINT,
            vec![],
            None,
            vec![scrypto_encode(&AccountSubstate {
                default_deposit_rule: AccountDefaultDepositRule::Accept,
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

    /// Method requires auth - if call goes through it performs the deposit with no questions asked
    pub fn deposit<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = bucket.resource_address(api)?;
        Self::get_vault(
            resource_address,
            |vault, api| vault.put(bucket, api),
            true,
            api,
        )?;
        Ok(())
    }

    /// Method requires auth - if call goes through it performs the deposit with no questions asked
    pub fn deposit_batch<Y>(buckets: Vec<Bucket>, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        for bucket in buckets {
            Self::deposit(bucket, api)?;
        }
        Ok(())
    }

    /// Method is public to all - if the resource can't be deposited it is returned.
    pub fn try_deposit_or_refund<Y>(
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<Option<Bucket>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = bucket.resource_address(api)?;

        let is_deposit_allowed = Self::is_deposit_allowed(&resource_address, api)?;
        if is_deposit_allowed {
            Self::get_vault(
                resource_address,
                |vault, api| vault.put(bucket, api),
                true,
                api,
            )?;
            Ok(None)
        } else {
            Ok(Some(bucket))
        }
    }

    /// Method is public to all - if ANY of the resources can't be deposited then ALL are returned.
    pub fn try_deposit_batch_or_refund<Y>(
        buckets: Vec<Bucket>,
        api: &mut Y,
    ) -> Result<Vec<Bucket>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let can_all_be_deposited = buckets
            .iter()
            .map(|bucket| {
                bucket
                    .resource_address(api)
                    .and_then(|resource_address| Self::is_deposit_allowed(&resource_address, api))
            })
            .all(|item| item == Ok(true));

        if can_all_be_deposited {
            Self::deposit_batch(buckets, api)?;
            Ok(vec![])
        } else {
            Ok(buckets)
        }
    }

    /// Method is public to all - if the resources can't be deposited then the execution panics.
    pub fn try_deposit_or_abort<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if let Some(bucket) = Self::try_deposit_or_refund(bucket, api)? {
            let resource_address = bucket.resource_address(api)?;
            Err(AccountError::DepositIsDisallowed { resource_address }.into())
        } else {
            Ok(())
        }
    }

    /// Method is public to all - if ANY of the resources can't be deposited then the execution
    /// panics.
    pub fn try_deposit_batch_or_abort<Y>(
        buckets: Vec<Bucket>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let buckets = Self::try_deposit_batch_or_refund(buckets, api)?;
        if buckets.len() != 0 {
            Err(AccountError::NotAllBucketsCouldBeDeposited.into())
        } else {
            Ok(())
        }
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

    pub fn burn<Y>(
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::get_vault(
            resource_address,
            |vault, api| vault.burn(amount, api),
            false,
            api,
        )
    }

    pub fn burn_non_fungibles<Y>(
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::get_vault(
            resource_address,
            |vault, api| vault.burn_non_fungibles(ids, api),
            false,
            api,
        )
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

    pub fn change_account_default_deposit_rule<Y>(
        default_deposit_rule: AccountDefaultDepositRule,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let substate_key = AccountField::Account.into();
        let handle = api.actor_open_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::MUTABLE)?;
        let mut account = api.field_lock_read_typed::<AccountSubstate>(handle)?;

        account.default_deposit_rule = default_deposit_rule;

        api.field_lock_write_typed(handle, account)?;
        api.field_lock_release(handle)?;

        Ok(())
    }

    pub fn configure_resource_deposit_rule<Y>(
        resource_address: ResourceAddress,
        resource_deposit_configuration: ResourceDepositRule,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        match resource_deposit_configuration {
            ResourceDepositRule::Allowed | ResourceDepositRule::Disallowed => {
                let kv_store_entry_lock_handle = api.actor_open_key_value_entry(
                    OBJECT_HANDLE_SELF,
                    ACCOUNT_RESOURCE_DEPOSIT_CONFIGURATION_INDEX,
                    &encoded_key,
                    LockFlags::MUTABLE,
                )?;

                api.key_value_entry_set_typed(
                    kv_store_entry_lock_handle,
                    &resource_deposit_configuration,
                )?;

                api.key_value_entry_release(kv_store_entry_lock_handle)?;
            }
            ResourceDepositRule::Neither => {
                api.actor_remove_key_value_entry(
                    OBJECT_HANDLE_SELF,
                    ACCOUNT_RESOURCE_DEPOSIT_CONFIGURATION_INDEX,
                    &encoded_key,
                )?;
            }
        };
        Ok(())
    }

    fn get_account_default_deposit_rule<Y>(
        api: &mut Y,
    ) -> Result<AccountDefaultDepositRule, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let substate_key = AccountField::Account.into();
        let handle =
            api.actor_open_field(OBJECT_HANDLE_SELF, substate_key, LockFlags::read_only())?;
        let account = api.field_lock_read_typed::<AccountSubstate>(handle)?;
        let default_deposit_rule = account.default_deposit_rule;
        api.field_lock_release(handle)?;

        Ok(default_deposit_rule)
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

        let kv_store_entry_lock_handle = api.actor_open_key_value_entry(
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

                        api.key_value_entry_set_typed(kv_store_entry_lock_handle, &vault.0)?;
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
        resource_address: &ResourceAddress,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_deposit_rule =
            Self::get_resource_deposit_configuration(resource_address, api)?;

        let is_deposit_allowed = match resource_deposit_rule {
            ResourceDepositRule::Allowed => true,
            ResourceDepositRule::Disallowed => false,
            ResourceDepositRule::Neither => {
                let default_deposit_rule = Self::get_account_default_deposit_rule(api)?;
                match default_deposit_rule {
                    AccountDefaultDepositRule::Accept => true,
                    AccountDefaultDepositRule::Reject => false,
                    AccountDefaultDepositRule::AllowExisting => {
                        *resource_address == RADIX_TOKEN
                            || Self::does_vault_exist(resource_address, api)?
                    }
                }
            }
        };

        Ok(is_deposit_allowed)
    }

    fn does_vault_exist<Y>(
        resource_address: &ResourceAddress,
        api: &mut Y,
    ) -> Result<bool, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let encoded_key = scrypto_encode(resource_address).expect("Impossible Case!");

        let kv_store_entry_lock_handle = api.actor_open_key_value_entry(
            OBJECT_HANDLE_SELF,
            ACCOUNT_VAULT_INDEX,
            &encoded_key,
            LockFlags::read_only(),
        )?;

        let does_vault_exist = {
            let entry: AccountVaultIndexEntry =
                api.key_value_entry_get_typed(kv_store_entry_lock_handle)?;

            match entry {
                Option::Some(_) => true,
                Option::None => false,
            }
        };

        api.key_value_entry_release(kv_store_entry_lock_handle)?;

        Ok(does_vault_exist)
    }

    fn get_resource_deposit_configuration<Y>(
        resource_address: &ResourceAddress,
        api: &mut Y,
    ) -> Result<ResourceDepositRule, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let encoded_key = scrypto_encode(&resource_address).expect("Impossible Case!");

        let kv_store_entry_lock_handle = api.actor_open_key_value_entry(
            OBJECT_HANDLE_SELF,
            ACCOUNT_RESOURCE_DEPOSIT_CONFIGURATION_INDEX,
            &encoded_key,
            LockFlags::read_only(),
        )?;

        let resource_deposit_configuration = {
            let entry =
                api.key_value_entry_get_typed::<ResourceDepositRule>(kv_store_entry_lock_handle)?;

            match entry {
                Option::Some(resource_deposit_configuration) => resource_deposit_configuration,
                Option::None => ResourceDepositRule::Neither,
            }
        };

        api.key_value_entry_release(kv_store_entry_lock_handle)?;

        Ok(resource_deposit_configuration)
    }
}

#[derive(ScryptoSbor)]
pub struct AccountOwnerBadgeData {
    pub name: String,
    pub account: ComponentAddress,
}

impl NonFungibleData for AccountOwnerBadgeData {
    const MUTABLE_FIELDS: &'static [&'static str] = &[];
}
