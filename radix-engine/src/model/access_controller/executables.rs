use std::collections::HashMap;

use crate::engine::{deref_and_update, ApplicationError, Executor, LockFlags, RENode};
use crate::engine::{
    CallFrameUpdate, ExecutableInvocation, ResolvedActor, ResolverApi, RuntimeError, SystemApi,
};
use crate::model::GlobalAddressSubstate;
use crate::wasm::WasmEngine;
use native_sdk::resource::{ComponentAuthZone, SysBucket, Vault};
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::types::*;
use radix_engine_interface::constants::CLOCK;
use radix_engine_interface::*;
use radix_engine_interface::{api::*, rule};

use super::AccessControllerSubstate;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum AccessControllerError {
    RecoveryForThisRoleAlreadyExists { role: Role },
    NoValidProposedRuleSetExists,
    TimeOverflow,
}

impl From<AccessControllerError> for RuntimeError {
    fn from(value: AccessControllerError) -> Self {
        RuntimeError::ApplicationError(ApplicationError::AccessControllerError(value))
    }
}

//=================================
// Access Controller Create Global
//=================================

impl ExecutableInvocation for AccessControllerCreateGlobalInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        self,
        _deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>
    where
        Self: Sized,
    {
        let actor =
            ResolvedActor::function(NativeFn::AccessController(AccessControllerFn::CreateGlobal));
        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Bucket(self.controlled_asset));

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessControllerCreateGlobalInvocation {
    type Output = ComponentAddress;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Creating a new vault and putting in it the controlled asset
        let vault = {
            let controlled_asset = Bucket(self.controlled_asset);

            let mut vault = controlled_asset
                .sys_resource_address(api)
                .and_then(|resource_address| Vault::sys_new(resource_address, api))?;
            vault.sys_put(controlled_asset, api)?;

            vault
        };

        // Creating the Access Controller substate
        let substate = AccessControllerSubstate {
            controlled_asset: vault.0,
            active_rule_set: self.rule_set,
            ongoing_recoveries: None,
            timed_recovery_delay_in_hours: self.timed_recovery_delay_in_hours,
            is_primary_role_locked: false,
        };

        // Allocating an RENodeId and creating the access controller RENode
        let node_id = api.allocate_node_id(RENodeType::AccessController)?;
        api.create_node(node_id, RENode::AccessController(substate))?;

        // Creating a global component address for the access controller RENode
        let global_node_id = api.allocate_node_id(RENodeType::GlobalAccessController)?;
        api.create_node(
            global_node_id,
            RENode::Global(GlobalAddressSubstate::AccessController(node_id.into())),
        )?;

        Ok((global_node_id.into(), CallFrameUpdate::empty()))
    }
}

//================================
// Access Controller Create Proof
//================================

impl ExecutableInvocation for AccessControllerCreateProofInvocation {
    type Exec = AccessControllerCreateProofExecutable;

    fn resolve<D: ResolverApi>(
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
            NativeFn::AccessController(AccessControllerFn::CreateProof),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerCreateProofExecutable {
    type Output = Proof;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Access Controller Substate Handle
        let node_id = self.receiver;
        let offset = SubstateOffset::AccessController(AccessControllerOffset::AccessController);
        let handle = api.lock_substate(node_id, offset, LockFlags::read_only())?;

        // Auth Check
        {
            let substate = api.get_ref(handle)?;
            let access_controller = substate.access_controller();

            // Proofs may only be created by the primary role when the primary role is NOT locked.
            // It doesn't matter whether the controller is in recovery mode or not.
            let rule = if !access_controller.is_primary_role_locked {
                access_controller.active_rule_set.primary_role.clone()
            } else {
                // TODO: Let's error out early instead of doing a check that we know will fail.
                rule!(deny_all)
            };
            ComponentAuthZone::assert_access_rule(rule, api)?;
        }

        // Creating a proof of the controlled asset
        let proof = {
            let substate = api.get_ref(handle)?;
            let access_controller = substate.access_controller();

            Vault(access_controller.controlled_asset).sys_create_proof(api)?
        };

        let call_frame_update = CallFrameUpdate::move_node(RENodeId::Proof(proof.0));
        api.drop_lock(handle)?;

        Ok((proof, call_frame_update))
    }
}

//===============================================
// Access Controller Update Timed Recovery Delay
//===============================================

impl ExecutableInvocation for AccessControllerUpdateTimedRecoveryDelayInvocation {
    type Exec = AccessControllerUpdateTimedRecoveryDelayExecutable;

    fn resolve<D: ResolverApi>(
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
            NativeFn::AccessController(AccessControllerFn::CreateProof),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            timed_recovery_delay_in_hours: self.timed_recovery_delay_in_hours,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerUpdateTimedRecoveryDelayExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Access Controller Substate Handle
        let node_id = self.receiver;
        let offset = SubstateOffset::AccessController(AccessControllerOffset::AccessController);
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        // Auth Check
        {
            let substate = api.get_ref(handle)?;
            let access_controller = substate.access_controller();

            // The timed recovery delay may only be updated by the primary role when:
            //    a) it's not locked
            //    b) we're not in recovery mode
            let rule = match (
                access_controller.is_primary_role_locked,
                access_controller.ongoing_recoveries.as_ref(),
            ) {
                (false, None) => access_controller.active_rule_set.primary_role.clone(),
                _ => rule!(deny_all), // TODO: Let's error out early instead of doing a check that we know will fail.
            };
            ComponentAuthZone::assert_access_rule(rule, api)?;
        }

        // Update the timed recovery delay
        {
            let mut substate = api.get_ref_mut(handle)?;
            let access_controller = substate.access_controller();
            access_controller.timed_recovery_delay_in_hours = self.timed_recovery_delay_in_hours;
        }

        api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//=====================================
// Access Controller Initiate Recovery
//=====================================

impl ExecutableInvocation for AccessControllerInitiateRecoveryInvocation {
    type Exec = AccessControllerInitiateRecoveryExecutable;

    fn resolve<D: ResolverApi>(
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
            NativeFn::AccessController(AccessControllerFn::InitiateRecovery),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
            role: self.role,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerInitiateRecoveryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Access Controller Substate Handle
        let node_id = self.receiver;
        let offset = SubstateOffset::AccessController(AccessControllerOffset::AccessController);
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        // Auth Check
        {
            let substate = api.get_ref(handle)?;
            let access_controller = substate.access_controller();

            // There are two cases here:
            //      - When primary is unlocked, then primary or recovery may initiate the recovery
            //        process.
            //      - When primary is locked, only recovery can initiate the recovery process.
            let rule = match (access_controller.is_primary_role_locked, self.role) {
                (false, Role::Primary) => access_controller.active_rule_set.primary_role.clone(),
                (_, Role::Recovery) => access_controller.active_rule_set.recovery_role.clone(),
                _ => rule!(deny_all), // TODO: Let's error out early instead of doing a check that we know will fail.
            };
            ComponentAuthZone::assert_access_rule(rule, api)?;
        }

        // Getting the current time
        let current_time = Runtime::sys_current_time(api, TimePrecision::Minute)?;

        // Initiate Recovery (if this role doesn't already have a recovery Ongoing)
        {
            let mut substate = api.get_ref_mut(handle)?;
            let access_controller = substate.access_controller();

            match access_controller.ongoing_recoveries.as_mut() {
                Some(ongoing_recoveries) => {
                    if !ongoing_recoveries.contains_key(&self.role) {
                        ongoing_recoveries.insert(self.role, (self.rule_set, current_time));
                    } else {
                        Err(AccessControllerError::RecoveryForThisRoleAlreadyExists {
                            role: self.role,
                        })?;
                    }
                }
                None => {
                    access_controller.ongoing_recoveries =
                        Some([(self.role.clone(), (self.rule_set, current_time))].into())
                }
            }
        }

        api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//==========================================
// Access Controller Quick Confirm Recovery
//==========================================

impl ExecutableInvocation for AccessControllerQuickConfirmRecoveryInvocation {
    type Exec = AccessControllerQuickConfirmRecoveryExecutable;

    fn resolve<D: ResolverApi>(
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
            NativeFn::AccessController(AccessControllerFn::QuickConfirmRecovery),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
            proposer: self.proposer,
            role: self.role,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerQuickConfirmRecoveryExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Access Controller Substate Handle
        let node_id = self.receiver;
        let offset = SubstateOffset::AccessController(AccessControllerOffset::AccessController);
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        // Auth Check
        {
            let substate = api.get_ref(handle)?;
            let access_controller = substate.access_controller();

            // All roles are allowed to confirm the recovery (even primary when they're locked)
            let rule = match self.role {
                Role::Primary => access_controller.active_rule_set.primary_role.clone(),
                Role::Recovery => access_controller.active_rule_set.recovery_role.clone(),
                Role::Confirmation => access_controller.active_rule_set.confirmation_role.clone(),
            };
            ComponentAuthZone::assert_access_rule(rule, api)?;
        }

        // Quick confirm and update active rule set
        {
            let mut substate = api.get_ref_mut(handle)?;
            let access_controller = substate.access_controller();

            let new_rule_set = access_controller
                .ongoing_recoveries
                .as_ref()
                .map(|ongoing_recoveries| ongoing_recoveries)
                .unwrap_or(&HashMap::new())
                .iter()
                .find(|(proposer, (proposed_rule_set, _))| {
                    **proposer == self.proposer
                        && *proposed_rule_set == self.rule_set
                        && self.proposer != self.role
                })
                .map_or(
                    Err(AccessControllerError::NoValidProposedRuleSetExists),
                    |(_, (rule_set, _))| Ok(rule_set.clone()),
                )?;

            access_controller.ongoing_recoveries = None;
            access_controller.active_rule_set = new_rule_set;
        }

        api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//==========================================
// Access Controller Timed Confirm Recovery
//==========================================

impl ExecutableInvocation for AccessControllerTimedConfirmRecoveryInvocation {
    type Exec = AccessControllerTimedConfirmRecoveryExecutable;

    fn resolve<D: ResolverApi>(
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
            rule_set: self.rule_set,
            proposer: self.proposer,
            role: self.role,
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
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Access Controller Substate Handle
        let node_id = self.receiver;
        let offset = SubstateOffset::AccessController(AccessControllerOffset::AccessController);
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        // Auth Check
        {
            let substate = api.get_ref(handle)?;
            let access_controller = substate.access_controller();

            // All roles are allowed to confirm the recovery (even primary when they're locked)
            let rule = match self.role {
                Role::Primary => access_controller.active_rule_set.primary_role.clone(),
                Role::Recovery => access_controller.active_rule_set.recovery_role.clone(),
                Role::Confirmation => access_controller.active_rule_set.confirmation_role.clone(),
            };
            ComponentAuthZone::assert_access_rule(rule, api)?;
        }

        // Getting the RuleSet (if exists) of the new active role
        let new_rule_set = {
            let substate = api.get_ref(handle)?;
            let access_controller = substate.access_controller();

            let (new_rule_set, proposed_at) = access_controller
                .ongoing_recoveries
                .as_ref()
                .map(|ongoing_recoveries| ongoing_recoveries)
                .unwrap_or(&HashMap::new())
                .iter()
                .find(|(proposer, (proposed_rule_set, _))| {
                    **proposer == self.proposer
                        && *proposed_rule_set == self.rule_set
                        && self.proposer == self.role
                })
                .map_or(
                    Err(AccessControllerError::NoValidProposedRuleSetExists),
                    |(_, (rule_set, proposed_at))| Ok((rule_set.clone(), proposed_at.clone())),
                )?;
            proposed_at
                .add_hours(access_controller.timed_recovery_delay_in_hours as i64)
                .map_or(
                    Err(RuntimeError::from(AccessControllerError::TimeOverflow)),
                    |instant| {
                        Runtime::sys_compare_against_current_time(
                            api,
                            instant,
                            TimePrecision::Minute,
                            time::TimeComparisonOperator::Lte,
                        )
                    },
                )?;

            new_rule_set
        };

        // Setting the new rules to the access controller
        {
            let mut substate = api.get_ref_mut(handle)?;
            let access_controller = substate.access_controller();

            access_controller.ongoing_recoveries = None;
            access_controller.active_rule_set = new_rule_set;
        }

        api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//===========================================
// Access Controller Cancel Recovery Attempt
//===========================================

impl ExecutableInvocation for AccessControllerCancelRecoveryAttemptInvocation {
    type Exec = AccessControllerCancelRecoveryAttemptExecutable;

    fn resolve<D: ResolverApi>(
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
            NativeFn::AccessController(AccessControllerFn::CancelRecoveryAttempt),
            resolved_receiver,
        );

        let executor = Self::Exec {
            receiver: resolved_receiver.receiver,
            rule_set: self.rule_set,
            role: self.role,
        };

        Ok((actor, call_frame_update, executor))
    }
}

impl Executor for AccessControllerCancelRecoveryAttemptExecutable {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Access Controller Substate Handle
        let node_id = self.receiver;
        let offset = SubstateOffset::AccessController(AccessControllerOffset::AccessController);
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        // Auth Check
        {
            let substate = api.get_ref(handle)?;
            let access_controller = substate.access_controller();

            // All roles are allowed to confirm the recovery (even primary when they're locked)
            let rule = match self.role {
                Role::Primary => access_controller.active_rule_set.primary_role.clone(),
                Role::Recovery => access_controller.active_rule_set.recovery_role.clone(),
                Role::Confirmation => access_controller.active_rule_set.confirmation_role.clone(),
            };
            ComponentAuthZone::assert_access_rule(rule, api)?;
        }

        // Removing the proposes rule set
        {
            let mut substate = api.get_ref_mut(handle)?;
            let access_controller = substate.access_controller();

            access_controller
                .ongoing_recoveries
                .as_mut()
                .unwrap_or(&mut HashMap::new())
                .remove_entry(&self.role)
                .map_or(
                    Err(RuntimeError::from(
                        AccessControllerError::NoValidProposedRuleSetExists,
                    )),
                    |_| Ok(()),
                )?;
        }

        api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//=====================================
// Access Controller Lock Primary Role
//=====================================

impl ExecutableInvocation for AccessControllerLockPrimaryRoleInvocation {
    type Exec = AccessControllerLockPrimaryRoleExecutable;

    fn resolve<D: ResolverApi>(
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
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Access Controller Substate Handle
        let node_id = self.receiver;
        let offset = SubstateOffset::AccessController(AccessControllerOffset::AccessController);
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        // Auth Check
        {
            let substate = api.get_ref(handle)?;
            let access_controller = substate.access_controller();

            // All roles are allowed to confirm the recovery (even primary when they're locked)
            let rule = access_rule_or(vec![
                access_controller.active_rule_set.primary_role.clone(),
                access_controller.active_rule_set.recovery_role.clone(),
            ]);
            ComponentAuthZone::assert_access_rule(rule, api)?;
        }

        // Lock the primary role
        {
            let mut substate = api.get_ref_mut(handle)?;
            let access_controller = substate.access_controller();

            access_controller.is_primary_role_locked = true
        }

        api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

//=======================================
// Access Controller Unlock Primary Role
//=======================================

impl ExecutableInvocation for AccessControllerUnlockPrimaryRoleInvocation {
    type Exec = AccessControllerUnlockPrimaryRoleExecutable;

    fn resolve<D: ResolverApi>(
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
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Access Controller Substate Handle
        let node_id = self.receiver;
        let offset = SubstateOffset::AccessController(AccessControllerOffset::AccessController);
        let handle = api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        // Auth Check
        {
            let substate = api.get_ref(handle)?;
            let access_controller = substate.access_controller();

            // All roles are allowed to confirm the recovery (even primary when they're locked)
            let rule = access_rule_or(vec![
                access_controller.active_rule_set.recovery_role.clone(),
                access_controller.active_rule_set.confirmation_role.clone(),
            ]);
            ComponentAuthZone::assert_access_rule(rule, api)?;
        }

        // Unlock the primary role
        {
            let mut substate = api.get_ref_mut(handle)?;
            let access_controller = substate.access_controller();

            access_controller.is_primary_role_locked = false
        }

        api.drop_lock(handle)?;

        Ok(((), CallFrameUpdate::empty()))
    }
}

pub fn access_rule_or(access_rules: Vec<AccessRule>) -> AccessRule {
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
