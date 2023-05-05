use super::call_frame::{CallFrame, LockSubstateError, Visibility};
use super::heap::Heap;
use super::id_allocator::IdAllocator;
use super::kernel_api::{
    KernelApi, KernelInternalApi, KernelInvokeApi, KernelNodeApi, KernelSubstateApi, LockInfo,
};
use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::*;
use crate::kernel::call_frame::Message;
use crate::kernel::kernel_api::{KernelInvocation, SystemState};
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::system::system::SystemService;
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::system_modules::execution_trace::{BucketSnapshot, ProofSnapshot};
use crate::track::interface::{AcquireLockError, NodeSubstates, SubstateStore};
use crate::types::*;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::ClientBlueprintApi;
use radix_engine_interface::blueprints::resource::*;
use resources_tracker_macro::trace_resources;
use sbor::rust::mem;

/// Organizes the radix engine stack to make a function entrypoint available for execution
pub struct KernelBoot<'g, V: SystemCallbackObject, S: SubstateStore> {
    pub id_allocator: &'g mut IdAllocator,
    pub callback: &'g mut SystemConfig<V>,
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

        SystemConfig::on_init(&mut kernel)?;

        let args = IndexedScryptoValue::from_vec(args).map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        for node_id in args.references() {
            if node_id.is_global_virtual() {
                // For virtual accounts and native packages, create a reference directly
                kernel.current_frame.add_ref(*node_id, Visibility::Normal);
                continue;
            }

            if kernel.current_frame.get_node_visibility(node_id).is_some() {
                continue;
            }

            let (handle, _) = kernel
                .store
                .acquire_lock(
                    node_id,
                    TYPE_INFO_BASE_MODULE,
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
                        kernel.current_frame.add_ref(*node_id, Visibility::Normal);
                    } else if blueprint.package_address.eq(&RESOURCE_PACKAGE)
                        && (blueprint.blueprint_name.eq(FUNGIBLE_VAULT_BLUEPRINT)
                            || blueprint.blueprint_name.eq(NON_FUNGIBLE_VAULT_BLUEPRINT))
                    {
                        kernel
                            .current_frame
                            .add_ref(*node_id, Visibility::DirectAccess);
                    } else {
                        return Err(RuntimeError::KernelError(KernelError::InvalidDirectAccess));
                    }
                }
                TypeInfoSubstate::KeyValueStore(..)
                | TypeInfoSubstate::Index
                | TypeInfoSubstate::SortedIndex => {
                    return Err(RuntimeError::KernelError(KernelError::InvalidDirectAccess));
                }
            }
        }

        let mut system = SystemService::new(&mut kernel);

        let rtn =
            system.call_function(package_address, blueprint_name, function_name, args.into())?;
        // Sanity check call frame
        assert!(kernel.prev_frame_stack.is_empty());

        SystemConfig::on_teardown(&mut kernel)?;

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
    current_frame: CallFrame<M::LockData>,
    // This stack could potentially be removed and just use the native stack
    // but keeping this call_frames stack may potentially prove useful if implementing
    // execution pause and/or for better debuggability
    prev_frame_stack: Vec<CallFrame<M::LockData>>,
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
        let mut message = Message::from_indexed_scrypto_value(&invocation.args);
        let sys_invocation = invocation.sys_invocation;
        let actor = invocation.resolved_actor;
        let args = &invocation.args;

        // Note that validity of `resolved_actor` should have been checked before getting here.

        // Before push call frame
        M::before_push_frame(&actor, &mut message, &args, self)?;

        // Push call frame
        {
            self.id_allocator.push();

            let frame = CallFrame::new_child_from_parent(&mut self.current_frame, actor, message)
                .map_err(CallFrameError::CreateFrameError)
                .map_err(KernelError::CallFrameError)?;
            let parent = mem::replace(&mut self.current_frame, frame);
            self.prev_frame_stack.push(parent);
        }

        // Execute
        let (output, message) = {
            // Handle execution start
            M::on_execution_start(self)?;

            // Auto drop locks
            self.current_frame
                .drop_all_locks(&mut self.heap, self.store)
                .map_err(CallFrameError::UnlockSubstateError)
                .map_err(KernelError::CallFrameError)?;

            // Run
            let output = M::invoke_upstream(sys_invocation, args, self)?;
            let mut message = Message::from_indexed_scrypto_value(&output);

            // Handle execution finish
            M::on_execution_finish(&mut message, self)?;

            // Auto-drop locks again in case module forgot to drop
            self.current_frame
                .drop_all_locks(&mut self.heap, self.store)
                .map_err(CallFrameError::UnlockSubstateError)
                .map_err(KernelError::CallFrameError)?;

            (output, message)
        };

        // Pop call frame
        {
            let mut parent = self.prev_frame_stack.pop().unwrap();

            // Move resource
            CallFrame::exchange(&mut self.current_frame, &mut parent, message)
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
    fn kernel_get_node_info(&self, node_id: &NodeId) -> Option<(Visibility, bool)> {
        let info = self.current_frame.get_node_visibility(node_id)?;
        Some(info)
    }

    fn kernel_get_current_depth(&self) -> usize {
        self.current_frame.depth()
    }

    fn kernel_get_system_state(&mut self) -> SystemState<'_, M> {
        let caller = self
            .prev_frame_stack
            .last()
            .and_then(|c| c.actor().as_ref());
        SystemState {
            system: &mut self.callback,
            caller,
            current: self.current_frame.actor().as_ref(),
        }
    }

    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot> {
        let (is_fungible_bucket, resource_address) = if let Some(substate) = self.heap.get_substate(
            &bucket_id,
            TYPE_INFO_BASE_MODULE,
            &TypeInfoOffset::TypeInfo.into(),
        ) {
            let type_info: TypeInfoSubstate = substate.as_typed().unwrap();
            match type_info {
                TypeInfoSubstate::Object(ObjectInfo {
                    blueprint,
                    outer_object,
                    ..
                }) if blueprint.package_address == RESOURCE_PACKAGE
                    && (blueprint.blueprint_name == FUNGIBLE_BUCKET_BLUEPRINT
                        || blueprint.blueprint_name == NON_FUNGIBLE_BUCKET_BLUEPRINT) =>
                {
                    let is_fungible = blueprint.blueprint_name.eq(FUNGIBLE_BUCKET_BLUEPRINT);
                    let parent = outer_object.unwrap();
                    let resource_address: ResourceAddress =
                        ResourceAddress::new_or_panic(parent.as_ref().clone().try_into().unwrap());
                    (is_fungible, resource_address)
                }
                _ => {
                    return None;
                }
            }
        } else {
            return None;
        };

        if is_fungible_bucket {
            let substate = self
                .heap
                .get_substate(
                    bucket_id,
                    OBJECT_BASE_MODULE,
                    &FungibleBucketOffset::Liquid.into(),
                )
                .unwrap();
            let liquid: LiquidFungibleResource = substate.as_typed().unwrap();

            Some(BucketSnapshot::Fungible {
                resource_address,
                liquid: liquid.amount(),
            })
        } else {
            let substate = self
                .heap
                .get_substate(
                    bucket_id,
                    OBJECT_BASE_MODULE,
                    &NonFungibleBucketOffset::Liquid.into(),
                )
                .unwrap();
            let liquid: LiquidNonFungibleResource = substate.as_typed().unwrap();

            Some(BucketSnapshot::NonFungible {
                resource_address,
                liquid: liquid.ids().clone(),
            })
        }
    }

    fn kernel_read_proof(&mut self, proof_id: &NodeId) -> Option<ProofSnapshot> {
        let is_fungible = if let Some(substate) = self.heap.get_substate(
            &proof_id,
            TYPE_INFO_BASE_MODULE,
            &TypeInfoOffset::TypeInfo.into(),
        ) {
            let type_info: TypeInfoSubstate = substate.as_typed().unwrap();
            match type_info {
                TypeInfoSubstate::Object(ObjectInfo { blueprint, .. })
                    if blueprint.package_address == RESOURCE_PACKAGE
                        && (blueprint.blueprint_name == NON_FUNGIBLE_PROOF_BLUEPRINT
                            || blueprint.blueprint_name == FUNGIBLE_PROOF_BLUEPRINT) =>
                {
                    blueprint.blueprint_name.eq(FUNGIBLE_PROOF_BLUEPRINT)
                }
                _ => {
                    return None;
                }
            }
        } else {
            return None;
        };

        if is_fungible {
            let substate = self
                .heap
                .get_substate(
                    proof_id,
                    TYPE_INFO_BASE_MODULE,
                    &TypeInfoOffset::TypeInfo.into(),
                )
                .unwrap();
            let info: TypeInfoSubstate = substate.as_typed().unwrap();
            let resource_address = ResourceAddress::new_or_panic(info.parent().unwrap().into());

            let substate = self
                .heap
                .get_substate(
                    proof_id,
                    OBJECT_BASE_MODULE,
                    &FungibleProofOffset::ProofRefs.into(),
                )
                .unwrap();
            let proof: FungibleProof = substate.as_typed().unwrap();

            Some(ProofSnapshot::Fungible {
                resource_address,
                total_locked: proof.amount(),
            })
        } else {
            let substate = self
                .heap
                .get_substate(
                    proof_id,
                    TYPE_INFO_BASE_MODULE,
                    &TypeInfoOffset::TypeInfo.into(),
                )
                .unwrap();
            let info: TypeInfoSubstate = substate.as_typed().unwrap();
            let resource_address = ResourceAddress::new_or_panic(info.parent().unwrap().into());

            let substate = self
                .heap
                .get_substate(
                    proof_id,
                    OBJECT_BASE_MODULE,
                    &NonFungibleProofOffset::ProofRefs.into(),
                )
                .unwrap();
            let proof: NonFungibleProof = substate.as_typed().unwrap();

            Some(ProofSnapshot::NonFungible {
                resource_address,
                total_locked: proof.non_fungible_local_ids().clone(),
            })
        }
    }
}

impl<'g, M, S> KernelSubstateApi<M::LockData> for Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: SubstateStore,
{
    #[trace_resources(log=node_id.entity_type(), log=module_num)]
    fn kernel_lock_substate_with_default(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<fn() -> IndexedScryptoValue>,
        data: M::LockData,
    ) -> Result<LockHandle, RuntimeError> {
        M::before_lock_substate(&node_id, &module_num, substate_key, &flags, self)?;

        let maybe_lock_handle = self.current_frame.acquire_lock(
            &mut self.heap,
            self.store,
            node_id,
            module_num,
            substate_key,
            flags,
            default,
            data,
        );

        let (lock_handle, first_lock_from_db) = match &maybe_lock_handle {
            Ok((lock_handle, first_lock_from_db)) => (*lock_handle, *first_lock_from_db),
            Err(LockSubstateError::TrackError(track_err)) => {
                if matches!(track_err.as_ref(), AcquireLockError::NotFound(..)) {
                    let retry =
                        M::on_substate_lock_fault(*node_id, module_num, &substate_key, self)?;

                    if retry {
                        self.current_frame
                            .acquire_lock(
                                &mut self.heap,
                                self.store,
                                &node_id,
                                module_num,
                                &substate_key,
                                flags,
                                None,
                                M::LockData::default(),
                            )
                            .map_err(CallFrameError::LockSubstateError)
                            .map_err(KernelError::CallFrameError)?
                    } else {
                        return maybe_lock_handle
                            .map(|(lock_handle, _)| lock_handle)
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
                        let (handle, first_lock_from_db) = self
                            .store
                            .acquire_lock(
                                node_id,
                                OBJECT_BASE_MODULE,
                                substate_key,
                                LockFlags::read_only(),
                            )
                            .map_err(|e| LockSubstateError::TrackError(Box::new(e)))
                            .map_err(CallFrameError::LockSubstateError)
                            .map_err(KernelError::CallFrameError)?;
                        self.store.release_lock(handle);

                        self.current_frame.add_ref(*node_id, Visibility::Normal);
                        let (lock_handle, _) = self
                            .current_frame
                            .acquire_lock(
                                &mut self.heap,
                                self.store,
                                &node_id,
                                OBJECT_BASE_MODULE,
                                substate_key,
                                flags,
                                None,
                                M::LockData::default(),
                            )
                            .map_err(CallFrameError::LockSubstateError)
                            .map_err(KernelError::CallFrameError)?;
                        (lock_handle, first_lock_from_db)
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
        M::after_lock_substate(lock_handle, 0, first_lock_from_db, self)?;

        Ok(lock_handle)
    }

    #[trace_resources]
    fn kernel_get_lock_info(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<LockInfo<M::LockData>, RuntimeError> {
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
        M::on_write_substate(lock_handle, value.len(), self)?;

        self.current_frame
            .write_substate(&mut self.heap, self.store, lock_handle, value)
            .map_err(CallFrameError::WriteSubstateError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)
    }

    fn kernel_set_substate(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        substate_key: SubstateKey,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        self.current_frame
            .set_substate(
                node_id,
                module_num,
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
        module_num: ModuleNumber,
        substate_key: &SubstateKey,
    ) -> Result<Option<IndexedScryptoValue>, RuntimeError> {
        self.current_frame
            .remove_substate(
                node_id,
                module_num,
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
        module_num: ModuleNumber,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError> {
        self.current_frame
            .scan_sorted(node_id, module_num, count, &mut self.heap, self.store)
            .map_err(CallFrameError::ScanSortedSubstatesError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)
    }

    fn kernel_scan_substates(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError> {
        self.current_frame
            .scan_substates(node_id, module_num, count, &mut self.heap, self.store)
            .map_err(CallFrameError::ScanSubstatesError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)
    }

    fn kernel_take_substates(
        &mut self,
        node_id: &NodeId,
        module_num: ModuleNumber,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError> {
        self.current_frame
            .take_substates(node_id, module_num, count, &mut self.heap, self.store)
            .map_err(CallFrameError::TakeSubstatesError)
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
        M::before_invoke(invocation.as_ref(), self)?;

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
