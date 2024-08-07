use crate::modules::metadata::Metadata;
use crate::modules::role_assignment::RoleAssignment;
use radix_common::data::scrypto::scrypto_encode;
use radix_common::prelude::ScryptoEncode;
use radix_common::types::GlobalAddress;
use radix_engine_interface::api::*;
use radix_engine_interface::object_modules::metadata::{
    MetadataSetInput, MetadataVal, METADATA_SET_IDENT,
};
use radix_engine_interface::object_modules::ModuleConfig;
use radix_engine_interface::prelude::*;
use radix_engine_interface::types::NodeId;
use radix_rust::indexmap;
use sbor::rust::prelude::*;
use sbor::rust::prelude::{Debug, ToOwned};

#[derive(Debug)]
pub struct BorrowedObject(pub NodeId);

impl BorrowedObject {
    pub fn new<T>(node_id: T) -> Self
    where
        T: Into<[u8; NodeId::LENGTH]>,
    {
        Self(NodeId(node_id.into()))
    }

    pub fn set_metadata<Y: SystemApi<E>, E: SystemApiError, S: AsRef<str>, V: MetadataVal>(
        &mut self,
        key: S,
        value: V,
        api: &mut Y,
    ) -> Result<(), E> {
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

pub fn globalize_object<Y: SystemApi<E>, E: SystemApiError>(
    object_id: NodeId,
    owner_role: OwnerRole,
    address_reservation: GlobalAddressReservation,
    main_roles: RoleAssignmentInit,
    metadata: ModuleConfig<MetadataInit>,
    api: &mut Y,
) -> Result<GlobalAddress, E> {
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

pub fn globalize_object_with_inner_object_and_event<
    Y: SystemApi<E>,
    E: SystemApiError,
    V: ScryptoEncode,
>(
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
) -> Result<(GlobalAddress, NodeId), E> {
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
