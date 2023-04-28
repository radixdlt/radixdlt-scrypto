use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::substate_lock_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::{FungibleResourceManagerOffset, NodeId};
use radix_engine_interface::*;
use crate::kernel::heap::DroppedProof;

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
    if !check_amount(Some(divisibility), amount) {
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
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        api: &mut Y,
    ) -> Result<ResourceAddress, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        verify_divisibility(divisibility)?;

        let object_id = api.new_object(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            vec![
                scrypto_encode(&divisibility).unwrap(),
                scrypto_encode(&Decimal::zero()).unwrap(),
            ],
        )?;

        let global_node_id = api.kernel_allocate_node_id(EntityType::GlobalFungibleResource)?;
        let resource_address = ResourceAddress::new_or_panic(global_node_id.into());
        globalize_resource_manager(object_id, resource_address, access_rules, metadata, api)?;

        Ok(resource_address)
    }

    pub(crate) fn create_with_initial_supply<Y>(
        divisibility: u8,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        initial_supply: Decimal,
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let global_node_id = api.kernel_allocate_node_id(EntityType::GlobalFungibleResource)?;
        let resource_address = ResourceAddress::new_or_panic(global_node_id.into());

        Self::create_with_initial_supply_and_address(
            divisibility,
            metadata,
            access_rules,
            initial_supply,
            resource_address.into(),
            api,
        )
    }

    pub(crate) fn create_with_initial_supply_and_address<Y>(
        divisibility: u8,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        initial_supply: Decimal,
        resource_address: [u8; NodeId::LENGTH], // TODO: Clean this up
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        verify_divisibility(divisibility)?;

        let object_id = api.new_object(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            vec![
                scrypto_encode(&divisibility).unwrap(),
                scrypto_encode(&initial_supply).unwrap(),
            ],
        )?;

        let resource_address = ResourceAddress::new_or_panic(resource_address);
        check_new_amount(divisibility, initial_supply)?;

        let bucket = globalize_fungible_with_initial_supply(
            object_id,
            resource_address,
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
            let divisibility_handle = api.lock_field(
                FungibleResourceManagerOffset::Divisibility.into(),
                LockFlags::read_only(),
            )?;
            let divisibility: u8 = api.sys_read_substate_typed(divisibility_handle)?;
            divisibility
        };

        // check amount
        check_new_amount(divisibility, amount)?;

        // Update total supply
        {
            let total_supply_handle = api.lock_field(
                FungibleResourceManagerOffset::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply: Decimal = api.sys_read_substate_typed(total_supply_handle)?;
            total_supply += amount;
            api.sys_write_substate_typed(total_supply_handle, &total_supply)?;
            api.sys_drop_lock(total_supply_handle)?;
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
            let total_supply_handle = api.lock_field(
                FungibleResourceManagerOffset::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply: Decimal = api.sys_read_substate_typed(total_supply_handle)?;
            total_supply -= other_bucket.liquid.amount();
            api.sys_write_substate_typed(total_supply_handle, &total_supply)?;
            api.sys_drop_lock(total_supply_handle)?;
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
        let bucket_id = api.new_object(
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
        let vault_id = api.new_object(
            FUNGIBLE_VAULT_BLUEPRINT,
            vec![
                scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
            ],
        )?;

        Runtime::emit_event(api, VaultCreationEvent { vault_id })?;

        Ok(Own(vault_id))
    }

    pub(crate) fn get_resource_type<Y>(api: &mut Y) -> Result<ResourceType, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let divisibility_handle = api.lock_field(
            FungibleResourceManagerOffset::Divisibility.into(),
            LockFlags::read_only(),
        )?;

        let divisibility: u8 = api.sys_read_substate_typed(divisibility_handle)?;
        let resource_type = ResourceType::Fungible { divisibility };

        Ok(resource_type)
    }

    pub(crate) fn get_total_supply<Y>(api: &mut Y) -> Result<Decimal, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let total_supply_handle = api.lock_field(
            FungibleResourceManagerOffset::TotalSupply.into(),
            LockFlags::read_only(),
        )?;
        let total_supply: Decimal = api.sys_read_substate_typed(total_supply_handle)?;
        Ok(total_supply)
    }

    pub(crate) fn drop_proof<Y>(proof: Proof, api: &mut Y) -> Result<(), RuntimeError>
        where
            Y: ClientApi<RuntimeError>,
    {
        let node_substates = api.drop_object(proof.0.as_node_id())?;
        let dropped_proof: DroppedProof = node_substates.into();
        dropped_proof.fungible_proof.drop_proof(api)?;

        Ok(())
    }
}
