use crate::engine::{
    deref_and_update, ApplicationError, CallFrameUpdate, ExecutableInvocation, InterpreterError,
    LockFlags, NativeExecutor, NativeProcedure, ResolvedActor, ResolvedMethod, ResolverApi, RuntimeError,
    SystemApi,
};
use crate::model::{MethodAuthorization, MethodAuthorizationError};
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::api::{EngineApi, Invocation, InvokableModel};
use radix_engine_interface::api::types::{
    AccessRulesChainMethod, GlobalAddress, NativeMethod, PackageOffset, RENodeId, SubstateOffset,
};
use radix_engine_interface::model::*;

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum AccessRulesChainError {
    BlueprintFunctionNotFound(String),
    InvalidIndex(u32),
    Unauthorized(MethodAuthorization, MethodAuthorizationError),
    ProtectedMethod(AccessRuleKey),
}

impl<W: WasmEngine> ExecutableInvocation<W> for AccessRulesAddAccessCheckInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolverApi<W>>(
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

        let actor = ResolvedActor::Method(
            ResolvedMethod::Native(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::AddAccessCheck,
            )),
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
        Y: SystemApi + EngineApi<RuntimeError>,
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
        let handle = system_api.lock_substate(self.receiver, offset, LockFlags::MUTABLE)?;

        let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
        let substate = substate_ref_mut.access_rules_chain();
        substate.access_rules_chain.push(self.access_rules);

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl<W: WasmEngine> ExecutableInvocation<W> for AccessRulesSetMethodAccessRuleInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolverApi<W>>(
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

        let actor = ResolvedActor::Method(
            ResolvedMethod::Native(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::SetMethodAccessRule,
            )),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AccessRulesSetMethodAccessRuleInvocation {
    type Output = ();

    fn main<Y>(
        self,
        api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // TODO: Should this invariant be enforced in a more static/structural way?
        if [
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::GetLength,
            ))),
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::SetGroupAccessRule,
            ))),
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::SetGroupMutability,
            ))),
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::SetMethodAccessRule,
            ))),
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::SetMethodMutability,
            ))),
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

impl<W: WasmEngine> ExecutableInvocation<W> for AccessRulesSetGroupAccessRuleInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolverApi<W>>(
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

        let actor = ResolvedActor::Method(
            ResolvedMethod::Native(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::SetGroupAccessRule,
            )),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AccessRulesSetGroupAccessRuleInvocation {
    type Output = ();

    fn main<Y>(
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

impl<W: WasmEngine> ExecutableInvocation<W> for AccessRulesSetMethodMutabilityInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolverApi<W>>(
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

        let actor = ResolvedActor::Method(
            ResolvedMethod::Native(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::SetMethodMutability,
            )),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AccessRulesSetMethodMutabilityInvocation {
    type Output = ();

    fn main<Y>(
        self,
        api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + EngineApi<RuntimeError> + InvokableModel<RuntimeError>,
    {
        // TODO: Should this invariant be enforced in a more static/structural way?
        if [
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::GetLength,
            ))),
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::SetGroupAccessRule,
            ))),
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::SetGroupMutability,
            ))),
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::SetMethodAccessRule,
            ))),
            AccessRuleKey::Native(NativeFn::Method(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::SetMethodMutability,
            ))),
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

impl<W: WasmEngine> ExecutableInvocation<W> for AccessRulesSetGroupMutabilityInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolverApi<W>>(
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

        let actor = ResolvedActor::Method(
            ResolvedMethod::Native(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::SetGroupMutability,
            )),
            resolved_receiver,
        );

        let executor = NativeExecutor(self);
        Ok((actor, call_frame_update, executor))
    }
}

impl NativeProcedure for AccessRulesSetGroupMutabilityInvocation {
    type Output = ();

    fn main<Y>(
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

impl<W: WasmEngine> ExecutableInvocation<W> for AccessRulesGetLengthInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: ResolverApi<W>>(
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

        let actor = ResolvedActor::Method(
            ResolvedMethod::Native(NativeMethod::AccessRulesChain(
                AccessRulesChainMethod::GetLength,
            )),
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
