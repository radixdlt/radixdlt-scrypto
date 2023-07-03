use crate::blueprints::resource::{FungibleResourceManagerError, NonFungibleResourceManagerError};
use crate::errors::{ApplicationError, RuntimeError};
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::*;

fn add_package_role(roles: &mut RolesInit) -> Result<(), String> {
    // Meta roles
    // TODO: Remove
    {
        roles.define_immutable_role(
            RESOURCE_PACKAGE_ROLE,
            rule!(require(package_of_direct_caller(RESOURCE_PACKAGE))),
        );
    }

    Ok(())
}

pub fn globalize_resource_manager<Y>(
    owner_role: OwnerRole,
    object_id: NodeId,
    resource_address_reservation: GlobalAddressReservation,
    mut main_roles: RolesInit,
    metadata: ModuleConfig<MetadataInit>,
    api: &mut Y,
) -> Result<ResourceAddress, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    add_package_role(&mut main_roles).map_err(|err| {
        if object_id.is_global_fungible_resource_manager() {
            RuntimeError::ApplicationError(ApplicationError::FungibleResourceManagerError(
                FungibleResourceManagerError::InvalidRole(err),
            ))
        } else {
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::InvalidRole(err),
            ))
        }
    })?;

    let roles = btreemap!(
        ObjectModuleId::Main => main_roles,
        ObjectModuleId::Metadata => metadata.roles,
    );

    let resman_access_rules = AccessRules::create(owner_role, roles, api)?.0;

    let metadata = Metadata::create_with_data(metadata.init, api)?;

    let address = api.globalize(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
        ),
        Some(resource_address_reservation),
    )?;

    Ok(ResourceAddress::new_or_panic(address.into()))
}

pub fn globalize_fungible_with_initial_supply<Y>(
    owner_role: OwnerRole,
    object_id: NodeId,
    resource_address_reservation: GlobalAddressReservation,
    mut main_roles: RolesInit,
    metadata: ModuleConfig<MetadataInit>,
    initial_supply: Decimal,
    api: &mut Y,
) -> Result<(ResourceAddress, Bucket), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    add_package_role(&mut main_roles).map_err(|err| {
        if object_id.is_global_fungible_resource_manager() {
            RuntimeError::ApplicationError(ApplicationError::FungibleResourceManagerError(
                FungibleResourceManagerError::InvalidRole(err),
            ))
        } else {
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::InvalidRole(err),
            ))
        }
    })?;

    let roles = btreemap!(
        ObjectModuleId::Main => main_roles,
        ObjectModuleId::Metadata => metadata.roles,
    );
    let resman_access_rules = AccessRules::create(owner_role, roles, api)?.0;
    let metadata = Metadata::create_with_data(metadata.init, api)?;

    let modules = btreemap!(
        ObjectModuleId::Main => object_id,
        ObjectModuleId::AccessRules => resman_access_rules.0,
        ObjectModuleId::Metadata => metadata.0,
    );

    let (address, bucket_id) = api.globalize_with_address_and_create_inner_object(
        modules,
        resource_address_reservation,
        FUNGIBLE_BUCKET_BLUEPRINT,
        vec![
            scrypto_encode(&LiquidFungibleResource::new(initial_supply)).unwrap(),
            scrypto_encode(&LockedFungibleResource::default()).unwrap(),
        ],
    )?;

    Ok((
        ResourceAddress::new_or_panic(address.into()),
        Bucket(Own(bucket_id)),
    ))
}

pub fn globalize_non_fungible_with_initial_supply<Y>(
    owner_role: OwnerRole,
    object_id: NodeId,
    resource_address_reservation: GlobalAddressReservation,
    mut main_roles: RolesInit,
    metadata: ModuleConfig<MetadataInit>,
    ids: BTreeSet<NonFungibleLocalId>,
    api: &mut Y,
) -> Result<(ResourceAddress, Bucket), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    add_package_role(&mut main_roles).map_err(|err| {
        if object_id.is_global_fungible_resource_manager() {
            RuntimeError::ApplicationError(ApplicationError::FungibleResourceManagerError(
                FungibleResourceManagerError::InvalidRole(err),
            ))
        } else {
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::InvalidRole(err),
            ))
        }
    })?;

    let roles = btreemap!(
        ObjectModuleId::Main => main_roles,
        ObjectModuleId::Metadata => metadata.roles,
    );
    let resman_access_rules = AccessRules::create(owner_role, roles, api)?.0;

    let metadata = Metadata::create_with_data(metadata.init, api)?;

    let (address, bucket_id) = api.globalize_with_address_and_create_inner_object(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
        ),
        resource_address_reservation,
        NON_FUNGIBLE_BUCKET_BLUEPRINT,
        vec![
            scrypto_encode(&LiquidNonFungibleResource::new(ids)).unwrap(),
            scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
        ],
    )?;

    Ok((
        ResourceAddress::new_or_panic(address.into()),
        Bucket(Own(bucket_id)),
    ))
}
