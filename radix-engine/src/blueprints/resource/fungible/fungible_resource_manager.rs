use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::heap::DroppedBucket;
use crate::kernel::heap::DroppedBucketResource;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
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
    MismatchingBucketResource,
    InvalidDivisibility(u8),
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

fn build_fungible_bucket<Y>(
    resource_address: ResourceAddress,
    divisibility: u8,
    amount: Decimal,
    api: &mut Y,
) -> Result<Bucket, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    // check amount
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

    let bucket_info = BucketInfoSubstate {
        resource_address,
        resource_type: ResourceType::Fungible { divisibility },
    };
    let liquid_resource = LiquidFungibleResource::new(amount);
    let bucket_id = api.new_object(
        BUCKET_BLUEPRINT,
        vec![
            scrypto_encode(&bucket_info).unwrap(),
            scrypto_encode(&liquid_resource).unwrap(),
            scrypto_encode(&LockedFungibleResource::default()).unwrap(),
            scrypto_encode(&LiquidNonFungibleResource::default()).unwrap(),
            scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
        ],
    )?;

    Ok(Bucket(Own(bucket_id)))
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
        resource_address: [u8; 27], // TODO: Clean this up
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
        let bucket = build_fungible_bucket(resource_address, divisibility, initial_supply, api)?;

        globalize_resource_manager(object_id, resource_address, access_rules, metadata, api)?;

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

        let bucket_id = {
            let resource_address = ResourceAddress::new_unchecked(api.get_global_address()?.into());

            let mut resource_manager: FungibleResourceManagerSubstate =
                api.sys_read_substate_typed(resman_handle)?;
            let divisibility = resource_manager.divisibility;
            let resource_type = ResourceType::Fungible { divisibility };

            // check amount
            if !resource_type.check_amount(amount) {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::ResourceManagerError(
                        FungibleResourceManagerError::InvalidAmount(amount, divisibility),
                    ),
                ));
            }

            // Practically impossible to overflow the Decimal type with this limit in place.
            if amount > dec!("1000000000000000000") {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::ResourceManagerError(
                        FungibleResourceManagerError::MaxMintAmountExceeded,
                    ),
                ));
            }

            resource_manager.total_supply += amount;

            let bucket_info = BucketInfoSubstate {
                resource_address,
                resource_type: ResourceType::Fungible { divisibility },
            };
            let liquid_resource = LiquidFungibleResource::new(amount);
            let bucket_id = api.new_object(
                BUCKET_BLUEPRINT,
                vec![
                    scrypto_encode(&bucket_info).unwrap(),
                    scrypto_encode(&liquid_resource).unwrap(),
                    scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                    scrypto_encode(&LiquidNonFungibleResource::default()).unwrap(),
                    scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
                ],
            )?;

            api.sys_write_substate_typed(resman_handle, &resource_manager)?;
            api.sys_drop_lock(resman_handle)?;

            bucket_id
        };

        Runtime::emit_event(api, MintFungibleResourceEvent { amount })?;

        Ok(Bucket(Own(bucket_id)))
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

        // FIXME: check if the bucket is locked
        let dropped_bucket: DroppedBucket = api.kernel_drop_node(bucket.0.as_node_id())?.into();

        // Construct the event and only emit it once all of the operations are done.
        match dropped_bucket.resource {
            DroppedBucketResource::Fungible(resource) => {
                Runtime::emit_event(
                    api,
                    BurnFungibleResourceEvent {
                        amount: resource.amount(),
                    },
                )?;

                // Check if resource matches
                // TODO: Move this check into actor check
                {
                    let resource_address =
                        ResourceAddress::new_unchecked(api.get_global_address()?.into());
                    if dropped_bucket.info.resource_address != resource_address {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::ResourceManagerError(
                                FungibleResourceManagerError::MismatchingBucketResource,
                            ),
                        ));
                    }

                    // Update total supply
                    // TODO: there might be better for maintaining total supply, especially for non-fungibles
                    // Update total supply
                    let mut resource_manager: FungibleResourceManagerSubstate =
                        api.sys_read_substate_typed(resman_handle)?;
                    resource_manager.total_supply -= resource.amount();

                    api.sys_write_substate_typed(resman_handle, &resource_manager)?;
                    api.sys_drop_lock(resman_handle)?;
                }
            }
            DroppedBucketResource::NonFungible(..) => {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::ResourceManagerError(
                        FungibleResourceManagerError::MismatchingBucketResource,
                    ),
                ));
            }
        }

        Ok(())
    }

    pub(crate) fn create_bucket<Y>(receiver: &NodeId, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_unchecked(api.get_global_address()?.into());
        let resman_handle = api.sys_lock_substate(
            receiver,
            &ResourceManagerOffset::ResourceManager.into(),
            LockFlags::read_only(),
        )?;
        let resource_manager: FungibleResourceManagerSubstate =
            api.sys_read_substate_typed(resman_handle)?;
        let divisibility = resource_manager.divisibility;
        let bucket_id = api.new_object(
            BUCKET_BLUEPRINT,
            vec![
                scrypto_encode(&BucketInfoSubstate {
                    resource_address,
                    resource_type: ResourceType::Fungible { divisibility },
                })
                .unwrap(),
                scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                scrypto_encode(&LiquidNonFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub(crate) fn create_vault<Y>(receiver: &NodeId, api: &mut Y) -> Result<Own, RuntimeError>
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
