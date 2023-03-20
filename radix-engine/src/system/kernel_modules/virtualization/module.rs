use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::account::ACCOUNT_BLUEPRINT;
use radix_engine_interface::blueprints::identity::IDENTITY_BLUEPRINT;

#[derive(Debug, Clone)]
pub struct VirtualizationModule;

impl KernelModule for VirtualizationModule {
    fn on_substate_lock_fault<Y: ClientApi<RuntimeError> + KernelModuleApi<RuntimeError>>(
        node_id: RENodeId,
        _module_id: NodeModuleId,
        _offset: &SubstateOffset,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match node_id {
            // TODO: Need to have a schema check in place before this in order to not create virtual components when accessing illegal substates
            RENodeId::GlobalObject(Address::Component(component_address)) => {
                // Lazy create component if missing
                let (package, blueprint, virtual_func_id, id) = match component_address {
                    ComponentAddress::EcdsaSecp256k1VirtualAccount(id) => (
                        ACCOUNT_PACKAGE,
                        ACCOUNT_BLUEPRINT,
                        0u8,
                        id,
                    ),
                    ComponentAddress::EddsaEd25519VirtualAccount(id) => (
                        ACCOUNT_PACKAGE,
                        ACCOUNT_BLUEPRINT,
                        1u8,
                        id,
                    ),
                    ComponentAddress::EcdsaSecp256k1VirtualIdentity(id) => (
                        IDENTITY_PACKAGE,
                        IDENTITY_BLUEPRINT,
                        0u8,
                        id,
                    ),
                    ComponentAddress::EddsaEd25519VirtualIdentity(id) => (
                        IDENTITY_PACKAGE,
                        IDENTITY_BLUEPRINT,
                        1u8,
                        id,
                    ),
                    _ => return Ok(false),
                };

                let rtn: Vec<u8> = api.kernel_invoke(VirtualLazyLoadInvocation {
                    package_address: package,
                    blueprint_name: blueprint.to_string(),
                    virtual_func_id: virtual_func_id,
                    args: id,
                })?.into();

                let (object_id, modules): (Own, BTreeMap<NodeModuleId, Own>) =
                    scrypto_decode(&rtn).unwrap();
                let modules = modules
                    .into_iter()
                    .map(|(id, own)| (id, own.id()))
                    .collect();
                api.kernel_allocate_virtual_node_id(node_id)?;
                api.globalize_with_address(
                    RENodeId::Object(object_id.id()),
                    modules,
                    node_id.into(),
                )?;

                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
