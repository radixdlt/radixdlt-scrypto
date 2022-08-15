use core::marker::PhantomData;

use sbor::rust::boxed::Box;
use sbor::rust::collections::*;
use sbor::rust::format;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::core::Receiver;
use scrypto::core::ScryptoActor;
use scrypto::engine::types::*;
use scrypto::prelude::FnIdentifier;
use scrypto::prelude::NativeFnIdentifier;
use scrypto::prelude::VaultFnIdentifier;
use scrypto::values::*;
use transaction::model::ExecutableInstruction;
use transaction::validation::*;

use crate::engine::*;
use crate::fee::*;
use crate::model::*;
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
    C,  // Fee reserve type
> where
    W: WasmEngine<I>,
    I: WasmInstance,
    C: FeeReserve,
{
    /// The transaction hash
    transaction_hash: Hash,
    /// The max call depth
    max_depth: usize,
    /// Whether to show trace messages
    #[allow(dead_code)] // for no_std
    trace: bool,

    /// State track
    track: &'g mut Track<'s>,
    /// Wasm engine
    wasm_engine: &'g mut W,
    /// Wasm Instrumenter
    wasm_instrumenter: &'g mut WasmInstrumenter,

    /// Fee reserve
    fee_reserve: &'g mut C,
    /// Fee table
    fee_table: &'g FeeTable,

    /// ID allocator
    id_allocator: IdAllocator,

    execution_trace: &'g mut ExecutionTrace,

    /// Call frames
    call_frames: Vec<CallFrame>,

    phantom: PhantomData<I>,
}

impl<'g, 's, W, I, C> Kernel<'g, 's, W, I, C>
where
    W: WasmEngine<I>,
    I: WasmInstance,
    C: FeeReserve,
{
    pub fn new(
        transaction_hash: Hash,
        transaction_signers: Vec<EcdsaPublicKey>,
        is_system: bool,
        max_depth: usize,
        trace: bool,
        track: &'g mut Track<'s>,
        wasm_engine: &'g mut W,
        wasm_instrumenter: &'g mut WasmInstrumenter,
        fee_reserve: &'g mut C,
        fee_table: &'g FeeTable,
        execution_trace: &'g mut ExecutionTrace,
    ) -> Self {
        let frame = CallFrame::new_root(transaction_signers);
        let mut kernel = Self {
            transaction_hash,
            max_depth,
            trace,
            track,
            wasm_engine,
            wasm_instrumenter,
            fee_reserve,
            fee_table,
            id_allocator: IdAllocator::new(IdSpace::Application),
            execution_trace,
            call_frames: vec![frame],
            phantom: PhantomData,
        };

        if is_system {
            let non_fungible_ids = [NonFungibleId::from_u32(0)].into_iter().collect();
            let bucket_id = match kernel
                .node_create(HeapRENode::Bucket(Bucket::new(
                    ResourceContainer::new_non_fungible(SYSTEM_TOKEN, non_fungible_ids),
                )))
                .unwrap()
            {
                RENodeId::Bucket(bucket_id) => bucket_id,
                _ => panic!("Unexpected RENodeID returned"),
            };
            let substate_id = SubstateId::Bucket(bucket_id);
            let mut node_ref = kernel
                .substate_borrow_mut(&substate_id)
                .expect("TODO check this unwrap");
            let bucket = node_ref.bucket();
            let system_proof = bucket
                .create_proof(bucket_id)
                .expect("TODO check this unwrap");
            Self::current_frame_mut(&mut kernel.call_frames)
                .auth_zone
                .proofs
                .push(system_proof);
        }
        kernel
    }

    fn process_call_data(validated: &ScryptoValue) -> Result<(), RuntimeError> {
        if !validated.kv_store_ids.is_empty() {
            return Err(RuntimeError::KeyValueStoreNotAllowed);
        }
        if !validated.vault_ids.is_empty() {
            return Err(RuntimeError::VaultNotAllowed);
        }
        Ok(())
    }

    fn process_return_data(validated: &ScryptoValue) -> Result<(), RuntimeError> {
        if !validated.kv_store_ids.is_empty() {
            return Err(RuntimeError::KeyValueStoreNotAllowed);
        }

        // TODO: Should we disallow vaults to be moved?

        Ok(())
    }

    fn read_value_internal(
        call_frames: &mut Vec<CallFrame>,
        track: &mut Track<'s>,
        substate_id: &SubstateId,
    ) -> Result<(RENodePointer, ScryptoValue), RuntimeError> {
        let node_id = SubstateProperties::get_node_id(substate_id);

        // Get location
        // Note this must be run AFTER values are taken, otherwise there would be inconsistent readable_values state
        let node_pointer = call_frames
            .last()
            .expect("Current frame always exists")
            .node_refs
            .get(&node_id)
            .cloned()
            .ok_or_else(|| RuntimeError::SubstateReadSubstateNotFound(substate_id.clone()))?;

        if matches!(substate_id, SubstateId::ComponentInfo(..)) {
            node_pointer
                .acquire_lock(substate_id.clone(), false, false, track)
                .expect("Should never fail");
        }

        // Read current value
        let current_value = {
            let mut node_ref = node_pointer.to_ref_mut(call_frames, track);
            node_ref.read_scrypto_value(&substate_id)?
        };

        // TODO: Remove, integrate with substate borrow mechanism
        if matches!(substate_id, SubstateId::ComponentInfo(..)) {
            node_pointer.release_lock(substate_id.clone(), false, track);
        }

        Ok((node_pointer.clone(), current_value))
    }

    fn new_uuid(id_allocator: &mut IdAllocator, transaction_hash: Hash) -> u128 {
        id_allocator.new_uuid(transaction_hash).unwrap()
    }

    fn new_node_id(
        id_allocator: &mut IdAllocator,
        transaction_hash: Hash,
        re_node: &HeapRENode,
    ) -> RENodeId {
        match re_node {
            HeapRENode::Bucket(..) => {
                let bucket_id = id_allocator.new_bucket_id().unwrap();
                RENodeId::Bucket(bucket_id)
            }
            HeapRENode::Proof(..) => {
                let proof_id = id_allocator.new_proof_id().unwrap();
                RENodeId::Proof(proof_id)
            }
            HeapRENode::Worktop(..) => RENodeId::Worktop,
            HeapRENode::Vault(..) => {
                let vault_id = id_allocator.new_vault_id(transaction_hash).unwrap();
                RENodeId::Vault(vault_id)
            }
            HeapRENode::KeyValueStore(..) => {
                let kv_store_id = id_allocator.new_kv_store_id(transaction_hash).unwrap();
                RENodeId::KeyValueStore(kv_store_id)
            }
            HeapRENode::Package(..) => {
                // Security Alert: ensure ID allocating will practically never fail
                let package_address = id_allocator.new_package_address(transaction_hash).unwrap();
                RENodeId::Package(package_address)
            }
            HeapRENode::Resource(..) => {
                let resource_address = id_allocator.new_resource_address(transaction_hash).unwrap();
                RENodeId::ResourceManager(resource_address)
            }
            HeapRENode::Component(ref component, ..) => {
                let component_address = id_allocator
                    .new_component_address(
                        transaction_hash,
                        &component.package_address(),
                        component.blueprint_name(),
                    )
                    .unwrap();
                RENodeId::Component(component_address)
            }
            HeapRENode::System(..) => {
                panic!("Should not get here.");
            }
        }
    }

    fn run(
        &mut self,
        auth_zone_frame_id: Option<usize>,
        input: ScryptoValue,
    ) -> Result<(ScryptoValue, HashMap<RENodeId, HeapRootRENode>), RuntimeError> {
        trace!(self, Level::Debug, "Run started!");

        let output = {
            let rtn = match Self::current_frame(&self.call_frames).actor.clone() {
                REActor {
                    receiver,
                    fn_identifier: FnIdentifier::Native(native_fn),
                } => NativeInterpreter::run(receiver, auth_zone_frame_id, native_fn, input, self),
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
                            .read_substate(SubstateId::Package(package_address))
                            .package();
                        self.fee_reserve
                            .consume(
                                self.fee_table.wasm_instantiation_per_byte()
                                    * package.code().len() as u32,
                                "instantiate_wasm",
                            )
                            .map_err(RuntimeError::CostingError)?;
                        let wasm_metering_params = self.fee_table.wasm_metering_params();
                        let instrumented_code = self
                            .wasm_instrumenter
                            .instrument(package.code(), &wasm_metering_params);
                        let mut instance = self.wasm_engine.instantiate(instrumented_code);
                        let blueprint_abi = package
                            .blueprint_abi(&blueprint_name)
                            .expect("Blueprint should exist");
                        let export_name = &blueprint_abi
                            .get_fn_abi(&ident)
                            .unwrap()
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
                                return Err(RuntimeError::MethodDoesNotExist(
                                    FnIdentifier::Scrypto {
                                        package_address,
                                        blueprint_name,
                                        ident,
                                    },
                                ))
                            }
                        };
                        let mut runtime: Box<dyn WasmRuntime> =
                            Box::new(RadixEngineWasmRuntime::new(scrypto_actor, self));
                        instance
                            .invoke_export(&export_name, &input, &mut runtime)
                            .map_err(|e| match e {
                                // Flatten error code for more readable transaction receipt
                                InvokeError::RuntimeError(e) => e,
                                e @ _ => RuntimeError::InvokeError(e.into()),
                            })?
                    };

                    let package = self
                        .track
                        .read_substate(SubstateId::Package(package_address))
                        .package();
                    let blueprint_abi = package
                        .blueprint_abi(&blueprint_name)
                        .expect("Blueprint should exist");
                    let fn_abi = blueprint_abi.get_fn_abi(&ident).unwrap();
                    if !fn_abi.output.matches(&output.dom) {
                        Err(RuntimeError::InvalidFnOutput {
                            fn_identifier: FnIdentifier::Scrypto {
                                package_address,
                                blueprint_name,
                                ident,
                            },
                            output: output.dom,
                        })
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
            return Err(RuntimeError::RENodeNotFound(missing_node));
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
                return Err(RuntimeError::InvokeMethodInvalidReferencePass(node_id));
            }
        }

        // drop proofs and check resource leak
        Self::current_frame_mut(&mut self.call_frames)
            .auth_zone
            .clear();
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

impl<'g, 's, W, I, C> SystemApi<'s, W, I, C> for Kernel<'g, 's, W, I, C>
where
    W: WasmEngine<I>,
    I: WasmInstance,
    C: FeeReserve,
{
    fn invoke_function(
        &mut self,
        fn_identifier: FnIdentifier,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        trace!(
            self,
            Level::Debug,
            "Invoking function: {:?}",
            &fn_identifier
        );

        if Self::current_frame(&self.call_frames).depth == self.max_depth {
            return Err(RuntimeError::MaxCallDepthLimitReached);
        }

        self.fee_reserve
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::InvokeFunction {
                        fn_identifier: fn_identifier.clone(),
                        input: &input,
                    }),
                "invoke_function",
            )
            .map_err(RuntimeError::CostingError)?;

        self.fee_reserve
            .consume(
                self.fee_table.run_method_cost(None, &fn_identifier, &input),
                "run_function",
            )
            .map_err(RuntimeError::CostingError)?;

        // Prevent vaults/kvstores from being moved
        Self::process_call_data(&input)?;

        // Figure out what buckets and proofs to move from this process
        let values_to_take = input.node_ids();
        let (taken_values, mut missing) = Self::current_frame_mut(&mut self.call_frames)
            .take_available_values(values_to_take, false)?;
        let first_missing_value = missing.drain().nth(0);
        if let Some(missing_value) = first_missing_value {
            return Err(RuntimeError::RENodeNotFound(missing_value));
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
                self.track
                    .acquire_lock(SubstateId::Package(package_address.clone()), false, false)
                    .map_err(|e| match e {
                        TrackError::NotFound => RuntimeError::PackageNotFound(*package_address),
                        TrackError::Reentrancy => {
                            panic!("Package reentrancy error should never occur.")
                        }
                        TrackError::StateTrackError(..) => panic!("Unexpected"),
                    })?;
                locked_values.insert(SubstateId::Package(package_address.clone()));
                let package = self
                    .track
                    .read_substate(SubstateId::Package(package_address.clone()))
                    .package();
                let abi = package.blueprint_abi(blueprint_name).ok_or(
                    RuntimeError::BlueprintNotFound(
                        package_address.clone(),
                        blueprint_name.clone(),
                    ),
                )?;
                let fn_abi = abi
                    .get_fn_abi(ident)
                    .ok_or(RuntimeError::MethodDoesNotExist(fn_identifier.clone()))?;
                if !fn_abi.input.matches(&input.dom) {
                    return Err(RuntimeError::InvalidFnInput { fn_identifier });
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
            let input: TransactionProcessorRunInput = scrypto_decode(&input.raw).unwrap();
            for instruction in &input.instructions {
                match instruction {
                    ExecutableInstruction::CallFunction { arg, .. }
                    | ExecutableInstruction::CallMethod { arg, .. } => {
                        let scrypto_value = ScryptoValue::from_slice(&arg).unwrap();
                        component_addresses.extend(scrypto_value.refed_component_addresses);
                    }
                    _ => {}
                }
            }

            // Make components visible
            for component_address in component_addresses {
                let node_id = RENodeId::Component(component_address);
                let substate_id = SubstateId::ComponentInfo(component_address);
                let node_pointer = RENodePointer::Store(node_id);
                // Check if component exists
                node_pointer.acquire_lock(substate_id.clone(), false, false, &mut self.track)?;
                node_pointer.release_lock(substate_id, false, &mut self.track);
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
                    return Err(RuntimeError::InvokeMethodInvalidReferencePass(node_id));
                }
            }
        }

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
            self.run(None, input)?
        };

        // Remove the last after clean-up
        self.call_frames.pop();

        // Release locked addresses
        for l in locked_values {
            // TODO: refactor after introducing `Lock` representation.
            self.track.release_lock(l.clone(), false);
        }

        // move buckets and proofs to this process.
        for (id, value) in received_values {
            trace!(self, Level::Debug, "Received value: {:?}", value);
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

        trace!(self, Level::Debug, "Invoking finished!");
        Ok(output)
    }

    fn invoke_method(
        &mut self,
        receiver: Receiver,
        fn_identifier: FnIdentifier,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        trace!(
            self,
            Level::Debug,
            "Invoking method: {:?} {:?}",
            receiver,
            &fn_identifier
        );

        if Self::current_frame(&self.call_frames).depth == self.max_depth {
            return Err(RuntimeError::MaxCallDepthLimitReached);
        }

        self.fee_reserve
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::InvokeMethod {
                        receiver: receiver.clone(),
                        input: &input,
                    }),
                "invoke_method",
            )
            .map_err(RuntimeError::CostingError)?;

        self.fee_reserve
            .consume(
                self.fee_table
                    .run_method_cost(Some(receiver), &fn_identifier, &input),
                "run_method",
            )
            .map_err(RuntimeError::CostingError)?;

        // Prevent vaults/kvstores from being moved
        Self::process_call_data(&input)?;

        // Figure out what buckets and proofs to move from this process
        let values_to_take = input.node_ids();
        let (taken_values, mut missing) = Self::current_frame_mut(&mut self.call_frames)
            .take_available_values(values_to_take, false)?;
        let first_missing_value = missing.drain().nth(0);
        if let Some(missing_value) = first_missing_value {
            return Err(RuntimeError::RENodeNotFound(missing_value));
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
        let auth_zone_frame_id = match &receiver {
            Receiver::Ref(node_id) | Receiver::Consumed(node_id) => {
                // Find node
                let current_frame = Self::current_frame(&self.call_frames);
                let node_pointer = if current_frame.owned_heap_nodes.contains_key(&node_id) {
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
                        RENodeId::ResourceManager(..) | RENodeId::System => {
                            RENodePointer::Store(*node_id)
                        }
                        _ => return Err(RuntimeError::InvokeMethodInvalidReceiver(*node_id)),
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
                    return Err(RuntimeError::LockFeeError(LockFeeError::RENodeNotInTrack));
                }
                node_pointer.acquire_lock(
                    substate_id.clone(),
                    true,
                    is_lock_fee,
                    &mut self.track,
                )?;
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
                            node_pointer.acquire_lock(
                                temporary_substate_id.clone(),
                                false,
                                false,
                                &mut self.track,
                            )?;
                            temporary_locks.push((node_pointer, temporary_substate_id, false));

                            let node_ref = node_pointer.to_ref(&self.call_frames, &mut self.track);
                            let component = node_ref.component_info();

                            // Don't support traits yet
                            if !package_address.eq(&component.package_address()) {
                                return Err(RuntimeError::MethodDoesNotExist(fn_identifier));
                            }
                            if !blueprint_name.eq(component.blueprint_name()) {
                                return Err(RuntimeError::MethodDoesNotExist(fn_identifier));
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
                            let node_ref = node_pointer.to_ref(&self.call_frames, &self.track);
                            node_ref.component_info().package_address()
                        };
                        let package_substate_id = SubstateId::Package(package_address);
                        let package_node_id = RENodeId::Package(package_address);
                        let package_node_pointer = RENodePointer::Store(package_node_id);
                        package_node_pointer.acquire_lock(
                            package_substate_id.clone(),
                            false,
                            false,
                            &mut self.track,
                        )?;
                        locked_pointers.push((
                            package_node_pointer,
                            package_substate_id.clone(),
                            false,
                        ));
                        next_frame_node_refs.insert(package_node_id, package_node_pointer);
                    }
                    RENodeId::Bucket(..) => {
                        let resource_address = {
                            let node_ref = node_pointer.to_ref(&self.call_frames, &self.track);
                            node_ref.bucket().resource_address()
                        };
                        let resource_substate_id = SubstateId::ResourceManager(resource_address);
                        let resource_node_id = RENodeId::ResourceManager(resource_address);
                        let resource_node_pointer = RENodePointer::Store(resource_node_id);
                        resource_node_pointer.acquire_lock(
                            resource_substate_id.clone(),
                            true,
                            false,
                            &mut self.track,
                        )?;
                        locked_pointers.push((resource_node_pointer, resource_substate_id, false));
                        next_frame_node_refs.insert(resource_node_id, resource_node_pointer);
                    }
                    RENodeId::Vault(..) => {
                        let resource_address = {
                            let node_ref = node_pointer.to_ref(&self.call_frames, &self.track);
                            node_ref.vault().resource_address()
                        };
                        let resource_substate_id = SubstateId::ResourceManager(resource_address);
                        let resource_node_id = RENodeId::ResourceManager(resource_address);
                        let resource_node_pointer = RENodePointer::Store(resource_node_id);
                        resource_node_pointer.acquire_lock(
                            resource_substate_id.clone(),
                            true,
                            false,
                            &mut self.track,
                        )?;
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
                        resource_node_pointer.acquire_lock(
                            resource_substate_id.clone(),
                            false,
                            false,
                            &mut self.track,
                        )?;
                        locked_pointers.push((resource_node_pointer, resource_substate_id, false));
                        next_frame_node_refs.insert(resource_node_id, resource_node_pointer);
                    }
                }

                ExecutionTraceModule::trace_invoke_method(
                    &self.call_frames,
                    &self.track,
                    &current_frame.actor,
                    &fn_identifier,
                    node_id,
                    node_pointer,
                    &input,
                    &next_owned_values,
                    &mut self.execution_trace,
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
                            .ok_or(RuntimeError::InvokeMethodInvalidReceiver(*node_id))?;
                        next_owned_values.insert(*node_id, heap_node);
                    }
                    _ => {}
                }

                for (node_pointer, substate_id, write_through) in temporary_locks {
                    node_pointer.release_lock(substate_id, write_through, &mut self.track);
                }

                next_frame_node_refs.insert(node_id.clone(), node_pointer.clone());
                None
            }
            Receiver::CurrentAuthZone => {
                for resource_address in &input.resource_addresses {
                    let resource_substate_id =
                        SubstateId::ResourceManager(resource_address.clone());
                    let resource_node_id = RENodeId::ResourceManager(resource_address.clone());
                    let resource_node_pointer = RENodePointer::Store(resource_node_id);
                    resource_node_pointer.acquire_lock(
                        resource_substate_id.clone(),
                        false,
                        false,
                        &mut self.track,
                    )?;
                    locked_pointers.push((resource_node_pointer, resource_substate_id, false));
                    next_frame_node_refs.insert(resource_node_id, resource_node_pointer);
                }
                Some(Self::current_frame(&self.call_frames).depth)
            }
        };

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
                return Err(RuntimeError::InvokeMethodInvalidReferencePass(node_id));
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
            self.run(auth_zone_frame_id, input)?
        };

        // Remove the last after clean-up
        self.call_frames.pop();

        // Release locked addresses
        for (node_pointer, substate_id, write_through) in locked_pointers {
            // TODO: refactor after introducing `Lock` representation.
            node_pointer.release_lock(substate_id, write_through, &mut self.track);
        }

        // move buckets and proofs to this process.
        for (id, value) in received_values {
            trace!(self, Level::Debug, "Received value: {:?}", value);
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

        trace!(self, Level::Debug, "Invoking finished!");
        Ok(output)
    }

    fn borrow_node(&mut self, node_id: &RENodeId) -> Result<RENodeRef<'_, 's>, FeeReserveError> {
        trace!(self, Level::Debug, "Borrowing value: {:?}", node_id);

        self.fee_reserve.consume(
            self.fee_table.system_api_cost({
                match node_id {
                    RENodeId::Bucket(_) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: true,
                        size: 0,
                    },
                    RENodeId::Proof(_) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: true,
                        size: 0,
                    },
                    RENodeId::Worktop => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: true,
                        size: 0,
                    },
                    RENodeId::Vault(_) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::Component(_) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::KeyValueStore(_) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::ResourceManager(_) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::Package(_) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::System => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                }
            }),
            "borrow_substate",
        )?;

        let node_pointer = Self::current_frame(&self.call_frames)
            .node_refs
            .get(node_id)
            .expect(&format!("{:?} is unknown.", node_id));

        Ok(node_pointer.to_ref(&self.call_frames, &self.track))
    }

    fn substate_borrow_mut(
        &mut self,
        substate_id: &SubstateId,
    ) -> Result<NativeSubstateRef, FeeReserveError> {
        trace!(
            self,
            Level::Debug,
            "Borrowing substate (mut): {:?}",
            substate_id
        );

        // Costing
        self.fee_reserve.consume(
            self.fee_table.system_api_cost({
                match substate_id {
                    SubstateId::Bucket(_) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: true,
                        size: 0,
                    },
                    SubstateId::Proof(_) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: true,
                        size: 0,
                    },
                    SubstateId::Worktop => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: true,
                        size: 0,
                    },
                    SubstateId::Vault(_) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::ComponentState(..) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::ComponentInfo(..) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::KeyValueStoreSpace(_) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::KeyValueStoreEntry(..) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::ResourceManager(..) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::NonFungibleSpace(..) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::NonFungible(..) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::Package(..) => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::System => SystemApiCostingEntry::BorrowSubstate {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                }
            }),
            "borrow_substate",
        )?;

        // Authorization
        if !Self::current_frame(&self.call_frames)
            .actor
            .is_substate_readable(substate_id)
        {
            panic!("Trying to read substate which is not visible.")
        }

        let node_id = SubstateProperties::get_node_id(substate_id);

        // TODO: Clean this up
        let frame = Self::current_frame(&self.call_frames);
        let node_pointer = if frame.owned_heap_nodes.contains_key(&node_id) {
            RENodePointer::Heap {
                frame_id: frame.depth,
                root: node_id.clone(),
                id: None,
            }
        } else {
            Self::current_frame(&self.call_frames)
                .node_refs
                .get(&node_id)
                .cloned()
                .expect(&format!("Node should exist {:?}", node_id))
        };

        Ok(node_pointer.borrow_native_ref(
            substate_id.clone(),
            &mut self.call_frames,
            &mut self.track,
        ))
    }

    fn substate_return_mut(&mut self, val_ref: NativeSubstateRef) -> Result<(), FeeReserveError> {
        trace!(self, Level::Debug, "Returning value");

        self.fee_reserve.consume(
            self.fee_table.system_api_cost({
                match &val_ref {
                    NativeSubstateRef::Stack(..) => {
                        SystemApiCostingEntry::ReturnSubstate { size: 0 }
                    }
                    NativeSubstateRef::Track(substate_id, _) => match substate_id {
                        SubstateId::Vault(_) => SystemApiCostingEntry::ReturnSubstate { size: 0 },
                        SubstateId::KeyValueStoreSpace(_) => {
                            SystemApiCostingEntry::ReturnSubstate { size: 0 }
                        }
                        SubstateId::KeyValueStoreEntry(_, _) => {
                            SystemApiCostingEntry::ReturnSubstate { size: 0 }
                        }
                        SubstateId::ResourceManager(_) => {
                            SystemApiCostingEntry::ReturnSubstate { size: 0 }
                        }
                        SubstateId::Package(_) => SystemApiCostingEntry::ReturnSubstate { size: 0 },
                        SubstateId::NonFungibleSpace(_) => {
                            SystemApiCostingEntry::ReturnSubstate { size: 0 }
                        }
                        SubstateId::NonFungible(_, _) => {
                            SystemApiCostingEntry::ReturnSubstate { size: 0 }
                        }
                        SubstateId::ComponentInfo(..) => {
                            SystemApiCostingEntry::ReturnSubstate { size: 0 }
                        }
                        SubstateId::ComponentState(_) => {
                            SystemApiCostingEntry::ReturnSubstate { size: 0 }
                        }
                        SubstateId::System => SystemApiCostingEntry::ReturnSubstate { size: 0 },
                        SubstateId::Bucket(..) => SystemApiCostingEntry::ReturnSubstate { size: 0 },
                        SubstateId::Proof(..) => SystemApiCostingEntry::ReturnSubstate { size: 0 },
                        SubstateId::Worktop => SystemApiCostingEntry::ReturnSubstate { size: 0 },
                    },
                }
            }),
            "return_substate",
        )?;

        val_ref.return_to_location(&mut self.call_frames, &mut self.track);
        Ok(())
    }

    fn node_drop(&mut self, node_id: &RENodeId) -> Result<HeapRootRENode, FeeReserveError> {
        trace!(self, Level::Debug, "Dropping value: {:?}", node_id);

        self.fee_reserve.consume(
            self.fee_table
                .system_api_cost(SystemApiCostingEntry::DropNode { size: 0 }),
            "drop_node",
        )?;

        // TODO: Authorization

        Ok(Self::current_frame_mut(&mut self.call_frames)
            .owned_heap_nodes
            .remove(&node_id)
            .unwrap())
    }

    fn node_create(&mut self, re_node: HeapRENode) -> Result<RENodeId, RuntimeError> {
        trace!(self, Level::Debug, "Creating value");

        // Costing
        self.fee_reserve
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::CreateNode {
                        size: 0, // TODO: get size of the value
                    }),
                "create_node",
            )
            .map_err(RuntimeError::CostingError)?;

        // TODO: Authorization

        // Take any required child nodes
        let children = re_node.get_child_nodes()?;
        let (taken_root_nodes, mut missing) =
            Self::current_frame_mut(&mut self.call_frames).take_available_values(children, true)?;
        let first_missing_node = missing.drain().nth(0);
        if let Some(missing_node) = first_missing_node {
            return Err(RuntimeError::RENodeCreateNodeNotFound(missing_node));
        }
        let mut child_nodes = HashMap::new();
        for (id, taken_root_node) in taken_root_nodes {
            child_nodes.extend(taken_root_node.to_nodes(id));
        }

        // Insert node into heap
        let node_id = Self::new_node_id(&mut self.id_allocator, self.transaction_hash, &re_node);
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
                    .expect("Current frame always exists");
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
                    .expect("Current frame always exists");
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

        Ok(node_id)
    }

    fn node_globalize(&mut self, node_id: RENodeId) -> Result<(), RuntimeError> {
        trace!(self, Level::Debug, "Globalizing value: {:?}", node_id);

        // Costing
        self.fee_reserve
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::GlobalizeNode {
                        size: 0, // TODO: get size of the value
                    }),
                "globalize_node",
            )
            .map_err(RuntimeError::CostingError)?;

        if !RENodeProperties::can_globalize(node_id) {
            return Err(RuntimeError::RENodeGlobalizeTypeNotAllowed(node_id));
        }

        // TODO: Authorization

        let mut nodes_to_take = HashSet::new();
        nodes_to_take.insert(node_id);
        let (taken_nodes, missing_nodes) = Self::current_frame_mut(&mut self.call_frames)
            .take_available_values(nodes_to_take, false)?;
        assert!(missing_nodes.is_empty());
        assert!(taken_nodes.len() == 1);
        let root_node = taken_nodes.into_values().nth(0).unwrap();

        let (substates, maybe_non_fungibles) = match root_node.root {
            HeapRENode::Component(component, component_state) => {
                let mut substates = HashMap::new();
                let component_address = node_id.into();
                substates.insert(
                    SubstateId::ComponentInfo(component_address),
                    Substate::ComponentInfo(component),
                );
                substates.insert(
                    SubstateId::ComponentState(component_address),
                    Substate::ComponentState(component_state),
                );
                let mut visible_substates = HashSet::new();
                visible_substates.insert(SubstateId::ComponentInfo(component_address));
                (substates, None)
            }
            HeapRENode::Package(package) => {
                let mut substates = HashMap::new();
                let package_address = node_id.into();
                substates.insert(
                    SubstateId::Package(package_address),
                    Substate::Package(package),
                );
                (substates, None)
            }
            HeapRENode::Resource(resource_manager, non_fungibles) => {
                let mut substates = HashMap::new();
                let resource_address: ResourceAddress = node_id.into();
                substates.insert(
                    SubstateId::ResourceManager(resource_address),
                    Substate::Resource(resource_manager),
                );
                (substates, non_fungibles)
            }
            _ => panic!("Not expected"),
        };

        for (substate_id, substate) in substates {
            self.track
                .create_uuid_substate(substate_id.clone(), substate);
        }

        let mut to_store_values = HashMap::new();
        for (id, value) in root_node.child_nodes.into_iter() {
            to_store_values.insert(id, value);
        }
        insert_non_root_nodes(self.track, to_store_values);

        if let Some(non_fungibles) = maybe_non_fungibles {
            let resource_address: ResourceAddress = node_id.into();
            let parent_address = SubstateId::NonFungibleSpace(resource_address.clone());
            for (id, non_fungible) in non_fungibles {
                self.track.set_key_value(
                    parent_address.clone(),
                    id.to_vec(),
                    Substate::NonFungible(NonFungibleWrapper(Some(non_fungible))),
                );
            }
        }

        Self::current_frame_mut(&mut self.call_frames)
            .node_refs
            .insert(node_id, RENodePointer::Store(node_id));

        Ok(())
    }

    fn substate_read(&mut self, substate_id: SubstateId) -> Result<ScryptoValue, RuntimeError> {
        trace!(
            self,
            Level::Debug,
            "Reading substate data: {:?}",
            substate_id
        );

        // Costing
        self.fee_reserve
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::ReadSubstate {
                        size: 0, // TODO: get size of the value
                    }),
                "read_substate",
            )
            .map_err(RuntimeError::CostingError)?;

        // Authorization
        if !Self::current_frame(&self.call_frames)
            .actor
            .is_substate_readable(&substate_id)
        {
            return Err(RuntimeError::SubstateReadNotReadable(
                Self::current_frame(&self.call_frames).actor.clone(),
                substate_id.clone(),
            ));
        }

        let (parent_pointer, current_value) =
            Self::read_value_internal(&mut self.call_frames, self.track, &substate_id)?;
        let cur_children = current_value.node_ids();
        for child_id in cur_children {
            let child_pointer = parent_pointer.child(child_id);
            Self::current_frame_mut(&mut self.call_frames)
                .node_refs
                .insert(child_id, child_pointer);
        }
        Ok(current_value)
    }

    fn substate_take(&mut self, substate_id: SubstateId) -> Result<ScryptoValue, RuntimeError> {
        trace!(self, Level::Debug, "Taking substate: {:?}", substate_id);

        // Costing
        self.fee_reserve
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::TakeSubstate {
                        size: 0, // TODO: get size of the value
                    }),
                "read_substate",
            )
            .map_err(RuntimeError::CostingError)?;

        // Authorization
        if !Self::current_frame(&self.call_frames)
            .actor
            .is_substate_writeable(&substate_id)
        {
            return Err(RuntimeError::SubstateWriteNotWriteable(
                Self::current_frame(&self.call_frames).actor.clone(),
                substate_id,
            ));
        }

        let (pointer, current_value) =
            Self::read_value_internal(&mut self.call_frames, self.track, &substate_id)?;
        let cur_children = current_value.node_ids();
        if !cur_children.is_empty() {
            return Err(RuntimeError::ValueNotAllowed);
        }

        // Write values
        let mut node_ref = pointer.to_ref_mut(&mut self.call_frames, &mut self.track);
        node_ref.replace_value_with_default(&substate_id);

        Ok(current_value)
    }

    fn substate_write(
        &mut self,
        substate_id: SubstateId,
        value: ScryptoValue,
    ) -> Result<(), RuntimeError> {
        trace!(
            self,
            Level::Debug,
            "Writing substate data: {:?}",
            substate_id
        );

        // Costing
        self.fee_reserve
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::WriteSubstate {
                        size: 0, // TODO: get size of the value
                    }),
                "write_substate",
            )
            .map_err(RuntimeError::CostingError)?;

        // Authorization
        if !Self::current_frame(&self.call_frames)
            .actor
            .is_substate_writeable(&substate_id)
        {
            return Err(RuntimeError::SubstateWriteNotWriteable(
                Self::current_frame(&self.call_frames).actor.clone(),
                substate_id,
            ));
        }

        // If write, take values from current frame
        let (taken_nodes, missing_nodes) = {
            let node_ids = value.node_ids();
            if !node_ids.is_empty() {
                if !SubstateProperties::can_own_nodes(&substate_id) {
                    return Err(RuntimeError::ValueNotAllowed);
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
        node_ref.write_value(substate_id, value, taken_nodes);

        Ok(())
    }

    fn transaction_hash(&mut self) -> Result<Hash, FeeReserveError> {
        self.fee_reserve.consume(
            self.fee_table
                .system_api_cost(SystemApiCostingEntry::ReadTransactionHash),
            "read_transaction_hash",
        )?;
        Ok(self.transaction_hash)
    }

    fn generate_uuid(&mut self) -> Result<u128, FeeReserveError> {
        self.fee_reserve.consume(
            self.fee_table
                .system_api_cost(SystemApiCostingEntry::GenerateUuid),
            "generate_uuid",
        )?;
        Ok(Self::new_uuid(
            &mut self.id_allocator,
            self.transaction_hash,
        ))
    }

    fn emit_log(&mut self, level: Level, message: String) -> Result<(), FeeReserveError> {
        self.fee_reserve.consume(
            self.fee_table
                .system_api_cost(SystemApiCostingEntry::EmitLog {
                    size: message.len() as u32,
                }),
            "emit_log",
        )?;
        self.track.add_log(level, message);
        Ok(())
    }

    fn check_access_rule(
        &mut self,
        access_rule: scrypto::resource::AccessRule,
        proof_ids: Vec<ProofId>,
    ) -> Result<bool, RuntimeError> {
        // Costing
        self.fee_reserve
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::CheckAccessRule {
                        size: proof_ids.len() as u32,
                    }),
                "check_access_rule",
            )
            .map_err(RuntimeError::CostingError)?;

        let proofs = proof_ids
            .iter()
            .map(|proof_id| {
                Self::current_frame(&self.call_frames)
                    .owned_heap_nodes
                    .get(&RENodeId::Proof(*proof_id))
                    .map(|p| match p.root() {
                        HeapRENode::Proof(proof) => proof.clone(),
                        _ => panic!("Expected proof"),
                    })
                    .ok_or(RuntimeError::ProofNotFound(proof_id.clone()))
            })
            .collect::<Result<Vec<Proof>, RuntimeError>>()?;
        let mut simulated_auth_zone = AuthZone::new_with_proofs(proofs);

        let method_authorization = convert(&Type::Unit, &Value::Unit, &access_rule);
        let is_authorized = method_authorization.check(&[&simulated_auth_zone]).is_ok();
        simulated_auth_zone.clear();

        Ok(is_authorized)
    }

    fn fee_reserve(&mut self) -> &mut C {
        self.fee_reserve
    }

    fn auth_zone(&mut self, frame_id: usize) -> &mut AuthZone {
        &mut self.call_frames.get_mut(frame_id).unwrap().auth_zone
    }
}
