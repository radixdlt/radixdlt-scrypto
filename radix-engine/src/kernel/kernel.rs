use super::actor::ExecutionMode;
use super::call_frame::{CallFrame, LockSubstateError, RefType};
use super::heap::{Heap, HeapNode};
use super::id_allocator::IdAllocator;
use super::kernel_api::{
    KernelInternalApi, KernelInvokeDownstreamApi, KernelNodeApi, KernelSubstateApi,
    KernelApi, LockInfo,
};
use crate::blueprints::resource::*;
use crate::errors::*;
use crate::errors::{InvalidSubstateAccess, RuntimeError};
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::{KernelInvocation, KernelUpstream};
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::node_properties::NodeProperties;
use crate::system::system_downstream::SystemDownstream;
use crate::system::system_modules::execution_trace::{BucketSnapshot, ProofSnapshot};
use crate::system::system_upstream::SystemUpstream;
use crate::types::*;
use crate::vm::wasm::WasmEngine;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_stores::interface::{AcquireLockError, SubstateStore};
use resources_tracker_macro::trace_resources;
use sbor::rust::mem;

pub struct RadixEngine;

impl RadixEngine {
    pub fn call_function<'g, W: WasmEngine, S: SubstateStore>(
        id_allocator: &'g mut IdAllocator,
        upstream: &'g mut SystemUpstream<W>,
        store: &'g mut S,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        #[cfg(feature = "resource_tracker")]
        radix_engine_utils::QEMU_PLUGIN_CALIBRATOR.with(|v| {
            v.borrow_mut();
        });

        let mut kernel = Kernel {
            execution_mode: ExecutionMode::Kernel,
            heap: Heap::new(),
            store,
            id_allocator,
            current_frame: CallFrame::new_root(),
            prev_frame_stack: vec![],
            upstream,
        };

        kernel.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::KernelModule, |api| {
            SystemUpstream::on_init(api)
        })?;

        let args = IndexedScryptoValue::from_vec(args)
            .map_err(|e| RuntimeError::SystemInvokeError(SystemInvokeError::InputDecodeError(e)))?;

        for node_id in args.references() {
            if node_id.is_global_virtual() {
                // For virtual accounts and native packages, create a reference directly
                kernel.current_frame.add_ref(*node_id, RefType::Normal);
                continue;
            } else if node_id.is_global_package()
                && is_native_package(PackageAddress::new_unchecked(node_id.0))
            {
                // TODO: This is required for bootstrap, can we clean this up and remove it at some point?
                kernel.current_frame.add_ref(*node_id, RefType::Normal);
                continue;
            }

            if kernel.current_frame.get_node_visibility(node_id).is_some() {
                continue;
            }

            let handle = kernel
                .store
                .acquire_lock(
                    node_id,
                    SysModuleId::TypeInfo.into(),
                    &TypeInfoOffset::TypeInfo.into(),
                    LockFlags::read_only(),
                )
                .map_err(|_| KernelError::NodeNotFound(*node_id))?;
            let substate_ref = kernel.store.read_substate(handle);
            let type_substate: TypeInfoSubstate = substate_ref.as_typed().unwrap();
            kernel.store.release_lock(handle);
            match type_substate {
                TypeInfoSubstate::Object(ObjectInfo {
                    blueprint, global, ..
                }) => {
                    if global {
                        kernel.current_frame.add_ref(*node_id, RefType::Normal);
                    } else if blueprint.package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                        && (blueprint.blueprint_name.eq(FUNGIBLE_VAULT_BLUEPRINT)
                            || blueprint.blueprint_name.eq(NON_FUNGIBLE_VAULT_BLUEPRINT))
                    {
                        kernel
                            .current_frame
                            .add_ref(*node_id, RefType::DirectAccess);
                    } else {
                        return Err(RuntimeError::KernelError(KernelError::InvalidDirectAccess));
                    }
                }
                TypeInfoSubstate::KeyValueStore(..) => {
                    return Err(RuntimeError::KernelError(KernelError::InvalidDirectAccess));
                }
            }
        }

        let mut system = SystemDownstream::new(&mut kernel);

        let rtn =
            system.call_function(package_address, blueprint_name, function_name, args.into())?;
        // Sanity check call frame
        assert!(kernel.prev_frame_stack.is_empty());

        SystemUpstream::on_teardown(&mut kernel)?;

        Ok(rtn)
    }
}

pub struct Kernel<
    'g, // Lifetime of values outliving all frames
    M,  // Upstream System layer
    S,  // Substate store
> where
    M: KernelUpstream,
    S: SubstateStore,
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
    store: &'g mut S,

    /// ID allocator
    id_allocator: &'g mut IdAllocator,

    /// Upstream system layer
    upstream: &'g mut M,
}

impl<'g, M, S> Kernel<'g, M, S>
where
    M: KernelUpstream,
    S: SubstateStore,
{
    fn drop_node_internal(&mut self, node_id: NodeId) -> Result<HeapNode, RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::DropNode, |api| {
            api.current_frame
                .remove_node(&mut api.heap, &node_id)
                .map_err(|e| {
                    RuntimeError::KernelError(KernelError::CallFrameError(
                        CallFrameError::MoveError(e),
                    ))
                })
        })
    }

    fn auto_drop_nodes_in_frame(&mut self) -> Result<(), RuntimeError> {
        let owned_nodes = self.current_frame.owned_nodes();
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::AutoDrop, |api| {
            M::auto_drop(owned_nodes, api)
        })?;

        // Last check
        if let Some(node_id) = self.current_frame.owned_nodes().into_iter().next() {
            return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                node_id,
            )));
        }

        Ok(())
    }

    fn invoke(
        &mut self,
        invocation: Box<KernelInvocation>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        let caller = Box::new(self.current_frame.actor.clone());

        let mut call_frame_update = invocation.get_update();
        let sys_invocation = invocation.sys_invocation;
        let actor = &invocation.resolved_actor;
        let args = &invocation.args;

        // Before push call frame
        {
            self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                M::before_push_frame(actor, &mut call_frame_update, &args, api)
            })?;
        }

        // Push call frame
        {
            self.id_allocator.push();

            let frame = CallFrame::new_child_from_parent(
                &mut self.current_frame,
                actor.clone(),
                call_frame_update.clone(),
            )
            .map_err(CallFrameError::MoveError)
            .map_err(KernelError::CallFrameError)?;
            let parent = mem::replace(&mut self.current_frame, frame);
            self.prev_frame_stack.push(parent);
        }

        // Execute
        let (output, update) = {
            // Handle execution start
            {
                self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                    M::on_execution_start(&caller, api)
                })?;
            }

            // Auto drop locks
            self.current_frame
                .drop_all_locks(&mut self.heap, self.store)
                .map_err(CallFrameError::UnlockSubstateError)
                .map_err(KernelError::CallFrameError)?;

            // Run
            let output = self.execute_in_mode(ExecutionMode::Client, |api| {
                M::invoke_upstream(sys_invocation, args, api)
            })?;

            let mut update = CallFrameUpdate {
                nodes_to_move: output.owned_node_ids().clone(),
                node_refs_to_copy: output.references().clone(),
            };

            // Handle execution finish
            {
                self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                    M::on_execution_finish(&caller, &mut update, api)
                })?;
            }

            // Auto-drop locks again in case module forgot to drop
            self.current_frame
                .drop_all_locks(&mut self.heap, self.store)
                .map_err(CallFrameError::UnlockSubstateError)
                .map_err(KernelError::CallFrameError)?;

            (output, update)
        };

        // Pop call frame
        {
            let mut parent = self.prev_frame_stack.pop().unwrap();

            // Move resource
            CallFrame::update_upstream(&mut self.current_frame, &mut parent, update)
                .map_err(CallFrameError::MoveError)
                .map_err(KernelError::CallFrameError)?;

            // drop proofs and check resource leak
            self.auto_drop_nodes_in_frame()?;

            // Restore previous frame
            self.current_frame = parent;

            self.id_allocator.pop()?;
        }

        // After pop call frame
        {
            self.execute_in_mode(ExecutionMode::KernelModule, |api| M::after_pop_frame(api))?;
        }

        Ok(output)
    }

    fn verify_valid_mode_transition(
        cur: &ExecutionMode,
        next: &ExecutionMode,
    ) -> Result<(), RuntimeError> {
        match (cur, next) {
            (ExecutionMode::Kernel, ..) => Ok(()),
            (ExecutionMode::Client, ExecutionMode::System) => Ok(()),
            _ => Err(RuntimeError::KernelError(
                KernelError::InvalidModeTransition(*cur, *next),
            )),
        }
    }

    #[inline(always)]
    pub fn execute_in_mode<X, RTN, E>(
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

impl<'g, M, S> KernelNodeApi for Kernel<'g, M, S>
where
    M: KernelUpstream,
    S: SubstateStore,
{
    #[trace_resources]
    fn kernel_drop_node(&mut self, node_id: &NodeId) -> Result<HeapNode, RuntimeError> {
        M::before_drop_node(node_id, self)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        let node = self.drop_node_internal(*node_id)?;

        // Restore current mode
        self.execution_mode = current_mode;

        M::after_drop_node(self)?;

        Ok(node)
    }

    #[trace_resources]
    fn kernel_allocate_node_id(&mut self, entity_type: EntityType) -> Result<NodeId, RuntimeError> {
        // TODO: Add costing
        let node_id = self.id_allocator.allocate_node_id(entity_type)?;

        Ok(node_id)
    }

    #[trace_resources(log=node_id)]
    fn kernel_allocate_virtual_node_id(&mut self, node_id: NodeId) -> Result<(), RuntimeError> {
        self.id_allocator.allocate_virtual_node_id(node_id);

        Ok(())
    }

    #[trace_resources(log=node_id)]
    fn kernel_create_node(
        &mut self,
        node_id: NodeId,
        module_init: BTreeMap<SysModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>,
    ) -> Result<(), RuntimeError> {
        M::before_create_node(&node_id, &module_init, self)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        let push_to_store = node_id.is_global();

        self.id_allocator.take_node_id(node_id)?;
        self.current_frame
            .create_node(
                node_id,
                module_init,
                &mut self.heap,
                self.store,
                push_to_store,
            )
            .map_err(CallFrameError::UnlockSubstateError)
            .map_err(KernelError::CallFrameError)?;

        // Restore current mode
        self.execution_mode = current_mode;

        M::after_create_node(&node_id, self)?;

        Ok(())
    }
}

impl<'g, M, S> KernelInternalApi<M> for Kernel<'g, M, S>
where
    M: KernelUpstream,
    S: SubstateStore,
{
    #[trace_resources]
    fn kernel_get_node_info(&self, node_id: &NodeId) -> Option<(RefType, bool)> {
        let info = self.current_frame.get_node_visibility(node_id)?;
        Some(info)
    }

    #[trace_resources]
    fn kernel_get_system(&mut self) -> &mut M {
        self.upstream
    }

    #[trace_resources]
    fn kernel_get_current_depth(&self) -> usize {
        self.current_frame.depth
    }

    fn kernel_set_mode(&mut self, mode: ExecutionMode) {
        self.execution_mode = mode;
    }

    // TODO: Remove
    #[trace_resources]
    fn kernel_get_current_actor(&mut self) -> Option<Actor> {
        let actor = self.current_frame.actor.clone();
        if let Some(actor) = &actor {
            match actor {
                Actor::Method {
                    global_address: Some(address),
                    ..
                } => {
                    self.current_frame
                        .add_ref(address.as_node_id().clone(), RefType::Normal);
                }
                _ => {}
            }
            let package_address = actor.blueprint().package_address;
            self.current_frame
                .add_ref(package_address.as_node_id().clone(), RefType::Normal);
        }

        actor
    }

    // TODO: Remove
    #[trace_resources]
    fn kernel_load_package_package_dependencies(&mut self) {
        self.current_frame
            .add_ref(RADIX_TOKEN.as_node_id().clone(), RefType::Normal);
    }

    // TODO: Remove
    #[trace_resources]
    fn kernel_load_common(&mut self) {
        self.current_frame
            .add_ref(EPOCH_MANAGER.as_node_id().clone(), RefType::Normal);
        self.current_frame
            .add_ref(CLOCK.as_node_id().clone(), RefType::Normal);
        self.current_frame
            .add_ref(RADIX_TOKEN.as_node_id().clone(), RefType::Normal);
        self.current_frame
            .add_ref(PACKAGE_TOKEN.as_node_id().clone(), RefType::Normal);
        self.current_frame
            .add_ref(ECDSA_SECP256K1_TOKEN.as_node_id().clone(), RefType::Normal);
        self.current_frame
            .add_ref(EDDSA_ED25519_TOKEN.as_node_id().clone(), RefType::Normal);
    }

    #[trace_resources]
    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot> {
        if let Some(substate) = self.heap.get_substate(
            &bucket_id,
            SysModuleId::TypeInfo,
            &TypeInfoOffset::TypeInfo.into(),
        ) {
            let type_info: TypeInfoSubstate = substate.as_typed().unwrap();
            match type_info {
                TypeInfoSubstate::Object(ObjectInfo { blueprint, .. })
                    if blueprint.package_address == RESOURCE_MANAGER_PACKAGE
                        && blueprint.blueprint_name == BUCKET_BLUEPRINT => {}
                _ => {
                    return None;
                }
            }
        } else {
            return None;
        }

        if let Some(substate) = self.heap.get_substate(
            &bucket_id,
            SysModuleId::ObjectTuple,
            &BucketOffset::Info.into(),
        ) {
            let info: BucketInfoSubstate = substate.as_typed().unwrap();

            match info.resource_type {
                ResourceType::Fungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            bucket_id,
                            SysModuleId::ObjectTuple,
                            &BucketOffset::LiquidFungible.into(),
                        )
                        .unwrap();
                    let liquid: LiquidFungibleResource = substate.as_typed().unwrap();

                    Some(BucketSnapshot::Fungible {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        liquid: liquid.amount(),
                    })
                }
                ResourceType::NonFungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            bucket_id,
                            SysModuleId::ObjectTuple,
                            &BucketOffset::LiquidNonFungible.into(),
                        )
                        .unwrap();
                    let liquid: LiquidNonFungibleResource = substate.as_typed().unwrap();

                    Some(BucketSnapshot::NonFungible {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        liquid: liquid.ids().clone(),
                    })
                }
            }
        } else {
            None
        }
    }

    #[trace_resources]
    fn kernel_read_proof(&mut self, proof_id: &NodeId) -> Option<ProofSnapshot> {
        if let Some(substate) = self.heap.get_substate(
            &proof_id,
            SysModuleId::TypeInfo,
            &TypeInfoOffset::TypeInfo.into(),
        ) {
            let type_info: TypeInfoSubstate = substate.as_typed().unwrap();
            match type_info {
                TypeInfoSubstate::Object(ObjectInfo { blueprint, .. })
                    if blueprint.package_address == RESOURCE_MANAGER_PACKAGE
                        && blueprint.blueprint_name == PROOF_BLUEPRINT => {}
                _ => {
                    return None;
                }
            }
        } else {
            return None;
        }

        if let Some(substate) = self.heap.get_substate(
            proof_id,
            SysModuleId::ObjectTuple,
            &ProofOffset::Info.into(),
        ) {
            let info: ProofInfoSubstate = substate.as_typed().unwrap();

            match info.resource_type {
                ResourceType::Fungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            proof_id,
                            SysModuleId::ObjectTuple,
                            &ProofOffset::Fungible.into(),
                        )
                        .unwrap();
                    let proof: FungibleProof = substate.as_typed().unwrap();

                    Some(ProofSnapshot::Fungible {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        restricted: info.restricted,
                        total_locked: proof.amount(),
                    })
                }
                ResourceType::NonFungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            proof_id,
                            SysModuleId::ObjectTuple,
                            &ProofOffset::NonFungible.into(),
                        )
                        .unwrap();
                    let proof: NonFungibleProof = substate.as_typed().unwrap();

                    Some(ProofSnapshot::NonFungible {
                        resource_address: info.resource_address,
                        resource_type: info.resource_type,
                        restricted: info.restricted,
                        total_locked: proof.non_fungible_local_ids().clone(),
                    })
                }
            }
        } else {
            None
        }
    }
}

impl<'g, M, S> KernelSubstateApi for Kernel<'g, M, S>
where
    M: KernelUpstream,
    S: SubstateStore,
{
    #[trace_resources(log={*node_id}, log=module_id, log={substate_key.to_hex()})]
    fn kernel_lock_substate(
        &mut self,
        node_id: &NodeId,
        module_id: SysModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        M::before_lock_substate(&node_id, &module_id, substate_key, &flags, self)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        // TODO: Check if valid substate_key for node_id

        // Check node configs
        if let Some(actor) = &self.current_frame.actor {
            if !NodeProperties::can_substate_be_accessed(
                current_mode,
                actor,
                node_id,
                module_id,
                substate_key,
                flags,
            ) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidSubstateAccess(Box::new(InvalidSubstateAccess {
                        mode: current_mode,
                        actor: actor.clone(),
                        node_id: node_id.clone(),
                        substate_key: substate_key.clone(),
                        flags,
                    })),
                ));
            }
        }

        let maybe_lock_handle = self.current_frame.acquire_lock(
            &mut self.heap,
            self.store,
            node_id,
            module_id,
            substate_key,
            flags,
        );

        let lock_handle = match &maybe_lock_handle {
            Ok(lock_handle) => *lock_handle,
            Err(LockSubstateError::TrackError(track_err)) => {
                if matches!(track_err.as_ref(), AcquireLockError::NotFound(..)) {
                    let retry =
                        M::on_substate_lock_fault(*node_id, module_id, &substate_key, self)?;

                    if retry {
                        self.current_frame
                            .acquire_lock(
                                &mut self.heap,
                                self.store,
                                &node_id,
                                module_id,
                                &substate_key,
                                flags,
                            )
                            .map_err(CallFrameError::LockSubstateError)
                            .map_err(KernelError::CallFrameError)?
                    } else {
                        return maybe_lock_handle
                            .map_err(CallFrameError::LockSubstateError)
                            .map_err(KernelError::CallFrameError)
                            .map_err(RuntimeError::KernelError);
                    }
                } else {
                    return Err(RuntimeError::KernelError(KernelError::CallFrameError(
                        CallFrameError::LockSubstateError(LockSubstateError::TrackError(
                            track_err.clone(),
                        )),
                    )));
                }
            }
            Err(err) => {
                match &err {
                    // TODO: This is a hack to allow for package imports to be visible
                    // TODO: Remove this once we are able to get this information through the Blueprint ABI
                    LockSubstateError::NodeNotInCallFrame(node_id)
                        if node_id.is_global_package() =>
                    {
                        let module_id = SysModuleId::ObjectTuple;
                        let handle = self
                            .store
                            .acquire_lock(
                                node_id,
                                module_id.into(),
                                substate_key,
                                LockFlags::read_only(),
                            )
                            .map_err(|e| LockSubstateError::TrackError(Box::new(e)))
                            .map_err(CallFrameError::LockSubstateError)
                            .map_err(KernelError::CallFrameError)?;
                        self.store.release_lock(handle);

                        self.current_frame.add_ref(*node_id, RefType::Normal);
                        self.current_frame
                            .acquire_lock(
                                &mut self.heap,
                                self.store,
                                &node_id,
                                module_id,
                                substate_key,
                                flags,
                            )
                            .map_err(CallFrameError::LockSubstateError)
                            .map_err(KernelError::CallFrameError)?
                    }
                    _ => {
                        return Err(RuntimeError::KernelError(KernelError::CallFrameError(
                            CallFrameError::LockSubstateError(err.clone()),
                        )))
                    }
                }
            }
        };

        // Restore current mode
        self.execution_mode = current_mode;

        // TODO: pass the right size
        M::after_lock_substate(lock_handle, 0, self)?;

        Ok(lock_handle)
    }

    #[trace_resources]
    fn kernel_get_lock_info(&mut self, lock_handle: LockHandle) -> Result<LockInfo, RuntimeError> {
        self.current_frame
            .get_lock_info(lock_handle)
            .ok_or(RuntimeError::KernelError(KernelError::LockDoesNotExist(
                lock_handle,
            )))
    }

    #[trace_resources]
    fn kernel_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        M::on_drop_lock(lock_handle, self)?;

        self.current_frame
            .drop_lock(&mut self.heap, self.store, lock_handle)
            .map_err(CallFrameError::UnlockSubstateError)
            .map_err(KernelError::CallFrameError)?;

        Ok(())
    }

    #[trace_resources]
    fn kernel_read_substate(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<&IndexedScryptoValue, RuntimeError> {
        let mut len = self
            .current_frame
            .read_substate(&mut self.heap, self.store, lock_handle)
            .map_err(CallFrameError::ReadSubstateError)
            .map_err(KernelError::CallFrameError)?
            .as_slice()
            .len();

        // TODO: replace this overwrite with proper packing costing rule
        let lock_info = self.current_frame.get_lock_info(lock_handle).unwrap();
        if lock_info.node_id.is_global_package() {
            len = 0;
        }

        M::on_read_substate(lock_handle, len, self)?;

        Ok(self
            .current_frame
            .read_substate(&mut self.heap, self.store, lock_handle)
            .unwrap())
    }

    fn kernel_write_substate(
        &mut self,
        lock_handle: LockHandle,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        M::on_write_substate(lock_handle, value.as_slice().len(), self)?;

        self.current_frame
            .write_substate(&mut self.heap, self.store, lock_handle, value)
            .map_err(CallFrameError::WriteSubstateError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)
    }
}

impl<'g, M, S> KernelInvokeDownstreamApi for Kernel<'g, M, S>
where
    M: KernelUpstream,
    S: SubstateStore,
{
    #[trace_resources]
    fn kernel_invoke_downstream(
        &mut self,
        invocation: Box<KernelInvocation>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        M::before_invoke(&invocation, invocation.payload_size, self)?;

        // Change to kernel mode
        let saved_mode = self.execution_mode;

        self.execution_mode = ExecutionMode::Kernel;
        let rtn = self.invoke(invocation)?;

        // Restore previous mode
        self.execution_mode = saved_mode;

        M::after_invoke(
            0, // TODO: Pass the right size
            self,
        )?;

        Ok(rtn)
    }
}

impl<'g, M, S> KernelApi<M> for Kernel<'g, M, S>
where
    M: KernelUpstream,
    S: SubstateStore,
{
}
