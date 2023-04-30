use crate::blueprints::resource::ComposedProof;
use crate::errors::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::node_init::ModuleInit;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system_callback::SystemLockData;
use crate::types::*;
use native_sdk::resource::SysProof;
use radix_engine_interface::api::{ClientApi, LockFlags};
use radix_engine_interface::blueprints::resource::*;

use super::{compose_proof_by_amount, compose_proof_by_ids, AuthZone, ComposeProofError};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AuthZoneError {
    EmptyAuthZone,
    ComposeProofError(ComposeProofError),
}

pub struct AuthZoneBlueprint;

impl AuthZoneBlueprint {
    pub(crate) fn pop<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let _input: AuthZonePopInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let auth_zone_handle =
            api.lock_field(AuthZoneOffset::AuthZone.into(), LockFlags::MUTABLE)?;

        let mut auth_zone: AuthZone = api.field_lock_read_typed(auth_zone_handle)?;
        let proof = auth_zone.pop().ok_or(RuntimeError::ApplicationError(
            ApplicationError::AuthZoneError(AuthZoneError::EmptyAuthZone),
        ))?;

        api.field_lock_write_typed(auth_zone_handle, &auth_zone)?;

        Ok(IndexedScryptoValue::from_typed(&proof))
    }

    pub(crate) fn push<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: AuthZonePushInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let auth_zone_handle =
            api.lock_field(AuthZoneOffset::AuthZone.into(), LockFlags::MUTABLE)?;

        let mut auth_zone: AuthZone = api.field_lock_read_typed(auth_zone_handle)?;
        auth_zone.push(input.proof);

        api.field_lock_write_typed(auth_zone_handle, &auth_zone)?;
        api.field_lock_release(auth_zone_handle)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn create_proof<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>,
    {
        let input: AuthZoneCreateProofInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let auth_zone_handle =
            api.lock_field(AuthZoneOffset::AuthZone.into(), LockFlags::MUTABLE)?;

        let auth_zone: AuthZone = api.field_lock_read_typed(auth_zone_handle)?;
        let proofs: Vec<Proof> = auth_zone.proofs.iter().map(|p| Proof(p.0)).collect();
        let composed_proof = compose_proof_by_amount(&proofs, input.resource_address, None, api)?;

        let blueprint_name = match &composed_proof {
            ComposedProof::Fungible(..) => FUNGIBLE_PROOF_BLUEPRINT,
            ComposedProof::NonFungible(..) => NON_FUNGIBLE_PROOF_BLUEPRINT,
        };
        api.field_lock_write_typed(auth_zone_handle, &auth_zone)?;

        let node_id = api.kernel_allocate_node_id(EntityType::InternalGenericComponent)?;
        api.kernel_create_node(
            node_id,
            btreemap!(
                USER_BASE_MODULE => composed_proof.into(),
                SysModuleId::TypeInfo.into() => ModuleInit::TypeInfo(TypeInfoSubstate::Object(ObjectInfo {
                    blueprint: Blueprint::new(&RESOURCE_MANAGER_PACKAGE, blueprint_name),
                    global: false,
                    outer_object: Some(input.resource_address.into()),
                    instance_schema: None,
                })).to_substates()
            ),
        )?;

        Ok(IndexedScryptoValue::from_typed(&Proof(Own(node_id))))
    }

    pub(crate) fn create_proof_by_amount<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>,
    {
        let input: AuthZoneCreateProofByAmountInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let auth_zone_handle =
            api.lock_field(AuthZoneOffset::AuthZone.into(), LockFlags::read_only())?;

        let composed_proof = {
            let auth_zone: AuthZone = api.field_lock_read_typed(auth_zone_handle)?;
            let proofs: Vec<Proof> = auth_zone.proofs.iter().map(|p| Proof(p.0)).collect();
            compose_proof_by_amount(&proofs, input.resource_address, Some(input.amount), api)?
        };

        let node_id = api.kernel_allocate_node_id(EntityType::InternalGenericComponent)?;
        match composed_proof {
            ComposedProof::Fungible(..) => {
                api.kernel_create_node(
                    node_id,
                    btreemap!(
                USER_BASE_MODULE => composed_proof.into(),
                SysModuleId::TypeInfo.into() => ModuleInit::TypeInfo(TypeInfoSubstate::Object(ObjectInfo {
                    blueprint: Blueprint::new(&RESOURCE_MANAGER_PACKAGE, FUNGIBLE_PROOF_BLUEPRINT),
                    global: false,
                    outer_object: Some(input.resource_address.into()),
                    instance_schema: None,
                })).to_substates()
            ),
                )?;
            }
            ComposedProof::NonFungible(..) => {
                api.kernel_create_node(
                    node_id,
                    btreemap!(
                USER_BASE_MODULE => composed_proof.into(),
                SysModuleId::TypeInfo.into() => ModuleInit::TypeInfo(TypeInfoSubstate::Object(ObjectInfo {
                    blueprint: Blueprint::new(&RESOURCE_MANAGER_PACKAGE, NON_FUNGIBLE_PROOF_BLUEPRINT),
                    global: false,
                    outer_object: Some(input.resource_address.into()),
                    instance_schema: None,
                })).to_substates()))?;
            }
        }

        Ok(IndexedScryptoValue::from_typed(&Proof(Own(node_id))))
    }

    pub(crate) fn create_proof_by_ids<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi<SystemLockData> + ClientApi<RuntimeError>,
    {
        let input: AuthZoneCreateProofByIdsInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let auth_zone_handle =
            api.lock_field(AuthZoneOffset::AuthZone.into(), LockFlags::MUTABLE)?;

        let composed_proof = {
            let auth_zone: AuthZone = api.field_lock_read_typed(auth_zone_handle)?;
            let proofs: Vec<Proof> = auth_zone.proofs.iter().map(|p| Proof(p.0)).collect();
            compose_proof_by_ids(&proofs, input.resource_address, Some(input.ids), api)?
        };

        let node_id = api.kernel_allocate_node_id(EntityType::InternalGenericComponent)?;
        api.kernel_create_node(
            node_id,
            btreemap!(
                USER_BASE_MODULE => composed_proof.into(),
                SysModuleId::TypeInfo.into() => ModuleInit::TypeInfo(TypeInfoSubstate::Object(ObjectInfo {
                    blueprint: Blueprint::new(&RESOURCE_MANAGER_PACKAGE, NON_FUNGIBLE_PROOF_BLUEPRINT),
                    global: false,
                    outer_object: Some(input.resource_address.into()),
                    instance_schema: None,
                })).to_substates()
            ),
        )?;

        Ok(IndexedScryptoValue::from_typed(&Proof(Own(node_id))))
    }

    pub(crate) fn clear<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let _input: AuthZoneClearInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let handle = api.lock_field(AuthZoneOffset::AuthZone.into(), LockFlags::MUTABLE)?;
        let mut auth_zone: AuthZone = api.field_lock_read_typed(handle)?;
        auth_zone.clear_signature_proofs();
        let proofs = auth_zone.drain();
        api.field_lock_write_typed(handle, &auth_zone)?;
        api.field_lock_release(handle)?;

        for proof in proofs {
            proof.sys_drop(api)?;
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn clear_signature_proofs<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let _input: AuthZoneClearInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let handle = api.lock_field(AuthZoneOffset::AuthZone.into(), LockFlags::MUTABLE)?;
        let mut auth_zone: AuthZone = api.field_lock_read_typed(handle)?;
        auth_zone.clear_signature_proofs();
        api.field_lock_write_typed(handle, &auth_zone)?;
        api.field_lock_release(handle)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn drain<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let _input: AuthZoneDrainInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let auth_zone_handle =
            api.lock_field(AuthZoneOffset::AuthZone.into(), LockFlags::MUTABLE)?;

        let mut auth_zone: AuthZone = api.field_lock_read_typed(auth_zone_handle)?;
        let proofs = auth_zone.drain();

        api.field_lock_write_typed(auth_zone_handle, &auth_zone)?;

        Ok(IndexedScryptoValue::from_typed(&proofs))
    }
}
