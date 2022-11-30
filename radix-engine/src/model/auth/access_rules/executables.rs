use crate::engine::{
    deref_and_update, ApplicationError, CallFrameUpdate, ExecutableInvocation, InterpreterError,
    Invokable, LockFlags, MethodDeref, NativeExecutor, NativeProcedure, REActor, ResolvedMethod,
    RuntimeError, SystemApi,
};
use crate::model::{MethodAuthorization, MethodAuthorizationError};
use crate::types::*;
use radix_engine_interface::api::api::{EngineApi, Invocation, SysInvokableNative};
use radix_engine_interface::api::types::{
    AccessRulesMethod, GlobalAddress, NativeMethod, PackageOffset, RENodeId, SubstateOffset,
};
use radix_engine_interface::model::*;

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum AccessRulesError {
    BlueprintFunctionNotFound(String),
    InvalidIndex(u32),
    Unauthorized(MethodAuthorization, MethodAuthorizationError),
    CannotSetAccessRuleOnSetAccessRule,
}

impl ExecutableInvocation for AccessRulesAddAccessCheckInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        mut self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();

        let resolved_receiver = deref_and_update(self.receiver, &mut call_frame_update, deref)?;
        // TODO: Move this into a more static check once node types implemented
        if !matches!(resolved_receiver.receiver, RENodeId::Component(..)) {
            return Err(RuntimeError::InterpreterError(
                InterpreterError::InvalidInvocation,
            ));
        }
        self.receiver = resolved_receiver.receiver;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::AccessRules(AccessRulesMethod::AddAccessCheck)),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AccessRulesAddAccessCheckInvocation {
    type Output = ();

    fn main<Y>(
        self,
        system_api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + Invokable<ScryptoInvocation> + EngineApi<RuntimeError>,
    {
        // Abi checks
        {
            let offset = SubstateOffset::Component(ComponentOffset::Info);
            let handle = system_api.lock_substate(self.receiver, offset, LockFlags::read_only())?;

            let (package_id, blueprint_name) = {
                let substate_ref = system_api.get_ref(handle)?;
                let component_info = substate_ref.component_info();
                let package_address = component_info.package_address;
                let blueprint_name = component_info.blueprint_name.to_owned();
                (
                    RENodeId::Global(GlobalAddress::Package(package_address)),
                    blueprint_name,
                )
            };

            let package_offset = SubstateOffset::Package(PackageOffset::Info);
            let handle =
                system_api.lock_substate(package_id, package_offset, LockFlags::read_only())?;
            let substate_ref = system_api.get_ref(handle)?;
            let package = substate_ref.package_info();
            let blueprint_abi = package.blueprint_abi(&blueprint_name).unwrap_or_else(|| {
                panic!(
                    "Blueprint {} is not found in package node {:?}",
                    blueprint_name, package_id
                )
            });
            for (key, _) in self.access_rules.iter() {
                if let AccessRuleKey::ScryptoMethod(func_name) = key {
                    if !blueprint_abi.contains_fn(func_name.as_str()) {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::AccessRulesError(
                                AccessRulesError::BlueprintFunctionNotFound(func_name.to_string()),
                            ),
                        ));
                    }
                }
            }
        }

        let offset = SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
        let handle = system_api.lock_substate(self.receiver, offset, LockFlags::MUTABLE)?;

        let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
        let access_rules = substate_ref_mut.access_rules();
        access_rules.access_rules.push(self.access_rules);

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for AccessRulesSetAccessRuleInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        mut self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();

        let resolved_receiver = deref_and_update(self.receiver, &mut call_frame_update, deref)?;
        match resolved_receiver.receiver {
            RENodeId::Component(..) | RENodeId::Package(..) | RENodeId::ResourceManager(..) => {}
            _ => {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidInvocation,
                ));
            }
        }
        self.receiver = resolved_receiver.receiver;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::AccessRules(AccessRulesMethod::SetAccessRule)),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AccessRulesSetAccessRuleInvocation {
    type Output = ();

    fn main<Y>(
        self,
        api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + EngineApi<RuntimeError>
            + SysInvokableNative<RuntimeError>,
    {
        // TODO: Should this invariant be inforced in a more static/structural way?
        if self
            .selector
            .eq(&AccessRuleSelector::Method(AccessRuleKey::Native(
                NativeFn::Method(NativeMethod::AccessRules(AccessRulesMethod::SetAccessRule)),
            )))
        {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::AccessRulesError(
                    AccessRulesError::CannotSetAccessRuleOnSetAccessRule,
                ),
            ));
        }

        let offset = SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
        let handle = api.lock_substate(self.receiver, offset, LockFlags::MUTABLE)?;

        let authorization = {
            let substate_ref = api.get_ref(handle)?;
            let access_rules_substate = substate_ref.access_rules();
            access_rules_substate.mutability_authorization(&self.selector)
        };

        // Manual Auth
        {
            let owned_node_ids = api.sys_get_visible_nodes()?;
            let node_id = owned_node_ids
                .into_iter()
                .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
                .expect("AuthZone does not exist");

            let offset = SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack);
            let handle = api.lock_substate(node_id, offset, LockFlags::read_only())?;
            let substate_ref = api.get_ref(handle)?;
            let auth_zone_substate = substate_ref.auth_zone();

            auth_zone_substate
                .check_auth(false, authorization)
                .map_err(|(authorization, error)| {
                    RuntimeError::ApplicationError(ApplicationError::AccessRulesError(
                        AccessRulesError::Unauthorized(authorization, error),
                    ))
                })?;
        }

        let mut substate_ref_mut = api.get_ref_mut(handle)?;
        let access_rules_substate = substate_ref_mut.access_rules();
        let access_rules_list = &mut access_rules_substate.access_rules;
        let index: usize = self.index.try_into().unwrap();
        let access_rules =
            access_rules_list
                .get_mut(index)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::AccessRulesError(AccessRulesError::InvalidIndex(self.index)),
                ))?;

        match self.selector {
            AccessRuleSelector::Method(key) => access_rules.set_method_access_rule(key, self.rule),
            AccessRuleSelector::Group(name) => access_rules.set_group_access_rule(name, self.rule),
            AccessRuleSelector::Default => access_rules.set_default_access_rule(self.rule),
        };

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for AccessRulesSetMutabilityInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        mut self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();

        let resolved_receiver = deref_and_update(self.receiver, &mut call_frame_update, deref)?;
        match resolved_receiver.receiver {
            RENodeId::Component(..) | RENodeId::Package(..) | RENodeId::ResourceManager(..) => {}
            _ => {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidInvocation,
                ));
            }
        }
        self.receiver = resolved_receiver.receiver;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::AccessRules(AccessRulesMethod::SetMutability)),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AccessRulesSetMutabilityInvocation {
    type Output = ();

    fn main<Y>(
        self,
        api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + EngineApi<RuntimeError>
            + SysInvokableNative<RuntimeError>,
    {
        // TODO: Should this invariant be inforced in a more static/structural way?
        if self
            .selector
            .eq(&AccessRuleSelector::Method(AccessRuleKey::Native(
                NativeFn::Method(NativeMethod::AccessRules(AccessRulesMethod::SetAccessRule)),
            )))
        {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::AccessRulesError(
                    AccessRulesError::CannotSetAccessRuleOnSetAccessRule,
                ),
            ));
        }

        let offset = SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
        let handle = api.lock_substate(self.receiver, offset, LockFlags::MUTABLE)?;

        let authorization = {
            let substate_ref = api.get_ref(handle)?;
            let access_rules_substate = substate_ref.access_rules();
            access_rules_substate.mutability_authorization(&self.selector)
        };

        // Manual Auth
        {
            let owned_node_ids = api.sys_get_visible_nodes()?;
            let node_id = owned_node_ids
                .into_iter()
                .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
                .expect("AuthZone does not exist");

            let offset = SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack);
            let handle = api.lock_substate(node_id, offset, LockFlags::read_only())?;
            let substate_ref = api.get_ref(handle)?;
            let auth_zone_substate = substate_ref.auth_zone();

            auth_zone_substate
                .check_auth(false, authorization)
                .map_err(|(authorization, error)| {
                    RuntimeError::ApplicationError(ApplicationError::AccessRulesError(
                        AccessRulesError::Unauthorized(authorization, error),
                    ))
                })?;
        }

        let mut substate_ref_mut = api.get_ref_mut(handle)?;
        let access_rules_substate = substate_ref_mut.access_rules();
        let access_rules_list = &mut access_rules_substate.access_rules;
        let index: usize = self.index.try_into().unwrap();
        let access_rules =
            access_rules_list
                .get_mut(index)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::AccessRulesError(AccessRulesError::InvalidIndex(self.index)),
                ))?;

        match self.selector {
            AccessRuleSelector::Method(key) => access_rules.set_mutability(key, self.mutability),
            AccessRuleSelector::Group(name) => {
                access_rules.set_group_mutability(name, self.mutability)
            }
            AccessRuleSelector::Default => access_rules.set_default_mutability(self.mutability),
        };

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for AccessRulesGetLengthInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        mut self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();

        let resolved_receiver = deref_and_update(self.receiver, &mut call_frame_update, deref)?;
        match resolved_receiver.receiver {
            RENodeId::Component(..) | RENodeId::Package(..) | RENodeId::ResourceManager(..) => {}
            _ => {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidInvocation,
                ));
            }
        }
        self.receiver = resolved_receiver.receiver;

        let actor = REActor::Method(
            ResolvedMethod::Native(NativeMethod::AccessRules(AccessRulesMethod::SetMutability)),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AccessRulesGetLengthInvocation {
    type Output = u32;

    fn main<Y>(
        self,
        api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + EngineApi<RuntimeError>
            + SysInvokableNative<RuntimeError>,
    {
        let offset = SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
        let handle = api.lock_substate(self.receiver, offset, LockFlags::MUTABLE)?;

        let substate_ref = api.get_ref(handle)?;
        let access_rules_substate = substate_ref.access_rules();

        Ok((
            access_rules_substate.access_rules.len() as u32,
            CallFrameUpdate::empty(),
        ))
    }
}
