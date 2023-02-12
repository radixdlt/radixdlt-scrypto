use super::state_machine::*;
use super::*;
use crate::errors::{ApplicationError, InterpreterError, RuntimeError};
use crate::kernel::{
    deref_and_update, CallFrameUpdate, ExecutableInvocation, Executor, KernelNodeApi,
    KernelSubstateApi, LockFlags, ResolvedActor,
};
use crate::system::global::GlobalAddressSubstate;
use crate::system::node::{RENodeInit, RENodeModuleInit};
use crate::system::node_modules::auth::AccessRulesChainSubstate;
use crate::wasm::WasmEngine;
use native_sdk::resource::{SysBucket, Vault};
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::access_controller::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::{CLOCK, PACKAGE_TOKEN};
use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode, IndexedScryptoValue, ScryptoValue,
};
use radix_engine_interface::*;
use radix_engine_interface::{api::*, rule};
use sbor::rust::collections::BTreeMap;

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
            },
            ACCESS_CONTROLLER_CREATE_PROOF_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::create_proof(receiver, input, api)
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
}

//=====================================
// Access Controller Initiate Recovery
//=====================================

pub struct AccessControllerInitiateRecoveryAsPrimaryExecutable {
    pub receiver: RENodeId,
    pub proposal: RecoveryProposal,
}

impl ExecutableInvocation for AccessControllerInitiateRecoveryAsPrimaryInvocation {
    type Exec = AccessControllerInitiateRecoveryAsPrimaryExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Component(CLOCK)));

        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::InitiateRecoveryAsPrimary),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            proposal: self.proposal,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerInitiateRecoveryAsPrimaryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientNodeApi<RuntimeError>
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        transition_mut(
            self.receiver,
            api,
            AccessControllerInitiateRecoveryAsPrimaryStateMachineInput {
                proposal: self.proposal,
            },
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

pub struct AccessControllerInitiateRecoveryAsRecoveryExecutable {
    pub receiver: RENodeId,
    pub proposal: RecoveryProposal,
}

impl ExecutableInvocation for AccessControllerInitiateRecoveryAsRecoveryInvocation {
    type Exec = AccessControllerInitiateRecoveryAsRecoveryExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Component(CLOCK)));

        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::InitiateRecoveryAsRecovery),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            proposal: self.proposal,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerInitiateRecoveryAsRecoveryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientNodeApi<RuntimeError>
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        transition_mut(
            self.receiver,
            api,
            AccessControllerInitiateRecoveryAsRecoveryStateMachineInput {
                proposal: self.proposal,
            },
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//==========================================
// Access Controller Quick Confirm Recovery
//==========================================

pub struct AccessControllerQuickConfirmPrimaryRoleRecoveryProposalExecutable {
    pub receiver: RENodeId,
    pub proposal_to_confirm: RecoveryProposal,
}

impl ExecutableInvocation for AccessControllerQuickConfirmPrimaryRoleRecoveryProposalInvocation {
    type Exec = AccessControllerQuickConfirmPrimaryRoleRecoveryProposalExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::QuickConfirmPrimaryRoleRecoveryProposal),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            proposal_to_confirm: self.proposal_to_confirm,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerQuickConfirmPrimaryRoleRecoveryProposalExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientNodeApi<RuntimeError>
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        let recovery_proposal = transition_mut(
            self.receiver,
            api,
            AccessControllerQuickConfirmPrimaryRoleRecoveryProposalStateMachineInput {
                proposal_to_confirm: self.proposal_to_confirm,
            },
        )?;

        update_access_rules(
            api,
            self.receiver,
            access_rules_from_rule_set(recovery_proposal.rule_set),
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

pub struct AccessControllerQuickConfirmRecoveryRoleRecoveryProposalExecutable {
    pub receiver: RENodeId,
    pub proposal_to_confirm: RecoveryProposal,
}

impl ExecutableInvocation for AccessControllerQuickConfirmRecoveryRoleRecoveryProposalInvocation {
    type Exec = AccessControllerQuickConfirmRecoveryRoleRecoveryProposalExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(
                AccessControllerFn::QuickConfirmRecoveryRoleRecoveryProposal,
            ),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            proposal_to_confirm: self.proposal_to_confirm,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerQuickConfirmRecoveryRoleRecoveryProposalExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientNodeApi<RuntimeError>
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        let recovery_proposal = transition_mut(
            self.receiver,
            api,
            AccessControllerQuickConfirmRecoveryRoleRecoveryProposalStateMachineInput {
                proposal_to_confirm: self.proposal_to_confirm,
            },
        )?;

        update_access_rules(
            api,
            self.receiver,
            access_rules_from_rule_set(recovery_proposal.rule_set),
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//==========================================
// Access Controller Timed Confirm Recovery
//==========================================

pub struct AccessControllerTimedConfirmRecoveryExecutable {
    pub receiver: RENodeId,
    pub proposal_to_confirm: RecoveryProposal,
}

impl ExecutableInvocation for AccessControllerTimedConfirmRecoveryInvocation {
    type Exec = AccessControllerTimedConfirmRecoveryExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        call_frame_update.add_ref(RENodeId::Global(GlobalAddress::Component(CLOCK)));
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::TimedConfirmRecovery),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            proposal_to_confirm: self.proposal_to_confirm,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerTimedConfirmRecoveryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientNodeApi<RuntimeError>
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        let recovery_proposal = transition_mut(
            self.receiver,
            api,
            AccessControllerTimedConfirmRecoveryStateMachineInput {
                proposal_to_confirm: self.proposal_to_confirm,
            },
        )?;

        // Update the access rules
        update_access_rules(
            api,
            self.receiver,
            access_rules_from_rule_set(recovery_proposal.rule_set),
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//===========================================
// Access Controller Cancel Recovery Attempt
//===========================================

pub struct AccessControllerCancelPrimaryRoleRecoveryProposalExecutable {
    pub receiver: RENodeId,
}

impl ExecutableInvocation for AccessControllerCancelPrimaryRoleRecoveryProposalInvocation {
    type Exec = AccessControllerCancelPrimaryRoleRecoveryProposalExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::CancelPrimaryRoleRecoveryProposal),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerCancelPrimaryRoleRecoveryProposalExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientNodeApi<RuntimeError>
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        transition_mut(
            self.receiver,
            api,
            AccessControllerCancelPrimaryRoleRecoveryProposalStateMachineInput,
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

pub struct AccessControllerCancelRecoveryRoleRecoveryProposalExecutable {
    pub receiver: RENodeId,
}

impl ExecutableInvocation for AccessControllerCancelRecoveryRoleRecoveryProposalInvocation {
    type Exec = AccessControllerCancelRecoveryRoleRecoveryProposalExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::CancelRecoveryRoleRecoveryProposal),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerCancelRecoveryRoleRecoveryProposalExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientNodeApi<RuntimeError>
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        transition_mut(
            self.receiver,
            api,
            AccessControllerCancelRecoveryRoleRecoveryProposalStateMachineInput,
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//=====================================
// Access Controller Lock Primary Role
//=====================================

pub struct AccessControllerLockPrimaryRoleExecutable {
    pub receiver: RENodeId,
}

impl ExecutableInvocation for AccessControllerLockPrimaryRoleInvocation {
    type Exec = AccessControllerLockPrimaryRoleExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::LockPrimaryRole),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerLockPrimaryRoleExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientNodeApi<RuntimeError>
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        transition_mut(
            self.receiver,
            api,
            AccessControllerLockPrimaryRoleStateMachineInput,
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//=======================================
// Access Controller Unlock Primary Role
//=======================================

pub struct AccessControllerUnlockPrimaryRoleExecutable {
    pub receiver: RENodeId,
}

impl ExecutableInvocation for AccessControllerUnlockPrimaryRoleInvocation {
    type Exec = AccessControllerUnlockPrimaryRoleExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::UnlockPrimaryRole),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerUnlockPrimaryRoleExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientNodeApi<RuntimeError>
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        transition_mut(
            self.receiver,
            api,
            AccessControllerUnlockPrimaryRoleStateMachineInput,
        )?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//=======================================
// Access Controller Stop Timed Recovery
//=======================================

pub struct AccessControllerStopTimedRecoveryExecutable {
    pub receiver: RENodeId,
    pub proposal: RecoveryProposal,
}

impl ExecutableInvocation for AccessControllerStopTimedRecoveryInvocation {
    type Exec = AccessControllerStopTimedRecoveryExecutable;

    fn resolve<D: ClientDerefApi<RuntimeError>>(
        self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let mut call_frame_update = CallFrameUpdate::empty();
        let receiver = RENodeId::Global(GlobalAddress::Component(self.receiver));
        let resolved_receiver = deref_and_update(receiver, &mut call_frame_update, deref)?;

        let actor = ResolvedActor::method(
            NativeFn::AccessController(AccessControllerFn::StopTimedRecovery),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            proposal: self.proposal,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerStopTimedRecoveryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientNodeApi<RuntimeError>
            + ClientSubstateApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        transition_mut(
            self.receiver,
            api,
            AccessControllerStopTimedRecoveryStateMachineInput {
                proposal: self.proposal,
            },
        )?;

        Ok(((), CallFrameUpdate::empty()))
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
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::InitiateRecoveryAsPrimary,
        )),
        primary_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::CancelPrimaryRoleRecoveryProposal,
        )),
        primary_group.into(),
    );

    // Recovery Role Rules
    let recovery_group = "recovery";
    access_rules.set_group_access_rule(recovery_group.into(), rule_set.recovery_role.clone());
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::InitiateRecoveryAsRecovery,
        )),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::TimedConfirmRecovery,
        )),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::CancelRecoveryRoleRecoveryProposal,
        )),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::LockPrimaryRole,
        )),
        recovery_group.into(),
    );
    access_rules.set_method_access_rule_to_group(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::UnlockPrimaryRole,
        )),
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
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::StopTimedRecovery,
        )),
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
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::QuickConfirmPrimaryRoleRecoveryProposal,
        )),
        access_rule_or([rule_set.recovery_role, rule_set.confirmation_role.clone()].into()),
    );
    access_rules.set_method_access_rule(
        AccessRuleKey::Native(NativeFn::AccessController(
            AccessControllerFn::QuickConfirmRecoveryRoleRecoveryProposal,
        )),
        access_rule_or([rule_set.primary_role, rule_set.confirmation_role].into()),
    );

    let non_fungible_local_id = NonFungibleLocalId::Bytes(
        scrypto_encode(&PackageIdentifier::Native(NativePackage::AccessController)).unwrap(),
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
