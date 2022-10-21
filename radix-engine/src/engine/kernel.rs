use std::mem;
use transaction::errors::IdAllocationError;
use transaction::model::{AuthZoneParams, Instruction};
use transaction::validation::*;

use crate::engine::call_frame::RENodeLocation;
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
            println!("{}[{:5}] {}", "  ".repeat($self.current_frame.depth) , $level, sbor::rust::format!($msg, $( $arg ),*));
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
    /// Current execution mode, specifies permissions into state/invocations
    execution_mode: ExecutionMode,

    /// The transaction hash
    transaction_hash: Hash,
    /// Blobs attached to the transaction
    blobs: &'g HashMap<Hash, Vec<u8>>,
    /// ID allocator
    id_allocator: IdAllocator,

    /// Stack
    current_frame: CallFrame,
    // This stack could potentially be removed and just use the native stack
    // but keeping this call_frames stack may potentially prove useful if implementing
    // execution pause and/or for better debuggability
    prev_frame_stack: Vec<CallFrame>,
    /// Heap
    heap: Heap,
    /// Store
    track: &'g mut Track<'s, R>,

    /// Interpreter capable of running scrypto programs
    scrypto_interpreter: &'g mut ScryptoInterpreter<I, W>,

    /// Kernel modules
    modules: Vec<Box<dyn Module<R>>>,
    /// The max call depth, TODO: Move into costing module
    max_depth: usize,
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
        let mut kernel = Self {
            execution_mode: ExecutionMode::Kernel,
            transaction_hash,
            blobs,
            max_depth,
            heap: Heap::new(),
            track,
            scrypto_interpreter,
            id_allocator: IdAllocator::new(IdSpace::Application),
            current_frame: CallFrame::new_root(),
            prev_frame_stack: vec![],
            modules,
        };

        // Initial authzone
        // TODO: Move into module initialization
        kernel
            .execute_in_mode::<_, _, RuntimeError>(ExecutionMode::AuthModule, |system_api| {
                let auth_zone = AuthZoneStackSubstate::new(
                    vec![],
                    auth_zone_params.virtualizable_proofs_resource_addresses,
                    auth_zone_params.initial_proofs.into_iter().collect(),
                );

                system_api.create_node(RENode::AuthZone(auth_zone))?;

                Ok(())
            })
            .expect("AuthModule failed to initialize");

        kernel
    }

    fn new_uuid(
        id_allocator: &mut IdAllocator,
        transaction_hash: Hash,
    ) -> Result<u128, IdAllocationError> {
        id_allocator.new_uuid(transaction_hash)
    }

    // TODO: Move this into a native function
    fn create_global_node(
        &mut self,
        node_id: RENodeId,
    ) -> Result<(GlobalAddress, GlobalAddressSubstate), RuntimeError> {
        self.execute_in_mode(ExecutionMode::Globalize, |system_api| match node_id {
            RENodeId::Component(component_id) => {
                let transaction_hash = system_api.transaction_hash;
                let handle = system_api.lock_substate(
                    node_id,
                    SubstateOffset::Component(ComponentOffset::Info),
                    LockFlags::read_only(),
                )?;
                let substate_ref = system_api.get_ref(handle)?;
                let info = substate_ref.component_info();
                let (package_address, blueprint_name) =
                    (info.package_address, info.blueprint_name.clone());
                system_api.drop_lock(handle)?;

                let component_address = system_api
                    .id_allocator
                    .new_component_address(transaction_hash, package_address, &blueprint_name)
                    .map_err(|e| RuntimeError::KernelError(KernelError::IdAllocationError(e)))?;

                Ok((
                    GlobalAddress::Component(component_address),
                    GlobalAddressSubstate::Component(scrypto::component::Component(component_id)),
                ))
            }
            RENodeId::System(component_id) => {
                let transaction_hash = system_api.transaction_hash;

                let component_address = system_api
                    .id_allocator
                    .new_system_component_address(transaction_hash)
                    .map_err(|e| RuntimeError::KernelError(KernelError::IdAllocationError(e)))?;

                Ok((
                    GlobalAddress::Component(component_address),
                    GlobalAddressSubstate::SystemComponent(scrypto::component::Component(
                        component_id,
                    )),
                ))
            }
            RENodeId::ResourceManager(resource_id) => {
                let transaction_hash = system_api.transaction_hash;
                let resource_address = system_api
                    .id_allocator
                    .new_resource_address(transaction_hash)
                    .map_err(|e| RuntimeError::KernelError(KernelError::IdAllocationError(e)))?;

                Ok((
                    GlobalAddress::Resource(resource_address),
                    GlobalAddressSubstate::Resource(resource_id),
                ))
            }
            RENodeId::Package(package_id) => {
                let transaction_hash = system_api.transaction_hash;
                let package_address = system_api
                    .id_allocator
                    .new_package_address(transaction_hash)
                    .map_err(|e| RuntimeError::KernelError(KernelError::IdAllocationError(e)))?;

                Ok((
                    GlobalAddress::Package(package_address),
                    GlobalAddressSubstate::Package(package_id),
                ))
            }
            _ => Err(RuntimeError::KernelError(
                KernelError::RENodeGlobalizeTypeNotAllowed(node_id),
            )),
        })
    }

    fn new_node_id(
        id_allocator: &mut IdAllocator,
        transaction_hash: Hash,
        re_node: &RENode,
    ) -> Result<RENodeId, IdAllocationError> {
        match re_node {
            RENode::Global(..) => panic!("Should not get here"),
            RENode::AuthZone(..) => {
                let auth_zone_id = id_allocator.new_auth_zone_id()?;
                Ok(RENodeId::AuthZoneStack(auth_zone_id))
            }
            RENode::Bucket(..) => {
                let bucket_id = id_allocator.new_bucket_id()?;
                Ok(RENodeId::Bucket(bucket_id))
            }
            RENode::Proof(..) => {
                let proof_id = id_allocator.new_proof_id()?;
                Ok(RENodeId::Proof(proof_id))
            }
            RENode::Worktop(..) => Ok(RENodeId::Worktop),
            RENode::Vault(..) => {
                let vault_id = id_allocator.new_vault_id(transaction_hash)?;
                Ok(RENodeId::Vault(vault_id))
            }
            RENode::KeyValueStore(..) => {
                let kv_store_id = id_allocator.new_kv_store_id(transaction_hash)?;
                Ok(RENodeId::KeyValueStore(kv_store_id))
            }
            RENode::NonFungibleStore(..) => {
                let nf_store_id = id_allocator.new_nf_store_id(transaction_hash)?;
                Ok(RENodeId::NonFungibleStore(nf_store_id))
            }
            RENode::Package(..) => {
                // Security Alert: ensure ID allocating will practically never fail
                let package_id = id_allocator.new_package_id(transaction_hash)?;
                Ok(RENodeId::Package(package_id))
            }
            RENode::ResourceManager(..) => {
                let resource_manager_id = id_allocator.new_resource_manager_id(transaction_hash)?;
                Ok(RENodeId::ResourceManager(resource_manager_id))
            }
            RENode::Component(..) => {
                let component_id = id_allocator.new_component_id(transaction_hash)?;
                Ok(RENodeId::Component(component_id))
            }
            RENode::System(..) => {
                let component_id = id_allocator.new_component_id(transaction_hash)?;
                Ok(RENodeId::System(component_id))
            }
        }
    }

    pub fn prepare_move_downstream(
        &mut self,
        node_id: RENodeId,
        to: &REActor,
    ) -> Result<(), RuntimeError> {
        self.execute_in_mode(ExecutionMode::MoveDownstream, |system_api| {
            match node_id {
                RENodeId::Bucket(..) => {
                    let handle = system_api.lock_substate(
                        node_id,
                        SubstateOffset::Bucket(BucketOffset::Bucket),
                        LockFlags::read_only(),
                    )?;
                    let substate_ref = system_api.get_ref(handle)?;
                    let bucket = substate_ref.bucket();
                    let locked = bucket.is_locked();
                    system_api.drop_lock(handle)?;
                    if locked {
                        Err(RuntimeError::KernelError(KernelError::CantMoveDownstream(
                            node_id,
                        )))
                    } else {
                        Ok(())
                    }
                }
                RENodeId::Proof(..) => {
                    // TODO: Remove Proof consuming method
                    if let REActor::Method(ResolvedMethod::Native(NativeMethod::Proof(..)), ..) = to
                    {
                        return Ok(());
                    }

                    let from = system_api.get_actor();

                    if from.is_scrypto_or_transaction() || to.is_scrypto_or_transaction() {
                        let handle = system_api.lock_substate(
                            node_id,
                            SubstateOffset::Proof(ProofOffset::Proof),
                            LockFlags::MUTABLE,
                        )?;
                        let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
                        let mut raw_mut = substate_ref_mut.get_raw_mut();
                        let proof = raw_mut.proof();

                        let rtn = if proof.is_restricted() {
                            Err(RuntimeError::KernelError(KernelError::CantMoveDownstream(
                                node_id,
                            )))
                        } else {
                            proof.change_to_restricted();
                            Ok(())
                        };

                        substate_ref_mut.flush()?;
                        system_api.drop_lock(handle)?;

                        rtn
                    } else {
                        Ok(())
                    }
                }
                RENodeId::Component(..) => Ok(()),
                RENodeId::AuthZoneStack(..)
                | RENodeId::ResourceManager(..)
                | RENodeId::KeyValueStore(..)
                | RENodeId::NonFungibleStore(..)
                | RENodeId::Vault(..)
                | RENodeId::Package(..)
                | RENodeId::Worktop
                | RENodeId::System(..)
                | RENodeId::Global(..) => Err(RuntimeError::KernelError(
                    KernelError::CantMoveDownstream(node_id),
                )),
            }
        })
    }

    pub fn prepare_move_upstream(&mut self, node_id: RENodeId) -> Result<(), RuntimeError> {
        self.execute_in_mode(ExecutionMode::MoveDownstream, |system_api| match node_id {
            RENodeId::Bucket(..) => {
                let handle = system_api.lock_substate(
                    node_id,
                    SubstateOffset::Bucket(BucketOffset::Bucket),
                    LockFlags::read_only(),
                )?;
                let substate_ref = system_api.get_ref(handle)?;
                let bucket = substate_ref.bucket();
                let locked = bucket.is_locked();
                system_api.drop_lock(handle)?;
                if locked {
                    Err(RuntimeError::KernelError(KernelError::CantMoveUpstream(
                        node_id,
                    )))
                } else {
                    Ok(())
                }
            }
            RENodeId::Proof(..) | RENodeId::Component(..) | RENodeId::Vault(..) => Ok(()),

            RENodeId::AuthZoneStack(..)
            | RENodeId::ResourceManager(..)
            | RENodeId::KeyValueStore(..)
            | RENodeId::NonFungibleStore(..)
            | RENodeId::Package(..)
            | RENodeId::Worktop
            | RENodeId::System(..)
            | RENodeId::Global(..) => Err(RuntimeError::KernelError(
                KernelError::CantMoveUpstream(node_id),
            )),
        })
    }

    fn drop_nodes_in_frame(&mut self) -> Result<(), RuntimeError> {
        let mut worktops = Vec::new();
        let owned_nodes = self.current_frame.owned_nodes();
        for node_id in owned_nodes {
            if let RENodeId::Worktop = node_id {
                worktops.push(node_id);
            } else {
                self.drop_node(node_id)?;
            }
        }
        for worktop_id in worktops {
            self.drop_node(worktop_id)?;
        }

        Ok(())
    }

    fn run(
        &mut self,
        actor: REActor,
        input: ScryptoValue,
        nodes_to_pass: Vec<RENodeId>,
        mut refed_nodes: HashMap<RENodeId, RENodeLocation>,
    ) -> Result<ScryptoValue, RuntimeError> {
        let new_refed_nodes = self.execute_in_mode(ExecutionMode::AuthModule, |system_api| {
            AuthModule::on_before_frame_start(&actor, &input, system_api).map_err(|e| match e {
                InvokeError::Error(e) => RuntimeError::ModuleError(e.into()),
                InvokeError::Downstream(runtime_error) => runtime_error,
            })
        })?;

        // TODO: Do this in a better way by allowing module to execute in next call frame
        for new_refed_node in new_refed_nodes {
            let node_pointer = self
                .current_frame
                .get_node_location(new_refed_node)
                .unwrap();
            refed_nodes.insert(new_refed_node, node_pointer);
        }

        for node_id in &nodes_to_pass {
            self.prepare_move_downstream(*node_id, &actor)?;
        }

        for m in &mut self.modules {
            m.on_run(
                &actor,
                &input,
                &mut self.current_frame,
                &mut self.heap,
                &mut self.track,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // Call Frame Push
        let frame = CallFrame::new_child_from_parent(
            &mut self.heap,
            &mut self.current_frame,
            actor.clone(), // TODO: Remove clone
            nodes_to_pass,
            refed_nodes,
        )?;

        let parent = mem::replace(&mut self.current_frame, frame);
        self.prev_frame_stack.push(parent);

        // Execute
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
            REActor::Function(ResolvedFunction::Scrypto { code, .. })
            | REActor::Method(ResolvedMethod::Scrypto { code, .. }, ..) => {
                let mut executor = self.scrypto_interpreter.create_executor(&code);

                self.execute_in_mode(ExecutionMode::ScryptoInterpreter, |system_api| {
                    executor.run(input, system_api)
                })
            }
        }?;

        // Process return data
        let mut parent = self.prev_frame_stack.pop().unwrap();

        let nodes_to_return = output.node_ids();
        for node_id in &nodes_to_return {
            self.prepare_move_upstream(*node_id)?;
        }

        CallFrame::move_nodes_upstream(
            &mut self.heap,
            &mut self.current_frame,
            &mut parent,
            nodes_to_return,
        )?;
        CallFrame::copy_refs(
            &mut self.current_frame,
            &mut parent,
            output.global_references(),
        )?;

        // Auto drop locks
        self.current_frame.drop_all_locks(&mut self.track)?;

        self.execute_in_mode(ExecutionMode::AuthModule, |system_api| {
            AuthModule::on_frame_end(system_api).map_err(|e| match e {
                InvokeError::Error(e) => RuntimeError::ModuleError(e.into()),
                InvokeError::Downstream(runtime_error) => runtime_error,
            })
        })?;

        // Auto-drop locks again in case module forgot to drop
        self.current_frame.drop_all_locks(&mut self.track)?;

        // drop proofs and check resource leak
        self.execution_mode = ExecutionMode::Application;
        self.drop_nodes_in_frame()?;
        self.execution_mode = ExecutionMode::Kernel;

        // Restore previous frame
        self.current_frame = parent;

        Ok(output)
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
        invocation: &ScryptoInvocation,
    ) -> Result<
        (
            REActor,
            HashMap<RENodeId, RENodeLocation>,
            HashSet<RENodeId>,
        ),
        RuntimeError,
    > {
        let mut additional_ref_copy = HashMap::new();

        let actor = match invocation {
            ScryptoInvocation::Function(function_ident, args) => {
                // Load the package substate
                // TODO: Move this in a better spot when more refactors are done
                let package_address = match function_ident.package {
                    ScryptoPackage::Global(address) => address,
                };
                let global_node_id = RENodeId::Global(GlobalAddress::Package(package_address));
                let package_node_id = self.node_method_deref(global_node_id)?.unwrap();
                let package = self.execute_in_mode::<_, _, RuntimeError>(
                    ExecutionMode::ScryptoInterpreter,
                    |system_api| {
                        let handle = system_api.lock_substate(
                            package_node_id,
                            SubstateOffset::Package(PackageOffset::Package),
                            LockFlags::read_only(),
                        )?;
                        let substate_ref = system_api.get_ref(handle)?;
                        let package = substate_ref.package().clone(); // TODO: Remove clone()
                        system_api.drop_lock(handle)?;

                        Ok(package)
                    },
                )?;

                // Pass the package ref
                // TODO: remove? currently needed for `Runtime::package_address()` API.
                additional_ref_copy.insert(
                    global_node_id,
                    self.current_frame
                        .get_node_location(global_node_id)
                        .unwrap(),
                );
                additional_ref_copy.insert(
                    package_node_id,
                    self.current_frame
                        .get_node_location(package_node_id)
                        .unwrap(),
                );

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
                if !fn_abi.input.matches(&args.dom) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFunctionInvocation(
                            function_ident.clone(),
                            ScryptoFnResolvingError::InvalidInput,
                        ),
                    ));
                }

                // Emit event
                for m in &mut self.modules {
                    m.on_wasm_instantiation(
                        &self.current_frame,
                        &mut self.heap,
                        &mut self.track,
                        package.code(),
                    )
                    .map_err(RuntimeError::ModuleError)?;
                }

                REActor::Function(ResolvedFunction::Scrypto {
                    package_address: package_address,
                    package_id: package_node_id.into(),
                    blueprint_name: function_ident.blueprint_name.clone(),
                    ident: function_ident.function_name.clone(),
                    export_name: fn_abi.export_name.clone(),
                    return_type: fn_abi.output.clone(),
                    code: package.code,
                })
            }
            ScryptoInvocation::Method(method_ident, args) => {
                let original_node_id = match method_ident.receiver {
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

                // Load the package substate
                // TODO: Move this in a better spot when more refactors are done
                let component_node_id = resolved_receiver.node_id();
                let component_info = self.execute_in_mode::<_, _, RuntimeError>(
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

                        Ok(component_info)
                    },
                )?;
                let package_node_id = self
                    .node_method_deref(RENodeId::Global(GlobalAddress::Package(
                        component_info.package_address,
                    )))?
                    .unwrap();
                let package = self.execute_in_mode::<_, _, RuntimeError>(
                    ExecutionMode::ScryptoInterpreter,
                    |system_api| {
                        let handle = system_api.lock_substate(
                            package_node_id,
                            SubstateOffset::Package(PackageOffset::Package),
                            LockFlags::read_only(),
                        )?;
                        let substate_ref = system_api.get_ref(handle)?;
                        let package = substate_ref.package().clone(); // TODO: Remove clone()
                        system_api.drop_lock(handle)?;

                        Ok(package)
                    },
                )?;

                // Pass the component ref
                // TODO: remove? currently needed for `Runtime::package_address()` API.
                let global_node_id =
                    RENodeId::Global(GlobalAddress::Package(component_info.package_address));
                additional_ref_copy.insert(
                    global_node_id,
                    self.current_frame
                        .get_node_location(global_node_id)
                        .unwrap(),
                );
                additional_ref_copy.insert(
                    component_node_id,
                    self.current_frame
                        .get_node_location(component_node_id)
                        .unwrap(),
                );

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
                if !fn_abi.input.matches(&args.dom) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoMethodInvocation(
                            method_ident.clone(),
                            ScryptoFnResolvingError::InvalidInput,
                        ),
                    ));
                }

                // Emit event
                for m in &mut self.modules {
                    m.on_wasm_instantiation(
                        &self.current_frame,
                        &mut self.heap,
                        &mut self.track,
                        package.code(),
                    )
                    .map_err(RuntimeError::ModuleError)?;
                }

                REActor::Method(
                    ResolvedMethod::Scrypto {
                        package_address: component_info.package_address,
                        package_id: package_node_id.into(),
                        blueprint_name: component_info.blueprint_name,
                        ident: method_ident.method_name.clone(),
                        export_name: fn_abi.export_name.clone(),
                        return_type: fn_abi.output.clone(),
                        code: package.code,
                    },
                    resolved_receiver,
                )
            }
        };

        Ok((actor, additional_ref_copy, HashSet::new()))
    }

    fn resolve_native_actor(
        &mut self,
        invocation: &NativeInvocation,
    ) -> Result<
        (
            REActor,
            HashMap<RENodeId, RENodeLocation>,
            HashSet<RENodeId>,
        ),
        RuntimeError,
    > {
        let mut additional_ref_copy = HashMap::new();
        let mut additional_node_move = HashSet::new();

        let actor = match invocation {
            NativeInvocation::Function(native_function, _) => {
                REActor::Function(ResolvedFunction::Native(*native_function))
            }
            NativeInvocation::Method(native_method, receiver, _) => {
                let resolved_receiver = match receiver {
                    Receiver::Consumed(node_id) => {
                        additional_node_move.insert(*node_id);
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
                        let location = self.current_frame.get_node_location(resolved_node_id)?;
                        additional_ref_copy.insert(resolved_node_id, location);

                        resolved_receiver
                    }
                };

                REActor::Method(ResolvedMethod::Native(*native_method), resolved_receiver)
            }
        };

        Ok((actor, additional_ref_copy, additional_node_move))
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

    fn invoke(&mut self, invocation: Invocation) -> Result<ScryptoValue, RuntimeError> {
        let depth = self.current_frame.depth;

        for m in &mut self.modules {
            m.pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                match &invocation {
                    Invocation::Scrypto(i) => SysCallInput::InvokeScrypto {
                        invocation: i,
                        depth,
                    },
                    Invocation::Native(i) => SysCallInput::InvokeNative {
                        invocation: i,
                        depth,
                    },
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

        let mut nodes_to_pass_downstream = Vec::new();
        let mut next_node_refs = HashMap::new();

        nodes_to_pass_downstream.extend(invocation.args().node_ids());
        // Internal state update to taken values

        // Move this into higher layer, e.g. transaction processor
        if self.current_frame.depth == 0 {
            let mut static_refs = HashSet::new();
            static_refs.insert(GlobalAddress::Resource(RADIX_TOKEN));
            static_refs.insert(GlobalAddress::Resource(SYSTEM_TOKEN));
            static_refs.insert(GlobalAddress::Resource(ECDSA_SECP256K1_TOKEN));
            static_refs.insert(GlobalAddress::Component(SYS_SYSTEM_COMPONENT));
            static_refs.insert(GlobalAddress::Package(ACCOUNT_PACKAGE));
            static_refs.insert(GlobalAddress::Package(SYS_FAUCET_PACKAGE));

            // Make refs visible
            let mut global_references = invocation.args().global_references();
            global_references.extend(static_refs.clone());

            // TODO: This can be refactored out once any type in sbor is implemented
            let maybe_txn: Result<TransactionProcessorRunInput, DecodeError> =
                scrypto_decode(&invocation.args().raw);
            if let Ok(input) = maybe_txn {
                for instruction in &input.instructions {
                    match instruction {
                        Instruction::CallFunction { args, .. }
                        | Instruction::CallMethod { args, .. }
                        | Instruction::CallNativeFunction { args, .. }
                        | Instruction::CallNativeMethod { args, .. } => {
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

                // TODO: static check here is to support the current genesis transaction which
                // TODO: requires references to dynamically created resources. Can remove
                // TODO: when this is resolved.
                if !static_refs.contains(&global_address) {
                    self.track
                        .acquire_lock(SubstateId(node_id, offset.clone()), LockFlags::read_only())
                        .map_err(|_| KernelError::GlobalAddressNotFound(global_address))?;
                    self.track
                        .release_lock(SubstateId(node_id, offset), false)
                        .map_err(|_| KernelError::GlobalAddressNotFound(global_address))?;
                }

                self.current_frame
                    .node_refs
                    .insert(node_id, RENodeLocation::Store);
                next_node_refs.insert(node_id, RENodeLocation::Store);
            }
        } else {
            // Check that global references are owned by this call frame
            let mut global_references = invocation.args().global_references();
            global_references.insert(GlobalAddress::Resource(RADIX_TOKEN));
            global_references.insert(GlobalAddress::Component(SYS_SYSTEM_COMPONENT));
            for global_address in global_references {
                let node_id = RENodeId::Global(global_address);

                // As of now, once a component is made visible to the frame, client can directly
                // read the substates of the component. This will cause "Substate was never locked" issue.
                // We use the following temporary solution to work around this.
                // A better solution is to create node representation before issuing any reference.
                // TODO: remove
                if let Some(pointer) = self.current_frame.node_refs.get(&node_id) {
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

        let (next_actor, additional_ref_copy, additional_node_move) = match &invocation {
            Invocation::Scrypto(i) => self.resolve_scrypto_actor(&i)?,
            Invocation::Native(i) => self.resolve_native_actor(&i)?,
        };
        next_node_refs.extend(additional_ref_copy);
        nodes_to_pass_downstream.extend(additional_node_move);

        let output = self.run(
            next_actor,
            invocation.args().clone(), // TODO: Remove clone
            nodes_to_pass_downstream,
            next_node_refs,
        )?;

        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::InvokeScrypto { output: &output },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // TODO: Move this into higher layer, e.g. transaction processor
        if self.current_frame.depth == 0 {
            self.current_frame.drop_all_locks(&mut self.track)?;
            self.execution_mode = ExecutionMode::Application;
            self.drop_nodes_in_frame()?;
            self.execution_mode = ExecutionMode::Kernel;
        }

        // Restore previous mode
        self.execution_mode = saved_mode;

        Ok(output)
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
            m.on_wasm_costing(&self.current_frame, &mut self.heap, &mut self.track, units)
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
                    &self.current_frame,
                    &mut self.heap,
                    &mut self.track,
                    vault_id,
                    fee,
                    contingent,
                )
                .map_err(RuntimeError::ModuleError)?;
        }

        Ok(fee)
    }

    fn get_actor(&self) -> &REActor {
        &self.current_frame.actor
    }

    fn invoke_scrypto(
        &mut self,
        invocation: ScryptoInvocation,
    ) -> Result<ScryptoValue, RuntimeError> {
        self.invoke(Invocation::Scrypto(invocation))
    }

    fn invoke_native(
        &mut self,
        invocation: NativeInvocation,
    ) -> Result<ScryptoValue, RuntimeError> {
        self.invoke(Invocation::Native(invocation))
    }

    fn get_visible_node_ids(&mut self) -> Result<Vec<RENodeId>, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::ReadOwnedNodes,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let node_ids = self.current_frame.get_visible_nodes();

        Ok(node_ids)
    }

    fn drop_node(&mut self, node_id: RENodeId) -> Result<HeapRENode, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::DropNode { node_id: &node_id },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        if !VisibilityProperties::check_drop_node_visibility(
            current_mode,
            &self.current_frame.actor,
            node_id,
        ) {
            return Err(RuntimeError::KernelError(
                KernelError::InvalidDropNodeVisibility {
                    mode: current_mode,
                    actor: self.current_frame.actor.clone(),
                    node_id,
                },
            ));
        }

        let mut node = self.current_frame.drop_node(&mut self.heap, node_id)?;
        node.try_drop()
            .map_err(|e| RuntimeError::KernelError(KernelError::DropFailure(e)))?;

        // Restore current mode
        self.execution_mode = current_mode;

        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::DropNode { node: &node },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(node)
    }

    fn create_node(&mut self, re_node: RENode) -> Result<RENodeId, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::CreateNode { node: &re_node },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        if !VisibilityProperties::check_create_node_visibility(
            current_mode,
            &self.current_frame.actor,
            &re_node,
        ) {
            return Err(RuntimeError::KernelError(
                KernelError::InvalidCreateNodeVisibility {
                    mode: current_mode,
                    actor: self.current_frame.actor.clone(),
                },
            ));
        }

        // TODO: For Scrypto components, check state against blueprint schema

        let node_id = match &re_node {
            RENode::Global(global_re_node) => {
                let derefed = global_re_node.node_deref();
                let (global_address, global_substate) = self.create_global_node(derefed)?;
                let global_node_id = RENodeId::Global(global_address);
                self.track.insert_substate(
                    SubstateId(global_node_id, SubstateOffset::Global(GlobalOffset::Global)),
                    RuntimeSubstate::Global(global_substate),
                );
                self.current_frame
                    .node_refs
                    .insert(global_node_id, RENodeLocation::Store);
                self.current_frame.move_owned_node_to_store(
                    &mut self.heap,
                    &mut self.track,
                    derefed,
                )?;
                global_node_id
            }
            _ => {
                let node_id =
                    Self::new_node_id(&mut self.id_allocator, self.transaction_hash, &re_node)
                        .map_err(|e| {
                            RuntimeError::KernelError(KernelError::IdAllocationError(e))
                        })?;
                self.current_frame
                    .create_node(&mut self.heap, node_id, re_node)?;
                node_id
            }
        };

        // Restore current mode
        self.execution_mode = current_mode;

        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::CreateNode { node_id: &node_id },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(node_id)
    }

    fn lock_substate(
        &mut self,
        mut node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
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

        // TODO: Check if valid offset for node_id

        // Authorization
        let actor = &self.current_frame.actor;
        if !VisibilityProperties::check_substate_visibility(
            current_mode,
            actor,
            node_id,
            offset.clone(),
            flags,
        ) {
            return Err(RuntimeError::KernelError(
                KernelError::InvalidSubstateVisibility {
                    mode: current_mode,
                    actor: actor.clone(),
                    node_id,
                    offset,
                    flags,
                },
            ));
        }

        let lock_handle =
            self.current_frame
                .acquire_lock(&mut self.track, node_id, offset.clone(), flags)?;

        // Restore current mode
        self.execution_mode = current_mode;

        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::LockSubstate { lock_handle },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(lock_handle)
    }

    fn drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::DropLock {
                    lock_handle: &lock_handle,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        self.current_frame
            .drop_lock(&mut self.track, lock_handle)
            .map_err(RuntimeError::KernelError)?;

        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::DropLock,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(())
    }

    fn get_ref(&mut self, lock_handle: LockHandle) -> Result<SubstateRef, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::GetRef {
                    lock_handle: &lock_handle,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let substate_ref =
            self.current_frame
                .get_ref(lock_handle, &mut self.heap, &mut self.track)?;

        // TODO: Move post sys call to substate_ref drop()
        /*
        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::GetRef {
                    lock_handle,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }
         */

        Ok(substate_ref)
    }

    fn get_ref_mut<'f>(
        &'f mut self,
        lock_handle: LockHandle,
    ) -> Result<SubstateRefMut<'f, 's, R>, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::GetRefMut {
                    lock_handle: &lock_handle,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let substate_ref_mut =
            self.current_frame
                .get_ref_mut(lock_handle, &mut self.heap, &mut self.track)?;

        // TODO: Move post sys call to substate_ref drop()
        /*
        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::GetRefMut,
            )
            .map_err(RuntimeError::ModuleError)?;
        }
         */

        Ok(substate_ref_mut)
    }

    fn read_transaction_hash(&mut self) -> Result<Hash, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::ReadTransactionHash,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
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
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
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
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::ReadBlob { blob: &blob },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(blob)
    }

    fn generate_uuid(&mut self) -> Result<u128, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::GenerateUuid,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let uuid = Self::new_uuid(&mut self.id_allocator, self.transaction_hash)
            .map_err(|e| RuntimeError::KernelError(KernelError::IdAllocationError(e)))?;

        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::GenerateUuid { uuid },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(uuid)
    }

    fn emit_log(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
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
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::EmitLog,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(())
    }
}
