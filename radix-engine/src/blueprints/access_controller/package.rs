use super::events::*;
use super::state_machine::*;
use crate::errors::{ApplicationError, InterpreterError, RuntimeError};
use crate::event_schema;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::FIXED_LOW_FEE;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::resource::{SysBucket, Vault};
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::{ACCESS_CONTROLLER_PACKAGE, PACKAGE_TOKEN};
use radix_engine_interface::schema::BlueprintSchema;
use radix_engine_interface::schema::FunctionSchema;
use radix_engine_interface::schema::PackageSchema;
use radix_engine_interface::schema::Receiver;
use radix_engine_interface::time::Instant;
use radix_engine_interface::types::ClientCostingReason;
use radix_engine_interface::*;
use radix_engine_interface::{api::*, rule};
use resources_tracker_macro::trace_resources;
use sbor::rust::vec;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct AccessControllerSubstate {
    /// A vault where the asset controlled by the access controller lives.
    pub controlled_asset: Own,

    /// The amount of time (in minutes) that it takes for timed recovery to be done. Maximum is
    /// 4,294,967,295 minutes which is 8171.5511700913 years. When this is [`None`], then timed
    /// recovery can not be performed through this access controller.
    pub timed_recovery_delay_in_minutes: Option<u32>,

    /// The states of the Access Controller.
    pub state: (
        PrimaryRoleState,
        PrimaryOperationState,
        RecoveryOperationState,
    ),
}

impl AccessControllerSubstate {
    pub fn new(controlled_asset: Own, timed_recovery_delay_in_minutes: Option<u32>) -> Self {
        Self {
            controlled_asset,
            timed_recovery_delay_in_minutes,
            state: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Default)]
pub enum PrimaryRoleState {
    #[default]
    Unlocked,
    Locked,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Default)]
pub enum PrimaryOperationState {
    #[default]
    Normal,
    Recovery(RecoveryProposal),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, Default)]
pub enum RecoveryOperationState {
    #[default]
    Normal,
    Recovery(RecoveryRecoveryState),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum RecoveryRecoveryState {
    Untimed(RecoveryProposal),
    Timed {
        proposal: RecoveryProposal,
        timed_recovery_allowed_after: Instant,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum AccessControllerError {
    /// Occurs when some action requires that the primary role is unlocked to happen.
    OperationRequiresUnlockedPrimaryRole,

    /// Occurs when adding time to an [`Instant`] results in an overflow
    TimeOverflow,

    /// Occurs when a proposer attempts to initiate another recovery when they already have a
    /// recovery underway.
    RecoveryAlreadyExistsForProposer { proposer: Proposer },

    /// Occurs when no recovery can be found for a given proposer.
    NoRecoveryExistsForProposer { proposer: Proposer },

    /// Occurs when there is no timed recoveries on the controller - typically because it isn't in
    /// the state that allows for it.
    NoTimedRecoveriesFound,

    /// Occurs when trying to perform a timed confirm recovery on a recovery proposal that could
    /// be time-confirmed but whose delay has not yet elapsed.
    TimedRecoveryDelayHasNotElapsed,

    /// Occurs when the expected recovery proposal doesn't match that which was found
    RecoveryProposalMismatch {
        expected: Box<RecoveryProposal>,
        found: Box<RecoveryProposal>,
    },
}

impl From<AccessControllerError> for RuntimeError {
    fn from(value: AccessControllerError) -> Self {
        RuntimeError::ApplicationError(ApplicationError::AccessControllerError(value))
    }
}

pub struct AccessControllerNativePackage;

impl AccessControllerNativePackage {
    pub fn schema() -> PackageSchema {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut substates = Vec::new();
        substates.push(aggregator.add_child_type_and_descendents::<AccessControllerSubstate>());

        let mut functions = BTreeMap::new();
        functions.insert(
            ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerCreateGlobalInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerCreateGlobalOutput>(),
                export_name: ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerCreateProofInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerCreateProofOutput>(),
                export_name: ACCESS_CONTROLLER_CREATE_PROOF_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateRecoveryAsPrimaryInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateRecoveryAsPrimaryOutput>(),
                export_name: ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateRecoveryAsRecoveryInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerInitiateRecoveryAsRecoveryOutput>(),
                export_name: ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmPrimaryRoleRecoveryProposalOutput>(),
                export_name: ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerQuickConfirmRecoveryRoleRecoveryProposalOutput>(),
                export_name: ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerTimedConfirmRecoveryInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerTimedConfirmRecoveryOutput>(),
                export_name: ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelPrimaryRoleRecoveryProposalInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelPrimaryRoleRecoveryProposalOutput>(),
                export_name: ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelRecoveryRoleRecoveryProposalInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerCancelRecoveryRoleRecoveryProposalOutput>(),
                export_name: ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerLockPrimaryRoleInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerLockPrimaryRoleOutput>(),
                export_name: ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerUnlockPrimaryRoleInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerUnlockPrimaryRoleOutput>(),
                export_name: ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT.to_string(),
            },
        );
        functions.insert(
            ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerStopTimedRecoveryInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerStopTimedRecoveryOutput>(),
                export_name: ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT.to_string(),
            },
        );

        let event_schema = event_schema! {
            aggregator,
            [
                InitiateRecoveryEvent,
                RuleSetUpdateEvent,
                CancelRecoveryProposalEvent,
                LockPrimaryRoleEvent,
                UnlockPrimaryRoleEvent,
                StopTimedRecoveryEvent
            ]
        };

        let schema = generate_full_schema(aggregator);
        PackageSchema {
            blueprints: btreemap!(
                ACCESS_CONTROLLER_BLUEPRINT.to_string() => BlueprintSchema {
                    parent: None,
                    schema,
                    substates,
                    functions,
                    virtual_lazy_load_functions: btreemap!(),
                    event_schema
                }
            ),
        }
    }

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<&NodeId>,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match export_name {
            ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                Self::create_global(input, api)
            }
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::create_proof(receiver, input, api)
            }
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::initiate_recovery_as_primary(receiver, input, api)
            }
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::initiate_recovery_as_recovery(receiver, input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::quick_confirm_primary_role_recovery_proposal(receiver, input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::quick_confirm_recovery_role_recovery_proposal(receiver, input, api)
            }
            ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::timed_confirm_recovery(receiver, input, api)
            }
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::cancel_primary_role_recovery_proposal(receiver, input, api)
            }
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::cancel_recovery_role_recovery_proposal(receiver, input, api)
            }
            ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::lock_primary_role(receiver, input, api)
            }
            ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::unlock_primary_role(receiver, input, api)
            }
            ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::stop_timed_recovery(receiver, input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    fn create_global<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: AccessControllerCreateGlobalInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        // Creating a new vault and putting in it the controlled asset
        let vault = {
            let mut vault = input
                .controlled_asset
                .sys_resource_address(api)
                .and_then(|resource_address| Vault::sys_new(resource_address, api))?;
            vault.sys_put(input.controlled_asset, api)?;

            vault
        };

        let substate =
            AccessControllerSubstate::new(vault.0, input.timed_recovery_delay_in_minutes);
        let object_id = api.new_object(
            ACCESS_CONTROLLER_BLUEPRINT,
            vec![scrypto_encode(&substate).unwrap()],
        )?;

        let access_rules =
            AccessRules::sys_new(access_rules_from_rule_set(input.rule_set), btreemap!(), api)?.0;

        let metadata = Metadata::sys_create(api)?;
        let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

        // Creating a global component address for the access controller RENode
        let address = api.globalize(
            object_id,
            btreemap!(
                TypedModuleId::AccessRules => access_rules.0,
                TypedModuleId::Metadata => metadata.0,
                TypedModuleId::Royalty => royalty.0,
            ),
        )?;

        Ok(IndexedScryptoValue::from_typed(&address))
    }

    fn create_proof<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AccessControllerCreateProofInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let proof = transition(receiver, api, AccessControllerCreateProofStateMachineInput)?;

        Ok(IndexedScryptoValue::from_typed(&proof))
    }

    fn initiate_recovery_as_primary<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AccessControllerInitiateRecoveryAsPrimaryInput =
            input.as_typed().map_err(|e| {
                RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
            })?;
        let proposal = RecoveryProposal {
            rule_set: input.rule_set,
            timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
        };

        transition_mut(
            receiver,
            api,
            AccessControllerInitiateRecoveryAsPrimaryStateMachineInput {
                proposal: proposal.clone(),
            },
        )?;

        Runtime::emit_event(
            api,
            InitiateRecoveryEvent {
                proposal,
                proposer: Proposer::Primary,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn initiate_recovery_as_recovery<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AccessControllerInitiateRecoveryAsRecoveryInput =
            input.as_typed().map_err(|e| {
                RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
            })?;
        let proposal = RecoveryProposal {
            rule_set: input.rule_set,
            timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
        };

        transition_mut(
            receiver,
            api,
            AccessControllerInitiateRecoveryAsRecoveryStateMachineInput {
                proposal: proposal.clone(),
            },
        )?;

        Runtime::emit_event(
            api,
            InitiateRecoveryEvent {
                proposal,
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn quick_confirm_primary_role_recovery_proposal<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput =
            input.as_typed().map_err(|e| {
                RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
            })?;
        let proposal = RecoveryProposal {
            rule_set: input.rule_set,
            timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
        };

        let recovery_proposal = transition_mut(
            receiver,
            api,
            AccessControllerQuickConfirmPrimaryRoleRecoveryProposalStateMachineInput {
                proposal_to_confirm: proposal.clone(),
            },
        )?;

        update_access_rules(
            api,
            receiver,
            access_rules_from_rule_set(recovery_proposal.rule_set),
        )?;

        Runtime::emit_event(
            api,
            RuleSetUpdateEvent {
                proposal,
                proposer: Proposer::Primary,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn quick_confirm_recovery_role_recovery_proposal<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput =
            input.as_typed().map_err(|e| {
                RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
            })?;
        let proposal = RecoveryProposal {
            rule_set: input.rule_set,
            timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
        };

        let recovery_proposal = transition_mut(
            receiver,
            api,
            AccessControllerQuickConfirmRecoveryRoleRecoveryProposalStateMachineInput {
                proposal_to_confirm: proposal.clone(),
            },
        )?;

        update_access_rules(
            api,
            receiver,
            access_rules_from_rule_set(recovery_proposal.rule_set),
        )?;

        Runtime::emit_event(
            api,
            RuleSetUpdateEvent {
                proposal,
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn timed_confirm_recovery<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AccessControllerTimedConfirmRecoveryInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;
        let proposal = RecoveryProposal {
            rule_set: input.rule_set,
            timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
        };

        let recovery_proposal = transition_mut(
            receiver,
            api,
            AccessControllerTimedConfirmRecoveryStateMachineInput {
                proposal_to_confirm: proposal.clone(),
            },
        )?;

        // Update the access rules
        update_access_rules(
            api,
            receiver,
            access_rules_from_rule_set(recovery_proposal.rule_set),
        )?;

        Runtime::emit_event(
            api,
            RuleSetUpdateEvent {
                proposal,
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn cancel_primary_role_recovery_proposal<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AccessControllerCancelPrimaryRoleRecoveryProposalInput =
            input.as_typed().map_err(|e| {
                RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
            })?;

        transition_mut(
            receiver,
            api,
            AccessControllerCancelPrimaryRoleRecoveryProposalStateMachineInput,
        )?;

        Runtime::emit_event(
            api,
            CancelRecoveryProposalEvent {
                proposer: Proposer::Primary,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn cancel_recovery_role_recovery_proposal<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AccessControllerCancelRecoveryRoleRecoveryProposalInput =
            input.as_typed().map_err(|e| {
                RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
            })?;

        transition_mut(
            receiver,
            api,
            AccessControllerCancelRecoveryRoleRecoveryProposalStateMachineInput,
        )?;

        Runtime::emit_event(
            api,
            CancelRecoveryProposalEvent {
                proposer: Proposer::Recovery,
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn lock_primary_role<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AccessControllerLockPrimaryRoleInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        transition_mut(
            receiver,
            api,
            AccessControllerLockPrimaryRoleStateMachineInput,
        )?;
        Runtime::emit_event(api, LockPrimaryRoleEvent {})?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn unlock_primary_role<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: AccessControllerUnlockPrimaryRoleInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        transition_mut(
            receiver,
            api,
            AccessControllerUnlockPrimaryRoleStateMachineInput,
        )?;
        Runtime::emit_event(api, UnlockPrimaryRoleEvent {})?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn stop_timed_recovery<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: AccessControllerStopTimedRecoveryInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        transition_mut(
            receiver,
            api,
            AccessControllerStopTimedRecoveryStateMachineInput {
                proposal: RecoveryProposal {
                    rule_set: input.rule_set,
                    timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
                },
            },
        )?;
        Runtime::emit_event(api, StopTimedRecoveryEvent {})?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}

fn access_rule_or(access_rules: Vec<AccessRule>) -> AccessRule {
    let mut rule_nodes = Vec::new();
    for access_rule in access_rules.into_iter() {
        match access_rule {
            AccessRule::AllowAll => return AccessRule::AllowAll,
            AccessRule::DenyAll => {}
            AccessRule::Protected(rule_node) => rule_nodes.push(rule_node),
        }
    }
    AccessRule::Protected(AccessRuleNode::AnyOf(rule_nodes))
}

//=========
// Helpers
//=========

fn access_rules_from_rule_set(rule_set: RuleSet) -> AccessRulesConfig {
    let mut access_rules = AccessRulesConfig::new();

    // Primary Role Rules
    let primary_group = "primary";
    access_rules.set_group_access_rule(primary_group.into(), rule_set.primary_role.clone());
    access_rules.set_method_access_rule_to_group(
        MethodKey::new(
            TypedModuleId::ObjectState,
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT,
        ),
        primary_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        MethodKey::new(
            TypedModuleId::ObjectState,
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT,
        ),
        primary_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        MethodKey::new(
            TypedModuleId::ObjectState,
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT,
        ),
        primary_group.into(),
    );

    // Recovery Role Rules
    let recovery_group = "recovery";
    access_rules.set_group_access_rule(recovery_group.into(), rule_set.recovery_role.clone());
    access_rules.set_method_access_rule_to_group(
        MethodKey::new(
            TypedModuleId::ObjectState,
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT,
        ),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        MethodKey::new(
            TypedModuleId::ObjectState,
            ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT,
        ),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        MethodKey::new(
            TypedModuleId::ObjectState,
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT,
        ),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        MethodKey::new(
            TypedModuleId::ObjectState,
            ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE_IDENT,
        ),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        MethodKey::new(
            TypedModuleId::ObjectState,
            ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE_IDENT,
        ),
        recovery_group.into(),
    );

    // Confirmation Role Rules
    let confirmation_group = "confirmation";
    access_rules.set_group_access_rule(
        confirmation_group.into(),
        rule_set.confirmation_role.clone(),
    );

    // Other methods
    access_rules.set_method_access_rule(
        MethodKey::new(
            TypedModuleId::ObjectState,
            ACCESS_CONTROLLER_STOP_TIMED_RECOVERY_IDENT,
        ),
        access_rule_or(
            [
                rule_set.primary_role.clone(),
                rule_set.recovery_role.clone(),
                rule_set.confirmation_role.clone(),
            ]
            .into(),
        ),
    );
    access_rules.set_method_access_rule(
        MethodKey::new(
            TypedModuleId::ObjectState,
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT,
        ),
        access_rule_or([rule_set.recovery_role, rule_set.confirmation_role.clone()].into()),
    );
    access_rules.set_method_access_rule(
        MethodKey::new(
            TypedModuleId::ObjectState,
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT,
        ),
        access_rule_or([rule_set.primary_role, rule_set.confirmation_role].into()),
    );

    let non_fungible_local_id =
        NonFungibleLocalId::bytes(scrypto_encode(&ACCESS_CONTROLLER_PACKAGE).unwrap()).unwrap();
    let non_fungible_global_id = NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_local_id);

    access_rules.default(rule!(deny_all), rule!(require(non_fungible_global_id)))
}

fn transition<Y, I>(
    receiver: &NodeId,
    api: &mut Y,
    input: I,
) -> Result<<AccessControllerSubstate as Transition<I>>::Output, RuntimeError>
where
    Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    AccessControllerSubstate: Transition<I>,
{
    let substate_key = AccessControllerOffset::AccessController.into();
    let handle = api.sys_lock_substate(receiver, &substate_key, LockFlags::read_only())?;

    let access_controller = {
        let access_controller: AccessControllerSubstate = api.sys_read_substate_typed(handle)?;
        access_controller
    };

    let rtn = access_controller.transition(api, input)?;

    api.sys_drop_lock(handle)?;

    Ok(rtn)
}

fn transition_mut<Y, I>(
    receiver: &NodeId,
    api: &mut Y,
    input: I,
) -> Result<<AccessControllerSubstate as TransitionMut<I>>::Output, RuntimeError>
where
    Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    AccessControllerSubstate: TransitionMut<I>,
{
    let substate_key = AccessControllerOffset::AccessController.into();
    let handle = api.sys_lock_substate(receiver, &substate_key, LockFlags::MUTABLE)?;

    let mut access_controller = {
        let access_controller: AccessControllerSubstate = api.sys_read_substate_typed(handle)?;
        access_controller
    };

    let rtn = access_controller.transition_mut(api, input)?;

    {
        api.sys_write_substate_typed(handle, &access_controller)?;
    }

    api.sys_drop_lock(handle)?;

    Ok(rtn)
}

fn update_access_rules<Y>(
    api: &mut Y,
    receiver: &NodeId,
    access_rules: AccessRulesConfig,
) -> Result<(), RuntimeError>
where
    Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
{
    for (group_name, access_rule) in access_rules.get_all_grouped_auth().iter() {
        api.call_module_method(
            receiver,
            TypedModuleId::AccessRules,
            ACCESS_RULES_SET_GROUP_ACCESS_RULE_IDENT,
            scrypto_encode(&AccessRulesSetGroupAccessRuleInput {
                object_key: ObjectKey::SELF,
                name: group_name.into(),
                rule: access_rule.clone(),
            })
            .unwrap(),
        )?;
    }
    for (method_key, entry) in access_rules.get_all_method_auth().iter() {
        match entry {
            AccessRuleEntry::AccessRule(access_rule) => {
                api.call_module_method(
                    receiver,
                    TypedModuleId::AccessRules,
                    ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
                    scrypto_encode(&AccessRulesSetMethodAccessRuleInput {
                        object_key: ObjectKey::SELF,
                        method_key: method_key.clone(),
                        rule: AccessRuleEntry::AccessRule(access_rule.clone()),
                    })
                    .unwrap(),
                )?;
            }
            AccessRuleEntry::Group(..) => {} // Already updated above
        }
    }
    Ok(())
}
