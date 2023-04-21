use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::heap::DroppedBucket;
use crate::kernel::heap::DroppedBucketResource;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::substate_lock_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::{NodeId, FungibleResourceManagerOffset};
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
        verify_divisibility(divisibility)?;

        let object_id = api.new_object(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            vec![
                scrypto_encode(&divisibility).unwrap(),
                scrypto_encode(&Decimal::zero()).unwrap(),
            ],
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
        verify_divisibility(divisibility)?;

        let object_id = api.new_object(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            vec![
                scrypto_encode(&divisibility).unwrap(),
                scrypto_encode(&initial_supply).unwrap(),
            ],
        )?;

        let resource_address = ResourceAddress::new_unchecked(resource_address);
        let bucket = build_fungible_bucket(resource_address, divisibility, initial_supply, api)?;

        globalize_resource_manager(object_id, resource_address, access_rules, metadata, api)?;

        Ok((resource_address, bucket))
    }

    pub(crate) fn mint<Y>(amount: Decimal, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
         let divisibility = {
            let divisibility_handle =    api.lock_field(
                FungibleResourceManagerOffset::Divisibility.into(),
                LockFlags::read_only(),
            )?;
            let divisibility: u8 = api.sys_read_substate_typed(divisibility_handle)?;
            divisibility
        };

        let bucket_id = {
            let resource_address = ResourceAddress::new_unchecked(api.get_global_address()?.into());
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

            let total_supply_handle = api.lock_field(
                FungibleResourceManagerOffset::TotalSupply.into(),
                LockFlags::MUTABLE,
            )?;
            let mut total_supply: Decimal = api.sys_read_substate_typed(total_supply_handle)?;
            total_supply += amount;
            api.sys_write_substate_typed(total_supply_handle, &total_supply)?;
            api.sys_drop_lock(total_supply_handle)?;


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

            bucket_id
        };

        Runtime::emit_event(api, MintFungibleResourceEvent { amount })?;

        Ok(Bucket(Own(bucket_id)))
    }

    pub(crate) fn burn<Y>(bucket: Bucket, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
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
                    let total_supply_handle = api.lock_field(
                        FungibleResourceManagerOffset::TotalSupply.into(),
                        LockFlags::MUTABLE,
                    )?;
                    let mut total_supply: Decimal = api.sys_read_substate_typed(total_supply_handle)?;
                    total_supply -= resource.amount();
                    api.sys_write_substate_typed(total_supply_handle, &total_supply)?;
                    api.sys_drop_lock(total_supply_handle)?;
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

    pub(crate) fn create_bucket<Y>(api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let resource_address = ResourceAddress::new_unchecked(api.get_global_address()?.into());
        let divisbility_handle = api.lock_field(
            FungibleResourceManagerOffset::Divisibility.into(),
            LockFlags::read_only(),
        )?;
        let divisibility: u8 = api.sys_read_substate_typed(divisbility_handle)?;
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

    pub(crate) fn create_vault<Y>(api: &mut Y) -> Result<Own, RuntimeError>
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
        let resource_type = ResourceType::Fungible {
            divisibility,
        };

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
}
