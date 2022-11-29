use crate::engine::{
    deref_and_update, ApplicationError, CallFrameUpdate, ExecutableInvocation, InterpreterError,
    Invokable, LockFlags, MethodDeref, NativeExecutor, NativeProcedure, REActor, ResolvedMethod,
    RuntimeError, SystemApi,
};
use crate::types::*;
use radix_engine_interface::api::api::{EngineApi, Invocation};
use radix_engine_interface::api::types::{
    AccessRulesMethod, GlobalAddress, NativeMethod, PackageOffset, RENodeId, SubstateOffset,
};
use radix_engine_interface::data::IndexedScryptoValue;
use radix_engine_interface::model::*;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum AccessRulesError {
    BlueprintFunctionNotFound(String),
}

impl ExecutableInvocation for AccessRulesAddAccessCheckInvocation {
    type Exec = NativeExecutor<Self>;

    fn resolve<D: MethodDeref>(
        mut self,
        deref: &mut D,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let input = IndexedScryptoValue::from_typed(&self);
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

        let executor = NativeExecutor(self, input);
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
            for (func_name, _) in self.access_rules.iter() {
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
        let handle = system_api.lock_substate(self.receiver, offset, LockFlags::MUTABLE)?;

        let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
        let access_rules = substate_ref_mut.access_rules();
        access_rules.access_rules.push(self.access_rules);

        Ok(((), CallFrameUpdate::empty()))
    }
}
