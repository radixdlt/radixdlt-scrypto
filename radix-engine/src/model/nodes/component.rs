use crate::engine::{
    ApplicationError, CallFrameUpdate, InvokableNative, LockFlags, NativeExecutable,
    NativeInvocation, NativeInvocationInfo, RuntimeError, SystemApi,
};
use crate::types::*;
use radix_engine_lib::component::ComponentAddAccessCheckInvocation;
use radix_engine_lib::engine::types::{
    ComponentMethod, ComponentOffset, GlobalAddress, NativeMethod, PackageOffset, RENodeId,
    SubstateOffset,
};

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum ComponentError {
    InvalidRequestData(DecodeError),
    BlueprintFunctionNotFound(String),
}

impl NativeExecutable for ComponentAddAccessCheckInvocation {
    type NativeOutput = ();

    fn execute<'a, Y>(
        input: Self,
        system_api: &mut Y,
    ) -> Result<((), CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi + InvokableNative<'a>,
    {
        let node_id = RENodeId::Component(input.receiver);
        let offset = SubstateOffset::Component(ComponentOffset::Info);
        let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;

        // Abi checks
        {
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
                        ApplicationError::ComponentError(
                            ComponentError::BlueprintFunctionNotFound(func_name.to_string()),
                        ),
                    ));
                }
            }
        }

        let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
        substate_ref_mut
            .component_info()
            .access_rules
            .push(input.access_rules);

        Ok(((), CallFrameUpdate::empty()))
    }
}

impl NativeInvocation for ComponentAddAccessCheckInvocation {
    fn info(&self) -> NativeInvocationInfo {
        NativeInvocationInfo::Method(
            NativeMethod::Component(ComponentMethod::AddAccessCheck),
            RENodeId::Component(self.receiver),
            CallFrameUpdate::empty(),
        )
    }
}
