use crate::blueprints::resource::ComposedProof;
use crate::errors::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::node_init::type_info_partition;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system_callback::SystemLockData;
use crate::types::*;
use native_sdk::resource::NativeProof;
use radix_engine_interface::api::{ClientApi, LockFlags, ObjectModuleId, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::package::BlueprintVersion;
use radix_engine_interface::blueprints::resource::*;

use super::{compose_proof_by_amount, compose_proof_by_ids, AuthZone, ComposeProofError};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AuthZoneError {
    EmptyAuthZone,
    ComposeProofError(ComposeProofError),
}

pub struct AuthZoneBlueprint;

impl AuthZoneBlueprint {
    pub(crate) fn pop<Y>(api: &mut Y) -> Result<Proof, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let auth_zone_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            AuthZoneField::AuthZone.into(),
            LockFlags::MUTABLE,
        )?;

        let mut auth_zone: AuthZone = api.field_read_typed(auth_zone_handle)?;
        let proof = auth_zone.pop().ok_or(RuntimeError::ApplicationError(
            ApplicationError::AuthZoneError(AuthZoneError::EmptyAuthZone),
        ))?;

        api.field_write_typed(auth_zone_handle, &auth_zone)?;

        Ok(proof)
    }

    pub(crate) fn push<Y>(proof: Proof, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let auth_zone_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            AuthZoneField::AuthZone.into(),
            LockFlags::MUTABLE,
        )?;

        let mut auth_zone: AuthZone = api.field_read_typed(auth_zone_handle)?;
        auth_zone.push(proof);

        api.field_write_typed(auth_zone_handle, &auth_zone)?;
        api.field_close(auth_zone_handle)?;

        Ok(())
    }

    pub(crate) fn create_proof_of_amount<Y>(
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>,
    {
        let auth_zone_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            AuthZoneField::AuthZone.into(),
            LockFlags::read_only(),
        )?;

        let composed_proof = {
            let auth_zone: AuthZone = api.field_read_typed(auth_zone_handle)?;
            let proofs: Vec<Proof> = auth_zone.proofs.iter().map(|p| Proof(p.0)).collect();
            compose_proof_by_amount(&proofs, resource_address, Some(amount), api)?
        };

        let node_id = api.kernel_allocate_node_id(EntityType::InternalGenericComponent)?;
        match composed_proof {
            ComposedProof::Fungible(..) => {
                api.kernel_create_node(
                    node_id,
                    btreemap!(
                        MAIN_BASE_PARTITION => composed_proof.into(),
                        TYPE_INFO_FIELD_PARTITION => type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
                            global: false,

                            main_blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, FUNGIBLE_PROOF_BLUEPRINT),
                            module_versions: btreemap!(
                                ObjectModuleId::Main => BlueprintVersion::default(),
                            ),

                            outer_object: OuterObjectInfo::Some {
                                outer_object: resource_address.into(),
                            },
                            features: btreeset!(),
                            instance_schema: None,
                        })),
                    ),
                )?;
            }
            ComposedProof::NonFungible(..) => {
                api.kernel_create_node(
                    node_id,
                    btreemap!(
                    MAIN_BASE_PARTITION => composed_proof.into(),
                    TYPE_INFO_FIELD_PARTITION => type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
                        global: false,

                        main_blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_PROOF_BLUEPRINT),
                        module_versions: btreemap!(
                            ObjectModuleId::Main => BlueprintVersion::default(),
                        ),

                        outer_object: OuterObjectInfo::Some {
                            outer_object: resource_address.into(),
                        },
                        features: btreeset!(),
                        instance_schema: None,
                    }))),
                )?;
            }
        }

        Ok(Proof(Own(node_id)))
    }

    pub(crate) fn create_proof_of_non_fungibles<Y>(
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>,
    {
        let auth_zone_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            AuthZoneField::AuthZone.into(),
            LockFlags::MUTABLE,
        )?;

        let composed_proof = {
            let auth_zone: AuthZone = api.field_read_typed(auth_zone_handle)?;
            let proofs: Vec<Proof> = auth_zone.proofs.iter().map(|p| Proof(p.0)).collect();
            compose_proof_by_ids(&proofs, resource_address, Some(ids), api)?
        };

        let node_id = api.kernel_allocate_node_id(EntityType::InternalGenericComponent)?;
        api.kernel_create_node(
            node_id,
            btreemap!(
                MAIN_BASE_PARTITION => composed_proof.into(),
                TYPE_INFO_FIELD_PARTITION => type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
                    global: false,

                    main_blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_PROOF_BLUEPRINT),
                    module_versions: btreemap!(
                        ObjectModuleId::Main => BlueprintVersion::default(),
                    ),

                    outer_object: OuterObjectInfo::Some {
                        outer_object: resource_address.into(),
                    },
                    features: btreeset!(),
                    instance_schema: None,
                }))
            ),
        )?;

        Ok(Proof(Own(node_id)))
    }

    pub(crate) fn create_proof_of_all<Y>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>,
    {
        let auth_zone_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            AuthZoneField::AuthZone.into(),
            LockFlags::MUTABLE,
        )?;

        let auth_zone: AuthZone = api.field_read_typed(auth_zone_handle)?;
        let proofs: Vec<Proof> = auth_zone.proofs.iter().map(|p| Proof(p.0)).collect();
        let composed_proof = compose_proof_by_amount(&proofs, resource_address, None, api)?;

        let blueprint_name = match &composed_proof {
            ComposedProof::Fungible(..) => FUNGIBLE_PROOF_BLUEPRINT,
            ComposedProof::NonFungible(..) => NON_FUNGIBLE_PROOF_BLUEPRINT,
        };
        api.field_write_typed(auth_zone_handle, &auth_zone)?;

        let node_id = api.kernel_allocate_node_id(EntityType::InternalGenericComponent)?;
        api.kernel_create_node(
            node_id,
            btreemap!(
                MAIN_BASE_PARTITION => composed_proof.into(),
                TYPE_INFO_FIELD_PARTITION => type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
                    global: false,

                    main_blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, blueprint_name),
                    module_versions: btreemap!(
                        ObjectModuleId::Main => BlueprintVersion::default(),
                    ),

                    outer_object: OuterObjectInfo::Some {
                        outer_object: resource_address.into(),
                    },
                    features: btreeset!(),
                    instance_schema: None,
                }))
            ),
        )?;

        Ok(Proof(Own(node_id)))
    }

    pub(crate) fn clear<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            AuthZoneField::AuthZone.into(),
            LockFlags::MUTABLE,
        )?;
        let mut auth_zone: AuthZone = api.field_read_typed(handle)?;
        auth_zone.clear_signature_proofs();
        let proofs = auth_zone.drain();
        api.field_write_typed(handle, &auth_zone)?;
        api.field_close(handle)?;

        for proof in proofs {
            proof.drop(api)?;
        }

        Ok(())
    }

    pub(crate) fn clear_signature_proofs<Y>(api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            AuthZoneField::AuthZone.into(),
            LockFlags::MUTABLE,
        )?;
        let mut auth_zone: AuthZone = api.field_read_typed(handle)?;
        auth_zone.clear_signature_proofs();
        api.field_write_typed(handle, &auth_zone)?;
        api.field_close(handle)?;

        Ok(())
    }

    pub(crate) fn drain<Y>(api: &mut Y) -> Result<Vec<Proof>, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let auth_zone_handle = api.actor_open_field(
            OBJECT_HANDLE_SELF,
            AuthZoneField::AuthZone.into(),
            LockFlags::MUTABLE,
        )?;

        let mut auth_zone: AuthZone = api.field_read_typed(auth_zone_handle)?;
        let proofs = auth_zone.drain();

        api.field_write_typed(auth_zone_handle, &auth_zone)?;

        Ok(proofs)
    }
}
