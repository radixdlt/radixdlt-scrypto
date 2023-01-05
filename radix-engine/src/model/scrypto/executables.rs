use radix_engine_interface::data::*;

use crate::engine::*;
use crate::types::*;
use crate::wasm::*;

impl<W: WasmEngine> ExecutableInvocation<W> for ScryptoInvocation {
    type Exec = ScryptoExecutor<W::WasmInstance>;

    fn resolve<D: ResolverApi<W> + SystemApi>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut node_refs_to_copy = HashSet::new();
        let args = IndexedScryptoValue::from_slice(&self.args())
            .map_err(|e| RuntimeError::KernelError(KernelError::InvalidScryptoValue(e)))?;

        let nodes_to_move = args.owned_node_ids().into_iter().collect();
        for global_address in args.global_references() {
            node_refs_to_copy.insert(RENodeId::Global(global_address));
        }

        let (executor, actor) = match &self {
            ScryptoInvocation::Function(function_ident, _) => {
                // Load the package substate
                // TODO: Move this in a better spot when more refactors are done
                let package_address = match function_ident.package {
                    ScryptoPackage::Global(address) => address,
                };
                let global_node_id = RENodeId::Global(GlobalAddress::Package(package_address));

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
                let abi = package
                    .blueprint_abi(&function_ident.blueprint_name)
                    .ok_or(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFunctionInvocation(
                            function_ident.clone(),
                            ScryptoFnResolvingError::BlueprintNotFound,
                        ),
                    ))?;
                let fn_abi = abi.get_fn_abi(&function_ident.function_name).ok_or(
                    RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFunctionInvocation(
                            function_ident.clone(),
                            ScryptoFnResolvingError::FunctionNotFound,
                        ),
                    ),
                )?;
                if fn_abi.mutability.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFunctionInvocation(
                            function_ident.clone(),
                            ScryptoFnResolvingError::FunctionNotFound,
                        ),
                    ));
                }
                // Check input against the ABI

                if !match_schema_with_value(&fn_abi.input, &args.value) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFunctionInvocation(
                            function_ident.clone(),
                            ScryptoFnResolvingError::InvalidInput,
                        ),
                    ));
                }

                let scrypto_fn_ident = ScryptoFnIdentifier::new(
                    package_address,
                    function_ident.blueprint_name.clone(),
                    function_ident.function_name.clone(),
                );

                // Emit event
                api.on_wasm_instantiation(package.code())?;

                (
                    api.vm().create_executor(
                        package_address,
                        &package.code,
                        fn_abi.export_name.clone(),
                        None,
                        args.as_vec(),
                        fn_abi.output.clone(),
                    ),
                    ResolvedActor::function(FnIdentifier::Scrypto(scrypto_fn_ident)),
                )
            }
            ScryptoInvocation::Method(method_ident, _) => {
                let original_node_id = match method_ident.receiver {
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
                            method_ident.clone(),
                            ScryptoFnResolvingError::BlueprintNotFound,
                        ),
                    ))?;
                let fn_abi = abi.get_fn_abi(&method_ident.method_name).ok_or(
                    RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoMethodInvocation(
                            method_ident.clone(),
                            ScryptoFnResolvingError::MethodNotFound,
                        ),
                    ),
                )?;
                if fn_abi.mutability.is_none() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoMethodInvocation(
                            method_ident.clone(),
                            ScryptoFnResolvingError::MethodNotFound,
                        ),
                    ));
                }

                // Check input against the ABI
                if !match_schema_with_value(&fn_abi.input, &args.value) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoMethodInvocation(
                            method_ident.clone(),
                            ScryptoFnResolvingError::InvalidInput,
                        ),
                    ));
                }

                let scrypto_fn_ident = ScryptoFnIdentifier::new(
                    component_info.package_address,
                    component_info.blueprint_name,
                    method_ident.method_name.clone(),
                );

                // Emit event
                api.on_wasm_instantiation(package.code())?;

                (
                    api.vm().create_executor(
                        component_info.package_address,
                        &package.code,
                        fn_abi.export_name.clone(),
                        Some(component_node_id.into()),
                        args.as_vec(),
                        fn_abi.output.clone(),
                    ),
                    ResolvedActor::method(
                        FnIdentifier::Scrypto(scrypto_fn_ident),
                        resolved_receiver,
                    ),
                )
            }
        };

        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::System(CLOCK)));
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

impl<W: WasmEngine> ExecutableInvocation<W> for ParsedScryptoInvocation {
    type Exec = ScryptoExecutorToParsed<W::WasmInstance>;

    fn resolve<D: ResolverApi<W> + SystemApi>(
        self,
        api: &mut D,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut node_refs_to_copy = HashSet::new();

        let nodes_to_move = self.args().owned_node_ids().into_iter().collect();
        for global_address in self.args().global_references() {
            node_refs_to_copy.insert(RENodeId::Global(global_address));
        }

        let (executor, actor) = match self {
            ParsedScryptoInvocation::Function(function_ident, args) => {
                // Load the package substate
                // TODO: Move this in a better spot when more refactors are done
                let package_address = match function_ident.package {
                    ScryptoPackage::Global(address) => address,
                };
                let global_node_id = RENodeId::Global(GlobalAddress::Package(package_address));

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
                let abi = package
                    .blueprint_abi(&function_ident.blueprint_name)
                    .ok_or(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFunctionInvocation(
                            function_ident.clone(),
                            ScryptoFnResolvingError::BlueprintNotFound,
                        ),
                    ))?;
                let fn_abi = abi.get_fn_abi(&function_ident.function_name).ok_or(
                    RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFunctionInvocation(
                            function_ident.clone(),
                            ScryptoFnResolvingError::FunctionNotFound,
                        ),
                    ),
                )?;
                if fn_abi.mutability.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFunctionInvocation(
                            function_ident.clone(),
                            ScryptoFnResolvingError::FunctionNotFound,
                        ),
                    ));
                }
                // Check input against the ABI

                if !match_schema_with_value(&fn_abi.input, &args.value) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFunctionInvocation(
                            function_ident.clone(),
                            ScryptoFnResolvingError::InvalidInput,
                        ),
                    ));
                }

                let scrypto_fn_ident = ScryptoFnIdentifier::new(
                    package_address,
                    function_ident.blueprint_name.clone(),
                    function_ident.function_name.clone(),
                );

                // Emit event
                api.on_wasm_instantiation(package.code())?;

                (
                    api.vm().create_executor_to_parsed(
                        package_address,
                        &package.code,
                        fn_abi.export_name.clone(),
                        None,
                        args.as_vec(),
                        fn_abi.output.clone(),
                    ),
                    ResolvedActor::function(FnIdentifier::Scrypto(scrypto_fn_ident)),
                )
            }
            ParsedScryptoInvocation::Method(method_ident, args) => {
                let original_node_id = match method_ident.receiver {
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
                            method_ident.clone(),
                            ScryptoFnResolvingError::BlueprintNotFound,
                        ),
                    ))?;
                let fn_abi = abi.get_fn_abi(&method_ident.method_name).ok_or(
                    RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoMethodInvocation(
                            method_ident.clone(),
                            ScryptoFnResolvingError::MethodNotFound,
                        ),
                    ),
                )?;
                if fn_abi.mutability.is_none() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoMethodInvocation(
                            method_ident.clone(),
                            ScryptoFnResolvingError::MethodNotFound,
                        ),
                    ));
                }

                // Check input against the ABI
                if !match_schema_with_value(&fn_abi.input, &args.value) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoMethodInvocation(
                            method_ident.clone(),
                            ScryptoFnResolvingError::InvalidInput,
                        ),
                    ));
                }

                let scrypto_fn_ident = ScryptoFnIdentifier::new(
                    component_info.package_address,
                    component_info.blueprint_name,
                    method_ident.method_name.clone(),
                );

                // Emit event
                api.on_wasm_instantiation(package.code())?;

                (
                    api.vm().create_executor_to_parsed(
                        component_info.package_address,
                        &package.code,
                        fn_abi.export_name.clone(),
                        Some(component_node_id.into()),
                        args.as_vec(),
                        fn_abi.output.clone(),
                    ),
                    ResolvedActor::method(
                        FnIdentifier::Scrypto(scrypto_fn_ident),
                        resolved_receiver,
                    ),
                )
            }
        };

        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)));
        node_refs_to_copy.insert(RENodeId::Global(GlobalAddress::System(CLOCK)));
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
