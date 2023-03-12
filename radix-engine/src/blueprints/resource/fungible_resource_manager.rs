use crate::blueprints::resource::vault::VaultInfoSubstate;
use crate::blueprints::resource::*;
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::heap::DroppedBucket;
use crate::kernel::heap::DroppedBucketResource;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use native_sdk::access_rules::AccessRulesObject;
use native_sdk::metadata::Metadata;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::{RENodeId, ResourceManagerOffset, SubstateOffset};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::*;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum FungibleResourceManagerError {
    InvalidAmount(Decimal, u8),
    MaxMintAmountExceeded,
    MismatchingBucketResource,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct FungibleResourceManagerSubstate {
    pub resource_address: ResourceAddress, // TODO: Figure out a way to remove?
    pub divisibility: u8,
    pub total_supply: Decimal,
}

fn build_fungible_resource_manager_substate_with_initial_supply<Y>(
    resource_address: ResourceAddress,
    divisibility: u8,
    initial_supply: Decimal,
    api: &mut Y,
) -> Result<(FungibleResourceManagerSubstate, Bucket), RuntimeError>
where
    Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
{
    let mut resource_manager = FungibleResourceManagerSubstate {
        resource_address,
        divisibility,
        total_supply: 0.into(),
    };

    let bucket = {
        // check amount
        let resource_type = ResourceType::Fungible { divisibility };
        if !resource_type.check_amount(initial_supply) {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ResourceManagerError(
                    FungibleResourceManagerError::InvalidAmount(initial_supply, divisibility),
                ),
            ));
        }

        // TODO: refactor this into mint function
        if initial_supply > dec!("1000000000000000000") {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::ResourceManagerError(
                    FungibleResourceManagerError::MaxMintAmountExceeded,
                ),
            ));
        }
        resource_manager.total_supply = initial_supply;

        let bucket_info = BucketInfoSubstate {
            resource_address,
            resource_type: ResourceType::Fungible { divisibility },
        };
        let liquid_resource = LiquidFungibleResource::new(initial_supply);
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

        Bucket(bucket_id)
    };

    Ok((resource_manager, bucket))
}

fn create_fungible_resource_manager<Y>(
    global_node_id: RENodeId,
    divisibility: u8,
    metadata: BTreeMap<String, String>,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    api: &mut Y,
) -> Result<ResourceAddress, RuntimeError>
where
    Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
{
    let resource_address: ResourceAddress = global_node_id.into();

    let resource_manager_substate = FungibleResourceManagerSubstate {
        divisibility,
        resource_address,
        total_supply: 0.into(),
    };

    let object_id = api.new_object(
        FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
        vec![scrypto_encode(&resource_manager_substate).unwrap()],
    )?;

    let (resman_access_rules, vault_access_rules) = build_access_rules(access_rules);
    let resman_access_rules = AccessRulesObject::sys_new(resman_access_rules, api)?;
    let vault_access_rules = AccessRulesObject::sys_new(vault_access_rules, api)?;
    let metadata = Metadata::sys_create_with_data(metadata, api)?;

    api.globalize_with_address(
        RENodeId::Object(object_id),
        btreemap!(
            NodeModuleId::AccessRules => resman_access_rules.id(),
            NodeModuleId::AccessRules1 => vault_access_rules.id(),
            NodeModuleId::Metadata => metadata.id(),
        ),
        resource_address.into(),
    )?;

    Ok(resource_address)
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
        let global_node_id = api.kernel_allocate_node_id(RENodeType::GlobalResourceManager)?;
        let address = create_fungible_resource_manager(
            global_node_id,
            divisibility,
            metadata,
            access_rules,
            api,
        )?;
        Ok(address)
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
        let global_node_id = api.kernel_allocate_node_id(RENodeType::GlobalResourceManager)?;
        let resource_address: ResourceAddress = global_node_id.into();

        let (resource_manager_substate, bucket) =
            build_fungible_resource_manager_substate_with_initial_supply(
                resource_address,
                divisibility,
                initial_supply,
                api,
            )?;

        let object_id = api.new_object(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            vec![scrypto_encode(&resource_manager_substate).unwrap()],
        )?;

        let (resman_access_rules, vault_access_rules) = build_access_rules(access_rules);
        let resman_access_rules = AccessRulesObject::sys_new(resman_access_rules, api)?;
        let vault_access_rules = AccessRulesObject::sys_new(vault_access_rules, api)?;
        let metadata = Metadata::sys_create_with_data(metadata, api)?;

        api.globalize_with_address(
            RENodeId::Object(object_id),
            btreemap!(
                NodeModuleId::AccessRules => resman_access_rules.id(),
                NodeModuleId::AccessRules1 => vault_access_rules.id(),
                NodeModuleId::Metadata => metadata.id(),
            ),
            resource_address.into(),
        )?;

        Ok((resource_address, bucket))
    }

    pub(crate) fn create_with_initial_supply_and_address<Y>(
        divisibility: u8,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        initial_supply: Decimal,
        resource_address: [u8; 26], // TODO: Clean this up
        api: &mut Y,
    ) -> Result<(ResourceAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let global_node_id =
            RENodeId::GlobalObject(ResourceAddress::Normal(resource_address).into());
        let resource_address: ResourceAddress = global_node_id.into();

        let (resource_manager_substate, bucket) =
            build_fungible_resource_manager_substate_with_initial_supply(
                resource_address,
                divisibility,
                initial_supply,
                api,
            )?;

        let object_id = api.new_object(
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            vec![scrypto_encode(&resource_manager_substate).unwrap()],
        )?;

        let (resman_access_rules, vault_access_rules) = build_access_rules(access_rules);
        let resman_access_rules = AccessRulesObject::sys_new(resman_access_rules, api)?;
        let vault_access_rules = AccessRulesObject::sys_new(vault_access_rules, api)?;
        let metadata = Metadata::sys_create_with_data(metadata, api)?;

        api.globalize_with_address(
            RENodeId::Object(object_id),
            btreemap!(
                NodeModuleId::AccessRules => resman_access_rules.id(),
                NodeModuleId::AccessRules1 => vault_access_rules.id(),
                NodeModuleId::Metadata => metadata.id(),
            ),
            resource_address.into(),
        )?;

        Ok((resource_address, bucket))
    }

    pub(crate) fn mint<Y>(
        receiver: RENodeId,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resman_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
            LockFlags::MUTABLE,
        )?;

        let bucket_id = {
            let resource_manager: &mut FungibleResourceManagerSubstate =
                api.kernel_get_substate_ref_mut(resman_handle)?;
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
                resource_address: resource_manager.resource_address,
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

        Ok(Bucket(bucket_id))
    }

    pub(crate) fn burn<Y>(
        receiver: RENodeId,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resman_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
            LockFlags::MUTABLE,
        )?;

        // FIXME: check if the bucket is locked!!!
        let dropped_bucket: DroppedBucket =
            api.kernel_drop_node(RENodeId::Object(bucket.0))?.into();

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
                    let resource_manager: &mut FungibleResourceManagerSubstate =
                        api.kernel_get_substate_ref_mut(resman_handle)?;
                    if dropped_bucket.info.resource_address != resource_manager.resource_address {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::ResourceManagerError(
                                FungibleResourceManagerError::MismatchingBucketResource,
                            ),
                        ));
                    }

                    // Update total supply
                    // TODO: there might be better for maintaining total supply, especially for non-fungibles
                    // Update total supply
                    resource_manager.total_supply -= resource.amount();
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

    pub(crate) fn create_bucket<Y>(receiver: RENodeId, api: &mut Y) -> Result<Bucket, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resman_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
            LockFlags::MUTABLE,
        )?;

        let resource_manager: &FungibleResourceManagerSubstate =
            api.kernel_get_substate_ref(resman_handle)?;
        let resource_address = resource_manager.resource_address;
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

        Ok(Bucket(bucket_id))
    }

    pub(crate) fn create_vault<Y>(receiver: RENodeId, api: &mut Y) -> Result<Own, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resman_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
            LockFlags::MUTABLE,
        )?;

        let resource_manager: &FungibleResourceManagerSubstate =
            api.kernel_get_substate_ref(resman_handle)?;
        let resource_address = resource_manager.resource_address;
        let divisibility = resource_manager.divisibility;
        let info = VaultInfoSubstate {
            resource_address,
            resource_type: ResourceType::Fungible { divisibility },
        };
        let vault_id = api.new_object(
            VAULT_BLUEPRINT,
            vec![
                scrypto_encode(&info).unwrap(),
                scrypto_encode(&LiquidFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedFungibleResource::default()).unwrap(),
                scrypto_encode(&LiquidNonFungibleResource::default()).unwrap(),
                scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
            ],
        )?;

        Runtime::emit_event(
            api,
            VaultCreationEvent {
                vault_id: RENodeId::Object(vault_id),
            },
        )?;

        Ok(Own::Vault(vault_id))
    }

    pub(crate) fn get_resource_type<Y>(
        receiver: RENodeId,
        api: &mut Y,
    ) -> Result<ResourceType, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resman_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
            LockFlags::read_only(),
        )?;

        let resource_manager: &FungibleResourceManagerSubstate =
            api.kernel_get_substate_ref(resman_handle)?;
        let resource_type = ResourceType::Fungible {
            divisibility: resource_manager.divisibility,
        };

        Ok(resource_type)
    }

    pub(crate) fn get_total_supply<Y>(
        receiver: RENodeId,
        api: &mut Y,
    ) -> Result<Decimal, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let resman_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager),
            LockFlags::read_only(),
        )?;
        let resource_manager: &FungibleResourceManagerSubstate =
            api.kernel_get_substate_ref(resman_handle)?;
        let total_supply = resource_manager.total_supply;
        Ok(total_supply)
    }
}
