use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi, LockFlags};
use crate::{errors::RuntimeError, types::*};
use radix_engine_interface::api::node_modules::auth::ACCESS_RULES_BLUEPRINT;
use radix_engine_interface::api::node_modules::metadata::METADATA_BLUEPRINT;
use radix_engine_interface::api::node_modules::royalty::{
    COMPONENT_ROYALTY_BLUEPRINT, PACKAGE_ROYALTY_BLUEPRINT,
};
use radix_engine_interface::api::types::{ScryptoInvocation, ScryptoReceiver};

pub fn resolve_method<Y: KernelNodeApi + KernelSubstateApi>(
    receiver: ScryptoReceiver,
    module_id: NodeModuleId,
    method_name: &str,
    args: &[u8],
    api: &mut Y,
) -> Result<ScryptoInvocation, RuntimeError> {
    let node_id = receiver.into();

    let (package_address, blueprint_name) = match module_id {
        NodeModuleId::SELF => {
            let handle = api.kernel_lock_substate(
                node_id,
                NodeModuleId::ComponentTypeInfo,
                SubstateOffset::ComponentTypeInfo(ComponentTypeInfoOffset::TypeInfo),
                LockFlags::read_only(),
            )?;
            let substate_ref = api.kernel_get_substate_ref(handle)?;
            let component_info = substate_ref.component_info().clone(); // TODO: Remove clone()
            let object_info = (
                component_info.package_address,
                component_info.blueprint_name,
            );
            api.kernel_drop_lock(handle)?;

            object_info
        }
        NodeModuleId::Metadata => {
            // TODO: Check if type has metadata
            (METADATA_PACKAGE, METADATA_BLUEPRINT.to_string())
        }
        NodeModuleId::ComponentRoyalty => {
            // TODO: Check if type has royalty
            (ROYALTY_PACKAGE, COMPONENT_ROYALTY_BLUEPRINT.to_string())
        }
        NodeModuleId::PackageRoyalty => {
            // TODO: Check if type has royalty
            (ROYALTY_PACKAGE, PACKAGE_ROYALTY_BLUEPRINT.to_string())
        }
        NodeModuleId::AccessRules => {
            // TODO: Check if type has royalty
            (ACCESS_RULES_PACKAGE, ACCESS_RULES_BLUEPRINT.to_string())
        }
        _ => todo!(),
    };

    let invocation = ScryptoInvocation {
        package_address,
        blueprint_name,
        receiver: Some((receiver, module_id)),
        fn_name: method_name.to_string(),
        args: args.to_owned(),
    };

    Ok(invocation)
}
