use crate::errors::*;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::auth::convert_contextless;
use crate::system::kernel_modules::costing::{FIXED_HIGH_FEE, FIXED_LOW_FEE};
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::types::*;
use native_sdk::resource::SysProof;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::types::ClientCostingReason;
use radix_engine_interface::api::{ClientApi, LockFlags};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::schema::{BlueprintSchema, FunctionSchema, PackageSchema, Receiver};

use super::{
    compose_proof_by_amount, compose_proof_by_ids, AuthZoneStackSubstate, ComposeProofError,
};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AuthZoneError {
    EmptyAuthZone,
    AssertAccessRuleFailed,
    ComposeProofError(ComposeProofError),
}

pub struct AuthZoneNativePackage;

impl AuthZoneNativePackage {
    pub fn schema() -> PackageSchema {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut substates = Vec::new();
        substates.push(aggregator.add_child_type_and_descendents::<AuthZoneStackSubstate>());

        let mut functions = BTreeMap::new();
        functions.insert(
            AUTH_ZONE_POP_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<AuthZonePopInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZonePopOutput>(),
                export_name: AUTH_ZONE_POP_IDENT.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_PUSH_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<AuthZonePushInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZonePushOutput>(),
                export_name: AUTH_ZONE_PUSH_IDENT.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_CREATE_PROOF_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<AuthZoneCreateProofInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZoneCreateProofOutput>(),
                export_name: AUTH_ZONE_CREATE_PROOF_IDENT.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_CREATE_PROOF_BY_AMOUNT_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AuthZoneCreateProofByAmountInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AuthZoneCreateProofByAmountOutput>(),
                export_name: AUTH_ZONE_CREATE_PROOF_BY_AMOUNT_IDENT.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_CREATE_PROOF_BY_IDS_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<AuthZoneCreateProofByIdsInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AuthZoneCreateProofByIdsOutput>(),
                export_name: AUTH_ZONE_CREATE_PROOF_BY_IDS_IDENT.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_CLEAR_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<AuthZoneClearInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZoneClearOutput>(),
                export_name: AUTH_ZONE_CLEAR_IDENT.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AuthZoneClearVirtualProofsInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AuthZoneClearVirtualProofsOutput>(),
                export_name: AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_IDENT.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_DRAIN_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<AuthZoneDrainInput>(),
                output: aggregator.add_child_type_and_descendents::<AuthZoneDrainOutput>(),
                export_name: AUTH_ZONE_DRAIN_IDENT.to_string(),
            },
        );
        functions.insert(
            AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<AuthZoneAssertAccessRuleInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AuthZoneAssertAccessRuleOutput>(),
                export_name: AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        PackageSchema {
            blueprints: btreemap!(
                AUTH_ZONE_BLUEPRINT.to_string() => BlueprintSchema {
                    schema,
                    substates,
                    functions,
                    event_schema: [].into()
                }
            ),
        }
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match export_name {
            AUTH_ZONE_POP_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::pop(receiver, input, api)
            }
            AUTH_ZONE_PUSH_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::push(receiver, input, api)
            }
            AUTH_ZONE_CREATE_PROOF_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::create_proof(receiver, input, api)
            }
            AUTH_ZONE_CREATE_PROOF_BY_AMOUNT_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::create_proof_by_amount(receiver, input, api)
            }
            AUTH_ZONE_CREATE_PROOF_BY_IDS_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::create_proof_by_ids(receiver, input, api)
            }
            AUTH_ZONE_CLEAR_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::clear(receiver, input, api)
            }
            AUTH_ZONE_CLEAR_SIGNATURE_PROOFS_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::clear_signature_proofs(receiver, input, api)
            }
            AUTH_ZONE_DRAIN_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                AuthZoneBlueprint::drain(receiver, input, api)
            }
            AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

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
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AuthZonePopInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;

        let proof = {
            let auth_zone_stack: &mut AuthZoneStackSubstate =
                api.kernel_get_substate_ref_mut(auth_zone_handle)?;
            auth_zone_stack
                .cur_auth_zone_mut()
                .pop()
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::AuthZoneError(AuthZoneError::EmptyAuthZone),
                ))?
        };

        Ok(IndexedScryptoValue::from_typed(&proof))
    }

    pub(crate) fn push<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AuthZonePushInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;

        let auth_zone_stack: &mut AuthZoneStackSubstate =
            api.kernel_get_substate_ref_mut(auth_zone_handle)?;
        auth_zone_stack.cur_auth_zone_mut().push(input.proof);
        api.sys_drop_lock(auth_zone_handle)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn create_proof<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AuthZoneCreateProofInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;

        let composed_proof = {
            let auth_zone_stack_ref: &AuthZoneStackSubstate =
                api.kernel_get_substate_ref_mut(auth_zone_handle)?;
            let auth_zone = auth_zone_stack_ref.cur_auth_zone().clone();
            compose_proof_by_amount(&auth_zone.proofs, input.resource_address, None, api)?
        };

        let node_id = api.kernel_allocate_node_id(AllocateEntityType::Object)?;
        api.kernel_create_node(
            node_id,
            composed_proof.into(),
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate::Object {
                    package_address: RESOURCE_MANAGER_PACKAGE,
                    blueprint_name: PROOF_BLUEPRINT.to_string(),
                    global: false,
                })
            ),
        )?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn create_proof_by_amount<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AuthZoneCreateProofByAmountInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::read_only(),
        )?;

        let composed_proof = {
            let auth_zone_stack_ref: &AuthZoneStackSubstate =
                api.kernel_get_substate_ref(auth_zone_handle)?;
            let auth_zone = auth_zone_stack_ref.cur_auth_zone().clone();
            compose_proof_by_amount(
                &auth_zone.proofs,
                input.resource_address,
                Some(input.amount),
                api,
            )?
        };

        let node_id = api.kernel_allocate_node_id(AllocateEntityType::Object)?;
        api.kernel_create_node(
            node_id,
            composed_proof.into(),
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate::Object {
                    package_address: RESOURCE_MANAGER_PACKAGE,
                    blueprint_name: PROOF_BLUEPRINT.to_string(),
                    global: false,
                })
            ),
        )?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn create_proof_by_ids<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AuthZoneCreateProofByIdsInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;

        let composed_proof = {
            let auth_zone_stack_ref: &AuthZoneStackSubstate =
                api.kernel_get_substate_ref(auth_zone_handle)?;
            let auth_zone = auth_zone_stack_ref.cur_auth_zone().clone();
            compose_proof_by_ids(
                &auth_zone.proofs,
                input.resource_address,
                Some(input.ids),
                api,
            )?
        };

        let node_id = api.kernel_allocate_node_id(AllocateEntityType::Object)?;
        api.kernel_create_node(
            node_id,
            composed_proof.into(),
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate::Object {
                    package_address: RESOURCE_MANAGER_PACKAGE,
                    blueprint_name: PROOF_BLUEPRINT.to_string(),
                    global: false,
                })
            ),
        )?;
        let proof_id = node_id.into();

        Ok(IndexedScryptoValue::from_typed(&Proof(proof_id)))
    }

    pub(crate) fn clear<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AuthZoneClearInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;
        let auth_zone_stack: &mut AuthZoneStackSubstate =
            api.kernel_get_substate_ref_mut(handle)?;
        let auth_zone = auth_zone_stack.cur_auth_zone_mut();
        auth_zone.clear_signature_proofs();
        let proofs = auth_zone.drain();
        api.sys_drop_lock(handle)?;

        for proof in proofs {
            proof.sys_drop(api)?;
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn clear_signature_proofs<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AuthZoneClearInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;
        let auth_zone_stack: &mut AuthZoneStackSubstate =
            api.kernel_get_substate_ref_mut(handle)?;
        auth_zone_stack.cur_auth_zone_mut().clear_signature_proofs();
        api.sys_drop_lock(handle)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn drain<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AuthZoneDrainInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let auth_zone_handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::MUTABLE,
        )?;

        let proofs = {
            let auth_zone_stack: &mut AuthZoneStackSubstate =
                api.kernel_get_substate_ref_mut(auth_zone_handle)?;
            auth_zone_stack.cur_auth_zone_mut().drain()
        };

        Ok(IndexedScryptoValue::from_typed(&proofs))
    }

    pub(crate) fn assert_access_rule<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AuthZoneAssertAccessRuleInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
            LockFlags::read_only(),
        )?;
        let auth_zone_stack_ref: &AuthZoneStackSubstate = api.kernel_get_substate_ref(handle)?;
        let auth_zone_stack = auth_zone_stack_ref.clone();
        let authorization = convert_contextless(&input.access_rule);

        // Authorization check
        if !auth_zone_stack.check_auth(false, &authorization, api)? {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::AuthZoneError(AuthZoneError::AssertAccessRuleFailed),
            ));
        }

        api.sys_drop_lock(handle)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}
