use crate::{
    errors::RuntimeError,
    kernel::kernel_api::LockFlags,
    kernel::{kernel_api::KernelSubstateApi, KernelNodeApi},
    types::*,
};
use radix_engine_interface::api::types::{ScryptoInvocation, ScryptoReceiver};

pub fn resolve_method<Y: KernelNodeApi + KernelSubstateApi>(
    receiver: ScryptoReceiver,
    method_name: &str,
    args: &[u8],
    api: &mut Y,
) -> Result<ScryptoInvocation, RuntimeError> {
    let node_id = match receiver {
        ScryptoReceiver::Global(component_address) => {
            RENodeId::Global(GlobalAddress::Component(component_address))
        }
        ScryptoReceiver::Resource(resource_address) => {
            RENodeId::Global(GlobalAddress::Resource(resource_address))
        }
        ScryptoReceiver::Component(component_id) => {
            // TODO: Fix this as this is wrong id for native components
            // TODO: Will be easier to fix this when local handles are implemented
            RENodeId::Component(component_id)
        }
        ScryptoReceiver::Vault(vault_id) => RENodeId::Vault(vault_id),
        ScryptoReceiver::Proof(proof_id) => RENodeId::Proof(proof_id),
    };

    let component_info = {
        let handle = api.lock_substate(
            node_id,
            NodeModuleId::ComponentTypeInfo,
            SubstateOffset::ComponentTypeInfo(ComponentTypeInfoOffset::TypeInfo),
            LockFlags::read_only(),
        )?;
        let substate_ref = api.get_ref(handle)?;
        let component_info = substate_ref.component_info().clone(); // TODO: Remove clone()
        api.drop_lock(handle)?;

        component_info
    };

    let invocation = ScryptoInvocation {
        package_address: component_info.package_address,
        blueprint_name: component_info.blueprint_name,
        receiver: Some(receiver),
        fn_name: method_name.to_string(),
        args: args.to_owned(),
    };

    Ok(invocation)
}
