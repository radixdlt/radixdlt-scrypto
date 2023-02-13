use super::state_machine::*;
use crate::errors::{ApplicationError, InterpreterError, RuntimeError};
use crate::kernel::{KernelNodeApi, KernelSubstateApi, LockFlags};
use crate::system::global::GlobalAddressSubstate;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_modules::auth::AccessRulesChainSubstate;
use native_sdk::resource::{SysBucket, Vault};
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::types::*;
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, Default)]
pub enum PrimaryRoleState {
    #[default]
    Unlocked,
    Locked,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, Default)]
pub enum PrimaryOperationState {
    #[default]
    Normal,
    Recovery(RecoveryProposal),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode, Default)]
pub enum RecoveryOperationState {
    #[default]
    Normal,
    Recovery(RecoveryRecoveryState),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum RecoveryRecoveryState {
    Untimed(RecoveryProposal),
    Timed {
        proposal: RecoveryProposal,
        timed_recovery_allowed_after: Instant,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

pub struct AccessControllerNativePackage;

//=================================
// Access Controller Create Global
//=================================

impl AccessControllerNativePackage {
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<ComponentId>,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        match export_name {
            ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT => {
                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                Self::create_global(input, api)
            }
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::create_proof(receiver, input, api)
            }
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::initial_recovery_as_primary(receiver, input, api)
            }
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::initial_recovery_as_recovery(receiver, input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::quick_confirm_primary_role_recovery_proposal(receiver, input, api)
            }
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::quick_confirm_recovery_role_recovery_proposal(receiver, input, api)
            }
            ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::timed_confirm_recovery(receiver, input, api)
            }
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::cancel_primary_role_recovery_proposal(receiver, input, api)
            }
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::cancel_recovery_role_recovery_proposal(receiver, input, api)
            }
            ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::lock_primary_role(receiver, input, api)
            }
            ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::unlock_primary_role(receiver, input, api)
            }
            ACCESS_CONTROLLER_STOP_TIMED_RECOVERY => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::stop_timed_recovery(receiver, input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::InvalidInvocation,
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
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
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
            RENodeModuleInit::AccessRulesChain(AccessRulesChainSubstate {
                access_rules_chain: [access_rules_from_rule_set(input.rule_set)].into(),
            }),
        );

        // Constructing the Access Controller RENode and Substates
        let access_controller = RENodeInit::AccessController(AccessControllerSubstate::new(
            vault.0,
            input.timed_recovery_delay_in_minutes,
        ));

        // Allocating an RENodeId and creating the access controller RENode
        let node_id = api.allocate_node_id(RENodeType::AccessController)?;
        api.create_node(node_id, access_controller, node_modules)?;

        // Creating a global component address for the access controller RENode
        let global_node_id = api.allocate_node_id(RENodeType::GlobalAccessController)?;
        api.create_node(
            global_node_id,
            RENodeInit::Global(GlobalAddressSubstate::AccessController(node_id.into())),
            BTreeMap::new(),
        )?;

        let address: ComponentAddress = global_node_id.into();
        Ok(IndexedScryptoValue::from_typed(&address))
    }

    fn create_proof<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let _input: AccessControllerCreateProofInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let proof = transition(
            RENodeId::AccessController(receiver),
            api,
            AccessControllerCreateProofStateMachineInput,
        )?;

        Ok(IndexedScryptoValue::from_typed(&proof))
    }

    fn initial_recovery_as_primary<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        let input: AccessControllerInitiateRecoveryAsPrimaryInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        transition_mut(
            RENodeId::AccessController(receiver),
            api,
            AccessControllerInitiateRecoveryAsPrimaryStateMachineInput {
                proposal: RecoveryProposal {
                    rule_set: input.rule_set,
                    timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
                },
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn initial_recovery_as_recovery<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        let input: AccessControllerInitiateRecoveryAsRecoveryInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        transition_mut(
            RENodeId::AccessController(receiver),
            api,
            AccessControllerInitiateRecoveryAsRecoveryStateMachineInput {
                proposal: RecoveryProposal {
                    rule_set: input.rule_set,
                    timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
                },
            },
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn quick_confirm_primary_role_recovery_proposal<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        let input: AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let recovery_proposal = transition_mut(
            RENodeId::AccessController(receiver),
            api,
            AccessControllerQuickConfirmPrimaryRoleRecoveryProposalStateMachineInput {
                proposal_to_confirm: RecoveryProposal {
                    rule_set: input.rule_set,
                    timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
                },
            },
        )?;

        update_access_rules(
            api,
            RENodeId::AccessController(receiver),
            access_rules_from_rule_set(recovery_proposal.rule_set),
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn quick_confirm_recovery_role_recovery_proposal<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        let input: AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let recovery_proposal = transition_mut(
            RENodeId::AccessController(receiver),
            api,
            AccessControllerQuickConfirmRecoveryRoleRecoveryProposalStateMachineInput {
                proposal_to_confirm: RecoveryProposal {
                    rule_set: input.rule_set,
                    timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
                },
            },
        )?;

        update_access_rules(
            api,
            RENodeId::AccessController(receiver),
            access_rules_from_rule_set(recovery_proposal.rule_set),
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn timed_confirm_recovery<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        let input: AccessControllerTimedConfirmRecoveryInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let recovery_proposal = transition_mut(
            RENodeId::AccessController(receiver),
            api,
            AccessControllerTimedConfirmRecoveryStateMachineInput {
                proposal_to_confirm: RecoveryProposal {
                    rule_set: input.rule_set,
                    timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
                },
            },
        )?;

        // Update the access rules
        update_access_rules(
            api,
            RENodeId::AccessController(receiver),
            access_rules_from_rule_set(recovery_proposal.rule_set),
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn cancel_primary_role_recovery_proposal<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        let _input: AccessControllerCancelPrimaryRoleRecoveryProposalInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        transition_mut(
            RENodeId::AccessController(receiver),
            api,
            AccessControllerCancelPrimaryRoleRecoveryProposalStateMachineInput,
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn cancel_recovery_role_recovery_proposal<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        let _input: AccessControllerCancelRecoveryRoleRecoveryProposalInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        transition_mut(
            RENodeId::AccessController(receiver),
            api,
            AccessControllerCancelRecoveryRoleRecoveryProposalStateMachineInput,
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn lock_primary_role<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        let _input: AccessControllerLockPrimaryRoleInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        transition_mut(
            RENodeId::AccessController(receiver),
            api,
            AccessControllerLockPrimaryRoleStateMachineInput,
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn unlock_primary_role<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        let _input: AccessControllerUnlockPrimaryRoleInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        transition_mut(
            RENodeId::AccessController(receiver),
            api,
            AccessControllerUnlockPrimaryRoleStateMachineInput,
        )?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn stop_timed_recovery<Y>(
        receiver: ComponentId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        let input: AccessControllerStopTimedRecoveryInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        transition_mut(
            RENodeId::AccessController(receiver),
            api,
            AccessControllerStopTimedRecoveryStateMachineInput {
                proposal: RecoveryProposal {
                    rule_set: input.rule_set,
                    timed_recovery_delay_in_minutes: input.timed_recovery_delay_in_minutes,
                },
            },
        )?;

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
        AccessRuleKey::ScryptoMethod(ACCESS_CONTROLLER_CREATE_PROOF_IDENT.to_string()),
        primary_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::ScryptoMethod(
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_PRIMARY_IDENT.to_string(),
        ),
        primary_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::ScryptoMethod(
            ACCESS_CONTROLLER_CANCEL_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
        ),
        primary_group.into(),
    );

    // Recovery Role Rules
    let recovery_group = "recovery";
    access_rules.set_group_access_rule(recovery_group.into(), rule_set.recovery_role.clone());
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::ScryptoMethod(
            ACCESS_CONTROLLER_INITIATE_RECOVERY_AS_RECOVERY_IDENT.to_string(),
        ),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::ScryptoMethod(ACCESS_CONTROLLER_TIMED_CONFIRM_RECOVERY_IDENT.to_string()),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::ScryptoMethod(
            ACCESS_CONTROLLER_CANCEL_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
        ),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::ScryptoMethod(ACCESS_CONTROLLER_LOCK_PRIMARY_ROLE.to_string()),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::ScryptoMethod(ACCESS_CONTROLLER_UNLOCK_PRIMARY_ROLE.to_string()),
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
        AccessRuleKey::ScryptoMethod(ACCESS_CONTROLLER_STOP_TIMED_RECOVERY.to_string()),
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
        AccessRuleKey::ScryptoMethod(
            ACCESS_CONTROLLER_QUICK_CONFIRM_PRIMARY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
        ),
        access_rule_or([rule_set.recovery_role, rule_set.confirmation_role.clone()].into()),
    );
    access_rules.set_method_access_rule(
        AccessRuleKey::ScryptoMethod(
            ACCESS_CONTROLLER_QUICK_CONFIRM_RECOVERY_ROLE_RECOVERY_PROPOSAL_IDENT.to_string(),
        ),
        access_rule_or([rule_set.primary_role, rule_set.confirmation_role].into()),
    );

    let non_fungible_local_id = NonFungibleLocalId::Bytes(
        scrypto_encode(&PackageIdentifier::Scrypto(ACCESS_CONTROLLER_PACKAGE)).unwrap(),
    );
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
        + ClientSubstateApi<RuntimeError>
        + ClientNativeInvokeApi<RuntimeError>,
    AccessControllerSubstate: Transition<I>,
{
    let offset = SubstateOffset::AccessController(AccessControllerOffset::AccessController);
    let handle = api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::read_only())?;

    let access_controller_clone = {
        let substate = api.get_ref(handle)?;
        let access_controller = substate.access_controller();
        access_controller.clone()
    };

    let rtn = access_controller_clone.transition(api, input)?;

    api.drop_lock(handle)?;

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
        + ClientSubstateApi<RuntimeError>
        + ClientNativeInvokeApi<RuntimeError>,
    AccessControllerSubstate: TransitionMut<I>,
{
    let offset = SubstateOffset::AccessController(AccessControllerOffset::AccessController);
    let handle = api.lock_substate(node_id, NodeModuleId::SELF, offset, LockFlags::MUTABLE)?;

    let mut access_controller_clone = {
        let substate = api.get_ref(handle)?;
        let access_controller = substate.access_controller();
        access_controller.clone()
    };

    let rtn = access_controller_clone.transition_mut(api, input)?;

    {
        let mut substate = api.get_ref_mut(handle)?;
        let access_controller = substate.access_controller();
        *access_controller = access_controller_clone
    }

    api.drop_lock(handle)?;

    Ok(rtn)
}

fn update_access_rules<Y>(
    api: &mut Y,
    receiver: RENodeId,
    access_rules: AccessRules,
) -> Result<(), RuntimeError>
where
    Y: KernelNodeApi
        + KernelSubstateApi
        + ClientNodeApi<RuntimeError>
        + ClientSubstateApi<RuntimeError>
        + ClientNativeInvokeApi<RuntimeError>,
{
    for (group_name, access_rule) in access_rules.get_all_grouped_auth().iter() {
        api.call_native(AccessRulesSetGroupAccessRuleInvocation {
            receiver: receiver,
            index: 0,
            name: group_name.into(),
            rule: access_rule.clone(),
        })?;
    }
    for (method_key, entry) in access_rules.get_all_method_auth().iter() {
        match entry {
            AccessRuleEntry::AccessRule(access_rule) => {
                api.call_native(AccessRulesSetMethodAccessRuleInvocation {
                    receiver: receiver,
                    index: 0,
                    key: method_key.clone(),
                    rule: AccessRuleEntry::AccessRule(access_rule.clone()),
                })?;
            }
            AccessRuleEntry::Group(..) => {} // Already updated above
        }
    }
    Ok(())
}
