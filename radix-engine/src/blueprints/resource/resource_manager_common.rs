use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::role_assignment::RoleAssignment;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::api::object_api::ModuleId;
use radix_engine_interface::api::{AttachedModuleId, ClientApi, FieldValue};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::*;

use super::{MintNonFungibleResourceEvent};

pub fn globalize_non_fungible_with_initial_supply<Y>(
    owner_role: OwnerRole,
    object_id: NodeId,
    resource_address_reservation: GlobalAddressReservation,
    main_roles: RoleAssignmentInit,
    metadata: ModuleConfig<MetadataInit>,
    ids: IndexSet<NonFungibleLocalId>,
    api: &mut Y,
) -> Result<(ResourceAddress, Bucket), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let roles = indexmap!(
        ModuleId::Main => main_roles,
        ModuleId::Metadata => metadata.roles,
    );
    let role_assignment = RoleAssignment::create(owner_role, roles, api)?.0;

    let metadata = Metadata::create_with_data(metadata.init, api)?;

    let (address, bucket_id) = api.globalize_with_address_and_create_inner_object_and_emit_event(
        object_id,
        indexmap!(
            AttachedModuleId::RoleAssignment => role_assignment.0,
            AttachedModuleId::Metadata => metadata.0,
        ),
        resource_address_reservation,
        NON_FUNGIBLE_BUCKET_BLUEPRINT,
        indexmap! {
            0u8 => FieldValue::new(&LiquidNonFungibleResource::new(ids.clone())),
            1u8 => FieldValue::new(&LockedNonFungibleResource::default()),
        },
        MintNonFungibleResourceEvent::EVENT_NAME,
        scrypto_encode(&MintNonFungibleResourceEvent { ids }).unwrap(),
    )?;

    Ok((
        ResourceAddress::new_or_panic(address.into()),
        Bucket(Own(bucket_id)),
    ))
}
