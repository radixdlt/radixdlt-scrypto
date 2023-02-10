use native_sdk::resource::SysBucket;
use radix_engine_interface::api::types::{
    AuthZoneStackOffset, ComponentOffset, GlobalAddress, GlobalOffset, LockHandle, ProofOffset,
    RENodeId, SubstateId, SubstateOffset, VaultId, WorktopOffset,
};
use radix_engine_interface::api::{
    ActorApi, ComponentApi, EngineApi, Invocation, Invokable, InvokableModel,
};
use radix_engine_interface::data::*;
use radix_engine_interface::rule;
use sbor::rust::fmt::Debug;
use sbor::rust::mem;
use transaction::model::AuthZoneParams;

use crate::engine::node_move_module::NodeMoveModule;
use crate::engine::system_api::LockInfo;
use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use crate::wasm::*;

pub struct Kernel<
    'g, // Lifetime of values outliving all frames
    's, // Substate store lifetime
    W,  // WASM engine type
    R,  // Fee reserve type
    M,
> where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    /// Current execution mode, specifies permissions into state/invocations
    execution_mode: ExecutionMode,
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

    /// ID allocator
    id_allocator: &'g mut IdAllocator,
    /// Interpreter capable of running scrypto programs
    scrypto_interpreter: &'g ScryptoInterpreter<W>,
    /// Kernel module
    module: &'g mut M,
}

impl<'g, 's, W, R, M> Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    pub fn new(
        auth_zone_params: AuthZoneParams,
        id_allocator: &'g mut IdAllocator,
        track: &'g mut Track<'s, R>,
        scrypto_interpreter: &'g ScryptoInterpreter<W>,
        module: &'g mut M,
    ) -> Self {
        let mut kernel = Self {
            execution_mode: ExecutionMode::Kernel,
            heap: Heap::new(),
            track,
            scrypto_interpreter,
            id_allocator,
            current_frame: CallFrame::new_root(),
            prev_frame_stack: vec![],
            module,
        };

        // Initial authzone
        // TODO: Move into module initialization
        kernel
            .execute_in_mode::<_, _, RuntimeError>(ExecutionMode::AuthModule, |api| {
                let auth_zone = AuthZoneStackSubstate::new(
                    vec![],
                    auth_zone_params.virtualizable_proofs_resource_addresses,
                    auth_zone_params.initial_proofs.into_iter().collect(),
                );
                let node_id = api.allocate_node_id(RENodeType::AuthZoneStack)?;
                api.create_node(node_id, RENodeInit::AuthZoneStack(auth_zone))?;
                Ok(())
            })
            .expect("AuthModule failed to initialize");

        kernel
            .execute_in_mode::<_, _, RuntimeError>(ExecutionMode::LoggerModule, |api| {
                LoggerModule::initialize(api)
            })
            .expect("Logger failed to initialize");

        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Resource(RADIX_TOKEN)),
            RENodeVisibilityOrigin::Normal,
        );
        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Resource(SYSTEM_TOKEN)),
            RENodeVisibilityOrigin::Normal,
        );
        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Resource(ECDSA_SECP256K1_TOKEN)),
            RENodeVisibilityOrigin::Normal,
        );
        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Resource(EDDSA_ED25519_TOKEN)),
            RENodeVisibilityOrigin::Normal,
        );
        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Component(EPOCH_MANAGER)),
            RENodeVisibilityOrigin::Normal,
        );
        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Component(CLOCK)),
            RENodeVisibilityOrigin::Normal,
        );
        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Package(ACCOUNT_PACKAGE)),
            RENodeVisibilityOrigin::Normal,
        );
        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::Package(FAUCET_PACKAGE)),
            RENodeVisibilityOrigin::Normal,
        );

        kernel
    }

    fn create_virtual_account(
        &mut self,
        node_id: RENodeId,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Result<(), RuntimeError> {
        // TODO: Replace with trusted IndexedScryptoValue
        let access_rule = rule!(require(non_fungible_global_id));
        let result = self.invoke(ScryptoInvocation {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: "Account".to_string(),
            fn_name: "create".to_string(),
            receiver: None,
            args: args!(access_rule),
        })?;
        let component_id = IndexedScryptoValue::from_value(result)
            .owned_node_ids()
            .expect("No duplicates expected")
            .into_iter()
            .next()
            .unwrap()
            .into();

        // TODO: Use system_api to globalize component when create_node is refactored
        // TODO: to allow for address selection
        let global_substate = GlobalAddressSubstate::Component(component_id);

        self.current_frame.create_node(
            node_id,
            RENodeInit::Global(global_substate),
            &mut self.heap,
            &mut self.track,
            true,
        )?;

        Ok(())
    }

    fn create_virtual_identity(
        &mut self,
        node_id: RENodeId,
        non_fungible_global_id: NonFungibleGlobalId,
    ) -> Result<(), RuntimeError> {
        let access_rule = rule!(require(non_fungible_global_id));
        let underlying_node_id = Identity::create(access_rule, self)?;

        // TODO: Use system_api to globalize component when create_node is refactored
        // TODO: to allow for address selection
        let global_substate = GlobalAddressSubstate::Identity(underlying_node_id.into());
        self.current_frame.create_node(
            node_id,
            RENodeInit::Global(global_substate),
            &mut self.heap,
            &mut self.track,
            true,
        )?;

        Ok(())
    }

    fn try_virtualize(
        &mut self,
        node_id: RENodeId,
        offset: &SubstateOffset,
    ) -> Result<bool, RuntimeError> {
        match (node_id, offset) {
            (
                RENodeId::Global(GlobalAddress::Component(component_address)),
                SubstateOffset::Global(GlobalOffset::Global),
            ) => {
                // Lazy create component if missing
                match component_address {
                    ComponentAddress::EcdsaSecp256k1VirtualAccount(address) => {
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            ECDSA_SECP256K1_TOKEN,
                            NonFungibleLocalId::bytes(address).unwrap(),
                        );
                        self.create_virtual_account(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EddsaEd25519VirtualAccount(address) => {
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            EDDSA_ED25519_TOKEN,
                            NonFungibleLocalId::bytes(address).unwrap(),
                        );
                        self.create_virtual_account(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EcdsaSecp256k1VirtualIdentity(address) => {
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            ECDSA_SECP256K1_TOKEN,
                            NonFungibleLocalId::bytes(address).unwrap(),
                        );
                        self.create_virtual_identity(node_id, non_fungible_global_id)?;
                    }
                    ComponentAddress::EddsaEd25519VirtualIdentity(address) => {
                        let non_fungible_global_id = NonFungibleGlobalId::new(
                            EDDSA_ED25519_TOKEN,
                            NonFungibleLocalId::bytes(address).unwrap(),
                        );
                        self.create_virtual_identity(node_id, non_fungible_global_id)?;
                    }
                    _ => return Ok(false),
                };

                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn drop_node_internal(&mut self, node_id: RENodeId) -> Result<HeapRENode, RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::DropNode, |system_api| {
            match node_id {
                RENodeId::AuthZoneStack(..) => {
                    let handle = system_api.lock_substate(
                        node_id,
                        SubstateOffset::AuthZoneStack(AuthZoneStackOffset::AuthZoneStack),
                        LockFlags::MUTABLE,
                    )?;
                    let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
                    let auth_zone_stack = substate_ref_mut.auth_zone_stack();
                    auth_zone_stack.clear_all();
                    system_api.drop_lock(handle)?;
                    Ok(())
                }
                RENodeId::Proof(..) => {
                    let handle = system_api.lock_substate(
                        node_id,
                        SubstateOffset::Proof(ProofOffset::Proof),
                        LockFlags::MUTABLE,
                    )?;
                    let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
                    let proof = substate_ref_mut.proof();
                    proof.drop();
                    system_api.drop_lock(handle)?;
                    Ok(())
                }
                RENodeId::Logger => Ok(()),
                RENodeId::Worktop => {
                    let handle = system_api.lock_substate(
                        node_id,
                        SubstateOffset::Worktop(WorktopOffset::Worktop),
                        LockFlags::MUTABLE,
                    )?;

                    let buckets = {
                        let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
                        let worktop = substate_ref_mut.worktop();
                        mem::replace(&mut worktop.resources, BTreeMap::new())
                    };
                    for (_, bucket) in buckets {
                        let bucket = Bucket(bucket.bucket_id());
                        if !bucket.sys_is_empty(system_api)? {
                            return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                                RENodeId::Worktop,
                            )));
                        }
                    }

                    system_api.drop_lock(handle)?;
                    Ok(())
                }
                RENodeId::Bucket(..) => Ok(()),
                RENodeId::TransactionRuntime(..) => Ok(()),
                _ => Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                    node_id,
                ))),
            }
        })?;

        let node = self.current_frame.remove_node(&mut self.heap, node_id)?;
        for (_, substate) in &node.substates {
            let (_, child_nodes) = substate.to_ref().references_and_owned_nodes();
            for child_node in child_nodes {
                // Need to go through system_api so that visibility issues can be caught
                self.drop_node(child_node)?;
            }
        }
        // TODO: REmove
        Ok(node)
    }

    fn drop_nodes_in_frame(&mut self) -> Result<(), RuntimeError> {
        let mut worktops = Vec::new();
        let owned_nodes = self.current_frame.owned_nodes();

        // Need to go through system_api so that visibility issues can be caught
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Application, |system_api| {
            for node_id in owned_nodes {
                if let RENodeId::Worktop = node_id {
                    worktops.push(node_id);
                } else {
                    system_api.drop_node(node_id)?;
                }
            }
            for worktop_id in worktops {
                system_api.drop_node(worktop_id)?;
            }

            Ok(())
        })?;

        Ok(())
    }

    fn run<X: Executor>(
        &mut self,
        executor: X,
        actor: ResolvedActor,
        mut call_frame_update: CallFrameUpdate,
    ) -> Result<X::Output, RuntimeError> {
        let derefed_lock = if let Some(ResolvedReceiver {
            derefed_from: Some((_, derefed_lock)),
            ..
        }) = &actor.receiver
        {
            Some(*derefed_lock)
        } else {
            None
        };

        // Filter
        self.execute_in_mode(ExecutionMode::AuthModule, |system_api| {
            AuthModule::on_before_frame_start(&actor, system_api)
        })?;

        // New Call Frame pre-processing
        {
            // TODO: Abstract these away
            self.execute_in_mode(ExecutionMode::TransactionModule, |system_api| {
                TransactionHashModule::on_call_frame_enter(
                    &mut call_frame_update,
                    &actor,
                    system_api,
                )
            })?;
            self.execute_in_mode(ExecutionMode::LoggerModule, |system_api| {
                LoggerModule::on_call_frame_enter(&mut call_frame_update, &actor, system_api)
            })?;
            self.execute_in_mode(ExecutionMode::AuthModule, |system_api| {
                AuthModule::on_call_frame_enter(&mut call_frame_update, &actor, system_api)
            })?;
            self.execute_in_mode(ExecutionMode::NodeMoveModule, |system_api| {
                NodeMoveModule::on_call_frame_enter(
                    &mut call_frame_update,
                    &actor.identifier,
                    system_api,
                )
            })?;

            self.module
                .pre_execute_invocation(
                    &actor,
                    &call_frame_update,
                    &mut self.current_frame,
                    &mut self.heap,
                    &mut self.track,
                )
                .map_err(RuntimeError::ModuleError)?;
            self.id_allocator.pre_execute_invocation();
        }

        // Call Frame Push
        {
            let frame = CallFrame::new_child_from_parent(
                &mut self.current_frame,
                actor,
                call_frame_update,
            )?;
            let parent = mem::replace(&mut self.current_frame, frame);
            self.prev_frame_stack.push(parent);
        }

        // Execute
        let (output, update) = self.execute_in_mode(ExecutionMode::Application, |system_api| {
            executor.execute(system_api)
        })?;

        // Call Frame post-processing
        {
            // Auto drop locks
            self.current_frame
                .drop_all_locks(&mut self.heap, &mut self.track)?;

            self.id_allocator.post_execute_invocation()?;
            self.module
                .post_execute_invocation(
                    &self.prev_frame_stack.last().unwrap().actor,
                    &update,
                    &mut self.current_frame,
                    &mut self.heap,
                    &mut self.track,
                )
                .map_err(RuntimeError::ModuleError)?;

            // TODO: Abstract these away
            self.execute_in_mode(ExecutionMode::NodeMoveModule, |system_api| {
                NodeMoveModule::on_call_frame_exit(&update, system_api)
            })?;
            self.execute_in_mode(ExecutionMode::AuthModule, |system_api| {
                AuthModule::on_call_frame_exit(system_api)
            })?;

            // Auto-drop locks again in case module forgot to drop
            self.current_frame
                .drop_all_locks(&mut self.heap, &mut self.track)?;
        }

        // Call Frame Pop
        {
            let mut parent = self.prev_frame_stack.pop().unwrap();
            CallFrame::update_upstream(&mut self.current_frame, &mut parent, update)?;

            // drop proofs and check resource leak
            self.drop_nodes_in_frame()?;

            // Restore previous frame
            self.current_frame = parent;
        }

        if let Some(derefed_lock) = derefed_lock {
            self.current_frame
                .drop_lock(&mut self.heap, &mut self.track, derefed_lock)?;
        }

        Ok(output)
    }

    pub fn node_method_deref(
        &mut self,
        node_id: RENodeId,
    ) -> Result<Option<(RENodeId, LockHandle)>, RuntimeError> {
        if let RENodeId::Global(..) = node_id {
            let derefed =
                self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::Deref, |system_api| {
                    let offset = SubstateOffset::Global(GlobalOffset::Global);
                    let handle = system_api.lock_substate(node_id, offset, LockFlags::empty())?;
                    let substate_ref = system_api.get_ref(handle)?;
                    Ok((substate_ref.global_address().node_deref(), handle))
                })?;

            Ok(Some(derefed))
        } else {
            Ok(None)
        }
    }

    pub fn node_offset_deref(
        &mut self,
        node_id: RENodeId,
        offset: &SubstateOffset,
    ) -> Result<Option<(RENodeId, LockHandle)>, RuntimeError> {
        if let RENodeId::Global(..) = node_id {
            if !matches!(offset, SubstateOffset::Global(GlobalOffset::Global)) {
                let derefed = self.execute_in_mode::<_, _, RuntimeError>(
                    ExecutionMode::Deref,
                    |system_api| {
                        let handle = system_api.lock_substate(
                            node_id,
                            SubstateOffset::Global(GlobalOffset::Global),
                            LockFlags::empty(),
                        )?;
                        let substate_ref = system_api.get_ref(handle)?;
                        Ok((substate_ref.global_address().node_deref(), handle))
                    },
                )?;

                Ok(Some(derefed))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn verify_valid_mode_transition(
        cur: &ExecutionMode,
        next: &ExecutionMode,
    ) -> Result<(), RuntimeError> {
        match (cur, next) {
            (ExecutionMode::Kernel, ..) => Ok(()),
            (ExecutionMode::Resolver, ExecutionMode::Deref) => Ok(()),
            _ => Err(RuntimeError::KernelError(
                KernelError::InvalidModeTransition(*cur, *next),
            )),
        }
    }

    fn invoke_internal<X: Executor>(
        &mut self,
        executor: X,
        actor: ResolvedActor,
        call_frame_update: CallFrameUpdate,
    ) -> Result<X::Output, RuntimeError> {
        let depth = self.current_frame.depth;
        // TODO: Move to higher layer
        if depth == 0 {
            for node_id in &call_frame_update.node_refs_to_copy {
                match node_id {
                    RENodeId::Global(global_address) => {
                        if self.current_frame.get_node_location(*node_id).is_err() {
                            if matches!(
                                global_address,
                                GlobalAddress::Component(
                                    ComponentAddress::EcdsaSecp256k1VirtualAccount(..)
                                )
                            ) || matches!(
                                global_address,
                                GlobalAddress::Component(
                                    ComponentAddress::EddsaEd25519VirtualAccount(..)
                                )
                            ) || matches!(
                                global_address,
                                GlobalAddress::Component(
                                    ComponentAddress::EcdsaSecp256k1VirtualIdentity(..)
                                )
                            ) || matches!(
                                global_address,
                                GlobalAddress::Component(
                                    ComponentAddress::EddsaEd25519VirtualIdentity(..)
                                )
                            ) {
                                self.current_frame
                                    .add_stored_ref(*node_id, RENodeVisibilityOrigin::Normal);
                                continue;
                            }

                            let offset = SubstateOffset::Global(GlobalOffset::Global);
                            self.track
                                .acquire_lock(
                                    SubstateId(*node_id, offset.clone()),
                                    LockFlags::read_only(),
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.track
                                .release_lock(SubstateId(*node_id, offset), false)
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.current_frame
                                .add_stored_ref(*node_id, RENodeVisibilityOrigin::Normal);
                        }
                    }
                    RENodeId::Vault(..) => {
                        if self.current_frame.get_node_location(*node_id).is_err() {
                            let offset = SubstateOffset::Vault(VaultOffset::Vault);
                            self.track
                                .acquire_lock(
                                    SubstateId(*node_id, offset.clone()),
                                    LockFlags::read_only(),
                                )
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;
                            self.track
                                .release_lock(SubstateId(*node_id, offset), false)
                                .map_err(|_| KernelError::RENodeNotFound(*node_id))?;

                            self.current_frame
                                .add_stored_ref(*node_id, RENodeVisibilityOrigin::DirectAccess);
                        }
                    }
                    _ => {}
                }
            }
        }

        let output = self.run(executor, actor, call_frame_update)?;

        // TODO: Move to higher layer
        if depth == 0 {
            self.current_frame
                .drop_all_locks(&mut self.heap, &mut self.track)?;
            self.drop_nodes_in_frame()?;
        }

        Ok(output)
    }

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
}

impl<'g, 's, W, R, M> VmApi<W> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn vm(&mut self) -> &ScryptoInterpreter<W> {
        self.scrypto_interpreter
    }

    fn on_wasm_instantiation(&mut self, code: &[u8]) -> Result<(), RuntimeError> {
        self.module
            .on_wasm_instantiation(&self.current_frame, &mut self.heap, &mut self.track, code)
            .map_err(RuntimeError::ModuleError)?;

        Ok(())
    }
}

impl<'g, 's, W, R, M> ResolverApi for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn deref(&mut self, node_id: RENodeId) -> Result<Option<(RENodeId, LockHandle)>, RuntimeError> {
        self.node_method_deref(node_id)
    }
}

pub trait Executor {
    type Output: Debug;

    fn execute<Y, W>(self, api: &mut Y) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + EngineApi<RuntimeError>
            + InvokableModel<RuntimeError>
            + ActorApi<RuntimeError>
            + ComponentApi<RuntimeError>
            + VmApi<W>,
        W: WasmEngine;
}

pub trait ExecutableInvocation: Invocation {
    type Exec: Executor<Output = Self::Output>;

    fn resolve<Y: ResolverApi + SystemApi>(
        self,
        api: &mut Y,
    ) -> Result<(ResolvedActor, CallFrameUpdate, Self::Exec), RuntimeError>;
}

impl<'g, 's, W, R, M> ComponentApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn invoke_method(
        &mut self,
        receiver: ScryptoReceiver,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        // TODO: Use execution mode?
        let invocation = resolve_method(receiver, method_name, &args, self)?;
        invoke_call_table(invocation, self)
    }
}

impl<'g, 's, W, R, N, M> Invokable<N, RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
    N: ExecutableInvocation,
{
    fn invoke(&mut self, invocation: N) -> Result<<N as Invocation>::Output, RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::Invoke {
                    fn_identifier: invocation.fn_identifier(),
                    input_size: 0, // TODO: Fix this
                    depth: self.current_frame.depth,
                },
            )
            .map_err(RuntimeError::ModuleError)?;

        // Change to kernel mode
        let saved_mode = self.execution_mode;

        self.execution_mode = ExecutionMode::Resolver;
        let (actor, call_frame_update, executor) = invocation.resolve(self)?;

        self.execution_mode = ExecutionMode::Kernel;
        let rtn = self.invoke_internal(executor, actor, call_frame_update)?;

        // Restore previous mode
        self.execution_mode = saved_mode;

        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::Invoke { rtn: &rtn },
            )
            .map_err(RuntimeError::ModuleError)?;

        Ok(rtn)
    }
}

impl<'g, 's, W, R, M> SystemApi for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn consume_cost_units(&mut self, units: u32) -> Result<(), RuntimeError> {
        self.module
            .on_wasm_costing(&self.current_frame, &mut self.heap, &mut self.track, units)
            .map_err(RuntimeError::ModuleError)?;

        Ok(())
    }

    fn lock_fee(
        &mut self,
        vault_id: VaultId,
        mut fee: Resource,
        contingent: bool,
    ) -> Result<Resource, RuntimeError> {
        fee = self
            .module
            .on_lock_fee(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                vault_id,
                fee,
                contingent,
            )
            .map_err(RuntimeError::ModuleError)?;

        Ok(fee)
    }

    fn get_visible_nodes(&mut self) -> Result<Vec<RENodeId>, RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::ReadOwnedNodes,
            )
            .map_err(RuntimeError::ModuleError)?;

        let node_ids = self.current_frame.get_visible_nodes();

        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::ReadOwnedNodes,
            )
            .map_err(RuntimeError::ModuleError)?;

        Ok(node_ids)
    }

    fn get_visible_node_data(
        &mut self,
        node_id: RENodeId,
    ) -> Result<RENodeVisibilityOrigin, RuntimeError> {
        let visibility = self.current_frame.get_node_visibility(node_id)?;
        Ok(visibility)
    }

    fn drop_node(&mut self, node_id: RENodeId) -> Result<HeapRENode, RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::DropNode { node_id: &node_id },
            )
            .map_err(RuntimeError::ModuleError)?;

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

        let node = self.drop_node_internal(node_id)?;

        // Restore current mode
        self.execution_mode = current_mode;

        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::DropNode { node: &node },
            )
            .map_err(RuntimeError::ModuleError)?;

        Ok(node)
    }

    fn allocate_node_id(&mut self, node_type: RENodeType) -> Result<RENodeId, RuntimeError> {
        // TODO: Add costing
        let node_id = self.id_allocator.allocate_node_id(node_type)?;

        Ok(node_id)
    }

    fn create_node(&mut self, node_id: RENodeId, re_node: RENodeInit) -> Result<(), RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::CreateNode { node: &re_node },
            )
            .map_err(RuntimeError::ModuleError)?;

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

        match (node_id, &re_node) {
            (
                RENodeId::Global(GlobalAddress::Package(..)),
                RENodeInit::Global(GlobalAddressSubstate::Package(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Resource(..)),
                RENodeInit::Global(GlobalAddressSubstate::Resource(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Component(..)),
                RENodeInit::Global(GlobalAddressSubstate::EpochManager(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Component(..)),
                RENodeInit::Global(GlobalAddressSubstate::Clock(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Component(..)),
                RENodeInit::Global(GlobalAddressSubstate::Validator(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Component(..)),
                RENodeInit::Global(GlobalAddressSubstate::Identity(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Component(..)),
                RENodeInit::Global(GlobalAddressSubstate::AccessController(..)),
            ) => {}
            (
                RENodeId::Global(address),
                RENodeInit::Global(GlobalAddressSubstate::Component(component)),
            ) => {
                // TODO: Get rid of this logic
                let (package_address, blueprint_name) = self
                    .execute_in_mode::<_, _, RuntimeError>(
                        ExecutionMode::Globalize,
                        |system_api| {
                            let handle = system_api.lock_substate(
                                RENodeId::Component(*component),
                                SubstateOffset::Component(ComponentOffset::Info),
                                LockFlags::read_only(),
                            )?;
                            let substate_ref = system_api.get_ref(handle)?;
                            let info = substate_ref.component_info();
                            let package_blueprint =
                                (info.package_address, info.blueprint_name.clone());
                            system_api.drop_lock(handle)?;
                            Ok(package_blueprint)
                        },
                    )?;

                match address {
                    GlobalAddress::Component(ComponentAddress::Account(..)) => {
                        if !(package_address.eq(&ACCOUNT_PACKAGE)
                            && blueprint_name.eq(&ACCOUNT_BLUEPRINT))
                        {
                            return Err(RuntimeError::KernelError(KernelError::InvalidId(node_id)));
                        }
                    }
                    GlobalAddress::Component(ComponentAddress::Normal(..)) => {
                        if package_address.eq(&ACCOUNT_PACKAGE)
                            && blueprint_name.eq(&ACCOUNT_BLUEPRINT)
                        {
                            return Err(RuntimeError::KernelError(KernelError::InvalidId(node_id)));
                        }
                    }
                    _ => {
                        return Err(RuntimeError::KernelError(KernelError::InvalidId(node_id)));
                    }
                }
            }
            (RENodeId::Bucket(..), RENodeInit::Bucket(..)) => {}
            (RENodeId::TransactionRuntime(..), RENodeInit::TransactionRuntime(..)) => {}
            (RENodeId::Proof(..), RENodeInit::Proof(..)) => {}
            (RENodeId::AuthZoneStack(..), RENodeInit::AuthZoneStack(..)) => {}
            (RENodeId::Vault(..), RENodeInit::Vault(..)) => {}
            (RENodeId::Component(..), RENodeInit::Component(..)) => {}
            (RENodeId::Worktop, RENodeInit::Worktop(..)) => {}
            (RENodeId::Logger, RENodeInit::Logger(..)) => {}
            (RENodeId::Package(..), RENodeInit::Package(..)) => {}
            (RENodeId::KeyValueStore(..), RENodeInit::KeyValueStore(..)) => {}
            (RENodeId::NonFungibleStore(..), RENodeInit::NonFungibleStore(..)) => {}
            (RENodeId::ResourceManager(..), RENodeInit::ResourceManager(..)) => {}
            (RENodeId::EpochManager(..), RENodeInit::EpochManager(..)) => {}
            (RENodeId::Validator(..), RENodeInit::Validator(..)) => {}
            (RENodeId::Clock(..), RENodeInit::Clock(..)) => {}
            (RENodeId::Identity(..), RENodeInit::Identity(..)) => {}
            (RENodeId::AccessController(..), RENodeInit::AccessController(..)) => {}
            _ => return Err(RuntimeError::KernelError(KernelError::InvalidId(node_id))),
        }

        // TODO: For Scrypto components, check state against blueprint schema

        let push_to_store = match re_node {
            RENodeInit::Global(..) | RENodeInit::Logger(..) => true,
            _ => false,
        };

        self.id_allocator.take_node_id(node_id)?;
        self.current_frame.create_node(
            node_id,
            re_node,
            &mut self.heap,
            &mut self.track,
            push_to_store,
        )?;

        // Restore current mode
        self.execution_mode = current_mode;

        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::CreateNode { node_id: &node_id },
            )
            .map_err(RuntimeError::ModuleError)?;

        Ok(())
    }

    fn lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        self.module
            .pre_sys_call(
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

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        // Deref
        let (node_id, derefed_lock) =
            if let Some((node_id, derefed_lock)) = self.node_offset_deref(node_id, &offset)? {
                (node_id, Some(derefed_lock))
            } else {
                (node_id, None)
            };

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

        let maybe_lock_handle = self.current_frame.acquire_lock(
            &mut self.heap,
            &mut self.track,
            node_id,
            offset.clone(),
            flags,
        );

        let lock_handle = match maybe_lock_handle {
            Ok(lock_handle) => lock_handle,
            Err(RuntimeError::KernelError(KernelError::TrackError(TrackError::NotFound(
                SubstateId(node_id, ref offset),
            )))) => {
                if self.try_virtualize(node_id, &offset)? {
                    self.current_frame.acquire_lock(
                        &mut self.heap,
                        &mut self.track,
                        node_id,
                        offset.clone(),
                        flags,
                    )?
                } else {
                    return maybe_lock_handle;
                }
            }
            Err(err) => {
                match &err {
                    // TODO: This is a hack to allow for package imports to be visible
                    // TODO: Remove this once we are able to get this information through the Blueprint ABI
                    RuntimeError::CallFrameError(CallFrameError::RENodeNotVisible(
                        RENodeId::Global(GlobalAddress::Package(package_address)),
                    )) => {
                        let node_id = RENodeId::Global(GlobalAddress::Package(*package_address));
                        let offset = SubstateOffset::Global(GlobalOffset::Global);
                        self.track
                            .acquire_lock(
                                SubstateId(node_id, offset.clone()),
                                LockFlags::read_only(),
                            )
                            .map_err(|_| err.clone())?;
                        self.track
                            .release_lock(SubstateId(node_id, offset.clone()), false)
                            .map_err(|_| err)?;
                        self.current_frame
                            .add_stored_ref(node_id, RENodeVisibilityOrigin::Normal);
                        self.current_frame.acquire_lock(
                            &mut self.heap,
                            &mut self.track,
                            node_id,
                            offset.clone(),
                            flags,
                        )?
                    }
                    _ => return Err(err),
                }
            }
        };

        if let Some(lock_handle) = derefed_lock {
            self.current_frame
                .drop_lock(&mut self.heap, &mut self.track, lock_handle)?;
        }

        // Restore current mode
        self.execution_mode = current_mode;

        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::LockSubstate { lock_handle },
            )
            .map_err(RuntimeError::ModuleError)?;

        Ok(lock_handle)
    }

    fn get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError> {
        self.current_frame.get_lock_info(lock_handle)
    }

    fn drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::DropLock {
                    lock_handle: &lock_handle,
                },
            )
            .map_err(RuntimeError::ModuleError)?;

        self.current_frame
            .drop_lock(&mut self.heap, &mut self.track, lock_handle)?;

        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::DropLock,
            )
            .map_err(RuntimeError::ModuleError)?;

        Ok(())
    }

    fn get_ref(&mut self, lock_handle: LockHandle) -> Result<SubstateRef, RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::GetRef {
                    lock_handle: &lock_handle,
                },
            )
            .map_err(RuntimeError::ModuleError)?;

        // A little hacky: this post sys call is called before the sys call happens due to
        // a mutable borrow conflict for substate ref.
        // Some modules (specifically: ExecutionTraceModule) require that all
        // pre/post callbacks are balanced.
        // TODO: Move post sys call to substate_ref drop() so that it's actually
        // after the sys call processing, not before.
        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::GetRef { lock_handle },
            )
            .map_err(RuntimeError::ModuleError)?;

        let substate_ref =
            self.current_frame
                .get_ref(lock_handle, &mut self.heap, &mut self.track)?;

        Ok(substate_ref)
    }

    fn get_ref_mut(&mut self, lock_handle: LockHandle) -> Result<SubstateRefMut, RuntimeError> {
        self.module
            .pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::GetRefMut {
                    lock_handle: &lock_handle,
                },
            )
            .map_err(RuntimeError::ModuleError)?;

        // A little hacky: this post sys call is called before the sys call happens due to
        // a mutable borrow conflict for substate ref.
        // Some modules (specifically: ExecutionTraceModule) require that all
        // pre/post callbacks are balanced.
        // TODO: Move post sys call to substate_ref drop() so that it's actually
        // after the sys call processing, not before.
        self.module
            .post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::GetRefMut,
            )
            .map_err(RuntimeError::ModuleError)?;

        let substate_ref_mut =
            self.current_frame
                .get_ref_mut(lock_handle, &mut self.heap, &mut self.track)?;

        Ok(substate_ref_mut)
    }
}

impl<'g, 's, W, R, M> ActorApi<RuntimeError> for Kernel<'g, 's, W, R, M>
where
    W: WasmEngine,
    R: FeeReserve,
    M: BaseModule<R>,
{
    fn fn_identifier(&mut self) -> Result<FnIdentifier, RuntimeError> {
        Ok(self.current_frame.actor.identifier.clone())
    }
}
