use transaction::errors::IdAllocationError;
use transaction::model::Instruction;
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
        initial_proofs: Vec<NonFungibleAddress>,
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
        let mut proofs_to_create = BTreeMap::<ResourceAddress, BTreeSet<NonFungibleId>>::new();
        for non_fungible in initial_proofs {
            proofs_to_create
                .entry(non_fungible.resource_address())
                .or_insert(BTreeSet::new())
                .insert(non_fungible.non_fungible_id());
        }
        let mut proofs = Vec::new();
        for (resource_address, non_fungible_ids) in proofs_to_create {
            let bucket_id = match kernel
                .node_create(HeapRENode::Bucket(Bucket::new(Resource::new_non_fungible(
                    resource_address,
                    non_fungible_ids,
                ))))
                .expect("Failed to create bucket")
            {
                RENodeId::Bucket(bucket_id) => bucket_id,
                _ => panic!("Expected Bucket RENodeId but received something else"),
            };
            let mut node_ref = kernel
                .borrow_node_mut(&RENodeId::Bucket(bucket_id))
                .expect("Failed to borrow bucket node");
            let bucket = node_ref.bucket_mut();
            let proof = bucket
                .create_proof(bucket_id)
                .expect("Failed to create proof");
            proofs.push(proof);
        }
        kernel
            .node_create(HeapRENode::AuthZone(AuthZone::new_with_proofs(proofs)))
            .expect("Failed to create AuthZone");

        kernel
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

    fn read_value_internal(
        call_frames: &mut Vec<CallFrame>,
        track: &mut Track<'s, R>,
        substate_id: &SubstateId,
    ) -> Result<(RENodePointer, ScryptoValue), RuntimeError> {
        let node_id = SubstateProperties::get_node_id(substate_id);

        // Get location
        // Note this must be run AFTER values are taken, otherwise there would be inconsistent readable_values state
        let node_pointer = call_frames
            .last()
            .expect("Current call frame does not exist")
            .node_refs
            .get(&node_id)
            .cloned()
            .ok_or_else(|| {
                RuntimeError::KernelError(KernelError::SubstateReadSubstateNotFound(
                    substate_id.clone(),
                ))
            })?;

        if let SubstateId::ComponentInfo(address) = substate_id {
            node_pointer
                .acquire_lock(SubstateId::ComponentState(*address), false, false, track)
                .map_err(RuntimeError::KernelError)?;
            node_pointer
                .acquire_lock(SubstateId::ComponentInfo(*address), false, false, track)
                .map_err(RuntimeError::KernelError)?;
        }

        // Read current value
        let current_value = {
            let mut node_ref = node_pointer.to_ref_mut(call_frames, track);
            node_ref.read_scrypto_value(&substate_id)?
        };

        // TODO: Remove, integrate with substate borrow mechanism
        if let SubstateId::ComponentInfo(address) = substate_id {
            node_pointer
                .release_lock(SubstateId::ComponentState(*address), false, track)
                .map_err(RuntimeError::KernelError)?;
            node_pointer
                .release_lock(SubstateId::ComponentInfo(*address), false, track)
                .map_err(RuntimeError::KernelError)?;
        }

        Ok((node_pointer.clone(), current_value))
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
        // TODO: Move to a better spot
        self.node_create(HeapRENode::AuthZone(AuthZone::new()))?;

        let output = {
            let rtn = match Self::current_frame(&self.call_frames).actor.clone() {
                REActor {
                    receiver,
                    fn_identifier: FnIdentifier::Native(native_fn),
                } => NativeInterpreter::run(receiver, native_fn, input, self),
                REActor {
                    receiver,
                    fn_identifier:
                        FnIdentifier::Scrypto {
                            package_address,
                            blueprint_name,
                            ident,
                        },
                } => {
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
                        let scrypto_actor = match receiver {
                            Some(Receiver::Ref(RENodeId::Component(component_address))) => {
                                ScryptoActor::Component(
                                    component_address,
                                    package_address.clone(),
                                    blueprint_name.clone(),
                                )
                            }
                            None => {
                                ScryptoActor::blueprint(package_address, blueprint_name.clone())
                            }
                            _ => {
                                return Err(RuntimeError::KernelError(KernelError::MethodNotFound(
                                    FnIdentifier::Scrypto {
                                        package_address,
                                        blueprint_name,
                                        ident,
                                    },
                                )))
                            }
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
                            fn_identifier: FnIdentifier::Scrypto {
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

        // Check we have valid references to pass back
        for refed_component_address in &output.refed_component_addresses {
            let node_id = RENodeId::Component(*refed_component_address);
            if let Some(RENodePointer::Store(..)) = Self::current_frame_mut(&mut self.call_frames)
                .node_refs
                .get(&node_id)
            {
                // Only allow passing back global references
            } else {
                let node_id = RENodeId::System(refed_component_address.clone());
                if !Self::current_frame(&self.call_frames)
                    .node_refs
                    .contains_key(&node_id)
                {
                    return Err(RuntimeError::KernelError(
                        KernelError::InvokeMethodInvalidReferencePass(node_id),
                    ));
                }
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

    fn invoke_function(
        &mut self,
        fn_identifier: FnIdentifier,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        let depth = Self::current_frame(&self.call_frames).depth;

        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::InvokeFunction {
                    fn_identifier: &fn_identifier,
                    input: &input,
                    depth,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // Check call depth
        if depth == self.max_depth {
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

        let mut next_owned_values = HashMap::new();

        // Internal state update to taken values
        for (id, mut value) in taken_values {
            match &mut value.root_mut() {
                HeapRENode::Proof(proof) => proof.change_to_restricted(),
                _ => {}
            }
            next_owned_values.insert(id, value);
        }

        let mut locked_values = HashSet::<SubstateId>::new();

        // No authorization but state load
        match &fn_identifier {
            FnIdentifier::Scrypto {
                package_address,
                blueprint_name,
                ident,
            } => {
                let node_pointer = RENodePointer::Store(RENodeId::Package(package_address.clone()));
                node_pointer
                    .acquire_lock(
                        SubstateId::Package(package_address.clone()),
                        false,
                        false,
                        &mut self.track,
                    )
                    .map_err(RuntimeError::KernelError)?;

                locked_values.insert(SubstateId::Package(package_address.clone()));
                let package = self
                    .track
                    .borrow_node(&RENodeId::Package(package_address.clone()))
                    .package();
                let abi =
                    package
                        .blueprint_abi(blueprint_name)
                        .ok_or(RuntimeError::KernelError(KernelError::BlueprintNotFound(
                            package_address.clone(),
                            blueprint_name.clone(),
                        )))?;
                let fn_abi = abi.get_fn_abi(ident).ok_or(RuntimeError::KernelError(
                    KernelError::MethodNotFound(fn_identifier.clone()),
                ))?;
                if !fn_abi.input.matches(&input.dom) {
                    return Err(RuntimeError::KernelError(KernelError::InvalidFnInput {
                        fn_identifier,
                    }));
                }
            }
            _ => {}
        };

        // Move this into higher layer, e.g. transaction processor
        let mut next_frame_node_refs = HashMap::new();
        if Self::current_frame(&self.call_frames).depth == 0 {
            let mut component_addresses = HashSet::new();

            // Collect component addresses
            for component_address in &input.refed_component_addresses {
                component_addresses.insert(*component_address);
            }
            let input: TransactionProcessorRunInput =
                scrypto_decode(&input.raw).expect("Transaction processor received invalid input");
            for instruction in &input.instructions {
                match instruction {
                    Instruction::CallFunction { args, .. }
                    | Instruction::CallMethod { args, .. } => {
                        let scrypto_value =
                            ScryptoValue::from_slice(&args).expect("Invalid CALL arguments");
                        component_addresses.extend(scrypto_value.refed_component_addresses);
                    }
                    _ => {}
                }
            }

            // Make components visible
            for component_address in component_addresses {
                let node_id = RENodeId::Component(component_address);

                // Check if component exists as root
                if !self
                    .track
                    .is_root(&SubstateId::ComponentInfo(component_address))
                {
                    return Err(RuntimeError::KernelError(KernelError::RENodeNotFound(
                        node_id,
                    )));
                }
                let node_pointer = RENodePointer::Store(node_id);
                node_pointer
                    .acquire_lock(
                        SubstateId::ComponentState(component_address),
                        false,
                        false,
                        &mut self.track,
                    )
                    .map_err(RuntimeError::KernelError)?;
                node_pointer
                    .acquire_lock(
                        SubstateId::ComponentInfo(component_address),
                        false,
                        false,
                        &mut self.track,
                    )
                    .map_err(RuntimeError::KernelError)?;
                node_pointer
                    .release_lock(
                        SubstateId::ComponentState(component_address),
                        false,
                        &mut self.track,
                    )
                    .map_err(RuntimeError::KernelError)?;
                node_pointer
                    .release_lock(
                        SubstateId::ComponentInfo(component_address),
                        false,
                        &mut self.track,
                    )
                    .map_err(RuntimeError::KernelError)?;
                next_frame_node_refs.insert(node_id, node_pointer);
            }
        } else {
            // Pass argument references
            for refed_component_address in &input.refed_component_addresses {
                let node_id = RENodeId::Component(refed_component_address.clone());
                if let Some(pointer) = Self::current_frame_mut(&mut self.call_frames)
                    .node_refs
                    .get(&node_id)
                {
                    let mut visible = HashSet::new();
                    visible.insert(SubstateId::ComponentInfo(*refed_component_address));
                    next_frame_node_refs.insert(node_id.clone(), pointer.clone());
                } else {
                    let node_id = RENodeId::System(refed_component_address.clone());
                    if !Self::current_frame(&self.call_frames)
                        .node_refs
                        .contains_key(&node_id)
                    {
                        return Err(RuntimeError::KernelError(
                            KernelError::InvokeMethodInvalidReferencePass(node_id),
                        ));
                    }
                }
            }
        }

        AuthModule::function_auth(&fn_identifier, &mut self.call_frames)?;

        // start a new frame and run
        let (output, received_values) = {
            let frame = CallFrame::new_child(
                Self::current_frame(&self.call_frames).depth + 1,
                REActor {
                    fn_identifier: fn_identifier.clone(),
                    receiver: None,
                },
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
        for l in locked_values {
            // TODO: refactor after introducing `Lock` representation.
            self.track
                .release_lock(l.clone(), false)
                .map_err(KernelError::SubstateError)
                .map_err(RuntimeError::KernelError)?;
        }

        // move buckets and proofs to this process.
        for (id, value) in received_values {
            Self::current_frame_mut(&mut self.call_frames)
                .owned_heap_nodes
                .insert(id, value);
        }

        // Accept component references
        for refed_component_address in &output.refed_component_addresses {
            let node_id = RENodeId::Component(*refed_component_address);
            let mut visible = HashSet::new();
            visible.insert(SubstateId::ComponentInfo(*refed_component_address));
            Self::current_frame_mut(&mut self.call_frames)
                .node_refs
                .insert(node_id, RENodePointer::Store(node_id));
        }

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::InvokeFunction { output: &output },
            )
            .map_err(RuntimeError::ModuleError)?;
        }
        Ok(output)
    }

    fn invoke_method(
        &mut self,
        receiver: Receiver,
        fn_identifier: FnIdentifier,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        let depth = Self::current_frame(&self.call_frames).depth;

        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::InvokeMethod {
                    receiver: &receiver,
                    fn_identifier: &fn_identifier,
                    input: &input,
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

        let mut next_owned_values = HashMap::new();

        // Internal state update to taken values
        for (id, mut value) in taken_values {
            match &mut value.root_mut() {
                HeapRENode::Proof(proof) => proof.change_to_restricted(),
                _ => {}
            }
            next_owned_values.insert(id, value);
        }

        let mut locked_pointers = Vec::new();
        let mut next_frame_node_refs = HashMap::new();

        // Authorization and state load
        match &receiver {
            Receiver::Ref(node_id) | Receiver::Consumed(node_id) => {
                // Find node
                let node_pointer = {
                    let current_frame = Self::current_frame(&self.call_frames);
                    if current_frame.owned_heap_nodes.contains_key(&node_id) {
                        RENodePointer::Heap {
                            frame_id: current_frame.depth,
                            root: node_id.clone(),
                            id: None,
                        }
                    } else if let Some(pointer) = current_frame.node_refs.get(&node_id) {
                        pointer.clone()
                    } else {
                        match node_id {
                            // Let these be globally accessible for now
                            // TODO: Remove when references cleaned up
                            RENodeId::ResourceManager(..) | RENodeId::System(..) => {
                                RENodePointer::Store(*node_id)
                            }
                            _ => {
                                return Err(RuntimeError::KernelError(
                                    KernelError::InvokeMethodInvalidReceiver(*node_id),
                                ))
                            }
                        }
                    }
                };

                // Lock Primary Substate
                let substate_id =
                    RENodeProperties::to_primary_substate_id(&fn_identifier, *node_id)?;
                let is_lock_fee = matches!(node_id, RENodeId::Vault(..))
                    && (fn_identifier.eq(&FnIdentifier::Native(NativeFnIdentifier::Vault(
                        VaultFnIdentifier::LockFee,
                    ))) || fn_identifier.eq(&FnIdentifier::Native(NativeFnIdentifier::Vault(
                        VaultFnIdentifier::LockContingentFee,
                    ))));
                if is_lock_fee && matches!(node_pointer, RENodePointer::Heap { .. }) {
                    return Err(RuntimeError::KernelError(KernelError::RENodeNotInTrack));
                }
                node_pointer
                    .acquire_lock(substate_id.clone(), true, is_lock_fee, &mut self.track)
                    .map_err(RuntimeError::KernelError)?;
                locked_pointers.push((node_pointer, substate_id.clone(), is_lock_fee));

                // TODO: Refactor when locking model finalized
                let mut temporary_locks = Vec::new();

                // Load actor
                match &fn_identifier {
                    FnIdentifier::Scrypto {
                        package_address,
                        blueprint_name,
                        ..
                    } => match node_id {
                        RENodeId::Component(component_address) => {
                            let temporary_substate_id =
                                SubstateId::ComponentInfo(*component_address);
                            node_pointer
                                .acquire_lock(
                                    temporary_substate_id.clone(),
                                    false,
                                    false,
                                    &mut self.track,
                                )
                                .map_err(RuntimeError::KernelError)?;
                            temporary_locks.push((node_pointer, temporary_substate_id, false));

                            let mut node_ref =
                                node_pointer.to_ref(&self.call_frames, &mut self.track);
                            let component = node_ref.component();

                            // Don't support traits yet
                            if !package_address.eq(&component.info.package_address) {
                                return Err(RuntimeError::KernelError(
                                    KernelError::MethodNotFound(fn_identifier),
                                ));
                            }
                            if !blueprint_name.eq(&component.info.blueprint_name) {
                                return Err(RuntimeError::KernelError(
                                    KernelError::MethodNotFound(fn_identifier),
                                ));
                            }
                        }
                        _ => panic!("Should not get here."),
                    },
                    _ => {}
                };

                // Lock Parent Substates
                // TODO: Check Component ABI here rather than in auth
                match node_id {
                    RENodeId::Component(..) => {
                        let package_address = {
                            let mut node_ref =
                                node_pointer.to_ref(&self.call_frames, &mut self.track);
                            node_ref.component().info.package_address
                        };
                        let package_substate_id = SubstateId::Package(package_address);
                        let package_node_id = RENodeId::Package(package_address);
                        let package_node_pointer = RENodePointer::Store(package_node_id);
                        package_node_pointer
                            .acquire_lock(
                                package_substate_id.clone(),
                                false,
                                false,
                                &mut self.track,
                            )
                            .map_err(RuntimeError::KernelError)?;
                        locked_pointers.push((
                            package_node_pointer,
                            package_substate_id.clone(),
                            false,
                        ));
                        next_frame_node_refs.insert(package_node_id, package_node_pointer);
                    }
                    RENodeId::Bucket(..) => {
                        let resource_address = {
                            let node_ref = node_pointer.to_ref(&self.call_frames, &mut self.track);
                            node_ref.bucket().resource_address()
                        };
                        let resource_substate_id = SubstateId::ResourceManager(resource_address);
                        let resource_node_id = RENodeId::ResourceManager(resource_address);
                        let resource_node_pointer = RENodePointer::Store(resource_node_id);
                        resource_node_pointer
                            .acquire_lock(
                                resource_substate_id.clone(),
                                true,
                                false,
                                &mut self.track,
                            )
                            .map_err(RuntimeError::KernelError)?;
                        locked_pointers.push((resource_node_pointer, resource_substate_id, false));
                        next_frame_node_refs.insert(resource_node_id, resource_node_pointer);
                    }
                    RENodeId::Vault(..) => {
                        let resource_address = {
                            let mut node_ref =
                                node_pointer.to_ref(&self.call_frames, &mut self.track);
                            node_ref.vault().resource_address()
                        };
                        let resource_substate_id = SubstateId::ResourceManager(resource_address);
                        let resource_node_id = RENodeId::ResourceManager(resource_address);
                        let resource_node_pointer = RENodePointer::Store(resource_node_id);
                        resource_node_pointer
                            .acquire_lock(
                                resource_substate_id.clone(),
                                true,
                                false,
                                &mut self.track,
                            )
                            .map_err(RuntimeError::KernelError)?;
                        locked_pointers.push((resource_node_pointer, resource_substate_id, false));
                        next_frame_node_refs.insert(resource_node_id, resource_node_pointer);
                    }
                    _ => {}
                }

                // Lock Resource Managers in request
                // TODO: Remove when references cleaned up
                if let FnIdentifier::Native(..) = &fn_identifier {
                    for resource_address in &input.resource_addresses {
                        let resource_substate_id =
                            SubstateId::ResourceManager(resource_address.clone());
                        let resource_node_id = RENodeId::ResourceManager(resource_address.clone());
                        let resource_node_pointer = RENodePointer::Store(resource_node_id);

                        // This condition check is a hack to fix a resource manager locking issue when the receiver
                        // is a resource manager and its address is present in the argument lists.
                        //
                        // TODO: See the outer TODO for clean-up instruction.
                        if !locked_pointers.contains(&(
                            resource_node_pointer,
                            resource_substate_id.clone(),
                            false,
                        )) {
                            resource_node_pointer
                                .acquire_lock(
                                    resource_substate_id.clone(),
                                    false,
                                    false,
                                    &mut self.track,
                                )
                                .map_err(RuntimeError::KernelError)?;
                            locked_pointers.push((
                                resource_node_pointer,
                                resource_substate_id,
                                false,
                            ));
                        }
                        next_frame_node_refs.insert(resource_node_id, resource_node_pointer);
                    }
                }

                let current_frame = Self::current_frame(&self.call_frames);
                self.execution_trace.trace_invoke_method(
                    &self.call_frames,
                    &mut self.track,
                    &current_frame.actor,
                    &fn_identifier,
                    node_id,
                    node_pointer,
                    &input,
                    &next_owned_values,
                )?;

                // Check method authorization
                AuthModule::receiver_auth(
                    &fn_identifier,
                    receiver.clone(),
                    &input,
                    node_pointer.clone(),
                    &mut self.call_frames,
                    &mut self.track,
                )?;

                match &receiver {
                    Receiver::Consumed(..) => {
                        let heap_node = Self::current_frame_mut(&mut self.call_frames)
                            .owned_heap_nodes
                            .remove(node_id)
                            .ok_or(RuntimeError::KernelError(
                                KernelError::InvokeMethodInvalidReceiver(*node_id),
                            ))?;
                        next_owned_values.insert(*node_id, heap_node);
                    }
                    _ => {}
                }

                for (node_pointer, substate_id, write_through) in temporary_locks {
                    node_pointer
                        .release_lock(substate_id, write_through, &mut self.track)
                        .map_err(RuntimeError::KernelError)?;
                }

                next_frame_node_refs.insert(node_id.clone(), node_pointer.clone());
            }
        }

        // TODO: Cleanup
        // Pass argument references
        for refed_component_address in &input.refed_component_addresses {
            let node_id = RENodeId::Component(refed_component_address.clone());
            if let Some(pointer) = Self::current_frame(&self.call_frames)
                .node_refs
                .get(&node_id)
            {
                let mut visible = HashSet::new();
                visible.insert(SubstateId::ComponentInfo(*refed_component_address));
                next_frame_node_refs.insert(node_id.clone(), pointer.clone());
            } else {
                let node_id = RENodeId::System(refed_component_address.clone());
                if !Self::current_frame(&self.call_frames)
                    .node_refs
                    .contains_key(&node_id)
                {
                    return Err(RuntimeError::KernelError(
                        KernelError::InvokeMethodInvalidReferencePass(node_id),
                    ));
                }
            }
        }

        // start a new frame
        let (output, received_values) = {
            let frame = CallFrame::new_child(
                Self::current_frame(&self.call_frames).depth + 1,
                REActor {
                    fn_identifier: fn_identifier.clone(),
                    receiver: Some(receiver.clone()),
                },
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
        for (node_pointer, substate_id, write_through) in locked_pointers {
            // TODO: refactor after introducing `Lock` representation.
            node_pointer
                .release_lock(substate_id, write_through, &mut self.track)
                .map_err(RuntimeError::KernelError)?;
        }

        // move buckets and proofs to this process.
        for (id, value) in received_values {
            Self::current_frame_mut(&mut self.call_frames)
                .owned_heap_nodes
                .insert(id, value);
        }

        // Accept component references
        for refed_component_address in &output.refed_component_addresses {
            let node_id = RENodeId::Component(*refed_component_address);
            let mut visible = HashSet::new();
            visible.insert(SubstateId::ComponentInfo(*refed_component_address));
            Self::current_frame_mut(&mut self.call_frames)
                .node_refs
                .insert(node_id, RENodePointer::Store(node_id));
        }

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::InvokeMethod { output: &output },
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
        let node_pointer = if current_frame.owned_heap_nodes.get(node_id).is_some() {
            RENodePointer::Heap {
                frame_id: current_frame.depth,
                root: node_id.clone(),
                id: None,
            } // TODO: can I borrow  non-root node?
        } else {
            current_frame
                .node_refs
                .get(node_id)
                .cloned()
                .expect(&format!(
                    "Attempt to borrow node {:?}, which is not visible in current frame.",
                    node_id
                )) // TODO: Assumption will break if auth is optional
        };

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
        let node_pointer = if current_frame.owned_heap_nodes.get(node_id).is_some() {
            RENodePointer::Heap {
                frame_id: current_frame.depth,
                root: node_id.clone(),
                id: None,
            } // TODO: can I borrow  non-root node?
        } else {
            current_frame
                .node_refs
                .get(node_id)
                .cloned()
                .expect(&format!(
                    "Attempt to borrow node {:?}, which is not visible in current frame.",
                    node_id
                )) // TODO: Assumption will break if auth is optional
        };

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
        self.track.new_node_ids.push(node_id.clone());
        let heap_root_node = HeapRootRENode {
            root: re_node,
            child_nodes,
        };
        Self::current_frame_mut(&mut self.call_frames)
            .owned_heap_nodes
            .insert(node_id, heap_root_node);

        // TODO: Clean the following up
        match node_id {
            RENodeId::KeyValueStore(..) | RENodeId::ResourceManager(..) => {
                let frame = self
                    .call_frames
                    .last_mut()
                    .expect("Current call frame does not exist");
                frame.node_refs.insert(
                    node_id.clone(),
                    RENodePointer::Heap {
                        frame_id: frame.depth,
                        root: node_id.clone(),
                        id: None,
                    },
                );
            }
            RENodeId::Component(component_address) => {
                let mut visible = HashSet::new();
                visible.insert(SubstateId::ComponentInfo(component_address));

                let frame = self
                    .call_frames
                    .last_mut()
                    .expect("Current call frame does not exist");
                frame.node_refs.insert(
                    node_id.clone(),
                    RENodePointer::Heap {
                        frame_id: frame.depth,
                        root: node_id.clone(),
                        id: None,
                    },
                );
            }
            _ => {}
        }

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

    fn node_globalize(&mut self, node_id: RENodeId) -> Result<(), RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::GlobalizeNode { node_id: &node_id },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        if !RENodeProperties::can_globalize(node_id) {
            return Err(RuntimeError::KernelError(
                KernelError::RENodeGlobalizeTypeNotAllowed(node_id),
            ));
        }

        // TODO: Authorization

        let mut nodes_to_take = HashSet::new();
        nodes_to_take.insert(node_id);
        let (taken_nodes, missing_nodes) = Self::current_frame_mut(&mut self.call_frames)
            .take_available_values(nodes_to_take, false)?;
        assert!(missing_nodes.is_empty());
        assert!(taken_nodes.len() == 1);
        let root_node = taken_nodes.into_values().nth(0).unwrap();

        for (id, substate) in nodes_to_substates(root_node.to_nodes(node_id)) {
            self.track.put_substate(id, substate);
        }

        Self::current_frame_mut(&mut self.call_frames)
            .node_refs
            .insert(node_id, RENodePointer::Store(node_id));

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::GlobalizeNode,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(())
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

        // Authorization
        if !Self::current_frame(&self.call_frames)
            .actor
            .is_substate_readable(&substate_id)
        {
            return Err(RuntimeError::KernelError(
                KernelError::SubstateReadNotReadable(
                    Self::current_frame(&self.call_frames).actor.clone(),
                    substate_id.clone(),
                ),
            ));
        }

        let (parent_pointer, current_value) =
            Self::read_value_internal(&mut self.call_frames, self.track, &substate_id)?;

        // TODO: Clean the following referencing up
        for component_address in &current_value.refed_component_addresses {
            let node_id = RENodeId::Component(*component_address);
            Self::current_frame_mut(&mut self.call_frames)
                .node_refs
                .insert(node_id, RENodePointer::Store(node_id));
        }

        let cur_children = current_value.node_ids();
        for child_id in cur_children {
            let child_pointer = parent_pointer.child(child_id);
            Self::current_frame_mut(&mut self.call_frames)
                .node_refs
                .insert(child_id, child_pointer);
        }

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::ReadSubstate {
                    value: &current_value,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(current_value)
    }

    fn substate_take(&mut self, substate_id: SubstateId) -> Result<ScryptoValue, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::TakeSubstate {
                    substate_id: &substate_id,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // Authorization
        if !Self::current_frame(&self.call_frames)
            .actor
            .is_substate_writeable(&substate_id)
        {
            return Err(RuntimeError::KernelError(
                KernelError::SubstateWriteNotWriteable(
                    Self::current_frame(&self.call_frames).actor.clone(),
                    substate_id,
                ),
            ));
        }

        let (pointer, current_value) =
            Self::read_value_internal(&mut self.call_frames, self.track, &substate_id)?;
        let cur_children = current_value.node_ids();
        if !cur_children.is_empty() {
            return Err(RuntimeError::KernelError(KernelError::ValueNotAllowed));
        }

        // Write values
        let mut node_ref = pointer.to_ref_mut(&mut self.call_frames, &mut self.track);
        node_ref.replace_value_with_default(&substate_id);

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::TakeSubstate {
                    value: &current_value,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(current_value)
    }

    fn substate_write(
        &mut self,
        substate_id: SubstateId,
        value: ScryptoValue,
    ) -> Result<(), RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::WriteSubstate {
                    substate_id: &substate_id,
                    value: &value,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // Authorization
        if !Self::current_frame(&self.call_frames)
            .actor
            .is_substate_writeable(&substate_id)
        {
            return Err(RuntimeError::KernelError(
                KernelError::SubstateWriteNotWriteable(
                    Self::current_frame(&self.call_frames).actor.clone(),
                    substate_id,
                ),
            ));
        }

        // TODO: Do this in a better way once references cleaned up
        for component_address in &value.refed_component_addresses {
            if !self
                .track
                .is_root(&SubstateId::ComponentInfo(*component_address))
            {
                return Err(RuntimeError::KernelError(KernelError::ValueNotAllowed));
            }
        }

        // Take values from current frame
        let (taken_nodes, missing_nodes) = {
            let node_ids = value.node_ids();
            if !node_ids.is_empty() {
                if !SubstateProperties::can_own_nodes(&substate_id) {
                    return Err(RuntimeError::KernelError(KernelError::ValueNotAllowed));
                }

                Self::current_frame_mut(&mut self.call_frames)
                    .take_available_values(node_ids, true)?
            } else {
                (HashMap::new(), HashSet::new())
            }
        };

        let (pointer, current_value) =
            Self::read_value_internal(&mut self.call_frames, self.track, &substate_id)?;
        let cur_children = current_value.node_ids();

        // Fulfill method
        verify_stored_value_update(&cur_children, &missing_nodes)?;

        // TODO: verify against some schema

        // Write values
        let mut node_ref = pointer.to_ref_mut(&mut self.call_frames, &mut self.track);
        node_ref
            .write_value(substate_id, value, taken_nodes)
            .map_err(|e| RuntimeError::KernelError(KernelError::NodeToSubstateFailure(e)))?;

        for m in &mut self.modules {
            m.post_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallOutput::WriteSubstate,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(())
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

    fn transaction_hash(&mut self) -> Result<Hash, RuntimeError> {
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
