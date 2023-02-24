use super::state_machine::*;
use crate::errors::{ApplicationError, InterpreterError, RuntimeError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi, LockFlags};
use crate::system::global::GlobalSubstate;
use crate::system::kernel_modules::costing::FIXED_LOW_FEE;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_modules::access_rules::ObjectAccessRulesChainSubstate;
use native_sdk::resource::{SysBucket, Vault};
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::{ACCESS_CONTROLLER_PACKAGE, PACKAGE_TOKEN};
use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode, IndexedScryptoValue, ScryptoValue,
};
use radix_engine_interface::time::Instant;
use radix_engine_interface::*;
use radix_engine_interface::{api::*, rule};
use sbor::rust::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct AccessControllerSubstate {
    /// A vault where the asset controlled by the access controller lives.
    pub controlled_asset: VaultId,

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
    pub fn new(controlled_asset: VaultId, timed_recovery_delay_in_minutes: Option<u32>) -> Self {
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
        expected: RecoveryProposal,
        found: RecoveryProposal,
    },
}

impl From<AccessControllerError> for RuntimeError {
    fn from(value: AccessControllerError) -> Self {
        RuntimeError::ApplicationError(ApplicationError::AccessControllerError(value))
    }
}

#[derive(ScryptoSbor, LegacyDescribe)]
struct AccessControllerInitiateRecoveryEvent {
    proposer: Proposer,
    proposal: RecoveryProposal,
}

#[derive(ScryptoSbor, LegacyDescribe)]
struct AccessControllerRuleSetUpdateEvent {
    proposer: Proposer,
    proposal: RecoveryProposal,
}

#[derive(ScryptoSbor, LegacyDescribe)]
struct AccessControllerCancelRecoveryProposalEvent {
    proposer: Proposer,
}

#[derive(ScryptoSbor, LegacyDescribe)]
struct AccessControllerLockPrimaryRoleEvent {}

#[derive(ScryptoSbor, LegacyDescribe)]
struct AccessControllerUnlockPrimaryRoleEvent {}

#[derive(ScryptoSbor, LegacyDescribe)]
struct AccessControllerStopTimedRecoveryEvent {}

pub struct AccessControllerNativePackage;

impl AccessControllerNativePackage {
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        match export_name {
            ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                Self::create_global(input, api)
            }
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::create_proof(receiver, input, api)
            }
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::initial_recovery_as_primary(receiver, input, api)
            }
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::initial_recovery_as_recovery(receiver, input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::quick_confirm_primary_role_recovery_proposal(receiver, input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::quick_confirm_recovery_role_recovery_proposal(receiver, input, api)
            }
            ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::timed_confirm_recovery(receiver, input, api)
            }
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::cancel_primary_role_recovery_proposal(receiver, input, api)
            }
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::cancel_recovery_role_recovery_proposal(receiver, input, api)
            }
            ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::lock_primary_role(receiver, input, api)
            }
            ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::unlock_primary_role(receiver, input, api)
            }
            ACCESS_CONTROLLER_STOP_TIMED_RECOVERY => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

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
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: AccessControllerCreateGlobalInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        // Creating a new vault and putting in it the controlled asset
        let vault = {
            let mut vault = input
                .controlled_asset
                .sys_resource_address(api)
                .and_then(|resource_address| Vault::sys_new(resource_address, api))?;
            vault.sys_put(input.controlled_asset, api)?;

            vault
        };

        let mut node_modules = BTreeMap::new();
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::ObjectAccessRulesChain(ObjectAccessRulesChainSubstate {
                access_rules_chain: [access_rules_from_rule_set(input.rule_set)].into(),
            }),
        );

        // Constructing the Access Controller RENode and Substates
        let access_controller = RENodeInit::AccessController(AccessControllerSubstate::new(
            vault.0,
            input.timed_recovery_delay_in_minutes,
        ));

        // Allocating an RENodeId and creating the access controller RENode
        let node_id = api.kernel_allocate_node_id(RENodeType::AccessController)?;
        api.kernel_create_node(node_id, access_controller, node_modules)?;

        // Creating a global component address for the access controller RENode
        let global_node_id = api.kernel_allocate_node_id(RENodeType::GlobalAccessController)?;
        api.kernel_create_node(
            global_node_id,
            RENodeInit::Global(GlobalSubstate::AccessController(node_id.into())),
            BTreeMap::new(),
        )?;

        let address: ComponentAddress = global_node_id.into();
        Ok(IndexedScryptoValue::from_typed(&address))
    }

    fn create_proof<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: AccessControllerCreateProofInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let proof = transition(receiver, api, AccessControllerCreateProofStateMachineInput)?;

        Ok(IndexedScryptoValue::from_typed(&proof))
    }

    fn initial_recovery_as_primary<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let input: AccessControllerInitiateRecoveryAsPrimaryInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;
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

        api.emit_event(AccessControllerInitiateRecoveryEvent {
            proposal,
            proposer: Proposer::Primary,
        })?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn initial_recovery_as_recovery<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let input: AccessControllerInitiateRecoveryAsRecoveryInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;
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

        api.emit_event(AccessControllerInitiateRecoveryEvent {
            proposal,
            proposer: Proposer::Recovery,
        })?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn quick_confirm_primary_role_recovery_proposal<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let input: AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;
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

        api.emit_event(AccessControllerRuleSetUpdateEvent {
            proposal,
            proposer: Proposer::Primary,
        })?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn quick_confirm_recovery_role_recovery_proposal<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let input: AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;
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

        api.emit_event(AccessControllerRuleSetUpdateEvent {
            proposal,
            proposer: Proposer::Recovery,
        })?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn timed_confirm_recovery<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let input: AccessControllerTimedConfirmRecoveryInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;
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

        api.emit_event(AccessControllerRuleSetUpdateEvent {
            proposal,
            proposer: Proposer::Recovery,
        })?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn cancel_primary_role_recovery_proposal<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let _input: AccessControllerCancelPrimaryRoleRecoveryProposalInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        transition_mut(
            receiver,
            api,
            AccessControllerCancelPrimaryRoleRecoveryProposalStateMachineInput,
        )?;

        api.emit_event(AccessControllerCancelRecoveryProposalEvent {
            proposer: Proposer::Primary,
        })?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn cancel_recovery_role_recovery_proposal<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let _input: AccessControllerCancelRecoveryRoleRecoveryProposalInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        transition_mut(
            receiver,
            api,
            AccessControllerCancelRecoveryRoleRecoveryProposalStateMachineInput,
        )?;

        api.emit_event(AccessControllerCancelRecoveryProposalEvent {
            proposer: Proposer::Recovery,
        })?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn lock_primary_role<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let _input: AccessControllerLockPrimaryRoleInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        transition_mut(
            receiver,
            api,
            AccessControllerLockPrimaryRoleStateMachineInput,
        )?;
        api.emit_event(AccessControllerLockPrimaryRoleEvent {})?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn unlock_primary_role<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let _input: AccessControllerUnlockPrimaryRoleInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        transition_mut(
            receiver,
            api,
            AccessControllerUnlockPrimaryRoleStateMachineInput,
        )?;
        api.emit_event(AccessControllerUnlockPrimaryRoleEvent {})?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn stop_timed_recovery<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>,
    {
        let input: AccessControllerStopTimedRecoveryInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

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
        api.emit_event(AccessControllerStopTimedRecoveryEvent {})?;

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

fn access_rules_from_rule_set(rule_set: RuleSet) -> AccessRules {
    let mut access_rules = AccessRules::new();

    // Primary Role Rules
    let primary_group = "primary";
    access_rules.set_group_access_rule(primary_group.into(), rule_set.primary_role.clone());
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::new(
            NodeModuleId::SELF,
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT.to_string(),
        ),
        primary_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::new(
            NodeModuleId::SELF,
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT.to_string(),
        ),
        primary_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::new(
            NodeModuleId::SELF,
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
        ),
        primary_group.into(),
    );

    // Recovery Role Rules
    let recovery_group = "recovery";
    access_rules.set_group_access_rule(recovery_group.into(), rule_set.recovery_role.clone());
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::new(
            NodeModuleId::SELF,
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT.to_string(),
        ),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::new(
            NodeModuleId::SELF,
            ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT.to_string(),
        ),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::new(
            NodeModuleId::SELF,
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
        ),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::new(
            NodeModuleId::SELF,
            ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE.to_string(),
        ),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::new(
            NodeModuleId::SELF,
            ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE.to_string(),
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
        AccessRuleKey::new(
            NodeModuleId::SELF,
            ACCESS_CONTROLLER_STOP_TIMED_RECOVERY.to_string(),
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
        AccessRuleKey::new(
            NodeModuleId::SELF,
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
        ),
        access_rule_or([rule_set.recovery_role, rule_set.confirmation_role.clone()].into()),
    );
    access_rules.set_method_access_rule(
        AccessRuleKey::new(
            NodeModuleId::SELF,
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
        ),
        access_rule_or([rule_set.primary_role, rule_set.confirmation_role].into()),
    );

    let non_fungible_local_id =
        NonFungibleLocalId::bytes(scrypto_encode(&ACCESS_CONTROLLER_PACKAGE).unwrap()).unwrap();
    let non_fungible_global_id = NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_local_id);

    access_rules.default(rule!(deny_all), rule!(require(non_fungible_global_id)))
}

fn transition<Y, I>(
    node_id: RENodeId,
    api: &mut Y,
    input: I,
) -> Result<<AccessControllerSubstate as Transition<I>>::Output, RuntimeError>
where
    Y: KernelNodeApi
        + KernelSubstateApi
        + ClientApi<RuntimeError>
        + ClientNodeApi<RuntimeError>
        + ClientSubstateApi<RuntimeError>,
    AccessControllerSubstate: Transition<I>,
{
    let offset = SubstateOffset::AccessController(AccessControllerOffset::AccessController);
    let handle =
        api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;

    let access_controller_clone = {
        let substate = api.kernel_get_substate_ref(handle)?;
        let access_controller = substate.access_controller();
        access_controller.clone()
    };

    let rtn = access_controller_clone.transition(api, input)?;

    api.kernel_drop_lock(handle)?;

    Ok(rtn)
}

fn transition_mut<Y, I>(
    node_id: RENodeId,
    api: &mut Y,
    input: I,
) -> Result<<AccessControllerSubstate as TransitionMut<I>>::Output, RuntimeError>
where
    Y: KernelNodeApi
        + KernelSubstateApi
        + ClientApi<RuntimeError>
        + ClientNodeApi<RuntimeError>
        + ClientSubstateApi<RuntimeError>,
    AccessControllerSubstate: TransitionMut<I>,
{
    let offset = SubstateOffset::AccessController(AccessControllerOffset::AccessController);
    let handle =
        api.kernel_lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

    let mut access_controller_clone = {
        let substate = api.kernel_get_substate_ref(handle)?;
        let access_controller = substate.access_controller();
        access_controller.clone()
    };

    let rtn = access_controller_clone.transition_mut(api, input)?;

    {
        let mut substate = api.kernel_get_substate_ref_mut(handle)?;
        let access_controller = substate.access_controller();
        *access_controller = access_controller_clone
    }

    api.kernel_drop_lock(handle)?;

    Ok(rtn)
}

fn update_access_rules<Y>(
    api: &mut Y,
    receiver: RENodeId,
    access_rules: AccessRules,
) -> Result<(), RuntimeError>
where
    Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
{
    for (group_name, access_rule) in access_rules.get_all_grouped_auth().iter() {
        api.call_module_method(
            receiver.into(),
            NodeModuleId::AccessRules,
            ACCESS_RULES_SET_GROUP_ACCESS_RULE_IDENT,
            scrypto_encode(&AccessRulesSetGroupAccessRuleInput {
                index: 0,
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
                    receiver.into(),
                    NodeModuleId::AccessRules,
                    ACCESS_RULES_SET_METHOD_ACCESS_RULE_IDENT,
                    scrypto_encode(&AccessRulesSetMethodAccessRuleInput {
                        index: 0,
                        key: method_key.clone(),
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
