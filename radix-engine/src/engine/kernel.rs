use radix_engine_interface::api::api::{EngineApi, Invocation, SysInvokableNative};
use radix_engine_interface::api::types::{
    AuthZoneStackOffset, ComponentOffset, GlobalAddress, GlobalOffset, Level, LockHandle,
    PackageOffset, ProofOffset, RENodeId, ScryptoFunctionIdent, ScryptoPackage, ScryptoReceiver,
    SubstateId, SubstateOffset, VaultId, WorktopOffset,
};
use radix_engine_interface::crypto::Hash;
use radix_engine_interface::data::*;

use radix_engine_interface::rule;
use sbor::rust::fmt::Debug;
use sbor::rust::mem;
use transaction::errors::IdAllocationError;
use transaction::model::AuthZoneParams;
use transaction::validation::*;

use crate::engine::node_move_module::NodeMoveModule;
use crate::engine::system_api::Invokable;
use crate::engine::system_api::LockInfo;
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
    R,  // Fee reserve type
> where
    W: WasmEngine,
    R: FeeReserve,
{
    /// Current execution mode, specifies permissions into state/invocations
    execution_mode: ExecutionMode,

    /// The transaction hash
    transaction_hash: Hash,
    /// Blobs attached to the transaction
    blobs: &'g HashMap<Hash, &'g [u8]>,
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
    track: Track<'s, R>,

    /// Interpreter capable of running scrypto programs
    scrypto_interpreter: &'g ScryptoInterpreter<W>,

    /// Kernel modules
    modules: Vec<Box<dyn Module<R>>>,
    /// The max call depth, TODO: Move into costing module
    max_depth: usize,
}

impl<'g, 's, W, R> Kernel<'g, 's, W, R>
where
    W: WasmEngine,
    R: FeeReserve,
{
    pub fn new(
        transaction_hash: Hash,
        auth_zone_params: AuthZoneParams,
        blobs: &'g HashMap<Hash, &'g [u8]>,
        max_depth: usize,
        track: Track<'s, R>,
        scrypto_interpreter: &'g ScryptoInterpreter<W>,
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

                let node_id = system_api.allocate_node_id(RENodeType::AuthZoneStack)?;
                system_api.create_node(node_id, RENode::AuthZoneStack(auth_zone))?;

                Ok(())
            })
            .expect("AuthModule failed to initialize");

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
            RENodeId::Global(GlobalAddress::System(EPOCH_MANAGER)),
            RENodeVisibilityOrigin::Normal,
        );
        kernel.current_frame.add_stored_ref(
            RENodeId::Global(GlobalAddress::System(CLOCK)),
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

    fn new_uuid(
        id_allocator: &mut IdAllocator,
        transaction_hash: Hash,
    ) -> Result<u128, IdAllocationError> {
        id_allocator.new_uuid(transaction_hash)
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
                let non_fungible_address = match component_address {
                    ComponentAddress::EcdsaSecp256k1VirtualAccount(address) => {
                        NonFungibleAddress::new(
                            ECDSA_SECP256K1_TOKEN,
                            NonFungibleId::Bytes(address.into()),
                        )
                    }
                    ComponentAddress::EddsaEd25519VirtualAccount(address) => {
                        NonFungibleAddress::new(
                            EDDSA_ED25519_TOKEN,
                            NonFungibleId::Bytes(address.into()),
                        )
                    }
                    _ => return Ok(false),
                };

                // TODO: Replace with trusted IndexedScryptoValue
                let access_rule = rule!(require(non_fungible_address));
                let result = self.invoke(ScryptoInvocation::Function(
                    ScryptoFunctionIdent {
                        package: ScryptoPackage::Global(ACCOUNT_PACKAGE),
                        blueprint_name: "Account".to_string(),
                        function_name: "create".to_string(),
                    },
                    args!(access_rule),
                ))?;
                let result = IndexedScryptoValue::from_slice(&result).unwrap();
                let component_id = result.component_ids.into_iter().next().unwrap();

                // TODO: Use system_api to globalize component when create_node is refactored
                // TODO: to allow for address selection
                let global_substate = GlobalAddressSubstate::Component(component_id);

                self.current_frame.create_node(
                    node_id,
                    RENode::Global(global_substate),
                    &mut self.heap,
                    &mut self.track,
                    true,
                    true,
                )?;

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
                RENodeId::Worktop => {
                    let handle = system_api.lock_substate(
                        node_id,
                        SubstateOffset::Worktop(WorktopOffset::Worktop),
                        LockFlags::MUTABLE,
                    )?;
                    let mut substate_ref_mut = system_api.get_ref_mut(handle)?;
                    let worktop = substate_ref_mut.worktop();
                    worktop.drop().map_err(|_| {
                        RuntimeError::KernelError(KernelError::DropNodeFailure(node_id))
                    })?;
                    system_api.drop_lock(handle)?;
                    Ok(())
                }
                RENodeId::Bucket(..) => Ok(()),
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

        self.current_frame.verify_allocated_ids_empty()?;

        Ok(())
    }

    fn run<X: Executor>(
        &mut self,
        executor: X,
        actor: REActor,
        mut call_frame_update: CallFrameUpdate,
    ) -> Result<X::Output, RuntimeError> {
        let derefed_lock = if let REActor::Method(
            _,
            ResolvedReceiver {
                derefed_from: Some((_, derefed_lock)),
                ..
            },
        ) = &actor
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
            self.execute_in_mode(ExecutionMode::AuthModule, |system_api| {
                AuthModule::on_call_frame_enter(&mut call_frame_update, &actor, system_api)
            })?;
            self.execute_in_mode(ExecutionMode::NodeMoveModule, |system_api| {
                NodeMoveModule::on_call_frame_enter(&mut call_frame_update, &actor, system_api)
            })?;
            for m in &mut self.modules {
                m.pre_execute_invocation(
                    &actor,
                    &call_frame_update,
                    &mut self.current_frame,
                    &mut self.heap,
                    &mut self.track,
                )
                .map_err(RuntimeError::ModuleError)?;
            }
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

            for m in &mut self.modules {
                m.post_execute_invocation(
                    &self.prev_frame_stack.last().unwrap().actor,
                    &update,
                    &mut self.current_frame,
                    &mut self.heap,
                    &mut self.track,
                )
                .map_err(RuntimeError::ModuleError)?;
            }

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
            (ExecutionMode::ScryptoInterpreter, ExecutionMode::Application) => Ok(()),
            _ => Err(RuntimeError::KernelError(
                KernelError::InvalidModeTransition(*cur, *next),
            )),
        }
    }

    fn invoke_internal<X: Executor>(
        &mut self,
        executor: X,
        actor: REActor,
        call_frame_update: CallFrameUpdate,
    ) -> Result<X::Output, RuntimeError> {
        // check call depth
        let depth = self.current_frame.depth;
        if depth == self.max_depth {
            return Err(RuntimeError::KernelError(
                KernelError::MaxCallDepthLimitReached,
            ));
        }

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

    pub fn finalize(mut self, result: InvokeResult) -> TrackReceipt {
        let final_result = match result {
            Ok(res) => self.finalize_modules().map(|_| res),
            Err(err) => {
                // If there was an error, we still try to finalize the modules,
                // but forward the original error (even if module finalizer also errors).
                let _silently_ignored = self.finalize_modules();
                Err(err)
            }
        };
        self.track.finalize(final_result)
    }

    fn finalize_modules(&mut self) -> Result<(), RuntimeError> {
        for m in &mut self.modules {
            m.on_finished_processing(&mut self.heap, &mut self.track)
                .map_err(RuntimeError::ModuleError)?;
        }
        Ok(())
    }
}

pub trait ResolveApi<W: WasmEngine> {
    fn deref(&mut self, node_id: RENodeId) -> Result<Option<(RENodeId, LockHandle)>, RuntimeError>;
    fn vm(&mut self) -> &ScryptoInterpreter<W>;
    fn on_wasm_instantiation(&mut self, code: &[u8]) -> Result<(), RuntimeError>;
}

impl<'g, 's, W, R> ResolveApi<W> for Kernel<'g, 's, W, R>
where
    W: WasmEngine,
    R: FeeReserve,
{
    fn deref(&mut self, node_id: RENodeId) -> Result<Option<(RENodeId, LockHandle)>, RuntimeError> {
        self.node_method_deref(node_id)
    }

    fn vm(&mut self) -> &ScryptoInterpreter<W> {
        self.scrypto_interpreter
    }

    fn on_wasm_instantiation(&mut self, code: &[u8]) -> Result<(), RuntimeError> {
        for m in &mut self.modules {
            m.on_wasm_instantiation(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                code,
            )
                .map_err(RuntimeError::ModuleError)?;
        }

        Ok(())
    }
}

pub trait Executor {
    type Output: Debug;

    fn execute<Y>(
        self,
        system_api: &mut Y,
    ) -> Result<(Self::Output, CallFrameUpdate), RuntimeError>
    where
        Y: SystemApi
            + Invokable<ScryptoInvocation>
            + EngineApi<RuntimeError>
            + SysInvokableNative<RuntimeError>;
}

pub trait ExecutableInvocation<W:WasmEngine>: Invocation {
    type Exec: Executor<Output = Self::Output>;

    fn resolve<Y: ResolveApi<W> + SystemApi> (
        self,
        api: &mut Y,
    ) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError>;
}

impl<'g, 's, W, R, N> Invokable<N> for Kernel<'g, 's, W, R>
where
    W: WasmEngine,
    R: FeeReserve,
    N: ExecutableInvocation<W>,
{
    fn invoke(&mut self, invocation: N) -> Result<<N as Invocation>::Output, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::Invoke {
                    invocation: &invocation,
                    input_size: 0,  // TODO: Fix this
                    value_count: 0, // TODO: Fix this
                    depth: self.current_frame.depth,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // Change to kernel mode
        let saved_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        let (actor, call_frame_update, executor) = invocation.resolve(self)?;

        let rtn = self.invoke_internal(executor, actor, call_frame_update)?;

        // Restore previous mode
        self.execution_mode = saved_mode;

        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::Invoke { rtn: &rtn },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(rtn)
    }
}

// TODO: remove redundant code and move this method to the interpreter
/*
impl<'g, 's, W, R> Invokable<ScryptoInvocation> for Kernel<'g, 's, W, R>
where
    W: WasmEngine,
    R: FeeReserve,
{
    fn invoke(
        &mut self,
        invocation: ScryptoInvocation,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::Invoke {
                    invocation: &invocation,
                    input_size: invocation.args().raw.len() as u32,
                    value_count: invocation.args().value_count() as u32,
                    depth: self.current_frame.depth,
                },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        // Change to kernel mode
        let saved_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        let (executor, actor, call_frame_update) = self.resolve(invocation)?;
        let rtn = self.invoke_internal(executor, actor, call_frame_update)?;

        // Restore previous mode
        self.execution_mode = saved_mode;

        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::Invoke { rtn: &rtn },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(rtn)
    }
}
 */

impl<'g, 's, W, R> SystemApi for Kernel<'g, 's, W, R>
where
    W: WasmEngine,
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

        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::ReadOwnedNodes,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

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

        let node = self.drop_node_internal(node_id)?;

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

    fn allocate_node_id(&mut self, node_type: RENodeType) -> Result<RENodeId, RuntimeError> {
        // TODO: Add costing

        let node_id = match node_type {
            RENodeType::AuthZoneStack => self
                .id_allocator
                .new_auth_zone_id()
                .map(|id| RENodeId::AuthZoneStack(id)),
            RENodeType::Bucket => self
                .id_allocator
                .new_bucket_id()
                .map(|id| RENodeId::Bucket(id)),
            RENodeType::Proof => self
                .id_allocator
                .new_proof_id()
                .map(|id| RENodeId::Proof(id)),
            RENodeType::Worktop => Ok(RENodeId::Worktop),
            RENodeType::Vault => self
                .id_allocator
                .new_vault_id(self.transaction_hash)
                .map(|id| RENodeId::Vault(id)),
            RENodeType::KeyValueStore => self
                .id_allocator
                .new_kv_store_id(self.transaction_hash)
                .map(|id| RENodeId::KeyValueStore(id)),
            RENodeType::NonFungibleStore => self
                .id_allocator
                .new_nf_store_id(self.transaction_hash)
                .map(|id| RENodeId::NonFungibleStore(id)),
            RENodeType::Package => {
                // Security Alert: ensure ID allocating will practically never fail
                self.id_allocator
                    .new_package_id(self.transaction_hash)
                    .map(|id| RENodeId::Package(id))
            }
            RENodeType::ResourceManager => self
                .id_allocator
                .new_resource_manager_id(self.transaction_hash)
                .map(|id| RENodeId::ResourceManager(id)),
            RENodeType::Component => self
                .id_allocator
                .new_component_id(self.transaction_hash)
                .map(|id| RENodeId::Component(id)),
            RENodeType::EpochManager => self
                .id_allocator
                .new_component_id(self.transaction_hash)
                .map(|id| RENodeId::EpochManager(id)),
            RENodeType::Clock => self
                .id_allocator
                .new_component_id(self.transaction_hash)
                .map(|id| RENodeId::Clock(id)),
            RENodeType::GlobalPackage => self
                .id_allocator
                .new_package_address(self.transaction_hash)
                .map(|address| RENodeId::Global(GlobalAddress::Package(address))),
            RENodeType::GlobalEpochManager => self
                .id_allocator
                .new_epoch_manager_address(self.transaction_hash)
                .map(|address| RENodeId::Global(GlobalAddress::System(address))),
            RENodeType::GlobalClock => self
                .id_allocator
                .new_clock_address(self.transaction_hash)
                .map(|address| RENodeId::Global(GlobalAddress::System(address))),
            RENodeType::GlobalResourceManager => self
                .id_allocator
                .new_resource_address(self.transaction_hash)
                .map(|address| RENodeId::Global(GlobalAddress::Resource(address))),
            RENodeType::GlobalAccount => self
                .id_allocator
                .new_account_address(self.transaction_hash)
                .map(|address| RENodeId::Global(GlobalAddress::Component(address))),
            RENodeType::GlobalComponent => self
                .id_allocator
                .new_component_address(self.transaction_hash)
                .map(|address| RENodeId::Global(GlobalAddress::Component(address))),
        }
        .map_err(|e| RuntimeError::KernelError(KernelError::IdAllocationError(e)))?;

        self.current_frame.add_allocated_id(node_id);

        Ok(node_id)
    }

    fn create_node(&mut self, node_id: RENodeId, re_node: RENode) -> Result<(), RuntimeError> {
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

        match (node_id, &re_node) {
            (
                RENodeId::Global(GlobalAddress::Package(..)),
                RENode::Global(GlobalAddressSubstate::Package(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::Resource(..)),
                RENode::Global(GlobalAddressSubstate::Resource(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::System(..)),
                RENode::Global(GlobalAddressSubstate::EpochManager(..)),
            ) => {}
            (
                RENodeId::Global(GlobalAddress::System(..)),
                RENode::Global(GlobalAddressSubstate::Clock(..)),
            ) => {}
            (
                RENodeId::Global(address),
                RENode::Global(GlobalAddressSubstate::Component(component)),
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
            (RENodeId::Bucket(..), RENode::Bucket(..)) => {}
            (RENodeId::Proof(..), RENode::Proof(..)) => {}
            (RENodeId::AuthZoneStack(..), RENode::AuthZoneStack(..)) => {}
            (RENodeId::Vault(..), RENode::Vault(..)) => {}
            (RENodeId::Component(..), RENode::Component(..)) => {}
            (RENodeId::Worktop, RENode::Worktop(..)) => {}
            (RENodeId::Package(..), RENode::Package(..)) => {}
            (RENodeId::KeyValueStore(..), RENode::KeyValueStore(..)) => {}
            (RENodeId::NonFungibleStore(..), RENode::NonFungibleStore(..)) => {}
            (RENodeId::ResourceManager(..), RENode::ResourceManager(..)) => {}
            (RENodeId::EpochManager(..), RENode::EpochManager(..)) => {}
            (RENodeId::Clock(..), RENode::Clock(..)) => {}
            _ => return Err(RuntimeError::KernelError(KernelError::InvalidId(node_id))),
        }

        // TODO: For Scrypto components, check state against blueprint schema

        let push_to_store = matches!(re_node, RENode::Global(..));
        self.current_frame.create_node(
            node_id,
            re_node,
            &mut self.heap,
            &mut self.track,
            push_to_store,
            false,
        )?;

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

        Ok(())
    }

    fn lock_substate(
        &mut self,
        node_id: RENodeId,
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
            Err(err) => return Err(err),
        };

        if let Some(lock_handle) = derefed_lock {
            self.current_frame
                .drop_lock(&mut self.heap, &mut self.track, lock_handle)?;
        }

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

    fn get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError> {
        self.current_frame.get_lock_info(lock_handle)
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
            .drop_lock(&mut self.heap, &mut self.track, lock_handle)?;

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

        // A little hacky: this post sys call is called before the sys call happens due to
        // a mutable borrow conflict for substate ref.
        // Some modules (specifically: ExecutionTraceModule) require that all
        // pre/post callbacks are balanced.
        // TODO: Move post sys call to substate_ref drop() so that it's actually
        // after the sys call processing, not before.
        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::GetRef { lock_handle },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let substate_ref =
            self.current_frame
                .get_ref(lock_handle, &mut self.heap, &mut self.track)?;

        Ok(substate_ref)
    }

    fn get_ref_mut(&mut self, lock_handle: LockHandle) -> Result<SubstateRefMut, RuntimeError> {
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

        // A little hacky: this post sys call is called before the sys call happens due to
        // a mutable borrow conflict for substate ref.
        // Some modules (specifically: ExecutionTraceModule) require that all
        // pre/post callbacks are balanced.
        // TODO: Move post sys call to substate_ref drop() so that it's actually
        // after the sys call processing, not before.
        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::GetRefMut,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        let substate_ref_mut =
            self.current_frame
                .get_ref_mut(lock_handle, &mut self.heap, &mut self.track)?;

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

    fn emit_event(&mut self, event: Event) -> Result<(), RuntimeError> {
        for m in &mut self.modules {
            m.pre_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallInput::EmitEvent { event: &event },
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        if let Event::Tracked(tracked_event) = event {
            self.track.add_event(tracked_event);
        }

        for m in &mut self.modules {
            m.post_sys_call(
                &self.current_frame,
                &mut self.heap,
                &mut self.track,
                SysCallOutput::EmitEvent,
            )
            .map_err(RuntimeError::ModuleError)?;
        }

        Ok(())
    }
}

impl<W:WasmEngine> ExecutableInvocation<W> for ScryptoInvocation {
    type Exec = ScryptoExecutor<W::WasmInstance>;

    fn resolve<D: ResolveApi<W> + SystemApi>(self, api: &mut D) -> Result<(REActor, CallFrameUpdate, Self::Exec), RuntimeError> {
        let mut node_refs_to_copy = HashSet::new();
        let args = IndexedScryptoValue::from_slice(&self.args())
            .map_err(|e| RuntimeError::KernelError(KernelError::InvalidScryptoValue(e)))?;

        let nodes_to_move = args.node_ids().into_iter().collect();
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

                let package = api.execute_in_mode::<_, _, RuntimeError>(
                    ExecutionMode::ScryptoInterpreter,
                    |system_api| {
                        let handle = system_api.lock_substate(
                            global_node_id,
                            SubstateOffset::Package(PackageOffset::Info),
                            LockFlags::read_only(),
                        )?;
                        let substate_ref = system_api.get_ref(handle)?;
                        let package = substate_ref.package_info().clone(); // TODO: Remove clone()
                        system_api.drop_lock(handle)?;

                        Ok(package)
                    },
                )?;

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

                if !match_schema_with_value(&fn_abi.input, &args.dom) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoFunctionInvocation(
                            function_ident.clone(),
                            ScryptoFnResolvingError::InvalidInput,
                        ),
                    ));
                }

                // Emit event
                api.on_wasm_instantiation(package.code())?;

                (
                    api.vm().create_executor(&package.code, args),
                    REActor::Function(ResolvedFunction::Scrypto {
                        package_address,
                        blueprint_name: function_ident.blueprint_name.clone(),
                        ident: function_ident.function_name.clone(),
                        export_name: fn_abi.export_name.clone(),
                        return_type: fn_abi.output.clone(),
                    }),
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
                let resolved_receiver = if let Some((derefed, derefed_lock)) =
                api.deref(original_node_id)?
                {
                    ResolvedReceiver::derefed(derefed, original_node_id, derefed_lock)
                } else {
                    ResolvedReceiver::new(original_node_id)
                };

                // Load the package substate
                // TODO: Move this in a better spot when more refactors are done
                let component_node_id = resolved_receiver.receiver;
                let component_info = api.execute_in_mode::<_, _, RuntimeError>(
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
                let package = api.execute_in_mode::<_, _, RuntimeError>(
                    ExecutionMode::ScryptoInterpreter,
                    |system_api| {
                        let package_global = RENodeId::Global(GlobalAddress::Package(
                            component_info.package_address,
                        ));
                        let handle = system_api.lock_substate(
                            package_global,
                            SubstateOffset::Package(PackageOffset::Info),
                            LockFlags::read_only(),
                        )?;
                        let substate_ref = system_api.get_ref(handle)?;
                        let package = substate_ref.package_info().clone(); // TODO: Remove clone()
                        system_api.drop_lock(handle)?;

                        Ok(package)
                    },
                )?;

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
                if !match_schema_with_value(&fn_abi.input, &args.dom) {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::InvalidScryptoMethodInvocation(
                            method_ident.clone(),
                            ScryptoFnResolvingError::InvalidInput,
                        ),
                    ));
                }

                // Emit event
                api.on_wasm_instantiation(package.code())?;

                (
                    api.vm().create_executor(&package.code, args),
                    REActor::Method(
                        ResolvedMethod::Scrypto {
                            package_address: component_info.package_address,
                            blueprint_name: component_info.blueprint_name,
                            ident: method_ident.method_name.clone(),
                            export_name: fn_abi.export_name.clone(),
                            return_type: fn_abi.output.clone(),
                        },
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