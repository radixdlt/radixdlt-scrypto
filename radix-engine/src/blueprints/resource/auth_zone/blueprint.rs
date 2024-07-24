use crate::blueprints::resource::ComposedProof;
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::node_init::type_info_partition;
use crate::system::system_callback::SystemLockData;
use crate::system::system_modules::auth::{Authorization, AuthorizationCheckResult};
use crate::system::type_info::TypeInfoSubstate;
use radix_engine_interface::api::{LockFlags, SystemApi, ACTOR_REF_SELF, ACTOR_STATE_SELF};
use radix_engine_interface::blueprints::package::BlueprintVersion;
use radix_engine_interface::blueprints::resource::*;
use radix_native_sdk::resource::NativeProof;

use super::{compose_proof_by_amount, compose_proof_by_ids, AuthZone, ComposeProofError};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AuthZoneError {
    ComposeProofError(ComposeProofError),
}

pub struct AuthZoneBlueprint;

impl AuthZoneBlueprint {
    pub fn pop<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<Option<Proof>, RuntimeError> {
        let auth_zone_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            AuthZoneField::AuthZone.into(),
            LockFlags::MUTABLE,
        )?;

        let mut auth_zone: AuthZone = api.field_read_typed(auth_zone_handle)?;
        let maybe_proof = auth_zone.pop();
        api.field_write_typed(auth_zone_handle, &auth_zone)?;

        Ok(maybe_proof)
    }

    pub fn push<Y: SystemApi<RuntimeError>>(proof: Proof, api: &mut Y) -> Result<(), RuntimeError> {
        let auth_zone_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            AuthZoneField::AuthZone.into(),
            LockFlags::MUTABLE,
        )?;

        let mut auth_zone: AuthZone = api.field_read_typed(auth_zone_handle)?;
        auth_zone.push(proof);

        api.field_write_typed(auth_zone_handle, &auth_zone)?;
        api.field_close(auth_zone_handle)?;

        Ok(())
    }

    pub fn create_proof_of_amount<
        Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    >(
        resource_address: ResourceAddress,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError> {
        let auth_zone_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
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
                            blueprint_info: BlueprintInfo {
                                blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, FUNGIBLE_PROOF_BLUEPRINT),
                                blueprint_version: BlueprintVersion::default(),
                                outer_obj_info: OuterObjectInfo::Some {
                                    outer_object: resource_address.into(),
                                },
                                features: indexset!(),
                                generic_substitutions: vec![],
                            },
                            object_type: ObjectType::Owned,
                        })),
                    ),
                )?;
                api.kernel_pin_node(node_id)?;
            }
            ComposedProof::NonFungible(..) => {
                api.kernel_create_node(
                    node_id,
                    btreemap!(
                    MAIN_BASE_PARTITION => composed_proof.into(),
                    TYPE_INFO_FIELD_PARTITION => type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
                        blueprint_info: BlueprintInfo {
                            blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_PROOF_BLUEPRINT),
                            blueprint_version: BlueprintVersion::default(),
                            outer_obj_info: OuterObjectInfo::Some {
                                outer_object: resource_address.into(),
                            },
                            features: indexset!(),
                            generic_substitutions: vec![],
                        },
                        object_type: ObjectType::Owned,
                    }))),
                )?;
                api.kernel_pin_node(node_id)?;
            }
        }

        Ok(Proof(Own(node_id)))
    }

    pub fn create_proof_of_non_fungibles<
        Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    >(
        resource_address: ResourceAddress,
        ids: IndexSet<NonFungibleLocalId>,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError> {
        let auth_zone_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
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
                    blueprint_info: BlueprintInfo {
                        blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, NON_FUNGIBLE_PROOF_BLUEPRINT),
                        blueprint_version: BlueprintVersion::default(),
                        outer_obj_info: OuterObjectInfo::Some {
                            outer_object: resource_address.into(),
                        },
                        features: indexset!(),
                        generic_substitutions: vec![],
                    },
                    object_type: ObjectType::Owned,
                }))
            ),
        )?;
        api.kernel_pin_node(node_id)?;

        Ok(Proof(Own(node_id)))
    }

    pub fn create_proof_of_all<
        Y: SystemApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    >(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, RuntimeError> {
        let auth_zone_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
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
                    blueprint_info: BlueprintInfo {
                        blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, blueprint_name),
                        blueprint_version: BlueprintVersion::default(),
                        outer_obj_info: OuterObjectInfo::Some {
                            outer_object: resource_address.into(),
                        },
                        features: indexset!(),
                        generic_substitutions: vec![],
                    },
                    object_type: ObjectType::Owned,
                }))
            ),
        )?;
        api.kernel_pin_node(node_id)?;

        Ok(Proof(Own(node_id)))
    }

    pub fn drop_proofs<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        Self::drop_signature_proofs(api)?;
        Self::drop_regular_proofs(api)?;
        Ok(())
    }

    pub fn drop_signature_proofs<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            AuthZoneField::AuthZone.into(),
            LockFlags::MUTABLE,
        )?;
        let mut auth_zone: AuthZone = api.field_read_typed(handle)?;
        auth_zone.remove_signature_proofs();
        api.field_write_typed(handle, &auth_zone)?;
        api.field_close(handle)?;

        Ok(())
    }

    pub fn drop_regular_proofs<Y: SystemApi<RuntimeError>>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            AuthZoneField::AuthZone.into(),
            LockFlags::MUTABLE,
        )?;
        let mut auth_zone: AuthZone = api.field_read_typed(handle)?;
        let proofs = auth_zone.remove_regular_proofs();
        api.field_write_typed(handle, &auth_zone)?;
        api.field_close(handle)?;

        for proof in proofs {
            proof.drop(api)?;
        }

        Ok(())
    }

    pub fn drain<Y: SystemApi<RuntimeError>>(api: &mut Y) -> Result<Vec<Proof>, RuntimeError> {
        let auth_zone_handle = api.actor_open_field(
            ACTOR_STATE_SELF,
            AuthZoneField::AuthZone.into(),
            LockFlags::MUTABLE,
        )?;

        let mut auth_zone: AuthZone = api.field_read_typed(auth_zone_handle)?;
        let proofs = auth_zone.remove_regular_proofs();
        api.field_write_typed(auth_zone_handle, &auth_zone)?;

        Ok(proofs)
    }

    pub fn assert_access_rule<Y: SystemApi<RuntimeError> + KernelSubstateApi<L>, L: Default>(
        access_rule: AccessRule,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let node_id = api.actor_get_node_id(ACTOR_REF_SELF)?;
        let auth_result =
            Authorization::check_authorization_against_access_rule(api, &node_id, &access_rule)?;

        match auth_result {
            AuthorizationCheckResult::Authorized => Ok(()),
            AuthorizationCheckResult::Failed(..) => Err(RuntimeError::SystemError(
                SystemError::AssertAccessRuleFailed,
            )),
        }
    }
}
