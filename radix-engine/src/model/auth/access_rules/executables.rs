use crate::engine::{
    ApplicationError, CallFrameUpdate, InterpreterError, LockFlags, NativeExecutable,
    NativeInvocation, NativeInvocationInfo, RuntimeError, SystemApi,
};
use crate::types::*;
use radix_engine_interface::api::types::{
    AccessRulesMethod, GlobalAddress, NativeMethod, PackageOffset, RENodeId, SubstateOffset,
};
use radix_engine_interface::model::*;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum AccessRulesError {
    BlueprintFunctionNotFound(String),
    InvalidIndex(usize),
    MethodUsesDefaultAuth(String),
}

impl NativeExecutable for AccessRulesAddAccessCheckInvocation {
    type NativeOutput = ();

    fn execute<Y>(input: Self, system_api: &mut Y) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = input.receiver;

        // TODO: Move this into a more static check once node types implemented
        if !matches!(node_id, RENodeId::Component(..)) {
            return Err(RuntimeError::InterpreterError(
                InterpreterError::InvalidInvocation,
            ));
        }

        // Abi checks
        {
            let offset = SubstateOffset::Component(ComponentOffset::Info);
            let handle = system_api.lock_substate(node_id, offset, LockFlags::read_only())?;

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

            let package_offset = SubstateOffset::Package(PackageOffset::Package);
            let handle =
                system_api.lock_substate(package_id, package_offset, LockFlags::read_only())?;
            let substate_ref = system_api.get_ref(handle)?;
            let package = substate_ref.package();
            let blueprint_abi = package.blueprint_abi(&blueprint_name).unwrap_or_else(|| {
                panic!(
                    "Blueprint {} is not found in package node {:?}",
                    blueprint_name, package_id
                )
            });
            for (func_name, _) in input.access_rules.iter() {
                if !blueprint_abi.contains_fn(func_name.as_str()) {
                    return Err(RuntimeError::ApplicationError(
                        ApplicationError::AccessRulesError(
                            AccessRulesError::BlueprintFunctionNotFound(func_name.to_string()),
                        ),
                    ));
                }
            }
        }

        let offset = SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
        let access_rules = substate_ref_mut.access_rules();
        access_rules.access_rules.push(input.access_rules);

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for AccessRulesAddAccessCheckInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AccessRules(AccessRulesMethod::AddAccessCheck),
            self.receiver,
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for AccessRulesUpdateAuthInvocation {
    type NativeOutput = ();

    fn execute<'a, Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(Self::NativeOutput, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = input.receiver;
        let offset = SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
        let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
        let access_rules_substate = substate_ref_mut.access_rules();
        let access_rules = access_rules_substate
            .access_rules
            .get_mut(input.access_rules_index)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::AccessRulesError(AccessRulesError::InvalidIndex(
                    input.access_rules_index,
                )),
            ))?;

        match input.method {
            AccessRulesMethodIdent::Default => {
                let (_, current_default_mutability) = access_rules.get_default().clone();
                *access_rules = access_rules
                    .clone()
                    .default(input.access_rule, current_default_mutability.clone());
            }
            AccessRulesMethodIdent::Method(method_name) => {
                let current_access_rule_mutability = access_rules
                    .iter()
                    .filter(|(x, _)| **x == method_name)
                    .nth(0)
                    .ok_or(RuntimeError::ApplicationError(
                        ApplicationError::AccessRulesError(
                            AccessRulesError::MethodUsesDefaultAuth(method_name.clone()),
                        ),
                    ))?
                    .1
                     .1
                    .clone();
                *access_rules = access_rules.clone().method(
                    &method_name,
                    input.access_rule,
                    current_access_rule_mutability.clone(),
                );
            }
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for AccessRulesUpdateAuthInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AccessRules(AccessRulesMethod::UpdateAuth),
            self.receiver,
            CallFrameUpdate::empty(),
        )
    }
}

impl NativeExecutable for AccessRulesLockAuthInvocation {
    type NativeOutput = ();

    fn execute<'a, Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<(Self::NativeOutput, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi,
    {
        let node_id = input.receiver;
        let offset = SubstateOffset::AccessRules(AccessRulesOffset::AccessRules);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
        let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
        let access_rules_substate = substate_ref_mut.access_rules();
        let access_rules = access_rules_substate
            .access_rules
            .get_mut(input.access_rules_index)
            .ok_or(RuntimeError::ApplicationError(
                ApplicationError::AccessRulesError(AccessRulesError::InvalidIndex(
                    input.access_rules_index,
                )),
            ))?;

        match input.method {
            AccessRulesMethodIdent::Default => {
                let (current_access_rule, _) = access_rules.get_default().clone();
                *access_rules = access_rules.clone().default(current_access_rule, LOCKED);
            }
            AccessRulesMethodIdent::Method(method_name) => {
                let current_access_rule = access_rules
                    .iter()
                    .filter(|(x, _)| **x == method_name)
                    .nth(0)
                    .ok_or(RuntimeError::ApplicationError(
                        ApplicationError::AccessRulesError(
                            AccessRulesError::MethodUsesDefaultAuth(method_name.clone()),
                        ),
                    ))?
                    .1
                     .0
                    .clone();
                *access_rules =
                    access_rules
                        .clone()
                        .method(&method_name, current_access_rule, LOCKED);
            }
        }

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for AccessRulesLockAuthInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::AccessRules(AccessRulesMethod::LockAuth),
            self.receiver,
            CallFrameUpdate::empty(),
        )
    }
}
