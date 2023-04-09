use crate::errors::RuntimeError;
use crate::kernel::actor::Actor;
use crate::kernel::kernel_api::{KernelInvocation, KernelUpstreamApi};
use crate::system::system_downstream::SystemDownstream;
use crate::system::system_upstream::{SystemInvocation, SystemUpstream};
use crate::types::*;
use crate::vm::wasm::WasmEngine;
use radix_engine_interface::api::kernel_modules::virtualization::VirtualLazyLoadInput;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::account::{
    ACCOUNT_BLUEPRINT, ACCOUNT_CREATE_VIRTUAL_ECDSA_256K1_ID,
    ACCOUNT_CREATE_VIRTUAL_EDDSA_255519_ID,
};
use radix_engine_interface::blueprints::identity::{
    IDENTITY_BLUEPRINT, IDENTITY_CREATE_VIRTUAL_ECDSA_256K1_ID,
    IDENTITY_CREATE_VIRTUAL_EDDSA_25519_ID,
};

#[derive(Debug, Clone)]
pub struct VirtualizationModule;

impl VirtualizationModule {
    pub fn on_substate_lock_fault<
        'g,
        Y: KernelUpstreamApi<SystemUpstream<'g, W>>,
        W: WasmEngine + 'g,
    >(
        node_id: NodeId,
        _module_id: SysModuleId,
        _offset: &SubstateKey,
        api: &mut Y,
    ) -> Result<bool, RuntimeError> {
        match node_id.entity_type() {
            // TODO: Need to have a schema check in place before this in order to not create virtual components when accessing illegal substates
            Some(entity_type) => {
                // Lazy create component if missing
                let (blueprint, virtual_func_id) = match entity_type {
                    EntityType::GlobalVirtualEcdsaAccount => (
                        Blueprint::new(&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT),
                        ACCOUNT_CREATE_VIRTUAL_ECDSA_256K1_ID,
                    ),
                    EntityType::GlobalVirtualEddsaAccount => (
                        Blueprint::new(&ACCOUNT_PACKAGE, ACCOUNT_BLUEPRINT),
                        ACCOUNT_CREATE_VIRTUAL_EDDSA_255519_ID,
                    ),
                    EntityType::GlobalVirtualEcdsaIdentity => (
                        Blueprint::new(&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT),
                        IDENTITY_CREATE_VIRTUAL_ECDSA_256K1_ID,
                    ),
                    EntityType::GlobalVirtualEddsaIdentity => (
                        Blueprint::new(&IDENTITY_PACKAGE, IDENTITY_BLUEPRINT),
                        IDENTITY_CREATE_VIRTUAL_EDDSA_25519_ID,
                    ),
                    _ => return Ok(false),
                };

                let mut args = [0u8; 26];
                args.copy_from_slice(&node_id.as_ref()[1..]);

                let invocation = KernelInvocation {
                    resolved_actor: Actor::virtual_lazy_load(blueprint.clone(), virtual_func_id),
                    args: IndexedScryptoValue::from_typed(&VirtualLazyLoadInput { id: args }),
                    sys_invocation: SystemInvocation {
                        blueprint: blueprint,
                        ident: FnIdent::System(virtual_func_id),
                        receiver: None,
                    },
                    payload_size: 0,
                };

                let rtn: Vec<u8> = api.kernel_invoke_downstream(Box::new(invocation))?.into();

                let (own, modules): (Own, BTreeMap<SysModuleId, Own>) =
                    scrypto_decode(&rtn).unwrap();
                let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();
                api.kernel_allocate_virtual_node_id(node_id)?;

                let mut system = SystemDownstream::new(api);
                system.globalize_with_address(
                    own.0,
                    modules,
                    GlobalAddress::new_unchecked(node_id.into()),
                )?;

                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
