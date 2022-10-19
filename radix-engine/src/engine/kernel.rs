use transaction::errors::IdAllocationError;
use transaction::model::{AuthZoneParams, Instruction};
use transaction::validation::*;

use crate::engine::call_frame::SubstateLock;
use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use crate::wasm::*;

#[macro_export]
macro_rules! trace {
    ( $self: expr, $level: expr, $msg: expr $( , $arg:expr )* ) => {
        #[cfg(not(feature = "alloc"))]
        if $self.trace {
            println!("{}[{:5}] {}", "  ".repeat(Self::current_frame(&$self.call_frames).depth) , $level, sbor::rust::format!($msg, $( $arg ),*));
        }
    };
}

pub struct Kernel<
    'g, // Lifetime of values outliving all frames
    's, // Substate store lifetime
    W,  // WASM engine type
    I,  // WASM instance type
    R,  // Fee reserve type
> where
    W: WasmEngine<I>,
    I: WasmInstance,
    R: FeeReserve,
{
    execution_mode: ExecutionMode,

    /// The transaction hash
    transaction_hash: Hash,
    /// Blobs attached to the transaction
    blobs: &'g HashMap<Hash, Vec<u8>>,
    /// The max call depth
    max_depth: usize,

    /// State track
    track: &'g mut Track<'s, R>,

    /// Interpreter capable of running scrypto programs
    scrypto_interpreter: &'g mut ScryptoInterpreter<I, W>,

    /// ID allocator
    id_allocator: IdAllocator,

    /// Call frames
    call_frames: Vec<CallFrame>,

    /// Kernel modules
    modules: Vec<Box<dyn Module<R>>>,
}

impl<'g, 's, W, I, R> Kernel<'g, 's, W, I, R>
where
    W: WasmEngine<I>,
    I: WasmInstance,
    R: FeeReserve,
{
    pub fn new(
        transaction_hash: Hash,
        auth_zone_params: AuthZoneParams,
        blobs: &'g HashMap<Hash, Vec<u8>>,
        max_depth: usize,
        track: &'g mut Track<'s, R>,
        scrypto_interpreter: &'g mut ScryptoInterpreter<I, W>,
        modules: Vec<Box<dyn Module<R>>>,
    ) -> Self {
        let frame = CallFrame::new_root();
        let mut kernel = Self {
            execution_mode: ExecutionMode::Kernel,
            transaction_hash,
            blobs,
            max_depth,
            track,
            scrypto_interpreter,
            id_allocator: IdAllocator::new(IdSpace::Application),
            call_frames: vec![frame],
            modules,
        };

        // Initial authzone
        // TODO: Move into module initialization
        let virtualizable_proofs_resource_addresses =
            auth_zone_params.virtualizable_proofs_resource_addresses;
        let mut proofs_to_create = BTreeMap::<ResourceAddress, BTreeSet<NonFungibleId>>::new();
        for non_fungible in auth_zone_params.initial_proofs {
            proofs_to_create
                .entry(non_fungible.resource_address())
                .or_insert(BTreeSet::new())
                .insert(non_fungible.non_fungible_id());
        }
        let mut proofs = Vec::new();

        for (resource_address, non_fungible_ids) in proofs_to_create {
            let bucket_id =
                kernel.create_non_fungible_bucket_with_ids(resource_address, non_fungible_ids);

            let proof = kernel
                .execute_in_mode::<_, _, RuntimeError>(ExecutionMode::AuthModule, |system_api| {
                    let node_id = RENodeId::Bucket(bucket_id);
                    let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
                    let handle = system_api.lock_substate(node_id, offset, LockFlags::MUTABLE)?;
                    let mut substate_mut = system_api.get_ref_mut(handle)?;
                    let mut raw_mut = substate_mut.get_raw_mut();
                    let proof = raw_mut
                        .bucket()
                        .create_proof(bucket_id)
                        .expect("Failed to create proof");
                    substate_mut.flush()?;
                    Ok(proof)
                })
                .unwrap();

            proofs.push(proof);
        }

        // Create empty buckets for virtual proofs
        let mut virtual_proofs_buckets: BTreeMap<ResourceAddress, BucketId> = BTreeMap::new();
        for resource_address in virtualizable_proofs_resource_addresses {
            let bucket_id = kernel
                .create_non_fungible_bucket_with_ids(resource_address.clone(), BTreeSet::new());
            virtual_proofs_buckets.insert(resource_address, bucket_id);
        }

        let auth_zone = AuthZoneStackSubstate::new(proofs, virtual_proofs_buckets);

        kernel
            .node_create(HeapRENode::AuthZone(auth_zone))
            .expect("Failed to create AuthZone");

        kernel
    }

    fn create_non_fungible_bucket_with_ids(
        &mut self,
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleId>,
    ) -> BucketId {
        match self
            .node_create(HeapRENode::Bucket(BucketSubstate::new(
                Resource::new_non_fungible(resource_address, ids),
            )))
            .expect("Failed to create a bucket")
        {
            RENodeId::Bucket(bucket_id) => bucket_id,
            _ => panic!("Expected Bucket RENodeId but received something else"),
        }
    }

    fn new_uuid(
        id_allocator: &mut IdAllocator,
        transaction_hash: Hash,
    ) -> Result<u128, IdAllocationError> {
        id_allocator.new_uuid(transaction_hash)
    }

    // TODO: Move this into a native function
    fn globalize(
        id_allocator: &mut IdAllocator,
        transaction_hash: Hash,
        node_id: RENodeId,
        re_node: &HeapRENode,
    ) -> Result<(GlobalAddress, GlobalAddressSubstate), RuntimeError> {
        match re_node {
            HeapRENode::Component(component) => {
                let component_address = id_allocator
                    .new_component_address(
                        transaction_hash,
                        component.info.package_address,
                        &component.info.blueprint_name,
                    )
                    .map_err(|e| RuntimeError::KernelError(KernelError::IdAllocationError(e)))?;
                let component_id: ComponentId = node_id.into();
                Ok((
                    GlobalAddress::Component(component_address),
                    GlobalAddressSubstate::Component(scrypto::component::Component(component_id)),
                ))
            }
            HeapRENode::System(..) => {
                let component_address = id_allocator
                    .new_system_component_address(transaction_hash)
                    .map_err(|e| RuntimeError::KernelError(KernelError::IdAllocationError(e)))?;

                let component_id: ComponentId = node_id.into();
                Ok((
                    GlobalAddress::Component(component_address),
                    GlobalAddressSubstate::SystemComponent(scrypto::component::Component(
                        component_id,
                    )),
                ))
            }
            HeapRENode::ResourceManager(..) => {
                let resource_address: ResourceAddress = node_id.into();

                Ok((
                    GlobalAddress::Resource(resource_address),
                    GlobalAddressSubstate::Resource(resource_address),
                ))
            }
            HeapRENode::Package(..) => {
                let package_address: PackageAddress = node_id.into();

                Ok((
                    GlobalAddress::Package(package_address),
                    GlobalAddressSubstate::Package(package_address),
                ))
            }
            _ => Err(RuntimeError::KernelError(
                KernelError::RENodeGlobalizeTypeNotAllowed(node_id),
            )),
        }
    }

    fn new_node_id(
        id_allocator: &mut IdAllocator,
        transaction_hash: Hash,
        re_node: &HeapRENode,
    ) -> Result<RENodeId, IdAllocationError> {
        match re_node {
            HeapRENode::Global(..) => panic!("Should not get here"),
            HeapRENode::AuthZone(..) => {
                let auth_zone_id = id_allocator.new_auth_zone_id()?;
                Ok(RENodeId::AuthZoneStack(auth_zone_id))
            }
            HeapRENode::Bucket(..) => {
                let bucket_id = id_allocator.new_bucket_id()?;
                Ok(RENodeId::Bucket(bucket_id))
            }
            HeapRENode::Proof(..) => {
                let proof_id = id_allocator.new_proof_id()?;
                Ok(RENodeId::Proof(proof_id))
            }
            HeapRENode::Worktop(..) => Ok(RENodeId::Worktop),
            HeapRENode::Vault(..) => {
                let vault_id = id_allocator.new_vault_id(transaction_hash)?;
                Ok(RENodeId::Vault(vault_id))
            }
            HeapRENode::KeyValueStore(..) => {
                let kv_store_id = id_allocator.new_kv_store_id(transaction_hash)?;
                Ok(RENodeId::KeyValueStore(kv_store_id))
            }
            HeapRENode::NonFungibleStore(..) => {
                let non_fungible_store_id =
                    id_allocator.new_non_fungible_store_id(transaction_hash)?;
                Ok(RENodeId::NonFungibleStore(non_fungible_store_id))
            }
            HeapRENode::Package(..) => {
                // Security Alert: ensure ID allocating will practically never fail
                let package_address = id_allocator.new_package_address(transaction_hash)?;
                Ok(RENodeId::Package(package_address))
            }
            HeapRENode::ResourceManager(..) => {
                let resource_address = id_allocator.new_resource_address(transaction_hash)?;
                Ok(RENodeId::ResourceManager(resource_address))
            }
            HeapRENode::Component(..) => {
                let component_id = id_allocator.new_component_id(transaction_hash)?;
                Ok(RENodeId::Component(component_id))
            }
            HeapRENode::System(..) => {
                let component_id = id_allocator.new_component_id(transaction_hash)?;
                Ok(RENodeId::System(component_id))
            }
        }
    }

    fn run(
        &mut self,
        actor: REActor,
        input: ScryptoValue,
        owned_nodes: HashMap<RENodeId, HeapRootRENode>,
        mut refed_nodes: HashMap<RENodeId, RENodePointer>,
    ) -> Result<(ScryptoValue, HashMap<RENodeId, HeapRootRENode>), RuntimeError> {
        let new_refed_nodes = self.execute_in_mode(ExecutionMode::AuthModule, |system_api| {
            AuthModule::on_before_frame_start(&actor, &input, system_api).map_err(|e| match e {
                InvokeError::Error(e) => RuntimeError::ModuleError(e.into()),
                InvokeError::Downstream(runtime_error) => runtime_error,
            })
        })?;

        // TODO: Do this in a better way by allowing module to execute in next call frame
        for new_refed_node in new_refed_nodes {
            let node_pointer = Self::current_frame(&self.call_frames)
                .get_node_pointer(new_refed_node)
                .unwrap();
            refed_nodes.insert(new_refed_node, node_pointer);
        }

        let frame = CallFrame::new_child(
            Self::current_frame(&self.call_frames).depth + 1,
            actor,
            owned_nodes,
            refed_nodes,
        );
        self.call_frames.push(frame);

        let actor = Self::current_frame(&self.call_frames).actor.clone();
        let output = match actor.clone() {
            REActor::Function(ResolvedFunction::Native(native_fn)) => self
                .execute_in_mode(ExecutionMode::Application, |system_api| {
                    NativeInterpreter::run_function(native_fn, input, system_api)
                }),
            REActor::Method(ResolvedMethod::Native(native_method), resolved_receiver) => self
                .execute_in_mode(ExecutionMode::Application, |system_api| {
                    NativeInterpreter::run_method(
                        native_method,
                        resolved_receiver,
                        input,
                        system_api,
                    )
                }),
            REActor::Function(ResolvedFunction::Scrypto {
                package_address, ..
            })
            | REActor::Method(
                ResolvedMethod::Scrypto {
                    package_address, ..
                },
                ..,
            ) => {
                // TODO: Move into interpreter when interpreter trait implemented
                let package = self.execute_in_mode::<_, _, RuntimeError>(
                    ExecutionMode::ScryptoInterpreter,
                    |system_api| {
                        let package_id = RENodeId::Global(GlobalAddress::Package(package_address));
                        let package_offset = SubstateOffset::Package(PackageOffset::Package);
                        let handle = system_api.lock_substate(
                            package_id,
                            package_offset,
                            LockFlags::read_only(),
                        )?;
                        let substate_ref = system_api.get_ref(handle)?;
                        let package = substate_ref.package().clone();
                        system_api.drop_lock(handle)?;
                        Ok(package)
                    },
                )?;
                let mut executor = self.scrypto_interpreter.create_executor(package);

                self.execute_in_mode(ExecutionMode::ScryptoInterpreter, |system_api| {
                    executor.run(input, system_api)
                })
            }
        }?;

        // Process return data
        let mut nodes_to_return = HashMap::new();
        for node_id in output.node_ids() {
            let mut node = Self::current_frame_mut(&mut self.call_frames).take_node(node_id)?;
            let root_node = node.root_mut();
            root_node.prepare_move_upstream(node_id)?;
            nodes_to_return.insert(node_id, node);
        }

        // Check references returned
        for global_address in output.global_references() {
            let node_id = RENodeId::Global(global_address);
            if !Self::current_frame_mut(&mut self.call_frames)
                .node_refs
                .contains_key(&node_id)
            {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidReferenceReturn(global_address),
                ));
            }
        }

        // Auto drop locks
        let frame = Self::current_frame_mut(&mut self.call_frames);
        for (_, lock) in frame.drain_locks() {
            let SubstateLock {
                substate_pointer: (node_pointer, offset),
                flags,
                ..
            } = lock;
            if !(matches!(offset, SubstateOffset::KeyValueStore(..))
                || matches!(
                    offset,
                    SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..))
                ))
            {
                node_pointer
                    .release_lock(
                        offset,
                        flags.contains(LockFlags::UNMODIFIED_BASE),
                        self.track,
                    )
                    .map_err(RuntimeError::KernelError)?;
            }
        }

        // TODO: Auto drop locks of module execution as well
        self.execute_in_mode(ExecutionMode::AuthModule, |system_api| {
            AuthModule::on_frame_end(system_api).map_err(|e| match e {
                InvokeError::Error(e) => RuntimeError::ModuleError(e.into()),
                InvokeError::Downstream(runtime_error) => runtime_error,
            })
        })?;

        // drop proofs and check resource leak
        let call_frame = self.call_frames.pop().unwrap();
        call_frame.drop_frame()?;

        Ok((output, nodes_to_return))
    }

    fn current_frame_mut(call_frames: &mut Vec<CallFrame>) -> &mut CallFrame {
        call_frames.last_mut().expect("Current frame always exists")
    }

    fn current_frame(call_frames: &Vec<CallFrame>) -> &CallFrame {
        call_frames.last().expect("Current frame always exists")
    }

    pub fn node_method_deref(
        &mut self,
        node_id: RENodeId,
    ) -> Result<Option<RENodeId>, RuntimeError> {
        if let RENodeId::Global(..) = node_id {
            let node_id =
                self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Deref, |system_api| {
                    let offset = SubstateOffset::Global(GlobalOffset::Global);
                    let handle = system_api.lock_substate(node_id, offset, LockFlags::empty())?;
                    let substate_ref = system_api.get_ref(handle)?;
                    Ok(substate_ref.global_address().node_deref())
                })?;

            Ok(Some(node_id))
        } else {
            Ok(None)
        }
    }

    pub fn node_offset_deref(
        &mut self,
        node_id: RENodeId,
        offset: &SubstateOffset,
    ) -> Result<Option<RENodeId>, RuntimeError> {
        if let RENodeId::Global(..) = node_id {
            if !matches!(offset, SubstateOffset::Global(GlobalOffset::Global)) {
                let node_id = self.execute_in_mode::<_, _, RuntimeError>(
                    ExecutionMode::Deref,
                    |system_api| {
                        let handle = system_api.lock_substate(
                            node_id,
                            SubstateOffset::Global(GlobalOffset::Global),
                            LockFlags::empty(),
                        )?;
                        let substate_ref = system_api.get_ref(handle)?;
                        Ok(substate_ref.global_address().node_deref())
                    },
                )?;

                Ok(Some(node_id))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    // TODO: remove redundant code and move this method to the interpreter
    fn resolve_scrypto_actor(
        &mut self,
        fn_ident: &ScryptoFnIdent,
        args: &ScryptoValue,
    ) -> Result<(REActor, HashMap<RENodeId, RENodePointer>), RuntimeError> {
        let mut references_to_add = HashMap::new();

        let actor = match fn_ident.clone() {
            ScryptoFnIdent::Function(ScryptoFunctionIdent {
                package_address,
                blueprint_name,
                function_name,
            }) => {
                // Load the package substate
                // TODO: Move this in a better spot when more refactors are done
                let (package_node_id, package) = self.execute_in_mode::<_, _, RuntimeError>(
                    ExecutionMode::ScryptoInterpreter,
                    |system_api| {
                        let handle = system_api.lock_substate(
                            RENodeId::Global(GlobalAddress::Package(package_address)),
                            SubstateOffset::Global(GlobalOffset::Global),
                            LockFlags::read_only(),
                        )?;
                        let substate_ref = system_api.get_ref(handle)?;
                        let package_node_id = substate_ref.global_address().node_deref();
                        system_api.drop_lock(handle)?;

                        let handle = system_api.lock_substate(
                            package_node_id,
                            SubstateOffset::Package(PackageOffset::Package),
                            LockFlags::read_only(),
                        )?;
                        let substate_ref = system_api.get_ref(handle)?;
                        let package = substate_ref.package().clone(); // TODO: Remove clone()
                        system_api.drop_lock(handle)?;

                        Ok((package_node_id, package))
                    },
                )?;

                // Pass the package ref
                let node_pointer = Self::current_frame(&self.call_frames)
                    .get_node_pointer(package_node_id)
                    .unwrap();
                references_to_add.insert(package_node_id, node_pointer);

                // Find the abi
                let abi = package.blueprint_abi(&blueprint_name).ok_or(
                    RuntimeError::InterpreterError(InterpreterError::InvalidScryptoFnIdent(
                        fn_ident.clone(),
                        ScryptoActorError::BlueprintNotFound,
                    )),
                )?;
                let fn_abi =
                    abi.get_fn_abi(&function_name)
                        .ok_or(RuntimeError::InterpreterError(
                            InterpreterError::InvalidScryptoFnIdent(
                                fn_ident.clone(),
                                ScryptoActorError::FunctionNotFound,
                            ),
                        ))?;
                if fn_abi.mutability.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFnIdent(
                            fn_ident.clone(),
                            ScryptoActorError::FunctionNotFound,
                        ),
                    ));
                }
                // Check input against the ABI
                if !fn_abi.input.matches(&args.dom) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFnIdent(
                            fn_ident.clone(),
                            ScryptoActorError::InvalidInput,
                        ),
                    ));
                }

                // Emit event
                for m in &mut self.modules {
                    m.on_wasm_instantiation(&mut self.track, &mut self.call_frames, package.code())
                        .map_err(RuntimeError::ModuleError)?;
                }

                REActor::Function(ResolvedFunction::Scrypto {
                    package_address,
                    blueprint_name,
                    ident: function_name,
                    export_name: fn_abi.export_name.clone(),
                    return_type: fn_abi.output.clone(),
                })
            }
            ScryptoFnIdent::Method(ScryptoMethodIdent {
                receiver,
                method_name,
            }) => {
                let original_node_id = match receiver {
                    ScryptoReceiver::Global(address) => {
                        RENodeId::Global(GlobalAddress::Component(address))
                    }
                    ScryptoReceiver::Component(component_id) => RENodeId::Component(component_id),
                };

                // Deref if global
                let resolved_receiver =
                    if let Some(derefed) = self.node_method_deref(original_node_id)? {
                        ResolvedReceiver::derefed(Receiver::Ref(derefed), original_node_id)
                    } else {
                        ResolvedReceiver::new(Receiver::Ref(original_node_id))
                    };

                // Add the resolved receiver ref
                let component_node_id = resolved_receiver.node_id();
                let node_pointer =
                    Self::current_frame(&self.call_frames).get_node_pointer(component_node_id)?;
                references_to_add.insert(component_node_id, node_pointer);

                // Load the package substate
                // TODO: Move this in a better spot when more refactors are done
                let (component_info, _package_node_id, package) = self
                    .execute_in_mode::<_, _, RuntimeError>(
                        ExecutionMode::ScryptoInterpreter,
                        |system_api| {
                            let handle = system_api.lock_substate(
                                component_node_id,
                                SubstateOffset::Component(ComponentOffset::Info),
                                LockFlags::read_only(),
                            )?;
                            let substate_ref = system_api.get_ref(handle)?;
                            let component_info = substate_ref.component_info().clone(); // TODO: Remove clone()
                            system_api.drop_lock(handle)?;

                            let handle = system_api.lock_substate(
                                RENodeId::Global(GlobalAddress::Package(
                                    component_info.package_address,
                                )),
                                SubstateOffset::Global(GlobalOffset::Global),
                                LockFlags::read_only(),
                            )?;
                            let substate_ref = system_api.get_ref(handle)?;
                            let package_node_id = substate_ref.global_address().node_deref();
                            system_api.drop_lock(handle)?;

                            let handle = system_api.lock_substate(
                                package_node_id,
                                SubstateOffset::Package(PackageOffset::Package),
                                LockFlags::read_only(),
                            )?;
                            let substate_ref = system_api.get_ref(handle)?;
                            let package = substate_ref.package().clone(); // TODO: Remove clone()
                            system_api.drop_lock(handle)?;

                            Ok((component_info, package_node_id, package))
                        },
                    )?;

                // Pass the component ref
                let node_pointer = Self::current_frame(&self.call_frames)
                    .get_node_pointer(component_node_id)
                    .unwrap();
                references_to_add.insert(component_node_id, node_pointer);

                // Find the abi
                let abi = package
                    .blueprint_abi(&component_info.blueprint_name)
                    .ok_or(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFnIdent(
                            fn_ident.clone(),
                            ScryptoActorError::BlueprintNotFound,
                        ),
                    ))?;
                let fn_abi = abi
                    .get_fn_abi(&method_name)
                    .ok_or(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFnIdent(
                            fn_ident.clone(),
                            ScryptoActorError::MethodNotFound,
                        ),
                    ))?;
                if fn_abi.mutability.is_none() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFnIdent(
                            fn_ident.clone(),
                            ScryptoActorError::MethodNotFound,
                        ),
                    ));
                }

                // Check input against the ABI
                if !fn_abi.input.matches(&args.dom) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFnIdent(
                            fn_ident.clone(),
                            ScryptoActorError::InvalidInput,
                        ),
                    ));
                }

                // Emit event
                for m in &mut self.modules {
                    m.on_wasm_instantiation(&mut self.track, &mut self.call_frames, package.code())
                        .map_err(RuntimeError::ModuleError)?;
                }

                REActor::Method(
                    ResolvedMethod::Scrypto {
                        package_address: component_info.package_address,
                        blueprint_name: component_info.blueprint_name,
                        ident: method_name,
                        export_name: fn_abi.export_name.clone(),
                        return_type: fn_abi.output.clone(),
                    },
                    resolved_receiver,
                )
            }
        };

        Ok((actor, references_to_add))
    }

    fn resolve_native_actor(
        &mut self,
        fn_ident: &NativeFnIdent,
        _args: &ScryptoValue,
    ) -> Result<
        (
            REActor,
            HashMap<RENodeId, RENodePointer>,
            HashMap<RENodeId, HeapRootRENode>,
        ),
        RuntimeError,
    > {
        let mut references_to_add = HashMap::new();
        let mut nodes_to_move = HashMap::new();

        let not_found = RuntimeError::InterpreterError(InterpreterError::InvalidNativeFnIdent(
            fn_ident.clone(),
        ));

        let actor = match fn_ident {
            NativeFnIdent::Function(NativeFunctionIdent {
                blueprint_name,
                function_name,
            }) => {
                // TODO: use strum derive?
                let native_function = match blueprint_name.as_str() {
                    "System" => NativeFunction::System(
                        SystemFunction::from_str(function_name).map_err(|_| not_found)?,
                    ),
                    "ResourceManager" => NativeFunction::ResourceManager(
                        ResourceManagerFunction::from_str(function_name).map_err(|_| not_found)?,
                    ),
                    "Package" => NativeFunction::Package(
                        PackageFunction::from_str(function_name).map_err(|_| not_found)?,
                    ),
                    "TransactionProcessor" => NativeFunction::TransactionProcessor(
                        TransactionProcessorFunction::from_str(function_name)
                            .map_err(|_| not_found)?,
                    ),
                    _ => return Err(not_found),
                };
                REActor::Function(ResolvedFunction::Native(native_function))
            }
            NativeFnIdent::Method(NativeMethodIdent {
                receiver,
                method_name,
            }) => {
                let resolved_receiver = match receiver {
                    Receiver::Consumed(node_id) => {
                        let node =
                            Self::current_frame_mut(&mut self.call_frames).take_node(*node_id)?;
                        nodes_to_move.insert(*node_id, node);
                        ResolvedReceiver::new(Receiver::Consumed(*node_id))
                    }
                    Receiver::Ref(node_id) => {
                        // Deref
                        let resolved_receiver =
                            if let Some(derefed) = self.node_method_deref(*node_id)? {
                                ResolvedReceiver::derefed(Receiver::Ref(derefed), *node_id)
                            } else {
                                ResolvedReceiver::new(Receiver::Ref(*node_id))
                            };

                        let resolved_node_id = resolved_receiver.node_id();
                        let node_pointer = Self::current_frame(&self.call_frames)
                            .get_node_pointer(resolved_node_id)?;
                        references_to_add.insert(resolved_node_id, node_pointer);

                        resolved_receiver
                    }
                };

                let native_method = match receiver.node_id() {
                    RENodeId::Bucket(_) => NativeMethod::Bucket(
                        BucketMethod::from_str(method_name).map_err(|_| not_found)?,
                    ),
                    RENodeId::Proof(_) => NativeMethod::Proof(
                        ProofMethod::from_str(method_name).map_err(|_| not_found)?,
                    ),
                    RENodeId::AuthZoneStack(_) => NativeMethod::AuthZone(
                        AuthZoneMethod::from_str(method_name).map_err(|_| not_found)?,
                    ),
                    RENodeId::Worktop => NativeMethod::Worktop(
                        WorktopMethod::from_str(method_name).map_err(|_| not_found)?,
                    ),
                    RENodeId::Component(_) => NativeMethod::Component(
                        ComponentMethod::from_str(method_name).map_err(|_| not_found)?,
                    ),
                    RENodeId::System(_) => NativeMethod::System(
                        SystemMethod::from_str(method_name).map_err(|_| not_found)?,
                    ),
                    RENodeId::Vault(_) => NativeMethod::Vault(
                        VaultMethod::from_str(method_name).map_err(|_| not_found)?,
                    ),
                    RENodeId::ResourceManager(_) => NativeMethod::ResourceManager(
                        ResourceManagerMethod::from_str(method_name).map_err(|_| not_found)?,
                    ),
                    RENodeId::Global(_)
                    | RENodeId::KeyValueStore(_)
                    | RENodeId::NonFungibleStore(_)
                    | RENodeId::Package(_) => return Err(not_found),
                };
                REActor::Method(ResolvedMethod::Native(native_method), resolved_receiver)
            }
        };

        Ok((actor, references_to_add, nodes_to_move))
    }

    fn verify_valid_mode_transition(
        cur: &ExecutionMode,
        next: &ExecutionMode,
    ) -> Result<(), RuntimeError> {
        match (cur, next) {
            (ExecutionMode::Kernel, ..) => Ok(()),
            (ExecutionMode::ScryptoInterpreter, ExecutionMode::Application) => Ok(()),
            _ => Err(RuntimeError::KernelError(
                KernelError::InvalidModeTransition(*cur, *next),
            )),
        }
    }
}

impl<'g, 's, W, I, R> SystemApi<'s, R> for Kernel<'g, 's, W, I, R>
where
    W: WasmEngine<I>,
    I: WasmInstance,
    R: FeeReserve,
{
    fn execute_in_mode<X, RTN, E>(
        &mut self,
        execution_mode: ExecutionMode,
        execute: X,
    ) -> Result<RTN, RuntimeError>
    where
        RuntimeError: From<E>,
        X: FnOnce(&mut Self) -> Result<RTN, E>,
    {
        Self::verify_valid_mode_transition(&self.execution_mode, &execution_mode)?;

        // Save and replace kernel actor
        let saved = self.execution_mode;
        self.execution_mode = execution_mode;

        let rtn = execute(self)?;

        // Restore old kernel actor
        self.execution_mode = saved;

        Ok(rtn)
    }

    fn consume_cost_units(&mut self, units: u32) -> Result<(), RuntimeError> {
        for m in &mut self.modules {
            m.on_wasm_costing(&mut self.track, &mut self.call_frames, units)
                .map_err(RuntimeError::ModuleError)?;
        }

        Ok(())
    }

    fn lock_fee(
        &mut self,
        vault_id: VaultId,
        mut fee: Resource,
        contingent: bool,
    ) -> Result<Resource, RuntimeError> {
        for m in &mut self.modules {
            fee = m
                .on_lock_fee(
                    &mut self.track,
                    &mut self.call_frames,
                    vault_id,
                    fee,
                    contingent,
                )
                .map_err(RuntimeError::ModuleError)?;
        }

        Ok(fee)
    }

    fn get_actor(&self) -> &REActor {
        &Self::current_frame(&self.call_frames).actor
    }

    fn invoke_scrypto(
        &mut self,
        fn_ident: ScryptoFnIdent,
        args: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        let depth = Self::current_frame(&self.call_frames).depth;

        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::InvokeScrypto {
                    fn_ident: &fn_ident,
                    args: &args,
                    depth,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // check call depth
        if depth == self.max_depth {
            return Err(RuntimeError::KernelError(
                KernelError::MaxCallDepthLimitReached,
            ));
        }

        let mut nodes_to_pass_downstream = HashMap::new();
        let mut next_node_refs = HashMap::new();

        // Internal state update to taken values
        for node_id in args.node_ids() {
            let node = Self::current_frame_mut(&mut self.call_frames).take_node(node_id)?;
            nodes_to_pass_downstream.insert(node_id, node);
        }

        // Move this into higher layer, e.g. transaction processor
        if Self::current_frame(&self.call_frames).depth == 0 {
            let mut static_refs = HashSet::new();
            static_refs.insert(GlobalAddress::Resource(RADIX_TOKEN));
            static_refs.insert(GlobalAddress::Resource(SYSTEM_TOKEN));
            static_refs.insert(GlobalAddress::Resource(ECDSA_SECP256K1_TOKEN));
            static_refs.insert(GlobalAddress::Component(SYS_SYSTEM_COMPONENT));
            static_refs.insert(GlobalAddress::Package(ACCOUNT_PACKAGE));
            static_refs.insert(GlobalAddress::Package(SYS_FAUCET_PACKAGE));

            // Make refs visible
            let mut global_references = args.global_references();
            global_references.extend(static_refs.clone());

            // TODO: This can be refactored out once any type in sbor is implemented
            let maybe_txn: Result<TransactionProcessorRunInput, DecodeError> =
                scrypto_decode(&args.raw);
            if let Ok(input) = maybe_txn {
                for instruction in &input.instructions {
                    match instruction {
                        Instruction::CallFunction { args, .. }
                        | Instruction::CallMethod { args, .. } => {
                            let scrypto_value =
                                ScryptoValue::from_slice(&args).expect("Invalid CALL arguments");
                            global_references.extend(scrypto_value.global_references());
                        }
                        _ => {}
                    }
                }
            }

            // Check for existence
            for global_address in global_references {
                let node_id = RENodeId::Global(global_address);
                let offset = SubstateOffset::Global(GlobalOffset::Global);
                let node_pointer = RENodePointer::Store(node_id);

                // TODO: static check here is to support the current genesis transaction which
                // TODO: requires references to dynamically created resources. Can remove
                // TODO: when this is resolved.
                if !static_refs.contains(&global_address) {
                    node_pointer
                        .acquire_lock(offset.clone(), LockFlags::read_only(), &mut self.track)
                        .map_err(|e| match e {
                            KernelError::TrackError(TrackError::NotFound(..)) => {
                                RuntimeError::KernelError(KernelError::GlobalAddressNotFound(
                                    global_address,
                                ))
                            }
                            _ => RuntimeError::KernelError(e),
                        })?;
                    node_pointer
                        .release_lock(offset, false, &mut self.track)
                        .map_err(RuntimeError::KernelError)?;
                }

                Self::current_frame_mut(&mut self.call_frames)
                    .node_refs
                    .insert(node_id, node_pointer);
                next_node_refs.insert(node_id, node_pointer);
            }
        } else {
            // Check that global references are owned by this call frame
            let mut global_references = args.global_references();
            global_references.insert(GlobalAddress::Resource(RADIX_TOKEN));
            global_references.insert(GlobalAddress::Component(SYS_SYSTEM_COMPONENT));
            for global_address in global_references {
                let node_id = RENodeId::Global(global_address);

                // As of now, once a component is made visible to the frame, client can directly
                // read the substates of the component. This will cause "Substate was never locked" issue.
                // We use the following temporary solution to work around this.
                // A better solution is to create node representation before issuing any reference.
                // TODO: remove
                if let Some(pointer) = Self::current_frame_mut(&mut self.call_frames)
                    .node_refs
                    .get(&node_id)
                {
                    next_node_refs.insert(node_id.clone(), pointer.clone());
                } else {
                    return Err(RuntimeError::KernelError(
                        KernelError::InvalidReferencePass(global_address),
                    ));
                }
            }
        }

        // Change to kernel mode
        let saved_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        let (next_actor, references_to_add) = self.resolve_scrypto_actor(&fn_ident, &args)?;
        next_node_refs.extend(references_to_add);

        let cur_actor = &Self::current_frame(&self.call_frames).actor;

        for (node_id, node) in &mut nodes_to_pass_downstream {
            let root_node = node.root_mut();
            root_node.prepare_move_downstream(*node_id, cur_actor, &next_actor)?;
        }

        let (output, received_values) =
            self.run(next_actor, args, nodes_to_pass_downstream, next_node_refs)?;

        // move re nodes to this process.
        for (id, node) in received_values {
            Self::current_frame_mut(&mut self.call_frames).insert_owned_node(id, node);
        }

        // Accept global references
        for global_address in output.global_references() {
            let node_id = RENodeId::Global(global_address);
            Self::current_frame_mut(&mut self.call_frames)
                .node_refs
                .insert(node_id, RENodePointer::Store(node_id));
        }

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::InvokeScrypto { output: &output },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // TODO: Move this into higher layer, e.g. transaction processor
        if Self::current_frame(&self.call_frames).depth == 0 {
            self.call_frames.pop().unwrap().drop_frame()?;
        }

        // Restore previous mode
        self.execution_mode = saved_mode;

        Ok(output)
    }

    fn invoke_native(
        &mut self,
        fn_ident: NativeFnIdent,
        args: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        let depth = Self::current_frame(&self.call_frames).depth;

        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::InvokeNative {
                    fn_ident: &fn_ident,
                    args: &args,
                    depth,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // check call depth
        if depth == self.max_depth {
            return Err(RuntimeError::KernelError(
                KernelError::MaxCallDepthLimitReached,
            ));
        }

        let mut nodes_to_pass_downstream = HashMap::new();
        let mut next_node_refs = HashMap::new();

        // Internal state update to taken values
        for node_id in args.node_ids() {
            let node = Self::current_frame_mut(&mut self.call_frames).take_node(node_id)?;
            nodes_to_pass_downstream.insert(node_id, node);
        }

        // Move this into higher layer, e.g. transaction processor
        if Self::current_frame(&self.call_frames).depth == 0 {
            let mut static_refs = HashSet::new();
            static_refs.insert(GlobalAddress::Resource(RADIX_TOKEN));
            static_refs.insert(GlobalAddress::Resource(SYSTEM_TOKEN));
            static_refs.insert(GlobalAddress::Resource(ECDSA_SECP256K1_TOKEN));
            static_refs.insert(GlobalAddress::Component(SYS_SYSTEM_COMPONENT));
            static_refs.insert(GlobalAddress::Package(ACCOUNT_PACKAGE));
            static_refs.insert(GlobalAddress::Package(SYS_FAUCET_PACKAGE));

            // Make refs visible
            let mut global_references = args.global_references();
            global_references.extend(static_refs.clone());

            // TODO: This can be refactored out once any type in sbor is implemented
            let maybe_txn: Result<TransactionProcessorRunInput, DecodeError> =
                scrypto_decode(&args.raw);
            if let Ok(input) = maybe_txn {
                for instruction in &input.instructions {
                    match instruction {
                        Instruction::CallFunction { args, .. }
                        | Instruction::CallMethod { args, .. } => {
                            let scrypto_value =
                                ScryptoValue::from_slice(&args).expect("Invalid CALL arguments");
                            global_references.extend(scrypto_value.global_references());
                        }
                        _ => {}
                    }
                }
            }

            // Check for existence
            for global_address in global_references {
                let node_id = RENodeId::Global(global_address);
                let offset = SubstateOffset::Global(GlobalOffset::Global);
                let node_pointer = RENodePointer::Store(node_id);

                // TODO: static check here is to support the current genesis transaction which
                // TODO: requires references to dynamically created resources. Can remove
                // TODO: when this is resolved.
                if !static_refs.contains(&global_address) {
                    node_pointer
                        .acquire_lock(offset.clone(), LockFlags::read_only(), &mut self.track)
                        .map_err(|e| match e {
                            KernelError::TrackError(TrackError::NotFound(..)) => {
                                RuntimeError::KernelError(KernelError::GlobalAddressNotFound(
                                    global_address,
                                ))
                            }
                            _ => RuntimeError::KernelError(e),
                        })?;
                    node_pointer
                        .release_lock(offset, false, &mut self.track)
                        .map_err(RuntimeError::KernelError)?;
                }

                Self::current_frame_mut(&mut self.call_frames)
                    .node_refs
                    .insert(node_id, node_pointer);
                next_node_refs.insert(node_id, node_pointer);
            }
        } else {
            // Check that global references are owned by this call frame
            let mut global_references = args.global_references();
            global_references.insert(GlobalAddress::Resource(RADIX_TOKEN));
            global_references.insert(GlobalAddress::Component(SYS_SYSTEM_COMPONENT));
            for global_address in global_references {
                let node_id = RENodeId::Global(global_address);

                // As of now, once a component is made visible to the frame, client can directly
                // read the substates of the component. This will cause "Substate was never locked" issue.
                // We use the following temporary solution to work around this.
                // A better solution is to create node representation before issuing any reference.
                // TODO: remove
                if let Some(pointer) = Self::current_frame_mut(&mut self.call_frames)
                    .node_refs
                    .get(&node_id)
                {
                    next_node_refs.insert(node_id.clone(), pointer.clone());
                } else {
                    return Err(RuntimeError::KernelError(
                        KernelError::InvalidReferencePass(global_address),
                    ));
                }
            }
        }

        // Change to kernel mode
        let saved_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        let (next_actor, references_to_add, nodes_to_move) =
            self.resolve_native_actor(&fn_ident, &args)?;
        next_node_refs.extend(references_to_add);
        nodes_to_pass_downstream.extend(nodes_to_move);

        let cur_actor = &Self::current_frame(&self.call_frames).actor;

        for (node_id, node) in &mut nodes_to_pass_downstream {
            let root_node = node.root_mut();
            root_node.prepare_move_downstream(*node_id, cur_actor, &next_actor)?;
        }

        let (output, received_values) =
            self.run(next_actor, args, nodes_to_pass_downstream, next_node_refs)?;

        // move re nodes to this process.
        for (id, node) in received_values {
            Self::current_frame_mut(&mut self.call_frames).insert_owned_node(id, node);
        }

        // Accept global references
        for global_address in output.global_references() {
            let node_id = RENodeId::Global(global_address);
            Self::current_frame_mut(&mut self.call_frames)
                .node_refs
                .insert(node_id, RENodePointer::Store(node_id));
        }

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::InvokeNative { output: &output },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // TODO: Move this into higher layer, e.g. transaction processor
        if Self::current_frame(&self.call_frames).depth == 0 {
            self.call_frames.pop().unwrap().drop_frame()?;
        }

        // Restore previous mode
        self.execution_mode = saved_mode;

        Ok(output)
    }

    fn get_visible_node_ids(&mut self) -> Result<Vec<RENodeId>, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::ReadOwnedNodes,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let node_ids = Self::current_frame_mut(&mut self.call_frames).get_visible_nodes();

        Ok(node_ids)
    }

    fn node_drop(&mut self, node_id: RENodeId) -> Result<HeapRootRENode, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::DropNode { node_id: &node_id },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // TODO: Authorization

        let node = Self::current_frame_mut(&mut self.call_frames).take_node(node_id)?;

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::DropNode { node: &node },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(node)
    }

    fn node_create(&mut self, mut re_node: HeapRENode) -> Result<RENodeId, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::CreateNode { node: &re_node },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // TODO: Authorization

        // Take any required child nodes
        let mut taken_root_nodes = HashMap::new();
        for offset in re_node.get_substates() {
            let substate = re_node.borrow_substate(&offset)?;
            let (_, owned) = substate.references_and_owned_nodes();
            for child_id in owned {
                SubstateProperties::verify_can_own(&offset, child_id)?;
                let node = Self::current_frame_mut(&mut self.call_frames).take_node(child_id)?;
                taken_root_nodes.insert(child_id, node);
            }
        }

        let mut child_nodes = HashMap::new();
        for (id, taken_root_node) in taken_root_nodes {
            child_nodes.extend(taken_root_node.to_nodes(id));
        }

        // Insert node into heap
        let node_id = Self::new_node_id(&mut self.id_allocator, self.transaction_hash, &re_node)
            .map_err(|e| RuntimeError::KernelError(KernelError::IdAllocationError(e)))?;
        let heap_root_node = HeapRootRENode {
            root: re_node,
            child_nodes,
        };
        Self::current_frame_mut(&mut self.call_frames).insert_owned_node(node_id, heap_root_node);

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::CreateNode { node_id: &node_id },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(node_id)
    }

    fn node_globalize(&mut self, node_id: RENodeId) -> Result<GlobalAddress, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::GlobalizeNode { node_id: &node_id },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // TODO: Authorization

        let node = Self::current_frame_mut(&mut self.call_frames).take_node(node_id)?;

        let (global_address, global_substate) = Self::globalize(
            &mut self.id_allocator,
            self.transaction_hash,
            node_id,
            &node.root,
        )?;
        self.track.new_global_addresses.push(global_address);
        self.track.insert_substate(
            SubstateId(
                RENodeId::Global(global_address),
                SubstateOffset::Global(GlobalOffset::Global),
            ),
            RuntimeSubstate::GlobalRENode(global_substate),
        );
        for (id, substate) in nodes_to_substates(node.to_nodes(node_id)) {
            self.track.insert_substate(id, substate);
        }

        Self::current_frame_mut(&mut self.call_frames)
            .node_refs
            .insert(
                RENodeId::Global(global_address),
                RENodePointer::Store(RENodeId::Global(global_address)),
            );

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::GlobalizeNode,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(global_address)
    }

    fn lock_substate(
        &mut self,
        mut node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::LockSubstate {
                    node_id: &node_id,
                    offset: &offset,
                    flags: &flags,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        // Deref
        if let Some(derefed) = self.node_offset_deref(node_id, &offset)? {
            node_id = derefed;
        }

        let node_pointer = Self::current_frame(&self.call_frames).get_node_pointer(node_id)?;

        // TODO: Check if valid offset for node_id

        // Authorization
        let actor = &Self::current_frame(&self.call_frames).actor;
        if !SubstateProperties::check_substate_access(
            current_mode,
            actor,
            node_id,
            offset.clone(),
            flags,
        ) {
            return Err(RuntimeError::KernelError(
                KernelError::InvalidSubstateLock {
                    mode: current_mode,
                    actor: actor.clone(),
                    node_id,
                    offset,
                    flags,
                },
            ));
        }

        if !(matches!(offset, SubstateOffset::KeyValueStore(..))
            || matches!(
                offset,
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..))
            ))
        {
            node_pointer
                .acquire_lock(offset.clone(), flags, &mut self.track)
                .map_err(RuntimeError::KernelError)?;
        }

        let lock_handle = Self::current_frame_mut(&mut self.call_frames).create_lock(
            node_pointer,
            offset.clone(),
            flags,
        );

        // Restore current mode
        self.execution_mode = current_mode;

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::LockSubstate { lock_handle },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(lock_handle)
    }

    fn drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::DropLock {
                    lock_handle: &lock_handle,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let (node_pointer, offset, flags) = Self::current_frame_mut(&mut self.call_frames)
            .drop_lock(lock_handle)
            .map_err(RuntimeError::KernelError)?;

        if !(matches!(offset, SubstateOffset::KeyValueStore(..))
            || matches!(
                offset,
                SubstateOffset::NonFungibleStore(NonFungibleStoreOffset::Entry(..))
            ))
        {
            node_pointer
                .release_lock(
                    offset.clone(),
                    flags.contains(LockFlags::UNMODIFIED_BASE),
                    self.track,
                )
                .map_err(RuntimeError::KernelError)?;
        }

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::DropLock,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(())
    }

    fn get_ref(&mut self, lock_handle: LockHandle) -> Result<SubstateRef, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::GetRef {
                    lock_handle: &lock_handle,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let SubstateLock {
            substate_pointer: (node_pointer, offset),
            ..
        } = Self::current_frame_mut(&mut self.call_frames)
            .get_lock(lock_handle)
            .map_err(RuntimeError::KernelError)?
            .clone();

        let (global_references, children) = {
            let substate_ref =
                node_pointer.borrow_substate(&offset, &mut self.call_frames, &mut self.track)?;
            substate_ref.references_and_owned_nodes()
        };

        // Expand references
        {
            let cur_frame = Self::current_frame_mut(&mut self.call_frames);
            // TODO: Figure out how to drop these references as well on reference drop
            for global_address in global_references {
                let node_id = RENodeId::Global(global_address);
                cur_frame
                    .node_refs
                    .insert(node_id, RENodePointer::Store(node_id));
            }
            for child_id in children {
                let child_pointer = node_pointer.child(child_id);
                cur_frame.node_refs.insert(child_id, child_pointer);
                cur_frame
                    .add_lock_visible_node(lock_handle, child_id)
                    .map_err(RuntimeError::KernelError)?;
            }
        }

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::GetRef {
                    node_pointer: &node_pointer,
                    offset: &offset,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        node_pointer.borrow_substate(&offset, &mut self.call_frames, &mut self.track)
    }

    fn get_ref_mut<'f>(
        &'f mut self,
        lock_handle: LockHandle,
    ) -> Result<SubstateRefMut<'f, 's, R>, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::GetRefMut {
                    lock_handle: &lock_handle,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let SubstateLock {
            substate_pointer: (node_pointer, offset),
            flags,
            ..
        } = Self::current_frame_mut(&mut self.call_frames)
            .get_lock(lock_handle)
            .map_err(RuntimeError::KernelError)?
            .clone();

        if !flags.contains(LockFlags::MUTABLE) {
            return Err(RuntimeError::KernelError(KernelError::LockNotMutable(
                lock_handle,
            )));
        }

        let (global_references, children) = {
            let substate_ref =
                node_pointer.borrow_substate(&offset, &mut self.call_frames, &mut self.track)?;
            substate_ref.references_and_owned_nodes()
        };

        // Expand references
        {
            let cur_frame = Self::current_frame_mut(&mut self.call_frames);
            // TODO: Figure out how to drop these references as well on reference drop
            for global_address in global_references {
                let node_id = RENodeId::Global(global_address);
                cur_frame
                    .node_refs
                    .insert(node_id, RENodePointer::Store(node_id));
            }
            for child_id in &children {
                let child_pointer = node_pointer.child(*child_id);
                cur_frame.node_refs.insert(*child_id, child_pointer);
                cur_frame
                    .add_lock_visible_node(lock_handle, *child_id)
                    .map_err(RuntimeError::KernelError)?;
            }
        }

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::GetRefMut,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        SubstateRefMut::new(
            lock_handle,
            node_pointer,
            offset,
            children,
            &mut self.call_frames,
            &mut self.track,
        )
    }

    fn read_transaction_hash(&mut self) -> Result<Hash, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::ReadTransactionHash,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::ReadTransactionHash {
                    hash: &self.transaction_hash,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(self.transaction_hash)
    }

    fn read_blob(&mut self, blob_hash: &Hash) -> Result<&[u8], RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::ReadBlob { blob_hash },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let blob = self
            .blobs
            .get(blob_hash)
            .ok_or(KernelError::BlobNotFound(blob_hash.clone()))
            .map_err(RuntimeError::KernelError)?;

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::ReadBlob { blob: &blob },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(blob)
    }

    fn generate_uuid(&mut self) -> Result<u128, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::GenerateUuid,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let uuid = Self::new_uuid(&mut self.id_allocator, self.transaction_hash)
            .map_err(|e| RuntimeError::KernelError(KernelError::IdAllocationError(e)))?;

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::GenerateUuid { uuid },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(uuid)
    }

    fn emit_log(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::EmitLog {
                    level: &level,
                    message: &message,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        self.track.add_log(level, message);

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::EmitLog,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(())
    }
}
