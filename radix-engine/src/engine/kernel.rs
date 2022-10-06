use scrypto::core::{FnIdent, MethodIdent, ReceiverMethodIdent};
use transaction::errors::IdAllocationError;
use transaction::model::{AuthZoneParams, Instruction};
use transaction::validation::*;

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
    /// The transaction hash
    transaction_hash: Hash,
    /// Blobs attached to the transaction
    blobs: &'g HashMap<Hash, Vec<u8>>,
    /// The max call depth
    max_depth: usize,

    /// State track
    track: &'g mut Track<'s, R>,
    /// WASM engine
    wasm_engine: &'g mut W,
    /// WASM Instrumenter
    wasm_instrumenter: &'g mut WasmInstrumenter,
    /// WASM metering params
    wasm_metering_params: WasmMeteringParams,

    /// ID allocator
    id_allocator: IdAllocator,

    /// Execution trace
    execution_trace: &'g mut ExecutionTrace,

    /// Call frames
    call_frames: Vec<CallFrame>,

    /// Kernel modules
    /// TODO: move execution trace and  authorization to modules
    modules: Vec<Box<dyn Module<R>>>,

    phantom: PhantomData<I>,
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
        wasm_engine: &'g mut W,
        wasm_instrumenter: &'g mut WasmInstrumenter,
        wasm_metering_params: WasmMeteringParams,
        execution_trace: &'g mut ExecutionTrace,
        modules: Vec<Box<dyn Module<R>>>,
    ) -> Self {
        let frame = CallFrame::new_root();
        let mut kernel = Self {
            transaction_hash,
            blobs,
            max_depth,
            track,
            wasm_engine,
            wasm_instrumenter,
            wasm_metering_params,
            id_allocator: IdAllocator::new(IdSpace::Application),
            execution_trace,
            call_frames: vec![frame],
            modules,
            phantom: PhantomData,
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
            let mut node_ref = kernel
                .borrow_node_mut(&RENodeId::Bucket(bucket_id))
                .expect("Failed to borrow bucket node");
            let bucket = node_ref.bucket_mut();
            let proof = bucket
                .create_proof(bucket_id)
                .expect("Failed to create proof");
            proofs.push(proof);
        }

        // Create empty buckets for virtual proofs
        let mut virtual_proofs_buckets: BTreeMap<ResourceAddress, BucketId> = BTreeMap::new();
        for resource_address in virtualizable_proofs_resource_addresses {
            let bucket_id = kernel
                .create_non_fungible_bucket_with_ids(resource_address.clone(), BTreeSet::new());
            virtual_proofs_buckets.insert(resource_address, bucket_id);
        }

        let auth_zone = AuthZone::new_with_proofs(proofs, virtual_proofs_buckets);

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
            .node_create(HeapRENode::Bucket(Bucket::new(Resource::new_non_fungible(
                resource_address,
                ids,
            ))))
            .expect("Failed to create a bucket")
        {
            RENodeId::Bucket(bucket_id) => bucket_id,
            _ => panic!("Expected Bucket RENodeId but received something else"),
        }
    }

    fn process_call_data(validated: &ScryptoValue) -> Result<(), RuntimeError> {
        if !validated.kv_store_ids.is_empty() {
            return Err(RuntimeError::KernelError(
                KernelError::KeyValueStoreNotAllowed,
            ));
        }
        if !validated.vault_ids.is_empty() {
            return Err(RuntimeError::KernelError(KernelError::VaultNotAllowed));
        }
        Ok(())
    }

    fn process_return_data(validated: &ScryptoValue) -> Result<(), RuntimeError> {
        if !validated.kv_store_ids.is_empty() {
            return Err(RuntimeError::KernelError(
                KernelError::KeyValueStoreNotAllowed,
            ));
        }

        // TODO: Should we disallow vaults to be moved?

        Ok(())
    }

    fn read_substate_internal(
        call_frames: &mut Vec<CallFrame>,
        track: &mut Track<'s, R>,
        node_pointer: RENodePointer,
        offset: SubstateOffset,
    ) -> Result<ScryptoValue, RuntimeError> {
        // TODO: Cleanup
        if let SubstateOffset::Component(ComponentOffset::Info) = offset {
            node_pointer
                .acquire_lock(
                    SubstateOffset::Component(ComponentOffset::State),
                    false,
                    false,
                    track,
                )
                .map_err(RuntimeError::KernelError)?;
            node_pointer
                .acquire_lock(
                    SubstateOffset::Component(ComponentOffset::Info),
                    false,
                    false,
                    track,
                )
                .map_err(RuntimeError::KernelError)?;
        }

        // Read current value
        let substate = {
            let mut node_ref = node_pointer.to_ref_mut(call_frames, track);
            node_ref.read_substate(&SubstateId(node_pointer.node_id(), offset.clone()))?
        };

        // TODO: Remove, integrate with substate borrow mechanism
        if let SubstateOffset::Component(ComponentOffset::Info) = offset {
            node_pointer
                .release_lock(
                    SubstateOffset::Component(ComponentOffset::State),
                    false,
                    track,
                )
                .map_err(RuntimeError::KernelError)?;
            node_pointer
                .release_lock(
                    SubstateOffset::Component(ComponentOffset::Info),
                    false,
                    track,
                )
                .map_err(RuntimeError::KernelError)?;
        }

        Ok(substate)
    }

    fn new_uuid(
        id_allocator: &mut IdAllocator,
        transaction_hash: Hash,
    ) -> Result<u128, IdAllocationError> {
        id_allocator.new_uuid(transaction_hash)
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
                Ok(RENodeId::AuthZone(auth_zone_id))
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
            HeapRENode::Package(..) => {
                // Security Alert: ensure ID allocating will practically never fail
                let package_address = id_allocator.new_package_address(transaction_hash)?;
                Ok(RENodeId::Package(package_address))
            }
            HeapRENode::ResourceManager(..) => {
                let resource_address = id_allocator.new_resource_address(transaction_hash)?;
                Ok(RENodeId::ResourceManager(resource_address))
            }
            HeapRENode::Component(ref component) => {
                let component_address = id_allocator.new_component_address(
                    transaction_hash,
                    &component.info.package_address,
                    &component.info.blueprint_name,
                )?;
                Ok(RENodeId::Component(component_address))
            }
            HeapRENode::System(..) => {
                let system_component_address =
                    id_allocator.new_system_component_address(transaction_hash)?;
                Ok(RENodeId::System(system_component_address))
            }
        }
    }

    fn run(
        &mut self,
        input: ScryptoValue,
    ) -> Result<(ScryptoValue, HashMap<RENodeId, HeapRootRENode>), RuntimeError> {
        // Copy-over root frame's auth zone virtual_proofs_buckets
        // TODO: Clean this up at some point (possible move to auth zone?)
        let root_frame = self
            .call_frames
            .first()
            .expect("Failed to get a root frame");
        let virtual_proofs_buckets = AuthModule::get_auth_zone(root_frame)
            .virtual_proofs_buckets
            .clone();

        // TODO: Move to a better spot
        self.node_create(HeapRENode::AuthZone(AuthZone::new_with_proofs(
            vec![],
            virtual_proofs_buckets,
        )))?;

        let output = {
            let rtn = match Self::current_frame(&self.call_frames).actor.clone() {
                REActor::Function(FunctionIdent::Native(native_fn)) => {
                    NativeInterpreter::run_function(native_fn, input, self)
                }
                REActor::Method(FullyQualifiedReceiverMethod {
                    receiver,
                    method: FullyQualifiedMethod::Native(native_method),
                }) => NativeInterpreter::run_method(receiver, native_method, input, self),
                REActor::Function(FunctionIdent::Scrypto {
                    package_address,
                    blueprint_name,
                    ident,
                })
                | REActor::Method(FullyQualifiedReceiverMethod {
                    method:
                        FullyQualifiedMethod::Scrypto {
                            package_address,
                            blueprint_name,
                            ident,
                        },
                    ..
                }) => {
                    let output = {
                        let package = self
                            .track
                            .borrow_node(&RENodeId::Package(package_address))
                            .package()
                            .clone();
                        for m in &mut self.modules {
                            m.on_wasm_instantiation(
                                &mut self.track,
                                &mut self.call_frames,
                                package.code(),
                            )
                            .map_err(RuntimeError::ModuleError)?;
                        }
                        let instrumented_code = self
                            .wasm_instrumenter
                            .instrument(package.code(), &self.wasm_metering_params);
                        let mut instance = self.wasm_engine.instantiate(instrumented_code);
                        let blueprint_abi = package
                            .blueprint_abi(&blueprint_name)
                            .expect("Blueprint not found"); // TODO: assumption will break if auth module is optional
                        let export_name = &blueprint_abi
                            .get_fn_abi(&ident)
                            .expect("Function not found")
                            .export_name
                            .to_string();
                        let scrypto_actor = match &Self::current_frame(&self.call_frames).actor {
                            REActor::Method(FullyQualifiedReceiverMethod { receiver, .. }) => {
                                match receiver {
                                    Receiver::Ref(RENodeId::Component(component_address)) => {
                                        ScryptoActor::Component(
                                            *component_address,
                                            package_address.clone(),
                                            blueprint_name.clone(),
                                        )
                                    }
                                    _ => {
                                        return Err(RuntimeError::KernelError(
                                            KernelError::FunctionNotFound(FunctionIdent::Scrypto {
                                                package_address,
                                                blueprint_name,
                                                ident,
                                            }),
                                        ))
                                    }
                                }
                            }
                            _ => ScryptoActor::blueprint(package_address, blueprint_name.clone()),
                        };

                        let mut runtime: Box<dyn WasmRuntime> =
                            Box::new(RadixEngineWasmRuntime::new(scrypto_actor, self));
                        instance
                            .invoke_export(&export_name, &input, &mut runtime)
                            .map_err(|e| match e {
                                InvokeError::Error(e) => {
                                    RuntimeError::KernelError(KernelError::WasmError(e))
                                }
                                InvokeError::Downstream(runtime_error) => runtime_error,
                            })?
                    };

                    let package = self
                        .track
                        .borrow_node(&RENodeId::Package(package_address))
                        .package();
                    let blueprint_abi = package
                        .blueprint_abi(&blueprint_name)
                        .expect("Blueprint not found"); // TODO: assumption will break if auth module is optional
                    let fn_abi = blueprint_abi
                        .get_fn_abi(&ident)
                        .expect("Function not found");
                    if !fn_abi.output.matches(&output.dom) {
                        Err(RuntimeError::KernelError(KernelError::InvalidFnOutput {
                            fn_identifier: FunctionIdent::Scrypto {
                                package_address,
                                blueprint_name,
                                ident,
                            },
                        }))
                    } else {
                        Ok(output)
                    }
                }
            }?;

            rtn
        };

        // Process return data
        Self::process_return_data(&output)?;

        // Take values to return
        let values_to_take = output.node_ids();
        let (received_values, mut missing) = Self::current_frame_mut(&mut self.call_frames)
            .take_available_values(values_to_take, false)?;
        let first_missing_value = missing.drain().nth(0);
        if let Some(missing_node) = first_missing_value {
            return Err(RuntimeError::KernelError(KernelError::RENodeNotFound(
                missing_node,
            )));
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

        // drop proofs and check resource leak
        Self::current_frame_mut(&mut self.call_frames).drop_owned_values()?;

        Ok((output, received_values))
    }

    fn current_frame_mut(call_frames: &mut Vec<CallFrame>) -> &mut CallFrame {
        call_frames.last_mut().expect("Current frame always exists")
    }

    fn current_frame(call_frames: &Vec<CallFrame>) -> &CallFrame {
        call_frames.last().expect("Current frame always exists")
    }

    fn invoke_function(
        &mut self,
        function_ident: FunctionIdent,
        input: ScryptoValue,
        next_owned_values: HashMap<RENodeId, HeapRootRENode>,
        next_frame_node_refs: HashMap<RENodeId, RENodePointer>,
    ) -> Result<(ScryptoValue, HashMap<RENodeId, HeapRootRENode>), RuntimeError> {
        match &function_ident {
            FunctionIdent::Scrypto {
                package_address,
                blueprint_name,
                ident,
            } => {
                let node_id = RENodeId::Package(package_address.clone());
                let node_pointer = RENodePointer::Store(node_id);
                let offset = SubstateOffset::Package(PackageOffset::Package);
                node_pointer
                    .acquire_lock(offset.clone(), false, false, &mut self.track)
                    .map_err(RuntimeError::KernelError)?;

                let package = self.track.borrow_node(&node_id).package();
                let abi =
                    package
                        .blueprint_abi(blueprint_name)
                        .ok_or(RuntimeError::KernelError(KernelError::BlueprintNotFound(
                            package_address.clone(),
                            blueprint_name.clone(),
                        )))?;
                let fn_abi = abi.get_fn_abi(ident).ok_or(RuntimeError::KernelError(
                    KernelError::FunctionNotFound(function_ident.clone()),
                ))?;
                if !fn_abi.input.matches(&input.dom) {
                    return Err(RuntimeError::KernelError(KernelError::InvalidFnInput2(
                        FnIdent::Function(function_ident.clone()),
                    )));
                }

                node_pointer
                    .release_lock(offset, false, &mut self.track)
                    .map_err(RuntimeError::KernelError)?;
            }
            _ => {}
        };

        AuthModule::function_auth(function_ident.clone(), &mut self.call_frames)?;

        // start a new frame and run
        let (output, received_values) = {
            let frame = CallFrame::new_child(
                Self::current_frame(&self.call_frames).depth + 1,
                REActor::Function(function_ident.clone()),
                next_owned_values,
                next_frame_node_refs,
                self,
            );
            self.call_frames.push(frame);
            self.run(input)?
        };

        // Remove the last after clean-up
        self.call_frames.pop();

        Ok((output, received_values))
    }

    fn invoke_method(
        &mut self,
        mut method_ident: ReceiverMethodIdent,
        input: ScryptoValue,
        mut next_owned_values: HashMap<RENodeId, HeapRootRENode>,
        mut next_frame_node_refs: HashMap<RENodeId, RENodePointer>,
    ) -> Result<(ScryptoValue, HashMap<RENodeId, HeapRootRENode>), RuntimeError> {
        let mut locked_pointers = Vec::new();

        // Authorization and state load
        let re_actor = {
            let mut node_id = method_ident.receiver.node_id();
            let mut node_pointer =
                Self::current_frame(&self.call_frames).get_node_pointer(node_id)?;

            // Deref
            if let Receiver::Ref(..) = method_ident.receiver {
                if let Some(derefed) =
                    node_pointer.node_deref(&self.call_frames, &mut self.track)?
                {
                    node_id = derefed.node_id();
                    node_pointer = derefed;
                    method_ident = ReceiverMethodIdent {
                        receiver: Receiver::Ref(node_id),
                        method_ident: method_ident.method_ident,
                    }
                }
            }

            // Lock Primary Substate
            let offset = RENodeProperties::to_primary_offset(&method_ident)?;
            let is_lock_fee =
                matches!(node_id, RENodeId::Vault(..))
                    && (method_ident
                        .method_ident
                        .eq(&MethodIdent::Native(NativeMethod::Vault(
                            VaultMethod::LockFee,
                        )))
                        || method_ident.method_ident.eq(&MethodIdent::Native(
                            NativeMethod::Vault(VaultMethod::LockFee),
                        ))
                        || method_ident.method_ident.eq(&MethodIdent::Native(
                            NativeMethod::Vault(VaultMethod::LockContingentFee),
                        )));
            if is_lock_fee && matches!(node_pointer, RENodePointer::Heap { .. }) {
                return Err(RuntimeError::KernelError(KernelError::RENodeNotInTrack));
            }
            node_pointer
                .acquire_lock(offset.clone(), true, is_lock_fee, &mut self.track)
                .map_err(RuntimeError::KernelError)?;
            locked_pointers.push((node_pointer, offset.clone(), is_lock_fee));

            // Load actor
            let re_actor = match &method_ident {
                ReceiverMethodIdent {
                    method_ident: MethodIdent::Scrypto(ident),
                    receiver,
                } => match node_id {
                    RENodeId::Component(..) => {
                        let mut node_ref = node_pointer.to_ref(&self.call_frames, &mut self.track);
                        let component = node_ref.component();
                        REActor::Method(FullyQualifiedReceiverMethod {
                            receiver: receiver.clone(),
                            method: FullyQualifiedMethod::Scrypto {
                                package_address: component.info.package_address.clone(),
                                blueprint_name: component.info.blueprint_name.clone(),
                                ident: ident.to_string(),
                            },
                        })
                    }
                    _ => panic!("Should not get here."),
                },
                ReceiverMethodIdent {
                    method_ident: MethodIdent::Native(native_fn),
                    receiver,
                } => REActor::Method(FullyQualifiedReceiverMethod {
                    receiver: receiver.clone(),
                    method: FullyQualifiedMethod::Native(native_fn.clone()),
                }),
            };

            // TODO: Check Component ABI here rather than in auth
            match node_id {
                RENodeId::Component(..) => {
                    let package_address = {
                        let mut node_ref = node_pointer.to_ref(&self.call_frames, &mut self.track);
                        let component = node_ref.component();
                        component.info.package_address.clone()
                    };
                    let package_node_id = RENodeId::Package(package_address);
                    let package_node_pointer = RENodePointer::Store(package_node_id);
                    next_frame_node_refs.insert(package_node_id, package_node_pointer);
                }
                RENodeId::Proof(..) => {
                    let resource_address = {
                        let node_ref = node_pointer.to_ref(&self.call_frames, &mut self.track);
                        node_ref.proof().resource_address()
                    };
                    let global_resource_node_id =
                        RENodeId::Global(GlobalAddress::Resource(resource_address));
                    next_frame_node_refs.insert(
                        global_resource_node_id,
                        RENodePointer::Store(global_resource_node_id),
                    );
                }
                RENodeId::Bucket(..) => {
                    let resource_address = {
                        let node_ref = node_pointer.to_ref(&self.call_frames, &mut self.track);
                        node_ref.bucket().resource_address()
                    };

                    let global_resource_node_id =
                        RENodeId::Global(GlobalAddress::Resource(resource_address));
                    next_frame_node_refs.insert(
                        global_resource_node_id,
                        RENodePointer::Store(global_resource_node_id),
                    );

                    let resource_node_id = RENodeId::ResourceManager(resource_address);
                    let resource_node_pointer = RENodePointer::Store(resource_node_id);
                    next_frame_node_refs.insert(resource_node_id, resource_node_pointer);
                }
                RENodeId::Vault(..) => {
                    let resource_address = {
                        let mut node_ref = node_pointer.to_ref(&self.call_frames, &mut self.track);
                        node_ref.vault().resource_address()
                    };
                    let global_resource_node_id =
                        RENodeId::Global(GlobalAddress::Resource(resource_address));
                    next_frame_node_refs.insert(
                        global_resource_node_id,
                        RENodePointer::Store(global_resource_node_id),
                    );

                    let resource_node_id = RENodeId::ResourceManager(resource_address);
                    let resource_node_pointer = RENodePointer::Store(resource_node_id);
                    next_frame_node_refs.insert(resource_node_id, resource_node_pointer);
                }
                _ => {}
            }

            if let MethodIdent::Native(..) = method_ident.method_ident {
                for resource_address in &input.resource_addresses {
                    let resource_node_id = RENodeId::ResourceManager(resource_address.clone());
                    let resource_node_pointer = RENodePointer::Store(resource_node_id);
                    next_frame_node_refs.insert(resource_node_id, resource_node_pointer);
                }
            }

            let current_frame = Self::current_frame(&self.call_frames);
            self.execution_trace.trace_invoke_method(
                &self.call_frames,
                &mut self.track,
                &current_frame.actor,
                &node_id,
                node_pointer,
                FnIdent::Method(method_ident.clone()),
                &input,
                &next_owned_values,
            )?;

            // Check method authorization
            AuthModule::receiver_auth(
                method_ident.clone(),
                &input,
                node_pointer.clone(),
                &mut self.call_frames,
                &mut self.track,
            )?;

            match &method_ident.receiver {
                Receiver::Consumed(..) => {
                    let heap_node = Self::current_frame_mut(&mut self.call_frames)
                        .owned_heap_nodes
                        .remove(&node_id)
                        .ok_or(RuntimeError::KernelError(
                            KernelError::InvokeMethodInvalidReceiver(node_id),
                        ))?;
                    next_owned_values.insert(node_id, heap_node);
                }
                _ => {}
            }

            next_frame_node_refs.insert(node_id.clone(), node_pointer.clone());
            re_actor
        };

        // start a new frame
        let (output, received_values) = {
            let frame = CallFrame::new_child(
                Self::current_frame(&self.call_frames).depth + 1,
                re_actor,
                next_owned_values,
                next_frame_node_refs,
                self,
            );
            self.call_frames.push(frame);
            self.run(input)?
        };

        // Remove the last after clean-up
        self.call_frames.pop();

        // Release locked addresses
        for (node_pointer, offset, write_through) in locked_pointers {
            // TODO: refactor after introducing `Lock` representation.
            node_pointer
                .release_lock(offset, write_through, &mut self.track)
                .map_err(RuntimeError::KernelError)?;
        }

        Ok((output, received_values))
    }
}

impl<'g, 's, W, I, R> SystemApi<'s, W, I, R> for Kernel<'g, 's, W, I, R>
where
    W: WasmEngine<I>,
    I: WasmInstance,
    R: FeeReserve,
{
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

    fn invoke(
        &mut self,
        fn_ident: FnIdent,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::Invoke {
                    function_identifier: &fn_ident,
                    input: &input,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // Check call depth
        if Self::current_frame(&self.call_frames).depth == self.max_depth {
            return Err(RuntimeError::KernelError(
                KernelError::MaxCallDepthLimitReached,
            ));
        }

        // Prevent vaults/kvstores from being moved
        Self::process_call_data(&input)?;

        // Figure out what buckets and proofs to move from this process
        let values_to_take = input.node_ids();
        let (taken_values, mut missing) = Self::current_frame_mut(&mut self.call_frames)
            .take_available_values(values_to_take, false)?;
        let first_missing_value = missing.drain().nth(0);
        if let Some(missing_value) = first_missing_value {
            return Err(RuntimeError::KernelError(KernelError::RENodeNotFound(
                missing_value,
            )));
        }
        // Internal state update to taken values
        let mut next_owned_values = HashMap::new();
        for (id, mut value) in taken_values {
            match &mut value.root_mut() {
                HeapRENode::Proof(proof) => proof.change_to_restricted(),
                _ => {}
            }
            next_owned_values.insert(id, value);
        }

        let mut next_node_refs = HashMap::new();
        // Move this into higher layer, e.g. transaction processor
        if Self::current_frame(&self.call_frames).depth == 0 {
            let mut static_refs = HashSet::new();
            static_refs.insert(GlobalAddress::Resource(RADIX_TOKEN));
            static_refs.insert(GlobalAddress::Resource(SYSTEM_TOKEN));
            static_refs.insert(GlobalAddress::Resource(ECDSA_TOKEN));
            static_refs.insert(GlobalAddress::Component(SYS_SYSTEM_COMPONENT));

            // Make refs visible
            let mut global_references = input.global_references();
            global_references.extend(static_refs.clone());

            // TODO: This can be refactored out once any type in sbor is implemented
            let maybe_txn: Result<TransactionProcessorRunInput, DecodeError> =
                scrypto_decode(&input.raw);
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
                        .acquire_lock(offset.clone(), false, false, &mut self.track)
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
            let mut global_references = input.global_references();
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

        // TODO: Slowly unify these two
        let (output, received_values) = match fn_ident {
            FnIdent::Method(method_ident) => {
                self.invoke_method(method_ident, input, next_owned_values, next_node_refs)?
            }
            FnIdent::Function(function_ident) => {
                self.invoke_function(function_ident, input, next_owned_values, next_node_refs)?
            }
        };

        // move re nodes to this process.
        for (id, value) in received_values {
            Self::current_frame_mut(&mut self.call_frames)
                .owned_heap_nodes
                .insert(id, value);
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
                SysCallOutput::Invoke { output: &output },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(output)
    }

    fn borrow_node(&mut self, node_id: &RENodeId) -> Result<RENodeRef<'_, 's, R>, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::BorrowNode { node_id: node_id },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let current_frame = Self::current_frame(&self.call_frames);
        let mut node_pointer = current_frame.get_node_pointer(*node_id)?;

        // Deref
        if let Some(derefed) = node_pointer.node_deref(&self.call_frames, &mut self.track)? {
            node_pointer = derefed;
        }

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::BorrowNode {
                    // Can't return the NodeRef due to borrow checks on `call_frames`
                    node_pointer: &node_pointer,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(node_pointer.to_ref(&self.call_frames, &mut self.track))
    }

    fn borrow_node_mut(
        &mut self,
        node_id: &RENodeId,
    ) -> Result<RENodeRefMut<'_, 's, R>, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::BorrowNode { node_id: node_id },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let current_frame = Self::current_frame(&self.call_frames);
        let mut node_pointer = current_frame.get_node_pointer(*node_id)?;

        // Deref
        if let Some(derefed) = node_pointer.node_deref(&self.call_frames, &mut self.track)? {
            node_pointer = derefed;
        }

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::BorrowNode {
                    // Can't return the NodeRef due to borrow checks on `call_frames`
                    node_pointer: &node_pointer,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(node_pointer.to_ref_mut(&mut self.call_frames, &mut self.track))
    }

    fn get_owned_node_ids(&mut self) -> Result<Vec<RENodeId>, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::ReadOwnedNodes,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let node_ids = Self::current_frame_mut(&mut self.call_frames)
            .owned_heap_nodes
            .keys()
            .cloned()
            .collect();
        Ok(node_ids)
    }

    fn node_drop(&mut self, node_id: &RENodeId) -> Result<HeapRootRENode, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::DropNode { node_id: node_id },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // TODO: Authorization

        let node = Self::current_frame_mut(&mut self.call_frames)
            .owned_heap_nodes
            .remove(&node_id)
            .expect(&format!(
                "Attempt to drop node {:?}, which is not owned by current frame",
                node_id
            )); // TODO: Assumption will break if auth is optional

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

    fn node_create(&mut self, re_node: HeapRENode) -> Result<RENodeId, RuntimeError> {
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
        let children = re_node.get_child_nodes()?;
        let (taken_root_nodes, mut missing) =
            Self::current_frame_mut(&mut self.call_frames).take_available_values(children, true)?;
        let first_missing_node = missing.drain().nth(0);
        if let Some(missing_node) = first_missing_node {
            return Err(RuntimeError::KernelError(
                KernelError::RENodeCreateNodeNotFound(missing_node),
            ));
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
        Self::current_frame_mut(&mut self.call_frames)
            .owned_heap_nodes
            .insert(node_id, heap_root_node);

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

        let (global_address, global_substate) = RENodeProperties::to_global(node_id).ok_or(
            RuntimeError::KernelError(KernelError::RENodeGlobalizeTypeNotAllowed(node_id)),
        )?;
        self.track.new_global_addresses.push(global_address);

        self.track.put_substate(
            SubstateId(
                RENodeId::Global(global_address),
                SubstateOffset::Global(GlobalOffset::Global),
            ),
            Substate::GlobalRENode(global_substate),
        );
        for (id, substate) in nodes_to_substates(node.to_nodes(node_id)) {
            self.track.put_substate(id, substate);
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

    fn substate_read(&mut self, substate_id: SubstateId) -> Result<ScryptoValue, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::ReadSubstate {
                    substate_id: &substate_id,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // Get Pointer
        let mut node_id = substate_id.0;
        let mut node_pointer = Self::current_frame(&self.call_frames).get_node_pointer(node_id)?;

        // Deref
        if let Some(derefed) = node_pointer.node_deref(&self.call_frames, &mut self.track)? {
            node_id = derefed.node_id();
            node_pointer = derefed;
        }

        // Authorization
        // TODO: Check if valid offset for node_id
        let offset = substate_id.1;
        if !Self::current_frame(&self.call_frames)
            .actor
            .is_substate_readable(node_id, offset.clone())
        {
            return Err(RuntimeError::KernelError(
                KernelError::SubstateReadNotReadable(
                    Self::current_frame(&self.call_frames).actor.clone(),
                    SubstateId(node_id, offset.clone()),
                ),
            ));
        }

        let substate = Self::read_substate_internal(
            &mut self.call_frames,
            self.track,
            node_pointer,
            offset.clone(),
        )?;

        // TODO: Clean the following referencing up
        let contained_value = extract_value_from_substate(&offset, &substate);
        if let Some(value) = contained_value {
            for global_address in value.global_references() {
                let node_id = RENodeId::Global(global_address);
                Self::current_frame_mut(&mut self.call_frames)
                    .node_refs
                    .insert(node_id, RENodePointer::Store(node_id));
            }

            let cur_children = value.node_ids();
            for child_id in cur_children {
                let child_pointer = node_pointer.child(child_id);
                Self::current_frame_mut(&mut self.call_frames)
                    .node_refs
                    .insert(child_id, child_pointer);
            }
        }

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::ReadSubstate { value: &substate },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(substate)
    }

    fn substate_write(
        &mut self,
        substate_id: SubstateId,
        substate: ScryptoValue, // TODO: use substate?
    ) -> Result<ScryptoValue, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::WriteSubstate {
                    substate_id: &substate_id,
                    value: &substate,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let offset = substate_id.1;

        // TODO: Do this in a better way once references cleaned up
        let contained_value = extract_value_from_substate(&offset, &substate); // FIXME: error handling!!!
        if let Some(value) = &contained_value {
            // Verify references exist
            for global_address in value.global_references() {
                let node_id = RENodeId::Global(global_address);
                if !Self::current_frame_mut(&mut self.call_frames)
                    .node_refs
                    .contains_key(&node_id)
                {
                    return Err(RuntimeError::KernelError(
                        KernelError::InvalidReferenceWrite(global_address),
                    ));
                }
            }
        }

        // Take values from current frame
        let (taken_nodes, missing_nodes) = {
            let node_ids = contained_value
                .map(|value| value.node_ids())
                .unwrap_or_default();
            if !node_ids.is_empty() {
                if !SubstateProperties::can_own_nodes(&offset) {
                    return Err(RuntimeError::KernelError(KernelError::ValueNotAllowed));
                }

                Self::current_frame_mut(&mut self.call_frames)
                    .take_available_values(node_ids, true)?
            } else {
                (HashMap::new(), HashSet::new())
            }
        };

        // Get Pointer
        let mut node_id = substate_id.0;
        let mut node_pointer = Self::current_frame(&self.call_frames).get_node_pointer(node_id)?;
        if let Some(derefed) = node_pointer.node_deref(&self.call_frames, &mut self.track)? {
            node_id = derefed.node_id();
            node_pointer = derefed;
        }

        // Authorization
        if !Self::current_frame(&self.call_frames)
            .actor
            .is_substate_writeable(node_id, offset.clone())
        {
            return Err(RuntimeError::KernelError(
                KernelError::SubstateWriteNotWriteable(
                    Self::current_frame(&self.call_frames).actor.clone(),
                    SubstateId(node_id, offset.clone()),
                ),
            ));
        }

        let prev_substate = Self::read_substate_internal(
            &mut self.call_frames,
            self.track,
            node_pointer,
            offset.clone(),
        )?;
        let prev_contained_value = extract_value_from_substate(&offset, &prev_substate);

        // Fulfill method
        let prev_children = prev_contained_value
            .map(|v| v.node_ids())
            .unwrap_or_default();
        verify_stored_value_update(&prev_children, &missing_nodes)?;

        // TODO: verify against some schema

        // Write values
        let mut node_ref = node_pointer.to_ref_mut(&mut self.call_frames, &mut self.track);
        node_ref.write_substate(SubstateId(node_id, offset), substate, taken_nodes);

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::WriteSubstate,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(prev_substate)
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

// Parse contained value (for reference management and children detection)
fn extract_value_from_substate(
    offset: &SubstateOffset,
    substate: &ScryptoValue,
) -> Option<ScryptoValue> {
    // Not that in the future, we may store child node ids and node refs in fields
    // other than the "any" byte array. Then, we will have to change the implementation to read
    // all fields.
    match offset {
        SubstateOffset::Component(ComponentOffset::State) => {
            let substate: ComponentStateSubstate = scrypto_decode(&substate.raw).ok()?;
            Some(ScryptoValue::from_slice(&substate.raw).unwrap())
        }
        SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(..)) => {
            let substate: KeyValueStoreEntrySubstate = scrypto_decode(&substate.raw).ok()?;
            substate
                .0
                .map(|raw| ScryptoValue::from_slice(&raw).unwrap())
        }
        SubstateOffset::ResourceManager(ResourceManagerOffset::NonFungible(..)) => {
            let substate: NonFungibleSubstate = scrypto_decode(&substate.raw).ok()?;
            substate.0.map(|v| ScryptoValue::from_typed(&v))
        }
        _ => None,
    }
}
