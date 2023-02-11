use crate::{
    blueprints::transaction_processor::TransactionProcessorError,
    errors::{ApplicationError, RuntimeError},
    kernel::kernel_api::LockFlags,
    kernel::{kernel_api::KernelSubstateApi, KernelNodeApi},
    types::*,
};
use radix_engine_interface::api::{
    types::CallTableInvocation,
    types::{ScryptoInvocation, ScryptoReceiver},
};
use radix_engine_interface::blueprints::account::*;

pub fn resolve_method<Y: KernelNodeApi + KernelSubstateApi>(
    receiver: ScryptoReceiver,
    method_name: &str,
    args: &[u8],
    api: &mut Y,
) -> Result<CallTableInvocation, RuntimeError> {
    let invocation = match receiver {
        ScryptoReceiver::Global(component_address) => match component_address {
            ComponentAddress::Identity(..)
            | ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
            | ComponentAddress::EddsaEd25519VirtualIdentity(..) => {
                return Err(RuntimeError::ApplicationError(
                    ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::ResolveError(ResolveError::NotAMethod),
                    ),
                ));
            }
            ComponentAddress::EpochManager(..) | ComponentAddress::Validator(..) => {
                let invocation = EpochManagerPackage::resolve_method_invocation(
                    component_address,
                    method_name,
                    args,
                )
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::ResolveError(e),
                    ))
                })?;
                CallTableInvocation::Native(invocation)
            }
            ComponentAddress::Clock(..) => {
                let invocation =
                    ClockPackage::resolve_method_invocation(component_address, method_name, args)
                        .map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                            TransactionProcessorError::ResolveError(e),
                        ))
                    })?;
                CallTableInvocation::Native(NativeInvocation::Clock(invocation))
            }
            ComponentAddress::AccessController(..) => {
                let invocation = AccessControllerPackage::resolve_method_invocation(
                    component_address,
                    method_name,
                    args,
                )
                .map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                        TransactionProcessorError::ResolveError(e),
                    ))
                })?;
                CallTableInvocation::Native(NativeInvocation::AccessController(invocation))
            }
            ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
            | ComponentAddress::EddsaEd25519VirtualAccount(..)
            | ComponentAddress::Account(..) => {
                /*
                let component_node_id =
                    RENodeId::Global(GlobalAddress::Component(component_address));
                let component_info = {
                    let handle = api.lock_substate(
                        component_node_id,
                        NodeModuleId::SELF,
                        SubstateOffset::Component(ComponentOffset::Info),
                        LockFlags::read_only(),
                    )?;
                    let substate_ref = api.get_ref(handle)?;
                    let component_info = substate_ref.component_info().clone(); // TODO: Remove clone()
                    api.drop_lock(handle)?;

                    component_info
                };
                 */

                let method_invocation = ScryptoInvocation {
                    package_address: ACCOUNT_PACKAGE,
                    blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
                    receiver: Some(ScryptoReceiver::Global(component_address.clone())),
                    fn_name: method_name.to_string(),
                    args: args.to_owned(),
                };
                CallTableInvocation::Scrypto(method_invocation)
            }
            ComponentAddress::Normal(..) => {
                let component_node_id =
                    RENodeId::Global(GlobalAddress::Component(component_address));
                let component_info = {
                    let handle = api.lock_substate(
                        component_node_id,
                        NodeModuleId::ComponentTypeInfo,
                        SubstateOffset::ComponentTypeInfo(ComponentTypeInfoOffset::TypeInfo),
                        LockFlags::read_only(),
                    )?;
                    let substate_ref = api.get_ref(handle)?;
                    let component_info = substate_ref.component_info().clone(); // TODO: Remove clone()
                    api.drop_lock(handle)?;

                    component_info
                };

                let method_invocation = ScryptoInvocation {
                    package_address: component_info.package_address,
                    blueprint_name: component_info.blueprint_name,
                    receiver: Some(ScryptoReceiver::Global(component_address.clone())),
                    fn_name: method_name.to_string(),
                    args: args.to_owned(),
                };
                CallTableInvocation::Scrypto(method_invocation)
            }
        },
        ScryptoReceiver::Component(component_id) => {
            let component_node_id = RENodeId::Component(component_id);
            let component_info = {
                let handle = api.lock_substate(
                    component_node_id,
                    NodeModuleId::ComponentTypeInfo,
                    SubstateOffset::ComponentTypeInfo(ComponentTypeInfoOffset::TypeInfo),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.get_ref(handle)?;
                let component_info = substate_ref.component_info().clone(); // TODO: Remove clone()
                api.drop_lock(handle)?;

                component_info
            };

            CallTableInvocation::Scrypto(ScryptoInvocation {
                package_address: component_info.package_address,
                blueprint_name: component_info.blueprint_name,
                receiver: Some(ScryptoReceiver::Component(component_id)),
                fn_name: method_name.to_string(),
                args: args.to_owned(),
            })
        }
    };

    Ok(invocation)
}
