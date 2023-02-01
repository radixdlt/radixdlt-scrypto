use crate::engine::{
    deref_and_update, ApplicationError, CallFrameUpdate, ExecutableInvocation, Executor,
    InterpreterError, LockFlags, ResolvedActor, ResolverApi, RuntimeError, SystemApi,
};
use crate::model::{MethodAuthorization, MethodAuthorizationError};
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::types::{
    AccessRulesChainFn, GlobalAddress, NativeFn, PackageOffset, RENodeId, SubstateOffset,
};
use radix_engine_interface::api::{EngineApi, Invocation, InvokableModel};
use radix_engine_interface::model::*;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum AccessRulesChainError {
    BlueprintFunctionNotFound(String),
    InvalidIndex(u32),
    Unauthorized(MethodAuthorization, MethodAuthorizationError),
    ProtectedMethod(AccessRuleKey),
}

impl ExecutableInvocation for AccessRulesAddAccessCheckInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        mut self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();

        let resolved_receiver = deref_and_update(self.receiver, &mut call_frame_update, deref)?;
        // TODO: Move this into a more static check once node types implemented
        if !matches!(resolved_receiver.receiver, RENodeId::Component(..)) {
            return Err(RuntimeError::InterpreterError(
                InterpreterError::InvalidInvocation,
            ));
        }
        self.receiver = resolved_receiver.receiver;

        let actor = ResolvedActor::method(
            NativeFn::AccessRulesChain(AccessRulesChainFn::AddAccessCheck),
            resolved_receiver,
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessRulesAddAccessCheckInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // Abi checks
        {
            let offset = SubstateOffset::Component(ComponentOffset::Info);
            let handle = api.lock_substate(self.receiver, offset, LockFlags::read_only())?;

            let (package_id, blueprint_name) = {
                let substate_ref = api.get_ref(handle)?;
                let component_info = substate_ref.component_info();
                let package_address = component_info.package_address;
                let blueprint_name = component_info.blueprint_name.to_owned();
                (
                    RENodeId::Global(GlobalAddress::Package(package_address)),
                    blueprint_name,
                )
            };

            let package_offset = SubstateOffset::Package(PackageOffset::Info);
            let handle = api.lock_substate(package_id, package_offset, LockFlags::read_only())?;
            let substate_ref = api.get_ref(handle)?;
            let package = substate_ref.package_info();
            let blueprint_abi = package.blueprint_abi(&blueprint_name).unwrap_or_else(|| {
                panic!(
                    "Blueprint {} is not found in package node {:?}",
                    blueprint_name, package_id
                )
            });
            for (key, _) in self.access_rules.get_all_method_auth() {
                if let AccessRuleKey::ScryptoMethod(func_name) = key {
                    if !blueprint_abi.contains_fn(func_name.as_str()) {
                        return Err(RuntimeError::ApplicationError(
                            ApplicationError::AccessRulesChainError(
                                AccessRulesChainError::BlueprintFunctionNotFound(
                                    func_name.to_string(),
                                ),
                            ),
                        ));
                    }
                }
            }
        }

        let offset = SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain);
        let handle = api.lock_substate(self.receiver, offset, LockFlags::MUTABLE)?;

        let mut substate_ref_mut = api.get_ref_mut(handle)?;
        let substate = substate_ref_mut.access_rules_chain();
        substate.access_rules_chain.push(self.access_rules);

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for AccessRulesSetMethodAccessRuleInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        mut self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();

        let resolved_receiver = deref_and_update(self.receiver, &mut call_frame_update, deref)?;
        match resolved_receiver.receiver {
            RENodeId::Component(..)
            | RENodeId::Package(..)
            | RENodeId::ResourceManager(..)
            | RENodeId::Validator(..)
            | RENodeId::AccessController(..) => {}
            _ => {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidInvocation,
                ));
            }
        }
        self.receiver = resolved_receiver.receiver;

        let actor = ResolvedActor::method(
            NativeFn::AccessRulesChain(AccessRulesChainFn::SetMethodAccessRule),
            resolved_receiver,
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessRulesSetMethodAccessRuleInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // TODO: Should this invariant be enforced in a more static/structural way?
        if [
            AccessRuleKey::Native(NativeFn::AccessRulesChain(AccessRulesChainFn::GetLength)),
            AccessRuleKey::Native(NativeFn::AccessRulesChain(
                AccessRulesChainFn::SetGroupAccessRule,
            )),
            AccessRuleKey::Native(NativeFn::AccessRulesChain(
                AccessRulesChainFn::SetGroupMutability,
            )),
            AccessRuleKey::Native(NativeFn::AccessRulesChain(
                AccessRulesChainFn::SetMethodAccessRule,
            )),
            AccessRuleKey::Native(NativeFn::AccessRulesChain(
                AccessRulesChainFn::SetMethodMutability,
            )),
        ]
        .iter()
        .any(|x| self.key == *x)
        {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::AccessRulesChainError(AccessRulesChainError::ProtectedMethod(
                    self.key,
                )),
            ));
        }

        let offset = SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain);
        let handle = api.lock_substate(self.receiver, offset, LockFlags::MUTABLE)?;

        let authorization = {
            let substate_ref = api.get_ref(handle)?;
            let access_rules_substate = substate_ref.access_rules_chain();
            access_rules_substate.method_mutability_authorization(&self.key)
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
            let auth_zone_stack = substate_ref.auth_zone_stack();

            auth_zone_stack.check_auth(false, authorization).map_err(
                |(authorization, error)| {
                    RuntimeError::ApplicationError(ApplicationError::AccessRulesChainError(
                        AccessRulesChainError::Unauthorized(authorization, error),
                    ))
                },
            )?;
        }

        let mut substate_ref_mut = api.get_ref_mut(handle)?;
        let substate = substate_ref_mut.access_rules_chain();
        let access_rules_chain = &mut substate.access_rules_chain;
        let index: usize = self.index.try_into().unwrap();
        let access_rules =
            access_rules_chain
                .get_mut(index)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::AccessRulesChainError(AccessRulesChainError::InvalidIndex(
                        self.index,
                    )),
                ))?;

        access_rules.set_method_access_rule(self.key, self.rule);

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for AccessRulesSetGroupAccessRuleInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        mut self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut call_frame_update = CallFrameUpdate::empty();

        let resolved_receiver = deref_and_update(self.receiver, &mut call_frame_update, deref)?;
        match resolved_receiver.receiver {
            RENodeId::Component(..)
            | RENodeId::Package(..)
            | RENodeId::ResourceManager(..)
            | RENodeId::AccessController(..) => {}
            _ => {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidInvocation,
                ));
            }
        }
        self.receiver = resolved_receiver.receiver;

        let actor = ResolvedActor::method(
            NativeFn::AccessRulesChain(AccessRulesChainFn::SetGroupAccessRule),
            resolved_receiver,
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessRulesSetGroupAccessRuleInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let offset = SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain);
        let handle = api.lock_substate(self.receiver, offset, LockFlags::MUTABLE)?;

        let authorization = {
            let substate_ref = api.get_ref(handle)?;
            let access_rules_substate = substate_ref.access_rules_chain();
            access_rules_substate.group_mutability_authorization(&self.name)
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
            let auth_zone_substate = substate_ref.auth_zone_stack();

            auth_zone_substate
                .check_auth(false, authorization)
                .map_err(|(authorization, error)| {
                    RuntimeError::ApplicationError(ApplicationError::AccessRulesChainError(
                        AccessRulesChainError::Unauthorized(authorization, error),
                    ))
                })?;
        }

        let mut substate_ref_mut = api.get_ref_mut(handle)?;
        let access_rules_substate = substate_ref_mut.access_rules_chain();
        let access_rules_list = &mut access_rules_substate.access_rules_chain;
        let index: usize = self.index.try_into().unwrap();
        let access_rules =
            access_rules_list
                .get_mut(index)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::AccessRulesChainError(AccessRulesChainError::InvalidIndex(
                        self.index,
                    )),
                ))?;

        access_rules.set_group_access_rule(self.name, self.rule);

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for AccessRulesSetMethodMutabilityInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        mut self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
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

        let actor = ResolvedActor::method(
            NativeFn::AccessRulesChain(AccessRulesChainFn::SetMethodMutability),
            resolved_receiver,
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessRulesSetMethodMutabilityInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // TODO: Should this invariant be enforced in a more static/structural way?
        if [
            AccessRuleKey::Native(NativeFn::AccessRulesChain(AccessRulesChainFn::GetLength)),
            AccessRuleKey::Native(NativeFn::AccessRulesChain(
                AccessRulesChainFn::SetGroupAccessRule,
            )),
            AccessRuleKey::Native(NativeFn::AccessRulesChain(
                AccessRulesChainFn::SetGroupMutability,
            )),
            AccessRuleKey::Native(NativeFn::AccessRulesChain(
                AccessRulesChainFn::SetMethodAccessRule,
            )),
            AccessRuleKey::Native(NativeFn::AccessRulesChain(
                AccessRulesChainFn::SetMethodMutability,
            )),
        ]
        .iter()
        .any(|x| self.key == *x)
        {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::AccessRulesChainError(AccessRulesChainError::ProtectedMethod(
                    self.key,
                )),
            ));
        }

        let offset = SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain);
        let handle = api.lock_substate(self.receiver, offset, LockFlags::MUTABLE)?;

        let authorization = {
            let substate_ref = api.get_ref(handle)?;
            let access_rules_substate = substate_ref.access_rules_chain();
            access_rules_substate.method_mutability_authorization(&self.key)
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
            let auth_zone_stack = substate_ref.auth_zone_stack();

            auth_zone_stack.check_auth(false, authorization).map_err(
                |(authorization, error)| {
                    RuntimeError::ApplicationError(ApplicationError::AccessRulesChainError(
                        AccessRulesChainError::Unauthorized(authorization, error),
                    ))
                },
            )?;
        }

        let mut substate_ref_mut = api.get_ref_mut(handle)?;
        let substate = substate_ref_mut.access_rules_chain();
        let access_rules_chain = &mut substate.access_rules_chain;
        let index: usize = self.index.try_into().unwrap();
        let access_rules =
            access_rules_chain
                .get_mut(index)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::AccessRulesChainError(AccessRulesChainError::InvalidIndex(
                        self.index,
                    )),
                ))?;

        access_rules.set_mutability(self.key, self.mutability);

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for AccessRulesSetGroupMutabilityInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        mut self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
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

        let actor = ResolvedActor::method(
            NativeFn::AccessRulesChain(AccessRulesChainFn::SetGroupMutability),
            resolved_receiver,
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessRulesSetGroupMutabilityInvocation {
    type Output = ();

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let offset = SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain);
        let handle = api.lock_substate(self.receiver, offset, LockFlags::MUTABLE)?;

        let authorization = {
            let substate_ref = api.get_ref(handle)?;
            let access_rules_substate = substate_ref.access_rules_chain();
            access_rules_substate.group_mutability_authorization(&self.name)
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
            let auth_zone_substate = substate_ref.auth_zone_stack();

            auth_zone_substate
                .check_auth(false, authorization)
                .map_err(|(authorization, error)| {
                    RuntimeError::ApplicationError(ApplicationError::AccessRulesChainError(
                        AccessRulesChainError::Unauthorized(authorization, error),
                    ))
                })?;
        }

        let mut substate_ref_mut = api.get_ref_mut(handle)?;
        let access_rules_substate = substate_ref_mut.access_rules_chain();
        let access_rules_list = &mut access_rules_substate.access_rules_chain;
        let index: usize = self.index.try_into().unwrap();
        let access_rules =
            access_rules_list
                .get_mut(index)
                .ok_or(RuntimeError::ApplicationError(
                    ApplicationError::AccessRulesChainError(AccessRulesChainError::InvalidIndex(
                        self.index,
                    )),
                ))?;

        access_rules.set_group_mutability(self.name, self.mutability);

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl ExecutableInvocation for AccessRulesGetLengthInvocation {
    type Exec = Self;

    fn resolve<D: ResolverApi>(
        mut self,
        deref: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
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

        let actor = ResolvedActor::method(
            NativeFn::AccessRulesChain(AccessRulesChainFn::GetLength),
            resolved_receiver,
        );

        Ok((actor, call_frame_update, self))
    }
}

impl Executor for AccessRulesGetLengthInvocation {
    type Output = u32;

    fn execute<Y, W: WasmEngine>(
        self,
        api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        let offset = SubstateOffset::AccessRulesChain(AccessRulesChainOffset::AccessRulesChain);
        let handle = api.lock_substate(self.receiver, offset, LockFlags::MUTABLE)?;

        let substate_ref = api.get_ref(handle)?;
        let access_rules_substate = substate_ref.access_rules_chain();

        Ok((
            access_rules_substate.access_rules_chain.len() as u32,
            CallFrameUpdate::empty(),
        ))
    }
}
