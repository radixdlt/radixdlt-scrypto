use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use native_sdk::resource::ResourceManager;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::{NodeId, ResourceManagerOffset};
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct FungibleResourceManagerSubstate {
    pub divisibility: u8,
    pub total_supply: Decimal,
}

impl FungibleResourceManagerSubstate {
    pub fn create(divisibility: u8, total_supply: Decimal) -> Result<Self, RuntimeError> {
        if divisibility > DIVISIBILITY_MAXIMUM {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ResourceManagerError(
                    FungibleResourceManagerError::InvalidDivisibility(divisibility),
                ),
            ));
        }

        let substate = Self {
            divisibility,
            total_supply,
        };

        Ok(substate)
    }
}

fn check_new_amount(divisibility: u8, amount: Decimal) -> Result<(), RuntimeError> {
    let resource_type = ResourceType::Fungible { divisibility };
    if !resource_type.check_amount(amount) {
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resource_manager_substate =
            FungibleResourceManagerSubstate::create(divisibility, 0.into())?;

        let object_id = api.new_object(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            vec![scrypto_encode(&resource_manager_substate).unwrap()],
        )?;

        let global_node_id = api.kernel_allocate_node_id(EntityType::GlobalFungibleResource)?;
        let resource_address = ResourceAddress::new_unchecked(global_node_id.into());
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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let global_node_id = api.kernel_allocate_node_id(EntityType::GlobalFungibleResource)?;
        let resource_address = ResourceAddress::new_unchecked(global_node_id.into());

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
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resource_manager_substate =
            FungibleResourceManagerSubstate::create(divisibility, initial_supply)?;

        let object_id = api.new_object(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            vec![scrypto_encode(&resource_manager_substate).unwrap()],
        )?;

        let resource_address = ResourceAddress::new_unchecked(resource_address);
        check_new_amount(divisibility, initial_supply)?;

        globalize_resource_manager(object_id, resource_address, access_rules, metadata, api)?;

        let bucket = ResourceManager(resource_address).new_fungible_bucket(initial_supply, api)?;

        Ok((resource_address, bucket))
    }

    pub(crate) fn mint<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resman_handle = api.sys_lock_substate(
            receiver,
            &ResourceManagerOffset::ResourceManager.into(),
            LockFlags::MUTABLE,
        )?;

        let resource_address = ResourceAddress::new_unchecked(api.get_global_address()?.into());

        let mut resource_manager: FungibleResourceManagerSubstate =
            api.sys_read_substate_typed(resman_handle)?;
        let divisibility = resource_manager.divisibility;

        // check amount
        check_new_amount(divisibility, amount)?;

        resource_manager.total_supply += amount;
        api.sys_write_substate_typed(resman_handle, &resource_manager)?;
        api.sys_drop_lock(resman_handle)?;

        let bucket = ResourceManager(resource_address).new_fungible_bucket(amount, api)?;

        Runtime::emit_event(api, MintFungibleResourceEvent { amount })?;

        Ok(bucket)
    }

    pub(crate) fn burn<Y>(
        receiver: &NodeId,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resman_handle = api.sys_lock_substate(
            receiver,
            &ResourceManagerOffset::ResourceManager.into(),
            LockFlags::MUTABLE,
        )?;

        // Drop other bucket
        let resource_address = ResourceAddress::new_unchecked(api.get_global_address()?.into());
        let other_bucket =
            drop_fungible_bucket_of_address(resource_address, bucket.0.as_node_id(), api)?;

        // Construct the event and only emit it once all of the operations are done.
        Runtime::emit_event(
            api,
            BurnFungibleResourceEvent {
                amount: other_bucket.liquid.amount(),
            },
        )?;

        // Update total supply
        let mut resource_manager: FungibleResourceManagerSubstate =
            api.sys_read_substate_typed(resman_handle)?;
        resource_manager.total_supply -= other_bucket.liquid.amount();
        api.sys_write_substate_typed(resman_handle, &resource_manager)?;
        api.sys_drop_lock(resman_handle)?;

        Ok(())
    }

    pub(crate) fn drop_empty_bucket<Y>(
        _receiver: &NodeId,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_unchecked(api.get_global_address()?.into());
        let other_bucket =
            drop_fungible_bucket_of_address(resource_address, bucket.0.as_node_id(), api)?;

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

    pub(crate) fn create_empty_bucket<Y>(
        receiver: &NodeId,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        Self::create_bucket(receiver, 0.into(), api)
    }

    pub(crate) fn create_bucket<Y>(
        receiver: &NodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resman_handle = api.sys_lock_substate(
            receiver,
            &ResourceManagerOffset::ResourceManager.into(),
            LockFlags::read_only(),
        )?;
        let resource_manager: FungibleResourceManagerSubstate =
            api.sys_read_substate_typed(resman_handle)?;
        let divisibility = resource_manager.divisibility;
        let bucket_id = api.new_object(
            FUNGIBLE_BUCKET_BLUEPRINT,
            vec![
                scrypto_encode(&BucketInfoSubstate {
                    resource_type: ResourceType::Fungible { divisibility },
                })
                .unwrap(),
                scrypto_encode(&LiquidFungibleResource::new(amount)).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
            ],
        )?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub(crate) fn create_empty_vault<Y>(receiver: &NodeId, api: &mut Y) -> Result<Own, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resman_handle = api.sys_lock_substate(
            receiver,
            &ResourceManagerOffset::ResourceManager.into(),
            LockFlags::read_only(),
        )?;
        let resource_manager: FungibleResourceManagerSubstate =
            api.sys_read_substate_typed(resman_handle)?;
        let divisibility = resource_manager.divisibility;
        let info = FungibleVaultDivisibilitySubstate { divisibility };
        let vault_id = api.new_object(
            FUNGIBLE_VAULT_BLUEPRINT,
            vec![
                scrypto_encode(&info).unwrap(),
                scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
            ],
        )?;

        Runtime::emit_event(api, VaultCreationEvent { vault_id })?;

        Ok(Own(vault_id))
    }

    pub(crate) fn get_resource_type<Y>(
        receiver: &NodeId,
        api: &mut Y,
    ) -> Result<ResourceType, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resman_handle = api.sys_lock_substate(
            receiver,
            &ResourceManagerOffset::ResourceManager.into(),
            LockFlags::read_only(),
        )?;

        let resource_manager: FungibleResourceManagerSubstate =
            api.sys_read_substate_typed(resman_handle)?;
        let resource_type = ResourceType::Fungible {
            divisibility: resource_manager.divisibility,
        };

        Ok(resource_type)
    }

    pub(crate) fn get_total_supply<Y>(
        receiver: &NodeId,
        api: &mut Y,
    ) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resman_handle = api.sys_lock_substate(
            receiver,
            &ResourceManagerOffset::ResourceManager.into(),
            LockFlags::read_only(),
        )?;
        let resource_manager: FungibleResourceManagerSubstate =
            api.sys_read_substate_typed(resman_handle)?;
        let total_supply = resource_manager.total_supply;
        Ok(total_supply)
    }
}
