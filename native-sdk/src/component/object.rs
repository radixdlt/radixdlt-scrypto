use crate::modules::metadata::Metadata;
use crate::modules::role_assignment::RoleAssignment;
use module_blueprints_interface::metadata::*;
use radix_engine_common::data::scrypto::{scrypto_encode, ScryptoDecode};
use radix_engine_common::prelude::*;
use radix_engine_system_api::ClientApi;
use radix_engine_system_api::FieldValue;
use utils::indexmap;

#[derive(Debug)]
pub struct BorrowedObject(pub NodeId);

impl BorrowedObject {
    pub fn new<T>(node_id: T) -> Self
    where
        T: Into<[u8; NodeId::LENGTH]>,
    {
        Self(NodeId(node_id.into()))
    }

    pub fn set_metadata<Y, E, S, V>(&mut self, key: S, value: V, api: &mut Y) -> Result<(), E>
    where
        Y: ClientApi<E>,
        S: AsRef<str>,
        V: MetadataVal,
        E: Debug + ScryptoDecode,
    {
        api.call_module_method(
            &self.0,
            AttachedModuleId::Metadata,
            METADATA_SET_IDENT,
            scrypto_encode(&MetadataSetInput {
                key: key.as_ref().to_owned(),
                value: value.to_metadata_value(),
            })
            .unwrap(),
        )?;

        Ok(())
    }
}

pub fn globalize_object<Y, E>(
    object_id: NodeId,
    owner_role: OwnerRole,
    address_reservation: GlobalAddressReservation,
    main_roles: RoleAssignmentInit,
    metadata: ModuleConfig<MetadataInit>,
    api: &mut Y,
) -> Result<GlobalAddress, E>
where
    Y: ClientApi<E>,
    E: Debug + ScryptoDecode,
{
    let role_assignment = {
        let roles = indexmap!(
            ModuleId::Main => main_roles,
            ModuleId::Metadata => metadata.roles,
        );
        RoleAssignment::create(owner_role, roles, api)?.0 .0
    };

    let metadata = Metadata::create_with_data(metadata.init, api)?.0;

    let address = api.globalize(
        object_id,
        indexmap!(
            AttachedModuleId::RoleAssignment => role_assignment,
            AttachedModuleId::Metadata => metadata,
        ),
        Some(address_reservation),
    )?;

    Ok(address)
}

pub fn globalize_object_with_inner_object_and_event<Y, E, V>(
    object_id: NodeId,
    owner_role: OwnerRole,
    address_reservation: GlobalAddressReservation,
    main_roles: RoleAssignmentInit,
    metadata: ModuleConfig<MetadataInit>,
    inner_object_bp: &str,
    inner_object_fields: IndexMap<FieldIndex, FieldValue>,
    event_name: &str,
    event: V,
    api: &mut Y,
) -> Result<(GlobalAddress, NodeId), E>
where
    Y: ClientApi<E>,
    E: Debug + ScryptoDecode,
    V: ScryptoEncode,
{
    let role_assignment = {
        let roles = indexmap!(
            ModuleId::Main => main_roles,
            ModuleId::Metadata => metadata.roles,
        );
        RoleAssignment::create(owner_role, roles, api)?.0 .0
    };
    let metadata = Metadata::create_with_data(metadata.init, api)?.0;

    let (address, inner_object) = api
        .globalize_with_address_and_create_inner_object_and_emit_event(
            object_id,
            indexmap!(
                AttachedModuleId::RoleAssignment => role_assignment,
                AttachedModuleId::Metadata => metadata,
            ),
            address_reservation,
            inner_object_bp,
            inner_object_fields,
            event_name,
            scrypto_encode(&event).unwrap(),
        )?;

    Ok((address, inner_object))
}
