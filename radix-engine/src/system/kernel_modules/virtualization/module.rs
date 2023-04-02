use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::types::*;
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

impl KernelModule for VirtualizationModule {
    fn on_substate_lock_fault<Y: KernelModuleApi<RuntimeError>>(
        node_id: NodeId,
        _module_id: TypedModuleId,
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

                let rtn: Vec<u8> = api
                    .kernel_invoke(Box::new(VirtualLazyLoadInvocation {
                        blueprint,
                        virtual_func_id,
                        args,
                    }))?
                    .into();

                let (own, modules): (Own, BTreeMap<TypedModuleId, Own>) =
                    scrypto_decode(&rtn).unwrap();
                let modules = modules.into_iter().map(|(id, own)| (id, own.0)).collect();
                api.kernel_allocate_virtual_node_id(node_id)?;
                api.globalize_with_address(
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
