use core::marker::PhantomData;

use sbor::rust::collections::*;
use sbor::rust::format;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::core::Receiver;
use scrypto::engine::types::*;
use scrypto::prelude::{ScryptoActor, TypeName};
use scrypto::resource::AuthZoneClearInput;
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
            // TODO: add ident
            println!("[{:5}] {}", $level, sbor::rust::format!($msg, $( $arg ),*));
        }
    };
}

pub struct Kernel<
    'p, // Parent lifetime
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
    /// Whether running in sudo mode
    is_system: bool,
    /// The max call depth
    max_depth: usize,
    /// Whether to show trace messages
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
    /// Call frames
    call_frames: Vec<CallFrame<'p>>,

    phantom: PhantomData<I>,
}

impl<'p, 'g, 's, W, I, C> Kernel<'p, 'g, 's, W, I, C>
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
    ) -> Self {
        let mut kernel = Self {
            transaction_hash,
            is_system,
            max_depth,
            trace,
            track,
            wasm_engine,
            wasm_instrumenter,
            fee_reserve,
            fee_table,
            id_allocator: IdAllocator::new(IdSpace::Application),
            call_frames: vec![],
            phantom: PhantomData,
        };

        let frame = CallFrame::new_root(transaction_signers, is_system, &mut kernel);
        kernel.call_frames.push(frame);

        kernel
    }

    fn current_frame<'a>(frames: &'a Vec<CallFrame<'p>>) -> &'a CallFrame<'p> {
        frames.last().expect("Current frame always exists")
    }

    fn current_frame_mut<'a>(frames: &'a mut Vec<CallFrame<'p>>) -> &'a mut CallFrame<'p> {
        frames.last_mut().expect("Current frame always exists")
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

    pub fn read_value_internal(
        current_frame: &mut CallFrame<'p>,
        track: &mut Track<'s>,
        substate_id: &SubstateId,
    ) -> Result<(RENodePointer, ScryptoValue), RuntimeError> {
        let node_id = SubstateProperties::get_node_id(substate_id);

        // Get location
        // Note this must be run AFTER values are taken, otherwise there would be inconsistent readable_values state
        let node_pointer = current_frame
            .node_refs
            .get(&node_id)
            .cloned()
            .ok_or_else(|| RuntimeError::SubstateReadSubstateNotFound(substate_id.clone()))?;

        if matches!(substate_id, SubstateId::ComponentInfo(..))
            && matches!(node_pointer, RENodePointer::Store(..))
        {
            track
                .acquire_lock(substate_id.clone(), false, false)
                .expect("Should never fail");
        }

        // Read current value
        let current_value = {
            let mut node_ref = node_pointer.to_ref_mut(
                current_frame.depth,
                &mut current_frame.owned_heap_nodes,
                &mut current_frame.parent_heap_nodes,
                track,
            );
            node_ref.read_scrypto_value(&substate_id)?
        };

        // TODO: Remove, integrate with substate borrow mechanism
        if matches!(substate_id, SubstateId::ComponentInfo(..))
            && matches!(node_pointer, RENodePointer::Store(..))
        {
            track.release_lock(substate_id.clone(), false);
        }

        Ok((node_pointer.clone(), current_value))
    }
}

impl<'p, 'g, 's, W, I, C> SystemApi<'s, W, I, C> for Kernel<'p, 'g, 's, W, I, C>
where
    W: WasmEngine<I>,
    I: WasmInstance,
    C: FeeReserve,
{
    fn invoke_function(
        &mut self,
        type_name: TypeName,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        trace!(
            self,
            Level::Debug,
            "Invoking function: {:?} {:?}",
            type_name,
            &fn_ident
        );

        if self.call_frames.len() == self.max_depth {
            return Err(RuntimeError::MaxCallDepthLimitReached);
        }

        self.fee_reserve
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::InvokeFunction {
                        type_name: type_name.clone(),
                        input: &input,
                    }),
                "invoke_function",
            )
            .map_err(RuntimeError::CostingError)?;

        self.fee_reserve
            .consume(
                self.fee_table
                    .run_function_cost(&type_name, fn_ident.as_str(), &input),
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
        let actor = match &type_name {
            TypeName::Blueprint(package_address, blueprint_name) => {
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
                    .get_fn_abi(&fn_ident)
                    .ok_or(RuntimeError::MethodDoesNotExist(fn_ident.clone()))?;
                if !fn_abi.input.matches(&input.dom) {
                    return Err(RuntimeError::InvalidFnInput { fn_ident });
                }

                REActor::Scrypto(ScryptoActor::blueprint(
                    *package_address,
                    blueprint_name.clone(),
                ))
            }
            TypeName::Package | TypeName::ResourceManager | TypeName::TransactionProcessor => {
                REActor::Native
            }
        };

        // Move this into higher layer, e.g. transaction processor
        let mut next_frame_node_refs = HashMap::new();
        if self.call_frames.len() == 0 {
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
                // TODO: Check if component exists
                let node_id = RENodeId::Component(component_address);
                next_frame_node_refs.insert(node_id, RENodePointer::Store(node_id));
            }
        } else {
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
        }

        // Setup next parent frame
        let mut next_borrowed_values: Vec<&mut HashMap<RENodeId, HeapRootRENode>> = Vec::new();
        let current_frame = Self::current_frame_mut(&mut self.call_frames);
        for parent_values in &mut current_frame.parent_heap_nodes {
            next_borrowed_values.push(parent_values);
        }
        next_borrowed_values.push(&mut current_frame.owned_heap_nodes);

        // start a new frame
        let (result, received_values) = {
            let mut frame = CallFrame::new(
                current_frame.depth + 1,
                actor,
                match type_name {
                    TypeName::TransactionProcessor | TypeName::Blueprint(_, _) => {
                        Some(AuthZone::new())
                    }
                    _ => None,
                },
                next_owned_values,
                next_frame_node_refs,
                next_borrowed_values,
                current_frame.auth_zone.as_ref(),
            );

            // invoke the main function
            frame.run(ExecutionEntity::Function(type_name), &fn_ident, input, self)?
        };

        // Process return data
        Self::process_return_data(&result)?;

        // Release locked addresses
        for l in locked_values {
            // TODO: refactor after introducing `Lock` representation.
            self.track.release_lock(l.clone(), false);
        }

        // move buckets and proofs to this process.
        for (id, value) in received_values {
            trace!(self, Level::Debug, "Received value: {:?}", value);
            current_frame.owned_heap_nodes.insert(id, value);
        }

        // Accept component references
        for refed_component_address in &result.refed_component_addresses {
            let node_id = RENodeId::Component(*refed_component_address);
            let mut visible = HashSet::new();
            visible.insert(SubstateId::ComponentInfo(*refed_component_address));
            current_frame
                .node_refs
                .insert(node_id, RENodePointer::Store(node_id));
        }

        trace!(self, Level::Debug, "Invoking finished!");
        Ok(result)
    }

    fn invoke_method(
        &mut self,
        receiver: Receiver,
        fn_ident: String,
        input: ScryptoValue,
    ) -> Result<ScryptoValue, RuntimeError> {
        trace!(
            self,
            Level::Debug,
            "Invoking method: {:?} {:?}",
            receiver,
            &fn_ident
        );

        let current_frame = Self::current_frame_mut(&mut self.call_frames);
        if current_frame.depth == self.max_depth {
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
                    .run_method_cost(&receiver, fn_ident.as_str(), &input),
                "run_method",
            )
            .map_err(RuntimeError::CostingError)?;

        // Prevent vaults/kvstores from being moved
        Self::process_call_data(&input)?;

        // Figure out what buckets and proofs to move from this process
        let values_to_take = input.node_ids();
        let (taken_values, mut missing) =
            current_frame.take_available_values(values_to_take, false)?;
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

        let mut locked_values = HashSet::new();
        let mut next_frame_node_refs = HashMap::new();
        // TODO: Remove once heap is implemented
        let next_caller_auth_zone;

        // Authorization and state load
        let (actor, execution_state) = match &receiver {
            Receiver::Consumed(node_id) => {
                let native_substate_id = match node_id {
                    RENodeId::Bucket(bucket_id) => SubstateId::Bucket(*bucket_id),
                    RENodeId::Proof(proof_id) => SubstateId::Proof(*proof_id),
                    _ => return Err(RuntimeError::MethodDoesNotExist(fn_ident.clone())),
                };

                let heap_node = current_frame
                    .owned_heap_nodes
                    .remove(node_id)
                    .ok_or(RuntimeError::RENodeNotFound(*node_id))?;

                // Lock Additional Substates
                match heap_node.root() {
                    HeapRENode::Bucket(bucket) => {
                        let resource_address = bucket.resource_address();
                        self.track
                            .acquire_lock(
                                SubstateId::ResourceManager(resource_address),
                                true,
                                false,
                            )
                            .expect("Should not fail.");
                        locked_values
                            .insert((SubstateId::ResourceManager(resource_address.clone()), false));
                        let node_id = RENodeId::ResourceManager(resource_address);
                        next_frame_node_refs.insert(node_id, RENodePointer::Store(node_id));
                    }
                    _ => {}
                }

                AuthModule::consumed_auth(
                    &fn_ident,
                    &native_substate_id,
                    heap_node.root(),
                    &mut self.track,
                    current_frame.auth_zone.as_ref(),
                    current_frame.caller_auth_zone,
                )?;
                next_owned_values.insert(*node_id, heap_node);
                next_caller_auth_zone = current_frame.auth_zone.as_ref();

                Ok((REActor::Native, ExecutionState::Consumed(*node_id)))
            }
            Receiver::NativeRENodeRef(node_id) => {
                let native_substate_id = match node_id {
                    RENodeId::Bucket(bucket_id) => SubstateId::Bucket(*bucket_id),
                    RENodeId::Proof(proof_id) => SubstateId::Proof(*proof_id),
                    RENodeId::ResourceManager(resource_address) => {
                        SubstateId::ResourceManager(*resource_address)
                    }
                    RENodeId::System => SubstateId::System,
                    RENodeId::Worktop => SubstateId::Worktop,
                    RENodeId::Component(component_address) => {
                        SubstateId::ComponentInfo(*component_address)
                    }
                    RENodeId::Vault(vault_id) => SubstateId::Vault(*vault_id),
                    _ => return Err(RuntimeError::MethodDoesNotExist(fn_ident.clone())),
                };

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

                next_frame_node_refs.insert(node_id.clone(), node_pointer.clone());

                // Lock Substate
                let is_lock_fee = matches!(node_id, RENodeId::Vault(..)) && &fn_ident == "lock_fee";
                match node_pointer {
                    RENodePointer::Store(..) => {
                        self.track
                            .acquire_lock(native_substate_id.clone(), true, is_lock_fee)
                            .map_err(|e| match e {
                                TrackError::StateTrackError(
                                    StateTrackError::RENodeAlreadyTouched,
                                ) => RuntimeError::LockFeeError(LockFeeError::RENodeAlreadyTouched),
                                // TODO: Remove when references cleaned up
                                TrackError::NotFound => RuntimeError::RENodeNotFound(*node_id),
                                TrackError::Reentrancy => {
                                    RuntimeError::Reentrancy(native_substate_id.clone())
                                }
                            })?;
                        locked_values.insert((native_substate_id.clone(), is_lock_fee));
                    }
                    RENodePointer::Heap { .. } => {
                        if is_lock_fee {
                            return Err(RuntimeError::LockFeeError(LockFeeError::RENodeNotInTrack));
                        }
                    }
                }

                // Lock Additional Substates
                match node_id {
                    RENodeId::Component(..) => {
                        let package_address = {
                            let node_ref = node_pointer.to_ref(
                                current_frame.depth,
                                &mut current_frame.owned_heap_nodes,
                                &mut current_frame.parent_heap_nodes,
                                &mut self.track,
                            );
                            node_ref.component_info().package_address()
                        };
                        let package_substate_id = SubstateId::Package(package_address);
                        let package_node_id = RENodeId::Package(package_address);
                        self.track
                            .acquire_lock(package_substate_id.clone(), false, false)
                            .map_err(|e| match e {
                                TrackError::NotFound => panic!("Should exist"),
                                TrackError::Reentrancy => RuntimeError::PackageReentrancy,
                                TrackError::StateTrackError(..) => panic!("Unexpected"),
                            })?;
                        locked_values.insert((package_substate_id.clone(), false));
                        next_frame_node_refs
                            .insert(package_node_id, RENodePointer::Store(package_node_id));
                    }
                    RENodeId::Vault(..) => {
                        let resource_address = {
                            let node_ref = node_pointer.to_ref(
                                current_frame.depth,
                                &mut current_frame.owned_heap_nodes,
                                &mut current_frame.parent_heap_nodes,
                                &mut self.track,
                            );
                            node_ref.vault().resource_address()
                        };
                        let resource_substate_id = SubstateId::ResourceManager(resource_address);
                        let resource_node_id = RENodeId::ResourceManager(resource_address);
                        self.track
                            .acquire_lock(resource_substate_id.clone(), true, false)
                            .expect("Should never fail.");
                        locked_values.insert((resource_substate_id, false));
                        next_frame_node_refs
                            .insert(resource_node_id, RENodePointer::Store(resource_node_id));
                    }
                    _ => {}
                }

                // Lock Resource Managers in request
                // TODO: Remove when references cleaned up
                for resource_address in &input.resource_addresses {
                    let resource_substate_id =
                        SubstateId::ResourceManager(resource_address.clone());
                    let node_id = RENodeId::ResourceManager(resource_address.clone());
                    self.track
                        .acquire_lock(resource_substate_id.clone(), false, false)
                        .map_err(|e| match e {
                            TrackError::NotFound => RuntimeError::RENodeNotFound(node_id),
                            TrackError::Reentrancy => {
                                RuntimeError::Reentrancy(resource_substate_id)
                            }
                            TrackError::StateTrackError(..) => panic!("Unexpected"),
                        })?;

                    locked_values
                        .insert((SubstateId::ResourceManager(resource_address.clone()), false));
                    next_frame_node_refs.insert(node_id, RENodePointer::Store(node_id));
                }

                // Check method authorization
                AuthModule::ref_auth(
                    &fn_ident,
                    &input,
                    native_substate_id.clone(),
                    node_pointer.clone(),
                    current_frame.depth,
                    &mut current_frame.owned_heap_nodes,
                    &mut current_frame.parent_heap_nodes,
                    &mut self.track,
                    current_frame.auth_zone.as_ref(),
                    current_frame.caller_auth_zone,
                )?;
                next_caller_auth_zone = current_frame.auth_zone.as_ref();

                Ok((REActor::Native, ExecutionState::RENodeRef(*node_id)))
            }
            Receiver::AuthZoneRef => {
                next_caller_auth_zone = Option::None;
                if let Some(auth_zone) = &mut current_frame.auth_zone {
                    for resource_address in &input.resource_addresses {
                        self.track
                            .acquire_lock(
                                SubstateId::ResourceManager(resource_address.clone()),
                                false,
                                false,
                            )
                            .map_err(|e| match e {
                                TrackError::NotFound => {
                                    RuntimeError::ResourceManagerNotFound(resource_address.clone())
                                }
                                TrackError::Reentrancy => {
                                    panic!("Package reentrancy error should never occur.")
                                }
                                TrackError::StateTrackError(..) => panic!("Unexpected"),
                            })?;
                        locked_values
                            .insert((SubstateId::ResourceManager(resource_address.clone()), false));
                        let node_id = RENodeId::ResourceManager(resource_address.clone());
                        next_frame_node_refs.insert(node_id, RENodePointer::Store(node_id));
                    }
                    Ok((REActor::Native, ExecutionState::AuthZone(auth_zone)))
                } else {
                    Err(RuntimeError::AuthZoneDoesNotExist)
                }
            }
            Receiver::Component(component_address) => {
                let component_address = component_address.clone();

                // Find value
                let node_id = RENodeId::Component(component_address);
                let node_pointer = if current_frame.owned_heap_nodes.contains_key(&node_id) {
                    RENodePointer::Heap {
                        frame_id: current_frame.depth,
                        root: node_id.clone(),
                        id: None,
                    }
                } else if let Some(pointer) = current_frame.node_refs.get(&node_id) {
                    pointer.clone()
                } else {
                    return Err(RuntimeError::InvokeMethodInvalidReceiver(node_id));
                };

                // Lock values and setup next frame
                match node_pointer {
                    RENodePointer::Store(..) => {
                        let substate_id = SubstateId::ComponentState(component_address);
                        self.track
                            .acquire_lock(substate_id.clone(), true, false)
                            .map_err(|e| match e {
                                TrackError::NotFound => {
                                    RuntimeError::ComponentNotFound(component_address)
                                }
                                TrackError::Reentrancy => {
                                    RuntimeError::ComponentReentrancy(component_address)
                                }
                                TrackError::StateTrackError(..) => {
                                    panic!("Unexpected")
                                }
                            })?;
                        locked_values.insert((substate_id.clone(), false));
                    }
                    _ => {}
                };

                match node_pointer {
                    RENodePointer::Store(..) => {
                        self.track
                            .acquire_lock(
                                SubstateId::ComponentInfo(component_address),
                                false,
                                false,
                            )
                            .expect("Component Info should not be locked for long periods of time");
                    }
                    _ => {}
                }

                let scrypto_actor = {
                    let node_ref = node_pointer.to_ref(
                        current_frame.depth,
                        &current_frame.owned_heap_nodes,
                        &current_frame.parent_heap_nodes,
                        &mut self.track,
                    );
                    let component = node_ref.component_info();
                    ScryptoActor::component(
                        component_address,
                        component.package_address(),
                        component.blueprint_name().to_string(),
                    )
                };

                // Lock additional substates
                let package_substate_id =
                    SubstateId::Package(scrypto_actor.package_address().clone());
                self.track
                    .acquire_lock(package_substate_id.clone(), false, false)
                    .expect("Should never fail");
                locked_values.insert((package_substate_id.clone(), false));

                // Check Method Authorization
                AuthModule::ref_auth(
                    &fn_ident,
                    &input,
                    SubstateId::ComponentState(component_address),
                    node_pointer.clone(),
                    current_frame.depth,
                    &mut current_frame.owned_heap_nodes,
                    &mut current_frame.parent_heap_nodes,
                    &mut self.track,
                    current_frame.auth_zone.as_ref(),
                    current_frame.caller_auth_zone,
                )?;
                next_caller_auth_zone = current_frame.auth_zone.as_ref();

                match node_pointer {
                    RENodePointer::Store(..) => {
                        self.track
                            .release_lock(SubstateId::ComponentInfo(component_address), false);
                    }
                    _ => {}
                }

                next_frame_node_refs.insert(node_id, node_pointer);

                let execution_state = ExecutionState::Component(
                    scrypto_actor.package_address().clone(),
                    scrypto_actor.blueprint_name().clone(),
                    component_address,
                );
                Ok((REActor::Scrypto(scrypto_actor), execution_state))
            }
        }?;

        // Pass argument references
        for refed_component_address in &input.refed_component_addresses {
            let node_id = RENodeId::Component(refed_component_address.clone());
            if let Some(pointer) = current_frame.node_refs.get(&node_id) {
                let mut visible = HashSet::new();
                visible.insert(SubstateId::ComponentInfo(*refed_component_address));
                next_frame_node_refs.insert(node_id.clone(), pointer.clone());
            } else {
                return Err(RuntimeError::InvokeMethodInvalidReferencePass(node_id));
            }
        }

        // Setup next parent frame
        let mut next_borrowed_values: Vec<&mut HashMap<RENodeId, HeapRootRENode>> = Vec::new();
        for parent_values in &mut current_frame.parent_heap_nodes {
            next_borrowed_values.push(parent_values);
        }
        next_borrowed_values.push(&mut current_frame.owned_heap_nodes);

        // start a new frame
        let (result, received_values) = {
            let mut frame = CallFrame::new(
                current_frame.depth + 1,
                actor,
                match receiver {
                    Receiver::Component(_) => Some(AuthZone::new()),
                    _ => None,
                },
                next_owned_values,
                next_frame_node_refs,
                next_borrowed_values,
                next_caller_auth_zone,
            );

            // invoke the main function
            frame.run(
                ExecutionEntity::Method(receiver, execution_state),
                &fn_ident,
                input,
                self,
            )?
        };

        // Release locked addresses
        for (substate_id, write_through) in locked_values {
            // TODO: refactor after introducing `Lock` representation.
            self.track.release_lock(substate_id.clone(), write_through);
        }

        // move buckets and proofs to this process.
        for (id, value) in received_values {
            trace!(self, Level::Debug, "Received value: {:?}", value);
            current_frame.owned_heap_nodes.insert(id, value);
        }

        // Accept component references
        for refed_component_address in &result.refed_component_addresses {
            let node_id = RENodeId::Component(*refed_component_address);
            let mut visible = HashSet::new();
            visible.insert(SubstateId::ComponentInfo(*refed_component_address));
            current_frame
                .node_refs
                .insert(node_id, RENodePointer::Store(node_id));
        }

        trace!(self, Level::Debug, "Invoking finished!");
        Ok(result)
    }

    fn borrow_node(&mut self, node_id: &RENodeId) -> Result<RENodeRef<'_, 's>, FeeReserveError> {
        trace!(self, Level::Debug, "Borrowing value: {:?}", node_id);
        let current_frame = Self::current_frame_mut(&mut self.call_frames);

        self.fee_reserve.consume(
            self.fee_table.system_api_cost({
                match node_id {
                    RENodeId::Bucket(_) => SystemApiCostingEntry::BorrowLocal,
                    RENodeId::Proof(_) => SystemApiCostingEntry::BorrowLocal,
                    RENodeId::Worktop => SystemApiCostingEntry::BorrowLocal,
                    RENodeId::Vault(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::Component(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::KeyValueStore(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::ResourceManager(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::Package(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    RENodeId::System => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                }
            }),
            "borrow",
        )?;

        let node_pointer = current_frame
            .node_refs
            .get(node_id)
            .expect(&format!("{:?} is unknown.", node_id));

        Ok(node_pointer.to_ref(
            current_frame.depth,
            &current_frame.owned_heap_nodes,
            &current_frame.parent_heap_nodes,
            &self.track,
        ))
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
        let current_frame = Self::current_frame_mut(&mut self.call_frames);

        // Costing
        self.fee_reserve.consume(
            self.fee_table.system_api_cost({
                match substate_id {
                    SubstateId::Bucket(_) => SystemApiCostingEntry::BorrowLocal,
                    SubstateId::Proof(_) => SystemApiCostingEntry::BorrowLocal,
                    SubstateId::Worktop => SystemApiCostingEntry::BorrowLocal,
                    SubstateId::Vault(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::ComponentState(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::ComponentInfo(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::KeyValueStoreSpace(_) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::KeyValueStoreEntry(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::ResourceManager(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::NonFungibleSpace(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::NonFungible(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::Package(..) => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                    SubstateId::System => SystemApiCostingEntry::BorrowGlobal {
                        // TODO: figure out loaded state and size
                        loaded: false,
                        size: 0,
                    },
                }
            }),
            "borrow",
        )?;

        // Authorization
        if !current_frame.actor.is_substate_readable(substate_id) {
            panic!("Trying to read value which is not visible.")
        }

        let node_id = SubstateProperties::get_node_id(substate_id);

        let node_pointer = current_frame
            .node_refs
            .get(&node_id)
            .expect(&format!("Node should exist {:?}", node_id));

        Ok(node_pointer.borrow_native_ref(
            current_frame.depth,
            substate_id.clone(),
            &mut current_frame.owned_heap_nodes,
            &mut current_frame.parent_heap_nodes,
            &mut self.track,
        ))
    }

    fn substate_return_mut(&mut self, val_ref: NativeSubstateRef) -> Result<(), FeeReserveError> {
        trace!(self, Level::Debug, "Returning value");
        let current_frame = Self::current_frame_mut(&mut self.call_frames);

        self.fee_reserve.consume(
            self.fee_table.system_api_cost({
                match &val_ref {
                    NativeSubstateRef::Stack(..) => SystemApiCostingEntry::ReturnLocal,
                    NativeSubstateRef::Track(substate_id, _) => match substate_id {
                        SubstateId::Vault(_) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                        SubstateId::KeyValueStoreSpace(_) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        SubstateId::KeyValueStoreEntry(_, _) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        SubstateId::ResourceManager(_) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        SubstateId::Package(_) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                        SubstateId::NonFungibleSpace(_) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        SubstateId::NonFungible(_, _) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        SubstateId::ComponentInfo(..) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        SubstateId::ComponentState(_) => {
                            SystemApiCostingEntry::ReturnGlobal { size: 0 }
                        }
                        SubstateId::System => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                        SubstateId::Bucket(..) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                        SubstateId::Proof(..) => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                        SubstateId::Worktop => SystemApiCostingEntry::ReturnGlobal { size: 0 },
                    },
                }
            }),
            "return",
        )?;

        val_ref.return_to_location(
            current_frame.depth,
            &mut current_frame.owned_heap_nodes,
            &mut current_frame.parent_heap_nodes,
            &mut self.track,
        );
        Ok(())
    }

    fn node_drop(&mut self, node_id: &RENodeId) -> Result<HeapRootRENode, FeeReserveError> {
        trace!(self, Level::Debug, "Dropping value: {:?}", node_id);
        let current_frame = Self::current_frame_mut(&mut self.call_frames);

        // TODO: costing

        // TODO: Authorization

        Ok(current_frame.owned_heap_nodes.remove(&node_id).unwrap())
    }

    fn node_create(&mut self, re_node: HeapRENode) -> Result<RENodeId, RuntimeError> {
        trace!(self, Level::Debug, "Creating value");
        let current_frame = Self::current_frame_mut(&mut self.call_frames);

        // Costing
        self.fee_reserve
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::Create {
                        size: 0, // TODO: get size of the value
                    }),
                "create",
            )
            .map_err(RuntimeError::CostingError)?;

        // TODO: Authorization

        // Take any required child nodes
        let children = re_node.get_child_nodes()?;
        let (taken_root_nodes, mut missing) =
            current_frame.take_available_values(children, true)?;
        let first_missing_node = missing.drain().nth(0);
        if let Some(missing_node) = first_missing_node {
            return Err(RuntimeError::RENodeCreateNodeNotFound(missing_node));
        }
        let mut child_nodes = HashMap::new();
        for (id, taken_root_node) in taken_root_nodes {
            child_nodes.extend(taken_root_node.to_nodes(id));
        }

        // Insert node into heap
        let node_id = self.new_node_id(&re_node);
        let heap_root_node = HeapRootRENode {
            root: re_node,
            child_nodes,
        };
        current_frame
            .owned_heap_nodes
            .insert(node_id, heap_root_node);

        // TODO: Clean the following up
        match node_id {
            RENodeId::KeyValueStore(..) | RENodeId::ResourceManager(..) => {
                current_frame.node_refs.insert(
                    node_id.clone(),
                    RENodePointer::Heap {
                        frame_id: current_frame.depth,
                        root: node_id.clone(),
                        id: None,
                    },
                );
            }
            RENodeId::Component(component_address) => {
                let mut visible = HashSet::new();
                visible.insert(SubstateId::ComponentInfo(component_address));
                current_frame.node_refs.insert(
                    node_id.clone(),
                    RENodePointer::Heap {
                        frame_id: current_frame.depth,
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
        let current_frame = Self::current_frame_mut(&mut self.call_frames);

        // Costing
        self.fee_reserve
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::Globalize {
                        size: 0, // TODO: get size of the value
                    }),
                "globalize",
            )
            .map_err(RuntimeError::CostingError)?;

        if !RENodeProperties::can_globalize(node_id) {
            return Err(RuntimeError::RENodeGlobalizeTypeNotAllowed(node_id));
        }

        // TODO: Authorization

        let mut nodes_to_take = HashSet::new();
        nodes_to_take.insert(node_id);
        let (taken_nodes, missing_nodes) =
            current_frame.take_available_values(nodes_to_take, false)?;
        assert!(missing_nodes.is_empty());
        assert!(taken_nodes.len() == 1);
        let root_node = taken_nodes.into_values().nth(0).unwrap();

        let (substates, maybe_non_fungibles) = match root_node.root {
            HeapRENode::Component(component, component_state) => {
                let mut substates = HashMap::new();
                let component_address = node_id.into();
                substates.insert(
                    SubstateId::ComponentInfo(component_address),
                    Substate::Component(component),
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

        current_frame
            .node_refs
            .insert(node_id, RENodePointer::Store(node_id));

        Ok(())
    }

    fn substate_read(&mut self, substate_id: SubstateId) -> Result<ScryptoValue, RuntimeError> {
        trace!(self, Level::Debug, "Reading value data: {:?}", substate_id);
        let current_frame = Self::current_frame_mut(&mut self.call_frames);

        // Costing
        self.fee_reserve
            .consume(
                self.fee_table.system_api_cost(SystemApiCostingEntry::Read {
                    size: 0, // TODO: get size of the value
                }),
                "read",
            )
            .map_err(RuntimeError::CostingError)?;

        // Authorization
        if !current_frame.actor.is_substate_readable(&substate_id) {
            return Err(RuntimeError::SubstateReadNotReadable(
                current_frame.actor.clone(),
                substate_id.clone(),
            ));
        }

        let (parent_pointer, current_value) =
            Self::read_value_internal(current_frame, self.track, &substate_id)?;
        let cur_children = current_value.node_ids();
        for child_id in cur_children {
            let child_pointer = parent_pointer.child(child_id);
            current_frame.node_refs.insert(child_id, child_pointer);
        }
        Ok(current_value)
    }

    fn substate_take(&mut self, substate_id: SubstateId) -> Result<ScryptoValue, RuntimeError> {
        trace!(self, Level::Debug, "Removing value data: {:?}", substate_id);
        let current_frame = Self::current_frame_mut(&mut self.call_frames);

        // TODO: Costing

        // Authorization
        if !current_frame.actor.is_substate_writeable(&substate_id) {
            return Err(RuntimeError::SubstateWriteNotWriteable(
                current_frame.actor.clone(),
                substate_id,
            ));
        }

        let (pointer, current_value) =
            Self::read_value_internal(current_frame, self.track, &substate_id)?;
        let cur_children = current_value.node_ids();
        if !cur_children.is_empty() {
            return Err(RuntimeError::ValueNotAllowed);
        }

        // Write values
        let mut node_ref = pointer.to_ref_mut(
            current_frame.depth,
            &mut current_frame.owned_heap_nodes,
            &mut current_frame.parent_heap_nodes,
            &mut self.track,
        );
        node_ref.replace_value_with_default(&substate_id);

        Ok(current_value)
    }

    fn substate_write(
        &mut self,
        substate_id: SubstateId,
        value: ScryptoValue,
    ) -> Result<(), RuntimeError> {
        trace!(self, Level::Debug, "Writing value data: {:?}", substate_id);
        let current_frame = Self::current_frame_mut(&mut self.call_frames);

        // Costing
        self.fee_reserve
            .consume(
                self.fee_table
                    .system_api_cost(SystemApiCostingEntry::Write {
                        size: 0, // TODO: get size of the value
                    }),
                "write",
            )
            .map_err(RuntimeError::CostingError)?;

        // Authorization
        if !current_frame.actor.is_substate_writeable(&substate_id) {
            return Err(RuntimeError::SubstateWriteNotWriteable(
                current_frame.actor.clone(),
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

                current_frame.take_available_values(node_ids, true)?
            } else {
                (HashMap::new(), HashSet::new())
            }
        };

        let (pointer, current_value) =
            Self::read_value_internal(current_frame, self.track, &substate_id)?;
        let cur_children = current_value.node_ids();

        // Fulfill method
        verify_stored_value_update(&cur_children, &missing_nodes)?;

        // TODO: verify against some schema

        // Write values
        let mut node_ref = pointer.to_ref_mut(
            current_frame.depth,
            &mut current_frame.owned_heap_nodes,
            &mut current_frame.parent_heap_nodes,
            &mut self.track,
        );
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
        Ok(self.new_uuid())
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
        let current_frame = Self::current_frame(&self.call_frames);
        let proofs = proof_ids
            .iter()
            .map(|proof_id| {
                current_frame
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
        simulated_auth_zone
            .main(
                "clear",
                ScryptoValue::from_typed(&AuthZoneClearInput {}),
                self,
            )
            .map_err(RuntimeError::AuthZoneError)?;

        Ok(is_authorized)
    }

    fn fee_reserve(&mut self) -> &mut C {
        self.fee_reserve
    }

    fn fee_table(&self) -> &FeeTable {
        self.fee_table
    }

    fn new_uuid(&mut self) -> u128 {
        self.id_allocator.new_uuid(self.transaction_hash).unwrap()
    }

    fn new_node_id(&mut self, re_node: &HeapRENode) -> RENodeId {
        match re_node {
            HeapRENode::Bucket(..) => {
                let bucket_id = self.id_allocator.new_bucket_id().unwrap();
                RENodeId::Bucket(bucket_id)
            }
            HeapRENode::Proof(..) => {
                let proof_id = self.id_allocator.new_proof_id().unwrap();
                RENodeId::Proof(proof_id)
            }
            HeapRENode::Worktop(..) => RENodeId::Worktop,
            HeapRENode::Vault(..) => {
                let vault_id = self
                    .id_allocator
                    .new_vault_id(self.transaction_hash)
                    .unwrap();
                RENodeId::Vault(vault_id)
            }
            HeapRENode::KeyValueStore(..) => {
                let kv_store_id = self
                    .id_allocator
                    .new_kv_store_id(self.transaction_hash)
                    .unwrap();
                RENodeId::KeyValueStore(kv_store_id)
            }
            HeapRENode::Package(..) => {
                // Security Alert: ensure ID allocating will practically never fail
                let package_address = self
                    .id_allocator
                    .new_package_address(self.transaction_hash)
                    .unwrap();
                RENodeId::Package(package_address)
            }
            HeapRENode::Resource(..) => {
                let resource_address = self
                    .id_allocator
                    .new_resource_address(self.transaction_hash)
                    .unwrap();
                RENodeId::ResourceManager(resource_address)
            }
            HeapRENode::Component(ref component, ..) => {
                let component_address = self
                    .id_allocator
                    .new_component_address(
                        self.transaction_hash,
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

    fn track(&mut self) -> &mut Track<'s> {
        self.track
    }

    fn id_allocator(&mut self) -> &mut IdAllocator {
        &mut self.id_allocator
    }

    fn wasm_engine(&mut self) -> &mut W {
        self.wasm_engine
    }

    fn wasm_instrumenter(&mut self) -> &mut WasmInstrumenter {
        self.wasm_instrumenter
    }
}
