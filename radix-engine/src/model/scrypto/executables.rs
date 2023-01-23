use radix_engine_interface::data::*;

use crate::engine::*;
use crate::model::TransactionProcessorError;
use crate::types::*;

impl ExecutableInvocation for ScryptoInvocation {
    type Exec = ScryptoExecutor;

    fn resolve<D: ResolverApi + SystemApi>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut node_refs_to_copy = HashSet::new();
        let args = IndexedScryptoValue::from_slice(&self.args).map_err(|e| {
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::InvalidCallData(e),
            ))
        })?;

        let nodes_to_move = args
            .owned_node_ids()
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                    TransactionProcessorError::ReadOwnedNodesError(e),
                ))
            })?
            .into_iter()
            .collect();
        for global_address in args.global_references() {
            node_refs_to_copy.insert(RENodeId::Global(global_address));
        }

        let scrypto_fn_ident = ScryptoFnIdentifier::new(
            self.package_address,
            self.blueprint_name.clone(),
            self.fn_name.clone(),
        );

        let (receiver, actor) = if let Some(receiver) = self.receiver {
            let original_node_id = match receiver {
                ScryptoReceiver::Global(component_address) => match component_address {
                    ComponentAddress::Normal(..)
                    | ComponentAddress::Account(..)
                    | ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
                    | ComponentAddress::EddsaEd25519VirtualAccount(..)
                    | ComponentAddress::AccessController(..) => {
                        RENodeId::Global(GlobalAddress::Component(component_address))
                    }
                    ComponentAddress::Clock(..)
                    | ComponentAddress::EpochManager(..)
                    | ComponentAddress::Validator(..)
                    | ComponentAddress::Identity(..)
                    | ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
                    | ComponentAddress::EddsaEd25519VirtualIdentity(..) => {
                        return Err(RuntimeError::InterpreterError(
                            InterpreterError::InvalidInvocation,
                        ));
                    }
                },
                ScryptoReceiver::Component(component_id) => RENodeId::Component(component_id),
            };

            // Type Check
            {
                let handle = api.lock_substate(
                    original_node_id,
                    SubstateOffset::Component(ComponentOffset::Info),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.get_ref(handle)?;
                let component_info = substate_ref.component_info(); // TODO: Remove clone()

                // Type check
                if !component_info.package_address.eq(&self.package_address) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidInvocation,
                    ));
                }
                if !component_info.blueprint_name.eq(&self.blueprint_name) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidInvocation,
                    ));
                }

                api.drop_lock(handle)?;
            }

            // Deref if global
            // TODO: Move into kernel
            let resolved_receiver =
                if let Some((derefed, derefed_lock)) = api.deref(original_node_id)? {
                    ResolvedReceiver::derefed(derefed, original_node_id, derefed_lock)
                } else {
                    ResolvedReceiver::new(original_node_id)
                };

            // Pass the component ref
            node_refs_to_copy.insert(resolved_receiver.receiver);

            (
                Some(resolved_receiver.receiver.into()),
                ResolvedActor::method(FnIdentifier::Scrypto(scrypto_fn_ident), resolved_receiver),
            )
        } else {
            (
                None,
                ResolvedActor::function(FnIdentifier::Scrypto(scrypto_fn_ident)),
            )
        };

        // Signature check + retrieve export_name
        let export_name = {
            let package_global = RENodeId::Global(GlobalAddress::Package(self.package_address));
            let handle = api.lock_substate(
                package_global,
                SubstateOffset::Package(PackageOffset::Info),
                LockFlags::read_only(),
            )?;
            let substate_ref = api.get_ref(handle)?;
            let package = substate_ref.package_info(); // TODO: Remove clone()
                                                       // Find the abi
            let abi = package.blueprint_abi(&self.blueprint_name).ok_or(
                RuntimeError::InterpreterError(InterpreterError::InvalidScryptoInvocation(
                    self.package_address,
                    self.blueprint_name.clone(),
                    self.fn_name.clone(),
                    ScryptoFnResolvingError::BlueprintNotFound,
                )),
            )?;
            let fn_abi = abi
                .get_fn_abi(&self.fn_name)
                .ok_or(RuntimeError::InterpreterError(
                    InterpreterError::InvalidScryptoInvocation(
                        self.package_address,
                        self.blueprint_name.clone(),
                        self.fn_name.clone(),
                        ScryptoFnResolvingError::MethodNotFound,
                    ),
                ))?;

            if fn_abi.mutability.is_some() != self.receiver.is_some() {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidInvocation,
                ));
            }

            if !match_schema_with_value(&fn_abi.input, args.as_value()) {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidScryptoInvocation(
                        self.package_address,
                        self.blueprint_name.clone(),
                        self.fn_name.clone(),
                        ScryptoFnResolvingError::InvalidInput,
                    ),
                ));
            }

            let export_name = fn_abi.export_name.clone();
            api.drop_lock(handle)?;

            export_name
        };

        let executor = ScryptoExecutor {
            package_address: self.package_address,
            export_name,
            component_id: receiver,
            args: args.into(),
        };

        // TODO: remove? currently needed for `Runtime::package_address()` API.
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Package(
            self.package_address,
        )));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Component(EPOCH_MANAGER)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Component(CLOCK)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(
            ECDSA_SECP256K1_TOKEN,
        )));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(
            EDDSA_ED25519_TOKEN,
        )));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Package(ACCOUNT_PACKAGE)));

        Ok((
            actor,
            CallFrameUpdate {
                nodes_to_move,
                node_refs_to_copy,
            },
            executor,
        ))
    }
}
