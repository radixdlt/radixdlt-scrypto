use crate::errors::RuntimeError;
use crate::kernel::actor::Actor;
use crate::kernel::kernel_api::{KernelApi, KernelInvocation};
use crate::system::system::SystemService;
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::types::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::system_modules::virtualization::VirtualLazyLoadInput;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::account::{
    ACCOUNT_BLUEPRINT, ACCOUNT_CREATE_VIRTUAL_ED25519_ID, ACCOUNT_CREATE_VIRTUAL_SECP256K1_ID,
};
use radix_engine_interface::blueprints::identity::{
    IDENTITY_BLUEPRINT, IDENTITY_CREATE_VIRTUAL_ED25519_ID, IDENTITY_CREATE_VIRTUAL_SECP256K1_ID,
};

#[derive(Debug, Clone)]
pub struct VirtualizationModule;

impl VirtualizationModule {
    pub fn on_substate_lock_fault<'g, Y: KernelApi<SystemConfig<C>>, C: SystemCallbackObject>(
        node_id: NodeId,
        _partition_num: PartitionNumber,
        _offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match node_id.entity_type() {
            // FIXME: Need to have a schema check in place before this in order to not create virtual components when accessing illegal substates
            Some(entity_type) => {
                // Lazy create component if missing
                let (blueprint, virtual_func_id) = match entity_type {
                    EntityType::GlobalVirtualSecp256k1Account => (
                        BlueprintId::new(&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT),
                        ACCOUNT_CREATE_VIRTUAL_SECP256K1_ID,
                    ),
                    EntityType::GlobalVirtualEd25519Account => (
                        BlueprintId::new(&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT),
                        ACCOUNT_CREATE_VIRTUAL_ED25519_ID,
                    ),
                    EntityType::GlobalVirtualSecp256k1Identity => (
                        BlueprintId::new(&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT),
                        IDENTITY_CREATE_VIRTUAL_SECP256K1_ID,
                    ),
                    EntityType::GlobalVirtualEd25519Identity => (
                        BlueprintId::new(&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT),
                        IDENTITY_CREATE_VIRTUAL_ED25519_ID,
                    ),
                    _ => return Ok(false),
                };

                let mut args = [0u8; NodeId::UUID_LENGTH];
                args.copy_from_slice(&node_id.as_ref()[1..]);

                let invocation = KernelInvocation {
                    actor: Actor::VirtualLazyLoad {
                        blueprint_id: blueprint.clone(),
                        ident: virtual_func_id,
                    },
                    args: IndexedScryptoValue::from_typed(&VirtualLazyLoadInput { id: args }),
                };

                let rtn: Vec<u8> = api.kernel_invoke(Box::new(invocation))?.into();

                let modules: BTreeMap<ObjectModuleId, Own> = scrypto_decode(&rtn).unwrap();
                let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();
                let address = GlobalAddress::new_or_panic(node_id.into());

                let mut system = SystemService::new(api);
                let address_reservation =
                    system.allocate_virtual_global_address(blueprint, address)?;
                system.globalize_with_address(modules, address_reservation)?;

                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
