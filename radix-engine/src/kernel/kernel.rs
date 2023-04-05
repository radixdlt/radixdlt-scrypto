use super::actor::ExecutionMode;
use super::call_frame::{CallFrame, LockSubstateError, RefType};
use super::executor::{ExecutableInvocation, Executor, ResolvedInvocation};
use super::heap::{Heap, HeapNode};
use super::id_allocator::IdAllocator;
use super::interpreters::ScryptoInterpreter;
use super::kernel_api::{
    KernelApi, KernelInternalApi, KernelInvokeApi, KernelModuleApi, KernelNodeApi,
    KernelSubstateApi, KernelWasmApi, LockInfo,
};
use super::module::KernelModule;
use super::module_mixer::KernelModuleMixer;
use super::track::Track;
use crate::blueprints::resource::*;
use crate::errors::*;
use crate::errors::{InvalidDropNodeAccess, InvalidSubstateAccess, RuntimeError};
use crate::kernel::actor::Actor;
use crate::system::kernel_modules::execution_trace::{BucketSnapshot, ProofSnapshot};
use crate::system::node_init::NodeInit;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::node_properties::NodeProperties;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::package::PackageCodeSubstate;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_stores::interface::{AcquireLockError, SubstateStore};
use resources_tracker_macro::trace_resources;
use sbor::rust::mem;

pub struct Kernel<
    'g, // Lifetime of values outliving all frames
    's, // Substate store lifetime
    W,  // WASM engine type
> where
    W: WasmEngine,
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
    track: &'g mut Track<'s>,

    /// ID allocator
    id_allocator: &'g mut IdAllocator,
    /// Interpreter capable of running scrypto programs
    scrypto_interpreter: &'g ScryptoInterpreter<W>,
    /// Kernel module mixer
    module: KernelModuleMixer,
}

impl<'g, 's, W> Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    pub fn new(
        id_allocator: &'g mut IdAllocator,
        track: &'g mut Track<'s>,
        scrypto_interpreter: &'g ScryptoInterpreter<W>,
        module: KernelModuleMixer,
    ) -> Self {
        #[cfg(feature = "resource_tracker")]
        radix_engine_utils::QEMU_PLUGIN_CALIBRATOR.with(|v| {
            v.borrow_mut();
        });

        Self {
            execution_mode: ExecutionMode::Kernel,
            heap: Heap::new(),
            track,
            scrypto_interpreter,
            id_allocator,
            current_frame: CallFrame::new_root(),
            prev_frame_stack: vec![],
            module,
        }
    }

    pub fn initialize(&mut self) -> Result<(), RuntimeError> {
        self.execute_in_mode::<_, _, RuntimeError>(ExecutionMode::KernelModule, |api| {
            KernelModuleMixer::on_init(api)
        })
    }

    // TODO: Josh holds some concern about this interface; will look into this again.
    pub fn teardown<T>(
        mut self,
        previous_result: Result<T, RuntimeError>,
    ) -> (KernelModuleMixer, Result<T, RuntimeError>) {
        let new_result = match previous_result {
            Ok(output) => {
                // Sanity check call frame
                assert!(self.prev_frame_stack.is_empty());

                // Tear down kernel modules
                match self
                    .execute_in_mode::<_, _, RuntimeError>(ExecutionMode::KernelModule, |api| {
                        KernelModuleMixer::on_teardown(api)
                    }) {
                    Ok(_) => Ok(output),
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
        };

        (self.module, new_result)
    }

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
            for node_id in owned_nodes {
                if let Ok(blueprint) = api.get_object_info(&node_id).map(|x| x.blueprint) {
                    match (blueprint.package_address, blueprint.blueprint_name.as_str()) {
                        (RESOURCE_MANAGER_PACKAGE, PROOF_BLUEPRINT) => {
                            api.call_function(
                                RESOURCE_MANAGER_PACKAGE,
                                PROOF_BLUEPRINT,
                                PROOF_DROP_IDENT,
                                scrypto_encode(&ProofDropInput {
                                    proof: Proof(Own(node_id)),
                                })
                                .unwrap(),
                            )?;
                        }
                        _ => {
                            return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                                node_id,
                            )))
                        }
                    }
                } else {
                    return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                        node_id,
                    )));
                }
            }

            Ok(())
        })?;

        // Last check
        if let Some(node_id) = self.current_frame.owned_nodes().into_iter().next() {
            return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                node_id,
            )));
        }

        Ok(())
    }

    fn run<X: Executor>(
        &mut self,
        mut resolved: Box<ResolvedInvocation<X>>,
    ) -> Result<X::Output, RuntimeError> {
        let caller = Box::new(self.current_frame.actor.clone());

        let executor = resolved.executor;
        let actor = &resolved.resolved_actor;
        let args = &resolved.args;
        let call_frame_update = &mut resolved.update;

        // Before push call frame
        {
            self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                KernelModuleMixer::before_push_frame(api, actor, call_frame_update, &args)
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
                    KernelModuleMixer::on_execution_start(api, &caller)
                })?;
            }

            // Auto drop locks
            self.current_frame
                .drop_all_locks(&mut self.heap, &mut self.track)
                .map_err(CallFrameError::UnlockSubstateError)
                .map_err(KernelError::CallFrameError)?;

            // Run
            let (output, mut update) =
                self.execute_in_mode(ExecutionMode::Client, |api| executor.execute(args, api))?;

            // Handle execution finish
            {
                self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                    KernelModuleMixer::on_execution_finish(api, &caller, &mut update)
                })?;
            }

            // Auto-drop locks again in case module forgot to drop
            self.current_frame
                .drop_all_locks(&mut self.heap, &mut self.track)
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
            self.execute_in_mode(ExecutionMode::KernelModule, |api| {
                KernelModuleMixer::after_pop_frame(api)
            })?;
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

    fn invoke_internal<X: Executor>(
        &mut self,
        resolved: Box<ResolvedInvocation<X>>,
    ) -> Result<X::Output, RuntimeError> {
        let depth = self.current_frame.depth;
        // TODO: Move to higher layer
        if depth == 0 {
            for node_id in &resolved.update.node_refs_to_copy {
                if node_id.is_global_virtual() {
                    // For virtual accounts and native packages, create a reference directly
                    self.current_frame.add_ref(*node_id, RefType::Normal);
                    continue;
                } else if node_id.is_global_package()
                    && is_native_package(PackageAddress::new_unchecked(node_id.0))
                {
                    // TODO: This is required for bootstrap, can we clean this up and remove it at some point?
                    self.current_frame.add_ref(*node_id, RefType::Normal);
                    continue;
                }

                if self.current_frame.get_node_visibility(node_id).is_some() {
                    continue;
                }

                let handle = self
                    .track
                    .acquire_lock(
                        node_id,
                        TypedModuleId::TypeInfo.into(),
                        &TypeInfoOffset::TypeInfo.into(),
                        LockFlags::read_only(),
                    )
                    .map_err(|_| KernelError::NodeNotFound(*node_id))?;
                let substate_ref = self.track.read_substate(handle);
                let type_substate: TypeInfoSubstate = substate_ref.as_typed().unwrap();
                self.track.release_lock(handle);
                match type_substate {
                    TypeInfoSubstate::Object(ObjectInfo {
                        blueprint, global, ..
                    }) => {
                        if global {
                            self.current_frame.add_ref(*node_id, RefType::Normal);
                        } else if blueprint.package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                            && (blueprint.blueprint_name.eq(FUNGIBLE_VAULT_BLUEPRINT)
                                || blueprint.blueprint_name.eq(NON_FUNGIBLE_VAULT_BLUEPRINT))
                        {
                            self.current_frame.add_ref(*node_id, RefType::DirectAccess);
                        } else {
                            return Err(RuntimeError::KernelError(
                                KernelError::InvalidDirectAccess,
                            ));
                        }
                    }
                    TypeInfoSubstate::KeyValueStore(..) => {
                        return Err(RuntimeError::KernelError(KernelError::InvalidDirectAccess));
                    }
                }
            }
        }

        let output = self.run(resolved)?;

        Ok(output)
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

impl<'g, 's, W> KernelNodeApi for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    #[trace_resources]
    fn kernel_drop_node(&mut self, node_id: &NodeId) -> Result<HeapNode, RuntimeError> {
        KernelModuleMixer::before_drop_node(self, &node_id)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        // TODO: Move this into the system layer
        if let Some(actor) = self.current_frame.actor.clone() {
            let info = self.get_object_info(node_id)?;
            if !NodeProperties::can_be_dropped(
                current_mode,
                &actor,
                info.blueprint.package_address,
                info.blueprint.blueprint_name.as_str(),
            ) {
                return Err(RuntimeError::KernelError(
                    KernelError::InvalidDropNodeAccess(Box::new(InvalidDropNodeAccess {
                        mode: current_mode,
                        actor: actor.clone(),
                        node_id: node_id.clone(),
                        package_address: info.blueprint.package_address,
                        blueprint_name: info.blueprint.blueprint_name,
                    })),
                ));
            }
        }

        let node = self.drop_node_internal(*node_id)?;

        // Restore current mode
        self.execution_mode = current_mode;

        KernelModuleMixer::after_drop_node(self)?;

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
        node_init: NodeInit,
        module_init: BTreeMap<TypedModuleId, BTreeMap<SubstateKey, IndexedScryptoValue>>,
    ) -> Result<(), RuntimeError> {
        KernelModuleMixer::before_create_node(self, &node_id, &node_init, &module_init)?;

        // Change to kernel mode
        let current_mode = self.execution_mode;
        self.execution_mode = ExecutionMode::Kernel;

        let push_to_store = node_id.is_global();

        self.id_allocator.take_node_id(node_id)?;
        self.current_frame
            .create_node(
                node_id,
                node_init,
                module_init,
                &mut self.heap,
                &mut self.track,
                push_to_store,
            )
            .map_err(CallFrameError::UnlockSubstateError)
            .map_err(KernelError::CallFrameError)?;

        // Restore current mode
        self.execution_mode = current_mode;

        KernelModuleMixer::after_create_node(self, &node_id)?;

        Ok(())
    }
}

impl<'g, 's, W> KernelInternalApi for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    #[trace_resources]
    fn kernel_get_node_info(&self, node_id: &NodeId) -> Option<(RefType, bool)> {
        let info = self.current_frame.get_node_visibility(node_id)?;
        Some(info)
    }

    #[trace_resources]
    fn kernel_get_module_state(&mut self) -> &mut KernelModuleMixer {
        &mut self.module
    }

    #[trace_resources]
    fn kernel_get_current_depth(&self) -> usize {
        self.current_frame.depth
    }

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

    #[trace_resources]
    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot> {
        if let Some(substate) = self.heap.get_substate(
            &bucket_id,
            TypedModuleId::TypeInfo,
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
            TypedModuleId::ObjectState,
            &BucketOffset::Info.into(),
        ) {
            let info: BucketInfoSubstate = substate.as_typed().unwrap();

            match info.resource_type {
                ResourceType::Fungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            bucket_id,
                            TypedModuleId::ObjectState,
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
                            TypedModuleId::ObjectState,
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
            TypedModuleId::TypeInfo,
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
            TypedModuleId::ObjectState,
            &ProofOffset::Info.into(),
        ) {
            let info: ProofInfoSubstate = substate.as_typed().unwrap();

            match info.resource_type {
                ResourceType::Fungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            proof_id,
                            TypedModuleId::ObjectState,
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
                            TypedModuleId::ObjectState,
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

impl<'g, 's, W> KernelSubstateApi for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    #[trace_resources(log={*node_id}, log=module_id, log={substate_key.to_hex()})]
    fn kernel_lock_substate(
        &mut self,
        node_id: &NodeId,
        module_id: TypedModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        KernelModuleMixer::before_lock_substate(self, &node_id, &module_id, substate_key, &flags)?;

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
            &mut self.track,
            node_id,
            module_id,
            substate_key,
            flags,
        );

        let lock_handle = match &maybe_lock_handle {
            Ok(lock_handle) => *lock_handle,
            Err(LockSubstateError::TrackError(track_err)) => {
                if matches!(track_err.as_ref(), AcquireLockError::NotFound(..)) {
                    let retry = KernelModuleMixer::on_substate_lock_fault(
                        *node_id,
                        module_id,
                        &substate_key,
                        self,
                    )?;
                    if retry {
                        self.current_frame
                            .acquire_lock(
                                &mut self.heap,
                                &mut self.track,
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
                        let module_id = TypedModuleId::ObjectState;
                        let handle = self
                            .track
                            .acquire_lock(
                                node_id,
                                module_id.into(),
                                substate_key,
                                LockFlags::read_only(),
                            )
                            .map_err(|e| LockSubstateError::TrackError(Box::new(e)))
                            .map_err(CallFrameError::LockSubstateError)
                            .map_err(KernelError::CallFrameError)?;
                        self.track.release_lock(handle);

                        self.current_frame.add_ref(*node_id, RefType::Normal);
                        self.current_frame
                            .acquire_lock(
                                &mut self.heap,
                                &mut self.track,
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
        KernelModuleMixer::after_lock_substate(self, lock_handle, 0)?;

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
        KernelModuleMixer::on_drop_lock(self, lock_handle)?;

        self.current_frame
            .drop_lock(&mut self.heap, &mut self.track, lock_handle)
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
            .read_substate(&mut self.heap, &mut self.track, lock_handle)
            .map_err(CallFrameError::ReadSubstateError)
            .map_err(KernelError::CallFrameError)?
            .as_slice()
            .len();

        // TODO: replace this overwrite with proper packing costing rule
        let lock_info = self.current_frame.get_lock_info(lock_handle).unwrap();
        if lock_info.node_id.is_global_package() {
            len = 0;
        }

        KernelModuleMixer::on_read_substate(self, lock_handle, len)?;

        Ok(self
            .current_frame
            .read_substate(&mut self.heap, &mut self.track, lock_handle)
            .unwrap())
    }

    fn kernel_write_substate(
        &mut self,
        lock_handle: LockHandle,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        KernelModuleMixer::on_write_substate(self, lock_handle, value.as_slice().len())?;

        self.current_frame
            .write_substate(&mut self.heap, &mut self.track, lock_handle, value)
            .map_err(CallFrameError::WriteSubstateError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)
    }
}

impl<'g, 's, W> KernelWasmApi<W> for Kernel<'g, 's, W>
where
    W: WasmEngine,
{
    #[trace_resources]
    fn kernel_create_wasm_instance(
        &mut self,
        package_address: PackageAddress,
        handle: LockHandle,
    ) -> Result<W::WasmInstance, RuntimeError> {
        // TODO: check if save to unwrap
        let package_code: PackageCodeSubstate =
            self.kernel_read_substate(handle)?.as_typed().unwrap();

        Ok(self
            .scrypto_interpreter
            .create_instance(package_address, &package_code.code))
    }
}

impl<'g, 's, W, N> KernelInvokeApi<N, RuntimeError> for Kernel<'g, 's, W>
where
    W: WasmEngine,
    N: ExecutableInvocation,
{
    #[trace_resources]
    fn kernel_invoke(
        &mut self,
        invocation: Box<N>,
    ) -> Result<<N as Invocation>::Output, RuntimeError> {
        KernelModuleMixer::before_invoke(
            self,
            &invocation.debug_identifier(),
            invocation.payload_size(),
        )?;

        // Change to kernel mode
        let saved_mode = self.execution_mode;

        self.execution_mode = ExecutionMode::Resolver;
        let resolved = invocation.resolve(self)?;

        self.execution_mode = ExecutionMode::Kernel;
        let rtn = self.invoke_internal(resolved)?;

        // Restore previous mode
        self.execution_mode = saved_mode;

        KernelModuleMixer::after_invoke(
            self, 0, // TODO: Pass the right size
        )?;

        Ok(rtn)
    }
}

impl<'g, 's, W> KernelApi<W, RuntimeError> for Kernel<'g, 's, W> where W: WasmEngine {}

impl<'g, 's, W> KernelModuleApi<RuntimeError> for Kernel<'g, 's, W> where W: WasmEngine {}
