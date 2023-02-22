use crate::blueprints::auth_zone::AuthZoneStackSubstate;
use crate::blueprints::resource::{ProofError, ProofSubstate, ResourceManagerSubstate};
use crate::errors::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::auth::convert_contextless;
use crate::system::kernel_modules::auth::*;
use crate::system::kernel_modules::costing::{FIXED_HIGH_FEE, FIXED_LOW_FEE};
use crate::system::node::RENodeInit;
use crate::types::*;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::{
    Address, AuthZoneStackOffset, ProofOffset, RENodeId, ResourceManagerOffset, SubstateOffset,
};
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AuthZoneError {
    EmptyAuthZone,
    ProofError(ProofError),
    CouldNotCreateProof,
    InvalidRequestData(DecodeError),
    CouldNotGetProof,
    CouldNotGetResource,
    NoMethodSpecified,
    AssertAccessRuleError(MethodAuthorization, MethodAuthorizationError),
}

pub struct AuthZoneNativePackage;

impl AuthZoneNativePackage {
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match export_name {
            AUTH_ZONE_POP_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::pop(receiver, input, api)
            }
            AUTH_ZONE_PUSH_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::push(receiver, input, api)
            }
            AUTH_ZONE_CREATE_PROOF_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::create_proof(receiver, input, api)
            }
            AUTH_ZONE_CREATE_PROOF_BY_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::create_proof_by_amount(receiver, input, api)
            }
            AUTH_ZONE_CREATE_PROOF_BY_IDS_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::create_proof_by_ids(receiver, input, api)
            }
            AUTH_ZONE_CLEAR_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::clear(receiver, input, api)
            }
            AUTH_ZONE_DRAIN_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::drain(receiver, input, api)
            }
            AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::assert_access_rule(receiver, input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}

pub struct AuthZoneBlueprint;

impl AuthZoneBlueprint {
    pub(crate) fn pop<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AuthZonePopInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;

        let proof = {
            let auth_zone_stack: &mut AuthZoneStackSubstate = api.kernel_get_substate_ref_mut2(auth_zone_handle)?;
            let proof = auth_zone_stack.cur_auth_zone_mut().pop()?;
            proof
        };

        let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
        api.kernel_create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn push<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AuthZonePushInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;

        let handle = api.sys_lock_substate(
            RENodeId::Proof(input.proof.0),
            SubstateOffset::Proof(ProofOffset::Proof),
            LockFlags::read_only(),
        )?;
        let proof: &ProofSubstate = api.kernel_get_substate_ref(handle)?;
        // Take control of the proof lock as the proof in the call frame will lose it's lock once dropped
        let mut cloned_proof = proof.clone();
        cloned_proof.change_to_unrestricted();

        let auth_zone_stack: &mut AuthZoneStackSubstate = api.kernel_get_substate_ref_mut2(auth_zone_handle)?;
        auth_zone_stack.cur_auth_zone_mut().push(cloned_proof);
        api.sys_drop_lock(auth_zone_handle)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn create_proof<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AuthZoneCreateProofInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;

        let resource_type = {
            let resource_id = RENodeId::Global(Address::Resource(input.resource_address));
            let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
            let resource_handle =
                api.sys_lock_substate(resource_id, offset, LockFlags::read_only())?;
            let resource_manager: &ResourceManagerSubstate =
                api.kernel_get_substate_ref(resource_handle)?;
            resource_manager.resource_type
        };

        let proof = {
            let auth_zone_stack: &mut AuthZoneStackSubstate = api.kernel_get_substate_ref_mut2(auth_zone_handle)?;
            let proof = auth_zone_stack
                .cur_auth_zone()
                .create_proof(input.resource_address, resource_type)?;
            proof
        };

        let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
        api.kernel_create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn create_proof_by_amount<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AuthZoneCreateProofByAmountInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;

        let resource_type = {
            let resource_id = RENodeId::Global(Address::Resource(input.resource_address));
            let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
            let resource_handle =
                api.sys_lock_substate(resource_id, offset, LockFlags::read_only())?;
            let resource_manager: &ResourceManagerSubstate =
                api.kernel_get_substate_ref(resource_handle)?;
            resource_manager.resource_type
        };

        let proof = {
            let auth_zone_stack: &mut AuthZoneStackSubstate = api.kernel_get_substate_ref_mut2(auth_zone_handle)?;
            let proof = auth_zone_stack.cur_auth_zone().create_proof_by_amount(
                input.amount,
                input.resource_address,
                resource_type,
            )?;

            proof
        };

        let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
        api.kernel_create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn create_proof_by_ids<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AuthZoneCreateProofByIdsInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;

        let resource_type = {
            let resource_id = RENodeId::Global(Address::Resource(input.resource_address));
            let offset = SubstateOffset::ResourceManager(ResourceManagerOffset::ResourceManager);
            let resource_handle =
                api.sys_lock_substate(resource_id, offset, LockFlags::read_only())?;
            let resource_manager: &ResourceManagerSubstate =
                api.kernel_get_substate_ref(resource_handle)?;
            resource_manager.resource_type
        };

        let proof = {
            let auth_zone_stack: &AuthZoneStackSubstate =
                api.kernel_get_substate_ref(auth_zone_handle)?;
            let proof = auth_zone_stack.cur_auth_zone().create_proof_by_ids(
                &input.ids,
                input.resource_address,
                resource_type,
            )?;

            proof
        };

        let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
        api.kernel_create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn clear<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AuthZoneClearInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;
        let auth_zone_stack: &mut AuthZoneStackSubstate = api.kernel_get_substate_ref_mut2(auth_zone_handle)?;
        auth_zone_stack.cur_auth_zone_mut().clear();

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn drain<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AuthZoneDrainInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;

        let proofs = {
            let auth_zone_stack: &mut AuthZoneStackSubstate = api.kernel_get_substate_ref_mut2(auth_zone_handle)?;
            let proofs = auth_zone_stack.cur_auth_zone_mut().drain();
            proofs
        };

        let mut proof_ids: Vec<Proof> = Vec::new();
        let mut nodes_to_move = Vec::new();
        for proof in proofs {
            let node_id = api.kernel_allocate_node_id(RENodeType::Proof)?;
            api.kernel_create_node(node_id, RENodeInit::Proof(proof), BTreeMap::new())?;
            let proof_id = node_id.into();
            proof_ids.push(Proof(proof_id));
            nodes_to_move.push(RENodeId::Proof(proof_id));
        }

        Ok(IndexedScryptoValue::from_typed(&proof_ids))
    }

    pub(crate) fn assert_access_rule<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AuthZoneAssertAccessRuleInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::read_only(),
        )?;
        let auth_zone_stack: &AuthZoneStackSubstate = api.kernel_get_substate_ref(handle)?;
        let authorization = convert_contextless(&input.access_rule);

        // Authorization check
        auth_zone_stack
            .check_auth(false, vec![authorization])
            .map_err(|(authorization, error)| {
                RuntimeError::ApplicationError(ApplicationError::AuthZoneError(
                    AuthZoneError::AssertAccessRuleError(authorization, error),
                ))
            })?;

        api.sys_drop_lock(handle)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}
