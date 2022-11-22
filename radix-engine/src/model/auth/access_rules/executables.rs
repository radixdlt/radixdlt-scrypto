use crate::engine::{
    ApplicationError, CallFrameUpdate, InterpreterError, Invokable, LockFlags,
    NativeInvocationMethod, RuntimeError, SystemApi,
};
use crate::model::ResourceManagerSetResourceAddressInvocation;
use crate::types::*;
use radix_engine_interface::api::api::{EngineApi, Invocation, SysInvokableNative};
use radix_engine_interface::api::types::{
    AccessRulesMethod, GlobalAddress, NativeMethod, PackageOffset, RENodeId, SubstateOffset,
};
use radix_engine_interface::model::*;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum AccessRulesError {
    BlueprintFunctionNotFound(String),
}

impl NativeInvocationMethod for AccessRulesAddAccessCheckInvocation {
    type Args = AccessRules;

    fn resolve(self) -> (RENodeId, Self::Args, NativeMethod, CallFrameUpdate) {
        (
            self.receiver,
            self.access_rules,
            NativeMethod::AccessRules(AccessRulesMethod::AddAccessCheck),
            CallFrameUpdate::empty(),
        )
    }

    fn execute<Y>(
        receiver: RENodeId,
        args: Self::Args,
        system_api: &mut Y,
    ) -> Result<(<Self as Invocation>::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + EngineApi<RuntimeError>
            + SysInvokableNative<RuntimeError>
            + Invokable<ResourceManagerSetResourceAddressInvocation>,
    {
        // TODO: Move this into a more static check once node types implemented
        if !matches!(receiver, RENodeId::Component(..)) {
            return Err(RuntimeError::InterpreterError(
                InterpreterError::InvalidInvocation,
            ));
        }

        // Abi checks
        {
            let offset = SubstateOffset::Component(ComponentOffset::Info);
            let handle = system_api.lock_substate(receiver, offset, LockFlags::read_only())?;

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
            for (func_name, _) in args.iter() {
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
        let handle = system_api.lock_substate(receiver, offset, LockFlags::MUTABLE)?;

        let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
        let access_rules = substate_ref_mut.access_rules();
        access_rules.access_rules.push(args);

        Ok(((), CallFrameUpdate::empty()))
    }
}
