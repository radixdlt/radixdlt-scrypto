use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use radix_engine_interface::api::{ClientApi, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::FungibleResourceManagerField;
use radix_engine_interface::*;

const DIVISIBILITY_MAXIMUM: u8 = 18;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum FungibleResourceManagerError {
    InvalidAmount(Decimal, u8),
    MaxMintAmountExceeded,
    InvalidDivisibility(u8),
    DropNonEmptyBucket,
}

pub type FungibleResourceManagerDivisibilitySubstate = u8;
pub type FungibleResourceManagerTotalSupplySubstate = Decimal;

pub fn verify_divisibility(divisibility: u8) -> Result<(), RuntimeError> {
    if divisibility > DIVISIBILITY_MAXIMUM {
        return Err(RuntimeError::ApplicationError(
            ApplicationError::ResourceManagerError(
                FungibleResourceManagerError::InvalidDivisibility(divisibility),
            ),
        ));
    }

    Ok(())
}

fn check_new_amount(divisibility: u8, amount: Decimal) -> Result<(), RuntimeError> {
    if !check_fungible_amount(&amount, divisibility) {
        return Err(RuntimeError::ApplicationError(
            ApplicationError::ResourceManagerError(FungibleResourceManagerError::InvalidAmount(
                amount,
                divisibility,
            )),
        ));
    }

    // TODO: refactor this into mint function
    if amount > dec!("1000000000000000000") {
        return Err(RuntimeError::ApplicationError(
            ApplicationError::ResourceManagerError(
                FungibleResourceManagerError::MaxMintAmountExceeded,
            ),
        ));
    }

    Ok(())
}

pub struct FungibleResourceManagerBlueprint;

impl FungibleResourceManagerBlueprint {
    pub(crate) fn create<Y>(
        divisibility: u8,
        metadata: BTreeMap<String, MetadataValue>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        verify_divisibility(divisibility)?;

        let object_id = api.new_simple_object(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            vec![
                scrypto_encode(&divisibility).unwrap(),
                scrypto_encode(&Decimal::zero()).unwrap(),
            ],
        )?;

        let (address_ownership, address) = api.allocate_global_address(BlueprintId {
            package_address: RESOURCE_PACKAGE,
            blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
        })?;
        let resource_address = ResourceAddress::new_or_panic(address.into());
        globalize_resource_manager(
            object_id,
            Own(address_ownership),
            access_rules,
            metadata,
            api,
        )?;

        Ok(resource_address)
    }

    pub(crate) fn create_with_initial_supply<Y>(
        divisibility: u8,
        metadata: BTreeMap<String, MetadataValue>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        initial_supply: Decimal,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let (address_ownership, _address) = api.allocate_global_address(BlueprintId {
            package_address: RESOURCE_PACKAGE,
            blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
        })?;

        Self::create_with_initial_supply_and_address(
            divisibility,
            metadata,
            access_rules,
            initial_supply,
            Own(address_ownership),
            api,
        )
    }

    pub(crate) fn create_with_initial_supply_and_address<Y>(
        divisibility: u8,
        metadata: BTreeMap<String, MetadataValue>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        initial_supply: Decimal,
        resource_address_ownership: Own,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        verify_divisibility(divisibility)?;

        let object_id = api.new_simple_object(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            vec![
                scrypto_encode(&divisibility).unwrap(),
                scrypto_encode(&initial_supply).unwrap(),
            ],
        )?;

        check_new_amount(divisibility, initial_supply)?;

        let (resource_address, bucket) = globalize_fungible_with_initial_supply(
            object_id,
            resource_address_ownership,
            access_rules,
            metadata,
            initial_supply,
            api,
        )?;

        Ok((resource_address, bucket))
    }

    pub(crate) fn mint<Y>(amount: Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let divisibility = {
            let divisibility_handle = api.actor_lock_field(
                OBJECT_HANDLE_SELF,
                FungibleResourceManagerField::Divisibility.into(),
                LockFlags::read_only(),
            )?;
            let divisibility: u8 = api.field_lock_read_typed(divisibility_handle)?;
            divisibility
        };

        // check amount
        check_new_amount(divisibility, amount)?;

        // Update total supply
        {
            let total_supply_handle = api.actor_lock_field(
                OBJECT_HANDLE_SELF,
                FungibleResourceManagerField::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply: Decimal = api.field_lock_read_typed(total_supply_handle)?;
            total_supply += amount;
            api.field_lock_write_typed(total_supply_handle, &total_supply)?;
            api.field_lock_release(total_supply_handle)?;
        }

        let bucket = Self::create_bucket(amount, api)?;

        Runtime::emit_event(api, MintFungibleResourceEvent { amount })?;

        Ok(bucket)
    }

    pub(crate) fn burn<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        // Drop other bucket
        let other_bucket = drop_fungible_bucket(bucket.0.as_node_id(), api)?;

        // Construct the event and only emit it once all of the operations are done.
        Runtime::emit_event(
            api,
            BurnFungibleResourceEvent {
                amount: other_bucket.liquid.amount(),
            },
        )?;

        // Update total supply
        {
            let total_supply_handle = api.actor_lock_field(
                OBJECT_HANDLE_SELF,
                FungibleResourceManagerField::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply: Decimal = api.field_lock_read_typed(total_supply_handle)?;
            total_supply -= other_bucket.liquid.amount();
            api.field_lock_write_typed(total_supply_handle, &total_supply)?;
            api.field_lock_release(total_supply_handle)?;
        }

        Ok(())
    }

    pub(crate) fn drop_empty_bucket<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let other_bucket = drop_fungible_bucket(bucket.0.as_node_id(), api)?;

        if other_bucket.liquid.amount().is_zero() {
            Ok(())
        } else {
            Err(RuntimeError::ApplicationError(
                ApplicationError::ResourceManagerError(
                    FungibleResourceManagerError::DropNonEmptyBucket,
                ),
            ))
        }
    }

    pub(crate) fn create_empty_bucket<Y>(api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        Self::create_bucket(0.into(), api)
    }

    pub(crate) fn create_bucket<Y>(amount: Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let bucket_id = api.new_simple_object(
            FUNGIBLE_BUCKET_BLUEPRINT,
            vec![
                scrypto_encode(&LiquidFungibleResource::new(amount)).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
            ],
        )?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub(crate) fn create_empty_vault<Y>(api: &mut Y) -> Result<Own, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let vault_id = api.new_simple_object(
            FUNGIBLE_VAULT_BLUEPRINT,
            vec![
                scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
            ],
        )?;

        // TODO: Figure out how to use SELF_ROLE rather than package
        let mut roles = Roles::new();
        roles.define_role(
            "this_package",
            RoleEntry::immutable(rule!(require(package_of_direct_caller(RESOURCE_PACKAGE)))),
        );
        roles.define_role(
            VAULT_WITHDRAW_ROLE,
            RoleEntry::new(AccessRule::AllowAll, ["this_package"], true),
        );
        let access_rules = AccessRules::create(roles, api)?;
        api.attach_access_rules(&vault_id, access_rules.0.as_node_id())?;

        Runtime::emit_event(api, VaultCreationEvent { vault_id })?;

        Ok(Own(vault_id))
    }

    pub(crate) fn get_resource_type<Y>(api: &mut Y) -> Result<ResourceType, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let divisibility_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            FungibleResourceManagerField::Divisibility.into(),
            LockFlags::read_only(),
        )?;

        let divisibility: u8 = api.field_lock_read_typed(divisibility_handle)?;
        let resource_type = ResourceType::Fungible { divisibility };

        Ok(resource_type)
    }

    pub(crate) fn get_total_supply<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let total_supply_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            FungibleResourceManagerField::TotalSupply.into(),
            LockFlags::read_only(),
        )?;
        let total_supply: Decimal = api.field_lock_read_typed(total_supply_handle)?;
        Ok(total_supply)
    }
}
