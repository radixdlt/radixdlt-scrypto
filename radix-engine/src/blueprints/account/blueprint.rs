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
use radix_engine_interface::api::kernel_modules::virtualization::VirtualLazyLoadInput;
use radix_engine_interface::api::kernel_modules::virtualization::VirtualLazyLoadOutput;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::CollectionIndex;
use radix_engine_interface::api::{ClientApi, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::resource::{require, Bucket, Proof};

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
    const OWNER_BADGE: ResourceAddress = ACCOUNT_OWNER_BADGE;
    const SECURIFY_AUTHORITY: Option<&'static str> = Some("securify");

    fn method_authorities() -> MethodAuthorities {
        let mut method_authorities = MethodAuthorities::new();
        method_authorities.set_main_method_authority(ACCOUNT_SECURIFY_IDENT, "securify");
        method_authorities.set_main_method_authority(ACCOUNT_LOCK_FEE_IDENT, "lock_fee");
        method_authorities.set_main_method_authority(ACCOUNT_LOCK_CONTINGENT_FEE_IDENT, "lock_fee");
        method_authorities.set_main_method_authority(
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT,
            "lock_fee_and_withdraw",
        );
        method_authorities.set_main_method_authority(
            ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT,
            "lock_fee_and_withdraw",
        );
        method_authorities.set_main_method_authority(ACCOUNT_WITHDRAW_IDENT, "withdraw");
        method_authorities
            .set_main_method_authority(ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT, "withdraw");
        method_authorities.set_main_method_authority(ACCOUNT_CREATE_PROOF_IDENT, "create_proof");
        method_authorities
            .set_main_method_authority(ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT, "create_proof");
        method_authorities
            .set_main_method_authority(ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT, "create_proof");
        method_authorities
    }

    fn authority_rules() -> AuthorityRules {
        let mut authority_rules = AuthorityRules::new();
        authority_rules.set_main_authority_rule(
            "lock_fee",
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            "withdraw",
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            "create_proof",
            rule!(require_owner()),
            rule!(deny_all),
        );
        authority_rules.set_main_authority_rule(
            "lock_fee_and_withdraw",
            rule!(require("lock_fee") && require("withdraw")),
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
                    value: scrypto_decode(
                        &scrypto_encode(&MetadataEntry::List(vec![MetadataValue::PublicKeyHash(
                            public_key_hash,
                        )]))
                        .unwrap(),
                    )
                    .unwrap(),
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
        let account_id = api.new_object(ACCOUNT_BLUEPRINT, None, vec![], btreemap!())?;

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

        // Getting a read-only lock handle on the KVStore ENTRY
        let kv_store_entry_lock_handle = {
            let handle = api.actor_lock_key_value_entry(
                OBJECT_HANDLE_SELF,
                ACCOUNT_VAULT_INDEX,
                &encoded_key,
                if create {
                    LockFlags::MUTABLE
                } else {
                    LockFlags::read_only()
                },
            )?;
            handle
        };

        // Get the vault stored in the KeyValueStore entry - if it doesn't exist, then create it if
        // instructed to.
        let mut vault = {
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
        }?;

        // Withdraw to bucket
        let rtn = vault_fn(&mut vault, api)?;

        api.key_value_entry_release(kv_store_entry_lock_handle)?;

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
            |vault, api| vault.take(amount, api),
            true,
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
            true,
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
            true,
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
            true,
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
            true,
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
            true,
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
            true,
            api,
        )?;

        Ok(proof)
    }
}
