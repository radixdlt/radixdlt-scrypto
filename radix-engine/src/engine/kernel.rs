use scrypto::core::{FnIdent, MethodIdent, ReceiverMethodIdent};
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

pub enum ScryptoFnIdent {
    Function(PackageAddress, String, String),
    Method(ResolvedReceiver, String),
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

            let node_id = RENodeId::Bucket(bucket_id);
            let offset = SubstateOffset::Bucket(BucketOffset::Bucket);
            let handle = kernel
                .lock_substate(node_id, offset, LockFlags::MUTABLE)
                .unwrap();
            let mut substate_mut = kernel.get_ref_mut(handle).unwrap();
            let mut raw_mut = substate_mut.get_raw_mut();
            let proof = raw_mut
                .bucket()
                .create_proof(bucket_id)
                .expect("Failed to create proof");
            substate_mut.flush().unwrap();
            proofs.push(proof);
        }

        // Create empty buckets for virtual proofs
        let mut virtual_proofs_buckets: BTreeMap<ResourceAddress, BucketId> = BTreeMap::new();
        for resource_address in virtualizable_proofs_resource_addresses {
            let bucket_id = kernel
                .create_non_fungible_bucket_with_ids(resource_address.clone(), BTreeSet::new());
            virtual_proofs_buckets.insert(resource_address, bucket_id);
        }

        let auth_zone = AuthZoneSubstate::new(proofs, virtual_proofs_buckets);

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
        let new_refed_nodes = self
            .execute_in_kernel_mode(KernelActor::AuthModule, |system_api| {
                AuthModule::on_before_frame_start(&actor, &input, system_api)
            })
            .map_err(|e| match e {
                InvokeError::Error(e) => RuntimeError::ModuleError(e.into()),
                InvokeError::Downstream(runtime_error) => runtime_error,
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
        let output = {
            let rtn = match actor.clone() {
                REActor::Function(ResolvedFunction::Native(native_fn)) => {
                    NativeInterpreter::run_function(native_fn, input, self)
                }
                REActor::Method(ResolvedReceiverMethod {
                    receiver,
                    method: ResolvedMethod::Native(native_method),
                }) => {
                    NativeInterpreter::run_method(receiver.receiver(), native_method, input, self)
                }
                REActor::Function(ResolvedFunction::Scrypto {
                    package_address,
                    blueprint_name,
                    ident,
                    export_name,
                })
                | REActor::Method(ResolvedReceiverMethod {
                    method:
                        ResolvedMethod::Scrypto {
                            package_address,
                            blueprint_name,
                            ident,
                            export_name,
                        },
                    ..
                }) => {
                    let package_id = RENodeId::Package(package_address);
                    let package_offset = SubstateOffset::Package(PackageOffset::Package);

                    let output = {
                        let package = {
                            let substate = self
                                .track
                                .borrow_substate(package_id, package_offset.clone());
                            substate.package().clone() // TODO: Remove clone()
                        };
                        let mut instance = self.scrypto_interpreter.instance(package);

                        let scrypto_actor = match &Self::current_frame(&self.call_frames).actor {
                            REActor::Method(ResolvedReceiverMethod {
                                receiver: ResolvedReceiver { receiver, .. },
                                ..
                            }) => match receiver {
                                Receiver::Ref(RENodeId::Component(component_id)) => {
                                    ScryptoActor::Component(
                                        *component_id,
                                        package_address.clone(),
                                        blueprint_name.clone(),
                                    )
                                }
                                _ => panic!("Should not get here."),
                            },
                            _ => ScryptoActor::blueprint(package_address, blueprint_name.clone()),
                        };

                        let mut runtime: Box<dyn WasmRuntime> =
                            Box::new(RadixEngineWasmRuntime::new(scrypto_actor, self));
                        instance
                            .invoke_export(&export_name, &input, &mut runtime)
                            .map_err(|e| match e {
                                InvokeError::Error(e) => RuntimeError::KernelError(
                                    KernelError::WasmError(actor.clone(), e),
                                ),
                                InvokeError::Downstream(runtime_error) => runtime_error,
                            })?
                    };

                    let package = self
                        .track
                        .borrow_substate(package_id, package_offset)
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
        self.execute_in_kernel_mode(KernelActor::AuthModule, |system_api| {
            AuthModule::on_frame_end(system_api)
        })
        .map_err(|e| match e {
            InvokeError::Error(e) => RuntimeError::ModuleError(e.into()),
            InvokeError::Downstream(runtime_error) => runtime_error,
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

    pub fn execute_in_kernel_mode<X, RTN, E>(
        &mut self,
        kernel_actor: KernelActor,
        execute: X,
    ) -> Result<RTN, E>
    where
        X: FnOnce(&mut Self) -> Result<RTN, E>,
    {
        // Save and replace kernel actor
        let saved_kernel_actor = {
            let current_frame = Self::current_frame_mut(&mut self.call_frames);
            let cur = current_frame.kernel_actor;
            current_frame.kernel_actor = kernel_actor;
            cur
        };

        let rtn = execute(self)?;

        // Restore old kernel actor
        {
            let current_frame = Self::current_frame_mut(&mut self.call_frames);
            current_frame.kernel_actor = saved_kernel_actor;
        }

        Ok(rtn)
    }

    pub fn node_method_deref(
        &mut self,
        node_id: RENodeId,
    ) -> Result<Option<RENodeId>, RuntimeError> {
        if let RENodeId::Global(..) = node_id {
            let node_id = self.execute_in_kernel_mode::<_, _, RuntimeError>(
                KernelActor::Deref,
                |system_api| {
                    let offset = SubstateOffset::Global(GlobalOffset::Global);
                    let handle = system_api.lock_substate(node_id, offset, LockFlags::empty())?;
                    let substate_ref = system_api.get_ref(handle)?;
                    Ok(substate_ref.global_address().node_deref())
                },
            )?;

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
                let node_id = self.execute_in_kernel_mode::<_, _, RuntimeError>(
                    KernelActor::Deref,
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

    fn resolve_method_actor(
        &mut self,
        receiver: ResolvedReceiver,
        method_ident: MethodIdent,
        input: &ScryptoValue,
    ) -> Result<(REActor, HashMap<RENodeId, RENodePointer>), RuntimeError> {
        let mut references_to_add = HashMap::new();

        let actor = match &method_ident {
            MethodIdent::Scrypto(ident) => {
                let (actor, node_id) = self
                    .execute_in_kernel_mode(KernelActor::ScryptoLoader, |system_api| {
                        ScryptoInterpreter::load_scrypto_actor(
                            ScryptoFnIdent::Method(receiver.clone(), ident.clone()),
                            input,
                            system_api,
                        )
                    })
                    .map_err(|e| match e {
                        InvokeError::Downstream(runtime_error) => runtime_error,
                        InvokeError::Error(error) => {
                            RuntimeError::InterpreterError(InterpreterError::InvalidScryptoMethod(
                                receiver.clone(),
                                method_ident.clone(),
                                error,
                            ))
                        }
                    })?;

                // TODO: Move this in a better spot when more refactors are done
                let package = self.execute_in_kernel_mode::<_, _, RuntimeError>(
                    KernelActor::ScryptoLoader,
                    |system_api| {
                        let handle = system_api.lock_substate(
                            node_id,
                            SubstateOffset::Package(PackageOffset::Package),
                            LockFlags::read_only(),
                        )?;
                        let substate_ref = system_api.get_ref(handle)?;
                        let package = substate_ref.package().clone(); // TODO: Remove clone()
                        system_api.drop_lock(handle)?;
                        Ok(package)
                    },
                )?;
                for m in &mut self.modules {
                    m.on_wasm_instantiation(&mut self.track, &mut self.call_frames, package.code())
                        .map_err(RuntimeError::ModuleError)?;
                }

                let node_pointer = Self::current_frame(&self.call_frames)
                    .get_node_pointer(node_id)
                    .unwrap();
                references_to_add.insert(node_id, node_pointer);
                actor
            }
            MethodIdent::Native(native_fn) => REActor::Method(ResolvedReceiverMethod {
                receiver: receiver.clone(),
                method: ResolvedMethod::Native(native_fn.clone()),
            }),
        };

        Ok((actor, references_to_add))
    }

    fn resolve_function_actor(
        &mut self,
        function_ident: FunctionIdent,
        input: &ScryptoValue,
    ) -> Result<(REActor, HashMap<RENodeId, RENodePointer>), RuntimeError> {
        let mut references_to_add = HashMap::new();

        let actor = match &function_ident {
            FunctionIdent::Scrypto {
                package_address,
                blueprint_name,
                ident,
            } => {
                let (actor, node_id) = self
                    .execute_in_kernel_mode(KernelActor::ScryptoLoader, |system_api| {
                        ScryptoInterpreter::load_scrypto_actor(
                            ScryptoFnIdent::Function(
                                *package_address,
                                blueprint_name.clone(),
                                ident.to_string(),
                            ),
                            input,
                            system_api,
                        )
                    })
                    .map_err(|e| match e {
                        InvokeError::Downstream(runtime_error) => runtime_error,
                        InvokeError::Error(error) => RuntimeError::InterpreterError(
                            InterpreterError::InvalidScryptoFunction(function_ident, error),
                        ),
                    })?;

                // TODO: Move this in a better spot when more refactors are done
                let package = self.execute_in_kernel_mode::<_, _, RuntimeError>(
                    KernelActor::ScryptoLoader,
                    |system_api| {
                        let handle = system_api.lock_substate(
                            node_id,
                            SubstateOffset::Package(PackageOffset::Package),
                            LockFlags::read_only(),
                        )?;
                        let substate_ref = system_api.get_ref(handle)?;
                        let package = substate_ref.package().clone(); // TODO: Remove clone()
                        system_api.drop_lock(handle)?;
                        Ok(package)
                    },
                )?;
                for m in &mut self.modules {
                    m.on_wasm_instantiation(&mut self.track, &mut self.call_frames, package.code())
                        .map_err(RuntimeError::ModuleError)?;
                }

                let node_pointer = Self::current_frame(&self.call_frames)
                    .get_node_pointer(node_id)
                    .unwrap();
                references_to_add.insert(node_id, node_pointer);
                actor
            }
            FunctionIdent::Native(native_function) => {
                REActor::Function(ResolvedFunction::Native(native_function.clone()))
            }
        };

        Ok((actor, references_to_add))
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
        let depth = Self::current_frame(&self.call_frames).depth;

        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::Invoke {
                    fn_ident: &fn_ident,
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

        let mut nodes_to_pass_downstream = HashMap::new();
        let mut next_node_refs = HashMap::new();

        // Internal state update to taken values
        for node_id in input.node_ids() {
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

        let (next_actor, references_to_add) = match fn_ident {
            FnIdent::Method(ReceiverMethodIdent {
                receiver,
                method_ident,
            }) => {
                let resolved_receiver = match receiver {
                    Receiver::Consumed(node_id) => {
                        let node =
                            Self::current_frame_mut(&mut self.call_frames).take_node(node_id)?;
                        nodes_to_pass_downstream.insert(node_id, node);
                        ResolvedReceiver::new(Receiver::Consumed(node_id))
                    }
                    Receiver::Ref(node_id) => {
                        // Deref
                        let resolved_receiver =
                            if let Some(derefed) = self.node_method_deref(node_id)? {
                                ResolvedReceiver::derefed(Receiver::Ref(derefed), node_id)
                            } else {
                                ResolvedReceiver::new(Receiver::Ref(node_id))
                            };

                        let resolved_node_id = resolved_receiver.node_id();
                        let node_pointer = Self::current_frame(&self.call_frames)
                            .get_node_pointer(resolved_node_id)?;
                        next_node_refs.insert(resolved_node_id, node_pointer);

                        resolved_receiver
                    }
                };
                self.resolve_method_actor(resolved_receiver, method_ident, &input)?
            }
            FnIdent::Function(function_ident) => {
                self.resolve_function_actor(function_ident, &input)?
            }
        };
        next_node_refs.extend(references_to_add);

        let cur_actor = &Self::current_frame(&self.call_frames).actor;

        for (node_id, node) in &mut nodes_to_pass_downstream {
            let root_node = node.root_mut();
            root_node.prepare_move_downstream(*node_id, cur_actor, &next_actor)?;
        }

        let (output, received_values) =
            self.run(next_actor, input, nodes_to_pass_downstream, next_node_refs)?;

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
                SysCallOutput::Invoke { output: &output },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // TODO: Move this into higher layer, e.g. transaction processor
        if Self::current_frame(&self.call_frames).depth == 0 {
            self.call_frames.pop().unwrap().drop_frame()?;
        }

        Ok(output)
    }

    fn get_actor(&self) -> &REActor {
        &Self::current_frame(&self.call_frames).actor
    }

    fn get_all_referenceable_node_ids(&mut self) -> Result<Vec<RENodeId>, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &mut self.track,
                &mut self.call_frames,
                SysCallInput::ReadOwnedNodes,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let node_ids = Self::current_frame_mut(&mut self.call_frames).get_all_referenceable_nodes();

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

        // Deref
        if let Some(derefed) = self.node_offset_deref(node_id, &offset)? {
            node_id = derefed;
        }

        let node_pointer = Self::current_frame(&self.call_frames).get_node_pointer(node_id)?;

        // TODO: Check if valid offset for node_id

        // Authorization
        let kernel_actor = Self::current_frame(&self.call_frames).kernel_actor;
        let actor = &Self::current_frame(&self.call_frames).actor;
        if !SubstateProperties::check_substate_access(
            kernel_actor,
            actor,
            node_id,
            offset.clone(),
            flags,
        ) {
            return Err(RuntimeError::KernelError(
                KernelError::InvalidSubstateLock {
                    kernel_actor,
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
