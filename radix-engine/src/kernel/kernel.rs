use super::call_frame::{CallFrame, LockSubstateError, RefType};
use super::heap::Heap;
use super::id_allocator::IdAllocator;
use super::kernel_api::{
    KernelApi, KernelInternalApi, KernelInvokeApi, KernelNodeApi, KernelSubstateApi, LockInfo,
};
use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::*;
use crate::kernel::actor::Actor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::KernelInvocation;
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system::SystemDownstream;
use crate::system::system_callback::SystemCallback;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::execution_trace::{BucketSnapshot, ProofSnapshot};
use crate::types::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientBlueprintApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_stores::interface::{AcquireLockError, NodeSubstates, SubstateStore};
use resources_tracker_macro::trace_resources;
use sbor::rust::mem;

/// Organizes the radix engine stack to make a function entrypoint available for execution
pub struct KernelBoot<'g, V: SystemCallbackObject, S: SubstateStore> {
    pub id_allocator: &'g mut IdAllocator,
    pub callback: &'g mut SystemCallback<V>,
    pub store: &'g mut S,
}

impl<'g, 'h, V: SystemCallbackObject, S: SubstateStore> KernelBoot<'g, V, S> {
    /// Executes a transaction
    pub fn call_function(
        self,
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
            heap: Heap::new(),
            store: self.store,
            id_allocator: self.id_allocator,
            current_frame: CallFrame::new_root(),
            prev_frame_stack: vec![],
            callback: self.callback,
        };

        SystemCallback::on_init(&mut kernel)?;

        let args = IndexedScryptoValue::from_vec(args).map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        for node_id in args.references() {
            if node_id.is_global_virtual() {
                // For virtual accounts and native packages, create a reference directly
                kernel.current_frame.add_ref(*node_id, RefType::Normal);
                continue;
            } else if node_id.is_global_package()
                && is_native_package(PackageAddress::new_or_panic(node_id.0))
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
                TypeInfoSubstate::KeyValueStore(..) | TypeInfoSubstate::SortedStore => {
                    return Err(RuntimeError::KernelError(KernelError::InvalidDirectAccess));
                }
            }
        }

        let mut system = SystemDownstream::new(&mut kernel);

        let rtn =
            system.call_function(package_address, blueprint_name, function_name, args.into())?;
        // Sanity check call frame
        assert!(kernel.prev_frame_stack.is_empty());

        SystemCallback::on_teardown(&mut kernel)?;

        Ok(rtn)
    }
}

pub struct Kernel<
    'g, // Lifetime of values outliving all frames
    M,  // Upstream System layer
    S,  // Substate store
> where
    M: KernelCallbackObject,
    S: SubstateStore,
{
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
    callback: &'g mut M,
}

impl<'g, M, S> Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: SubstateStore,
{
    fn invoke(
        &mut self,
        invocation: Box<KernelInvocation<M::Invocation>>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        let caller = Box::new(self.current_frame.actor.clone());

        let mut call_frame_update = invocation.get_update();
        let sys_invocation = invocation.sys_invocation;
        let actor = &invocation.resolved_actor;
        let args = &invocation.args;

        // Before push call frame
        M::before_push_frame(actor, &mut call_frame_update, &args, self)?;

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
            M::on_execution_start(&caller, self)?;

            // Auto drop locks
            self.current_frame
                .drop_all_locks(&mut self.heap, self.store)
                .map_err(CallFrameError::UnlockSubstateError)
                .map_err(KernelError::CallFrameError)?;

            // Run
            let output = M::invoke_upstream(sys_invocation, args, self)?;

            let mut update = CallFrameUpdate {
                nodes_to_move: output.owned_node_ids().clone(),
                node_refs_to_copy: output.references().clone(),
            };

            // Handle execution finish
            M::on_execution_finish(&caller, &mut update, self)?;

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

            // auto drop
            {
                let owned_nodes = self.current_frame.owned_nodes();
                M::auto_drop(owned_nodes, self)?;
                // Last check
                if let Some(node_id) = self.current_frame.owned_nodes().into_iter().next() {
                    return Err(RuntimeError::KernelError(KernelError::DropNodeFailure(
                        node_id,
                    )));
                }
            }

            // Restore previous frame
            self.current_frame = parent;

            self.id_allocator.pop()?;
        }

        // After pop call frame
        M::after_pop_frame(self)?;

        Ok(output)
    }
}

impl<'g, M, S> KernelNodeApi for Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: SubstateStore,
{
    #[trace_resources(log=node_id.entity_type())]
    fn kernel_drop_node(&mut self, node_id: &NodeId) -> Result<NodeSubstates, RuntimeError> {
        M::before_drop_node(node_id, self)?;

        let node = self
            .current_frame
            .remove_node(&mut self.heap, node_id)
            .map_err(|e| {
                RuntimeError::KernelError(KernelError::CallFrameError(CallFrameError::MoveError(e)))
            })?;

        M::after_drop_node(self)?;

        Ok(node)
    }

    #[trace_resources(log=entity_type)]
    fn kernel_allocate_node_id(&mut self, entity_type: EntityType) -> Result<NodeId, RuntimeError> {
        M::on_allocate_node_id(Some(entity_type), false, self)?;

        let node_id = self.id_allocator.allocate_node_id(entity_type)?;

        Ok(node_id)
    }

    #[trace_resources(log=node_id.entity_type())]
    fn kernel_allocate_virtual_node_id(&mut self, node_id: NodeId) -> Result<(), RuntimeError> {
        M::on_allocate_node_id(node_id.entity_type(), true, self)?;

        self.id_allocator.allocate_virtual_node_id(node_id);

        Ok(())
    }

    #[trace_resources(log=node_id.entity_type())]
    fn kernel_create_node(
        &mut self,
        node_id: NodeId,
        node_substates: NodeSubstates,
    ) -> Result<(), RuntimeError> {
        M::before_create_node(&node_id, &node_substates, self)?;

        let push_to_store = node_id.is_global();

        self.id_allocator.take_node_id(node_id)?;
        self.current_frame
            .create_node(
                node_id,
                node_substates,
                &mut self.heap,
                self.store,
                push_to_store,
            )
            .map_err(CallFrameError::UnlockSubstateError)
            .map_err(KernelError::CallFrameError)?;

        M::after_create_node(&node_id, self)?;

        Ok(())
    }
}

impl<'g, M, S> KernelInternalApi<M> for Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: SubstateStore,
{
    fn kernel_get_node_info(&self, node_id: &NodeId) -> Option<(RefType, bool)> {
        let info = self.current_frame.get_node_visibility(node_id)?;
        Some(info)
    }

    fn kernel_get_callback(&mut self) -> &mut M {
        &mut self.callback
    }

    fn kernel_get_current_depth(&self) -> usize {
        self.current_frame.depth
    }

    fn kernel_get_node_type_info(&mut self, node_id: &NodeId) -> Option<TypeInfo> {
        // This is to solve the bootstrapping problem.
        // TODO: Can be removed if we flush bootstrap state updates without transactional execution.
        if node_id.eq(RADIX_TOKEN.as_node_id()) {
            return Some(TypeInfo::Object {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                global: true,
                type_parent: None,
            });
        } else if node_id.eq(ECDSA_SECP256K1_TOKEN.as_node_id())
            || node_id.eq(EDDSA_ED25519_TOKEN.as_node_id())
            || node_id.eq(SYSTEM_TOKEN.as_node_id())
            || node_id.eq(PACKAGE_TOKEN.as_node_id())
            || node_id.eq(GLOBAL_OBJECT_TOKEN.as_node_id())
            || node_id.eq(PACKAGE_OWNER_TOKEN.as_node_id())
            || node_id.eq(VALIDATOR_OWNER_TOKEN.as_node_id())
            || node_id.eq(IDENTITY_OWNER_TOKEN.as_node_id())
            || node_id.eq(ACCOUNT_OWNER_TOKEN.as_node_id())
        {
            return Some(TypeInfo::Object {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                global: true,
                type_parent: None,
            });
        }

        self.heap
            .get_substate(
                node_id,
                SysModuleId::TypeInfo.into(),
                &TypeInfoOffset::TypeInfo.into(),
            )
            .map(|x| x.as_typed::<TypeInfoSubstate>().unwrap())
            .or_else(|| {
                self.store
                    .acquire_lock(
                        node_id,
                        SysModuleId::TypeInfo.into(),
                        &TypeInfoOffset::TypeInfo.into(),
                        LockFlags::read_only(),
                    )
                    .and_then(|lock| {
                        let substate = self
                            .store
                            .read_substate(lock)
                            .as_typed::<TypeInfoSubstate>()
                            .unwrap();
                        self.store.release_lock(lock);
                        Ok(substate)
                    })
                    .ok()
            })
            .map(|substate| match substate {
                TypeInfoSubstate::Object(ObjectInfo {
                    blueprint,
                    global,
                    type_parent,
                }) => TypeInfo::Object {
                    package_address: blueprint.package_address,
                    blueprint_name: blueprint.blueprint_name,
                    global,
                    type_parent,
                },
                TypeInfoSubstate::KeyValueStore(_) => TypeInfo::KeyValueStore,
                TypeInfoSubstate::SortedStore => TypeInfo::SortedStore,
            })
    }

    // TODO: Remove
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
    fn kernel_load_package_package_dependencies(&mut self) {
        self.current_frame
            .add_ref(RADIX_TOKEN.as_node_id().clone(), RefType::Normal);
    }

    // TODO: Remove
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

    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot> {
        if let Some(substate) = self.heap.get_substate(
            &bucket_id,
            SysModuleId::TypeInfo.into(),
            &TypeInfoOffset::TypeInfo.into(),
        ) {
            let type_info: TypeInfoSubstate = substate.as_typed().unwrap();
            match type_info {
                TypeInfoSubstate::Object(ObjectInfo { blueprint, .. })
                    if blueprint.package_address == RESOURCE_MANAGER_PACKAGE
                        && (blueprint.blueprint_name == FUNGIBLE_BUCKET_BLUEPRINT
                            || blueprint.blueprint_name == NON_FUNGIBLE_BUCKET_BLUEPRINT) => {}
                _ => {
                    return None;
                }
            }
        } else {
            return None;
        }

        if let Some(substate) = self.heap.get_substate(
            &bucket_id,
            SysModuleId::Object.into(),
            &BucketOffset::Info.into(),
        ) {
            let info: BucketInfoSubstate = substate.as_typed().unwrap();

            match info.resource_type {
                ResourceType::Fungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            bucket_id,
                            SysModuleId::Object.into(),
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
                            SysModuleId::Object.into(),
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

    fn kernel_read_proof(&mut self, proof_id: &NodeId) -> Option<ProofSnapshot> {
        if let Some(substate) = self.heap.get_substate(
            &proof_id,
            SysModuleId::TypeInfo.into(),
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
            SysModuleId::Object.into(),
            &ProofOffset::Info.into(),
        ) {
            let info: ProofInfoSubstate = substate.as_typed().unwrap();

            match info.resource_type {
                ResourceType::Fungible { .. } => {
                    let substate = self
                        .heap
                        .get_substate(
                            proof_id,
                            SysModuleId::Object.into(),
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
                            SysModuleId::Object.into(),
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
    M: KernelCallbackObject,
    S: SubstateStore,
{
    #[trace_resources(log=node_id.entity_type(), log=module_id, log=substate_key.to_hex())]
    fn kernel_lock_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
        flags: LockFlags,
    ) -> Result<LockHandle, RuntimeError> {
        M::before_lock_substate(&node_id, &module_id, substate_key, &flags, self)?;

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
                        let module_id = SysModuleId::Object;
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
                                module_id.into(),
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

    #[trace_resources]
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

    fn kernel_set_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: SubstateKey,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        self.current_frame
            .set_substate(
                node_id,
                module_id,
                substate_key,
                value,
                &mut self.heap,
                self.store,
            )
            .map_err(CallFrameError::SetSubstatesError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)
    }

    fn kernel_remove_substate(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Result<Option<IndexedScryptoValue>, RuntimeError> {
        self.current_frame
            .remove_substate(
                node_id,
                module_id,
                &substate_key,
                &mut self.heap,
                self.store,
            )
            .map_err(CallFrameError::RemoveSubstatesError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)
    }

    fn kernel_scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        module_id: ModuleId,
        count: u32,
    ) -> Result<Vec<(SubstateKey, IndexedScryptoValue)>, RuntimeError> {
        self.current_frame
            .scan_sorted(node_id, module_id, count, &mut self.heap, self.store)
            .map_err(CallFrameError::ScanSortedSubstatesError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)
    }
}

impl<'g, M, S> KernelInvokeApi<M::Invocation> for Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: SubstateStore,
{
    #[trace_resources]
    fn kernel_invoke(
        &mut self,
        invocation: Box<KernelInvocation<M::Invocation>>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        M::before_invoke(invocation.as_ref(), invocation.payload_size, self)?;

        let rtn = self.invoke(invocation)?;

        M::after_invoke(
            0, // TODO: Pass the right size
            self,
        )?;

        Ok(rtn)
    }
}

impl<'g, M, S> KernelApi<M> for Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: SubstateStore,
{
}
