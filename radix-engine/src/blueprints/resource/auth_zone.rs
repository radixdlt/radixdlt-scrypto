use crate::errors::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::type_info::TypeInfoSubstate;
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
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AuthZonePopInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            LockFlags::MUTABLE,
        )?;

        let proof = {
            let auth_zone: &mut AuthZone = api.kernel_get_substate_ref_mut(auth_zone_handle)?;
            auth_zone.pop().ok_or(RuntimeError::ApplicationError(
                ApplicationError::AuthZoneError(AuthZoneError::EmptyAuthZone),
            ))?
        };

        Ok(IndexedScryptoValue::from_typed(&proof))
    }

    pub(crate) fn push<Y>(
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AuthZonePushInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            LockFlags::MUTABLE,
        )?;

        let auth_zone: &mut AuthZone = api.kernel_get_substate_ref_mut(auth_zone_handle)?;
        auth_zone.push(input.proof);
        api.sys_drop_lock(auth_zone_handle)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn create_proof<Y>(
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AuthZoneCreateProofInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            LockFlags::MUTABLE,
        )?;

        let composed_proof = {
            let auth_zone: &mut AuthZone = api.kernel_get_substate_ref_mut(auth_zone_handle)?;
            let proofs: Vec<Proof> = auth_zone.proofs.iter().map(|p| Proof(p.0)).collect();
            compose_proof_by_amount(&proofs, input.resource_address, None, api)?
        };

        let node_id = api.kernel_allocate_node_id(AllocateEntityType::Object)?;
        api.kernel_create_node(
            node_id,
            composed_proof.into(),
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate::Object {
                    blueprint: Blueprint::new(&RESOURCE_MANAGER_PACKAGE, PROOF_BLUEPRINT),
                    global: false,
                    parent: None,
                })
            ),
        )?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn create_proof_by_amount<Y>(
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AuthZoneCreateProofByAmountInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            LockFlags::read_only(),
        )?;

        let composed_proof = {
            let auth_zone: &AuthZone = api.kernel_get_substate_ref(auth_zone_handle)?;
            let proofs: Vec<Proof> = auth_zone.proofs.iter().map(|p| Proof(p.0)).collect();
            compose_proof_by_amount(&proofs, input.resource_address, Some(input.amount), api)?
        };

        let node_id = api.kernel_allocate_node_id(AllocateEntityType::Object)?;
        api.kernel_create_node(
            node_id,
            composed_proof.into(),
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate::Object {
                    blueprint: Blueprint::new(&RESOURCE_MANAGER_PACKAGE, PROOF_BLUEPRINT),
                    global: false,
                    parent: None,
                })
            ),
        )?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn create_proof_by_ids<Y>(
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AuthZoneCreateProofByIdsInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            LockFlags::MUTABLE,
        )?;

        let composed_proof = {
            let auth_zone: &AuthZone = api.kernel_get_substate_ref(auth_zone_handle)?;
            let proofs: Vec<Proof> = auth_zone.proofs.iter().map(|p| Proof(p.0)).collect();
            compose_proof_by_ids(&proofs, input.resource_address, Some(input.ids), api)?
        };

        let node_id = api.kernel_allocate_node_id(AllocateEntityType::Object)?;
        api.kernel_create_node(
            node_id,
            composed_proof.into(),
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate::Object {
                    blueprint: Blueprint::new(&RESOURCE_MANAGER_PACKAGE, PROOF_BLUEPRINT),
                    global: false,
                    parent: None,
                })
            ),
        )?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn clear<Y>(
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AuthZoneClearInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            LockFlags::MUTABLE,
        )?;
        let auth_zone: &mut AuthZone = api.kernel_get_substate_ref_mut(handle)?;
        auth_zone.clear_signature_proofs();
        let proofs = auth_zone.drain();
        api.sys_drop_lock(handle)?;

        for proof in proofs {
            proof.sys_drop(api)?;
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn clear_signature_proofs<Y>(
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AuthZoneClearInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            LockFlags::MUTABLE,
        )?;
        let auth_zone: &mut AuthZone = api.kernel_get_substate_ref_mut(handle)?;
        auth_zone.clear_signature_proofs();
        api.sys_drop_lock(handle)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn drain<Y>(
        receiver: &RENodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AuthZoneDrainInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::AuthZone(AuthZoneOffset::AuthZone),
            LockFlags::MUTABLE,
        )?;

        let proofs = {
            let auth_zone: &mut AuthZone = api.kernel_get_substate_ref_mut(auth_zone_handle)?;
            auth_zone.drain()
        };

        Ok(IndexedScryptoValue::from_typed(&proofs))
    }
}
