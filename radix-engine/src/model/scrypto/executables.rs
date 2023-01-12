use radix_engine_interface::data::*;

use crate::engine::*;
use crate::model::TransactionProcessorError;
use crate::types::*;

impl ExecutableInvocation for ScryptoMethodInvocation {
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

        let (executor, actor) = {
            let original_node_id = match self.receiver {
                ScryptoReceiver::Global(address) => {
                    RENodeId::Global(GlobalAddress::Component(address))
                }
                ScryptoReceiver::Component(component_id) => RENodeId::Component(component_id),
            };

            // Deref if global
            // TODO: Move into kernel
            let resolved_receiver =
                if let Some((derefed, derefed_lock)) = api.deref(original_node_id)? {
                    ResolvedReceiver::derefed(derefed, original_node_id, derefed_lock)
                } else {
                    ResolvedReceiver::new(original_node_id)
                };

            // Load the package substate
            // TODO: Move this in a better spot when more refactors are done
            let component_node_id = resolved_receiver.receiver;
            let component_info = {
                let handle = api.lock_substate(
                    component_node_id,
                    SubstateOffset::Component(ComponentOffset::Info),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.get_ref(handle)?;
                let component_info = substate_ref.component_info().clone(); // TODO: Remove clone()
                api.drop_lock(handle)?;

                component_info
            };
            let package = {
                let package_global =
                    RENodeId::Global(GlobalAddress::Package(component_info.package_address));
                let handle = api.lock_substate(
                    package_global,
                    SubstateOffset::Package(PackageOffset::Info),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.get_ref(handle)?;
                let package = substate_ref.package_info().clone(); // TODO: Remove clone()
                api.drop_lock(handle)?;

                package
            };

            // Pass the component ref
            // TODO: remove? currently needed for `Runtime::package_address()` API.
            let global_node_id =
                RENodeId::Global(GlobalAddress::Package(component_info.package_address));
            node_refs_to_copy.insert(global_node_id);
            node_refs_to_copy.insert(component_node_id);

            // Find the abi
            let abi = package
                .blueprint_abi(&component_info.blueprint_name)
                .ok_or(RuntimeError::InterpreterError(
                    InterpreterError::InvalidScryptoMethodInvocation(
                        self.method_name.clone(),
                        ScryptoFnResolvingError::BlueprintNotFound,
                    ),
                ))?;
            let fn_abi =
                abi.get_fn_abi(&self.method_name)
                    .ok_or(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoMethodInvocation(
                            self.method_name.clone(),
                            ScryptoFnResolvingError::MethodNotFound,
                        ),
                    ))?;
            if fn_abi.mutability.is_none() {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidScryptoMethodInvocation(
                        self.method_name.clone(),
                        ScryptoFnResolvingError::MethodNotFound,
                    ),
                ));
            }

            // Check input against the ABI
            if !match_schema_with_value(&fn_abi.input, args.as_value()) {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidScryptoMethodInvocation(
                        self.method_name.clone(),
                        ScryptoFnResolvingError::InvalidInput,
                    ),
                ));
            }

            let scrypto_fn_ident = ScryptoFnIdentifier::new(
                component_info.package_address,
                component_info.blueprint_name,
                self.method_name.clone(),
            );

            (
                ScryptoExecutor {
                    package_address: component_info.package_address,
                    export_name: fn_abi.export_name.clone(),
                    component_id: Some(component_node_id.into()),
                    args: args.into_vec(),
                },
                ResolvedActor::method(FnIdentifier::Scrypto(scrypto_fn_ident), resolved_receiver),
            )
        };

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

impl ExecutableInvocation for ScryptoFunctionInvocation {
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

        let (executor, actor) = {
            // Load the package substate
            // TODO: Move this in a better spot when more refactors are done
            let global_node_id = RENodeId::Global(GlobalAddress::Package(self.package_address));

            let package = {
                let handle = api.lock_substate(
                    global_node_id,
                    SubstateOffset::Package(PackageOffset::Info),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.get_ref(handle)?;
                let package = substate_ref.package_info().clone(); // TODO: Remove clone()
                api.drop_lock(handle)?;

                package
            };

            // Pass the package ref
            // TODO: remove? currently needed for `Runtime::package_address()` API.
            node_refs_to_copy.insert(global_node_id);

            // Find the abi
            let abi = package.blueprint_abi(&self.blueprint_name).ok_or(
                RuntimeError::InterpreterError(InterpreterError::InvalidScryptoFunctionInvocation(
                    self.package_address,
                    self.blueprint_name.clone(),
                    self.function_name.clone(),
                    ScryptoFnResolvingError::BlueprintNotFound,
                )),
            )?;
            let fn_abi =
                abi.get_fn_abi(&self.function_name)
                    .ok_or(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFunctionInvocation(
                            self.package_address,
                            self.blueprint_name.clone(),
                            self.function_name.clone(),
                            ScryptoFnResolvingError::FunctionNotFound,
                        ),
                    ))?;
            if fn_abi.mutability.is_some() {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidScryptoFunctionInvocation(
                        self.package_address,
                        self.blueprint_name.clone(),
                        self.function_name.clone(),
                        ScryptoFnResolvingError::FunctionNotFound,
                    ),
                ));
            }
            // Check input against the ABI

            if !match_schema_with_value(&fn_abi.input, args.as_value()) {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidScryptoFunctionInvocation(
                        self.package_address,
                        self.blueprint_name.clone(),
                        self.function_name.clone(),
                        ScryptoFnResolvingError::InvalidInput,
                    ),
                ));
            }

            let scrypto_fn_ident = ScryptoFnIdentifier::new(
                self.package_address,
                self.blueprint_name.clone(),
                self.function_name.clone(),
            );

            (
                ScryptoExecutor {
                    package_address: self.package_address,
                    export_name: fn_abi.export_name.clone(),
                    component_id: None,
                    args: args.into_vec(),
                },
                ResolvedActor::function(FnIdentifier::Scrypto(scrypto_fn_ident)),
            )
        };

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

impl ExecutableInvocation for ParsedScryptoFunctionInvocation {
    type Exec = ScryptoExecutorToParsed;

    fn resolve<D: ResolverApi + SystemApi>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut node_refs_to_copy = HashSet::new();

        let nodes_to_move = self
            .args
            .owned_node_ids()
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                    TransactionProcessorError::ReadOwnedNodesError(e),
                ))
            })?
            .into_iter()
            .collect();
        for global_address in self.args.global_references() {
            node_refs_to_copy.insert(RENodeId::Global(global_address));
        }

        let (executor, actor) = {
            // Load the package substate
            // TODO: Move this in a better spot when more refactors are done
            let global_node_id = RENodeId::Global(GlobalAddress::Package(self.package_address));

            let package = {
                let handle = api.lock_substate(
                    global_node_id,
                    SubstateOffset::Package(PackageOffset::Info),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.get_ref(handle)?;
                let package = substate_ref.package_info().clone(); // TODO: Remove clone()
                api.drop_lock(handle)?;

                package
            };

            // Pass the package ref
            // TODO: remove? currently needed for `Runtime::package_address()` API.
            node_refs_to_copy.insert(global_node_id);

            // Find the abi
            let abi = package.blueprint_abi(&self.blueprint_name).ok_or(
                RuntimeError::InterpreterError(InterpreterError::InvalidScryptoFunctionInvocation(
                    self.package_address,
                    self.blueprint_name.clone(),
                    self.function_name.clone(),
                    ScryptoFnResolvingError::BlueprintNotFound,
                )),
            )?;
            let fn_abi =
                abi.get_fn_abi(&self.function_name)
                    .ok_or(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFunctionInvocation(
                            self.package_address,
                            self.blueprint_name.clone(),
                            self.function_name.clone(),
                            ScryptoFnResolvingError::FunctionNotFound,
                        ),
                    ))?;
            if fn_abi.mutability.is_some() {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidScryptoFunctionInvocation(
                        self.package_address,
                        self.blueprint_name.clone(),
                        self.function_name.clone(),
                        ScryptoFnResolvingError::FunctionNotFound,
                    ),
                ));
            }
            // Check input against the ABI

            if !match_schema_with_value(&fn_abi.input, self.args.as_value()) {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidScryptoFunctionInvocation(
                        self.package_address,
                        self.blueprint_name.clone(),
                        self.function_name.clone(),
                        ScryptoFnResolvingError::InvalidInput,
                    ),
                ));
            }

            let scrypto_fn_ident = ScryptoFnIdentifier::new(
                self.package_address,
                self.blueprint_name.clone(),
                self.function_name.clone(),
            );

            (
                ScryptoExecutorToParsed {
                    //instance,
                    package_address: self.package_address,
                    export_name: fn_abi.export_name.clone(),
                    component_id: None,
                    args: self.args.into_vec(),
                },
                ResolvedActor::function(FnIdentifier::Scrypto(scrypto_fn_ident)),
            )
        };

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

impl ExecutableInvocation for ParsedScryptoMethodInvocation {
    type Exec = ScryptoExecutorToParsed;

    fn resolve<D: ResolverApi + SystemApi>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut node_refs_to_copy = HashSet::new();

        let nodes_to_move = self
            .args
            .owned_node_ids()
            .map_err(|e| {
                RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                    TransactionProcessorError::ReadOwnedNodesError(e),
                ))
            })?
            .into_iter()
            .collect();
        for global_address in self.args.global_references() {
            node_refs_to_copy.insert(RENodeId::Global(global_address));
        }

        let (executor, actor) = {
            let original_node_id = match self.receiver {
                ScryptoReceiver::Global(address) => {
                    RENodeId::Global(GlobalAddress::Component(address))
                }
                ScryptoReceiver::Component(component_id) => RENodeId::Component(component_id),
            };

            // Deref if global
            // TODO: Move into kernel
            let resolved_receiver =
                if let Some((derefed, derefed_lock)) = api.deref(original_node_id)? {
                    ResolvedReceiver::derefed(derefed, original_node_id, derefed_lock)
                } else {
                    ResolvedReceiver::new(original_node_id)
                };

            // Load the package substate
            // TODO: Move this in a better spot when more refactors are done
            let component_node_id = resolved_receiver.receiver;
            let component_info = {
                let handle = api.lock_substate(
                    component_node_id,
                    SubstateOffset::Component(ComponentOffset::Info),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.get_ref(handle)?;
                let component_info = substate_ref.component_info().clone(); // TODO: Remove clone()
                api.drop_lock(handle)?;

                component_info
            };
            let package = {
                let package_global =
                    RENodeId::Global(GlobalAddress::Package(component_info.package_address));
                let handle = api.lock_substate(
                    package_global,
                    SubstateOffset::Package(PackageOffset::Info),
                    LockFlags::read_only(),
                )?;
                let substate_ref = api.get_ref(handle)?;
                let package = substate_ref.package_info().clone(); // TODO: Remove clone()
                api.drop_lock(handle)?;

                package
            };

            // Pass the component ref
            // TODO: remove? currently needed for `Runtime::package_address()` API.
            let global_node_id =
                RENodeId::Global(GlobalAddress::Package(component_info.package_address));
            node_refs_to_copy.insert(global_node_id);
            node_refs_to_copy.insert(component_node_id);

            // Find the abi
            let abi = package
                .blueprint_abi(&component_info.blueprint_name)
                .ok_or(RuntimeError::InterpreterError(
                    InterpreterError::InvalidScryptoMethodInvocation(
                        self.method_name.clone(),
                        ScryptoFnResolvingError::BlueprintNotFound,
                    ),
                ))?;
            let fn_abi =
                abi.get_fn_abi(&self.method_name)
                    .ok_or(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoMethodInvocation(
                            self.method_name.clone(),
                            ScryptoFnResolvingError::MethodNotFound,
                        ),
                    ))?;
            if fn_abi.mutability.is_none() {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidScryptoMethodInvocation(
                        self.method_name.clone(),
                        ScryptoFnResolvingError::MethodNotFound,
                    ),
                ));
            }

            // Check input against the ABI
            if !match_schema_with_value(&fn_abi.input, self.args.as_value()) {
                return Err(RuntimeError::InterpreterError(
                    InterpreterError::InvalidScryptoMethodInvocation(
                        self.method_name.clone(),
                        ScryptoFnResolvingError::InvalidInput,
                    ),
                ));
            }

            let scrypto_fn_ident = ScryptoFnIdentifier::new(
                component_info.package_address,
                component_info.blueprint_name,
                self.method_name.clone(),
            );

            (
                ScryptoExecutorToParsed {
                    package_address: component_info.package_address,
                    export_name: fn_abi.export_name.clone(),
                    component_id: Some(component_node_id.into()),
                    args: self.args.into_vec(),
                },
                ResolvedActor::method(FnIdentifier::Scrypto(scrypto_fn_ident), resolved_receiver),
            )
        };

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
