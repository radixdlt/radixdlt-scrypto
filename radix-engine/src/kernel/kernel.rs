use super::actor::{Actor, MethodActor};
use super::call_frame::{CallFrame, NodeVisibility, OpenSubstateError};
use super::heap::Heap;
use super::id_allocator::IdAllocator;
use super::kernel_api::{
    KernelApi, KernelInternalApi, KernelInvokeApi, KernelNodeApi, KernelSubstateApi, LockInfo,
};
use crate::blueprints::resource::*;
use crate::blueprints::transaction_processor::TransactionProcessorRunInputEfficientEncodable;
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
use crate::track::interface::{AcquireLockError, NodeSubstates, StoreAccessInfo, SubstateStore};
use crate::types::*;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::ClientBlueprintApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::transaction_processor::{
    TRANSACTION_PROCESSOR_BLUEPRINT, TRANSACTION_PROCESSOR_RUN_IDENT,
};
use resources_tracker_macro::trace_resources;
use sbor::rust::mem;
use transaction::prelude::PreAllocatedAddress;

/// Organizes the radix engine stack to make a function entrypoint available for execution
pub struct KernelBoot<'g, V: SystemCallbackObject, S: SubstateStore> {
    pub id_allocator: &'g mut IdAllocator,
    pub callback: &'g mut SystemConfig<V>,
    pub store: &'g mut S,
}

impl<'g, 'h, V: SystemCallbackObject, S: SubstateStore> KernelBoot<'g, V, S> {
    pub fn create_kernel_for_test_only(&mut self) -> Kernel<SystemConfig<V>, S> {
        Kernel {
            heap: Heap::new(),
            store: self.store,
            id_allocator: self.id_allocator,
            current_frame: CallFrame::new_root(Actor::Root),
            prev_frame_stack: vec![],
            callback: self.callback,
        }
    }

    /// Executes a transaction
    pub fn call_transaction_processor<'a>(
        self,
        manifest_encoded_instructions: &'a [u8],
        pre_allocated_addresses: &'a Vec<PreAllocatedAddress>,
        references: &'a IndexSet<Reference>,
        blobs: &'a IndexMap<Hash, Vec<u8>>,
    ) -> Result<Vec<u8>, RuntimeError> {
        #[cfg(feature = "resource_tracker")]
        radix_engine_profiling::QEMU_PLUGIN_CALIBRATOR.with(|v| {
            v.borrow_mut();
        });

        let mut kernel = Kernel {
            heap: Heap::new(),
            store: self.store,
            id_allocator: self.id_allocator,
            current_frame: CallFrame::new_root(Actor::Root),
            prev_frame_stack: vec![],
            callback: self.callback,
        };

        SystemConfig::on_init(&mut kernel)?;

        // Reference management
        for reference in references.iter() {
            let node_id = &reference.0;
            if node_id.is_global_virtual() {
                // For virtual accounts, create a reference directly
                kernel
                    .current_frame
                    .add_global_reference(GlobalAddress::new_or_panic(node_id.clone().into()));
                continue;
            }

            if kernel
                .current_frame
                .get_node_visibility(node_id)
                .can_be_invoked(false)
            {
                continue;
            }

            // We have a reference to a node which can't be invoked - so it must be a direct access,
            // let's validate it as such

            let (handle, _store_access) = kernel
                .store
                .acquire_lock(
                    node_id,
                    TYPE_INFO_FIELD_PARTITION,
                    &TypeInfoField::TypeInfo.into(),
                    LockFlags::read_only(),
                )
                .map_err(|_| KernelError::InvalidReference(*node_id))?;
            let (substate_ref, _store_access) = kernel.store.read_substate(handle);
            let type_substate: TypeInfoSubstate = substate_ref.as_typed().unwrap();
            kernel.store.close_substate(handle);
            match type_substate {
                TypeInfoSubstate::Object(ObjectInfo {
                    blueprint_id: blueprint,
                    global,
                    ..
                }) => {
                    if global {
                        kernel
                            .current_frame
                            .add_global_reference(GlobalAddress::new_or_panic(
                                node_id.clone().into(),
                            ));
                    } else if blueprint.package_address.eq(&RESOURCE_PACKAGE)
                        && (blueprint.blueprint_name.eq(FUNGIBLE_VAULT_BLUEPRINT)
                            || blueprint.blueprint_name.eq(NON_FUNGIBLE_VAULT_BLUEPRINT))
                    {
                        kernel.current_frame.add_direct_access_reference(
                            InternalAddress::new_or_panic(node_id.clone().into()),
                        );
                    } else {
                        return Err(RuntimeError::KernelError(KernelError::InvalidDirectAccess));
                    }
                }
                _ => {
                    return Err(RuntimeError::KernelError(KernelError::InvalidDirectAccess));
                }
            }
        }

        // Allocate global addresses
        let mut global_address_reservations = Vec::new();
        for PreAllocatedAddress {
            blueprint_id,
            address,
        } in pre_allocated_addresses
        {
            let mut system = SystemService::new(&mut kernel);
            let global_address_reservation =
                system.prepare_global_address(blueprint_id.clone(), address.clone())?;
            global_address_reservations.push(global_address_reservation);
        }

        // Call TX processor
        let mut system = SystemService::new(&mut kernel);
        let rtn = system.call_function(
            TRANSACTION_PROCESSOR_PACKAGE,
            TRANSACTION_PROCESSOR_BLUEPRINT,
            TRANSACTION_PROCESSOR_RUN_IDENT,
            scrypto_encode(&TransactionProcessorRunInputEfficientEncodable {
                manifest_encoded_instructions,
                global_address_reservations,
                references,
                blobs,
            })
            .unwrap(),
        )?;

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
        invocation: Box<KernelInvocation>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        // Check actor visibility
        let can_be_invoked = match &invocation.actor {
            Actor::Method(MethodActor {
                node_id,
                is_direct_access,
                ..
            }) => self
                .current_frame
                .get_node_visibility(&node_id)
                .can_be_invoked(*is_direct_access),
            Actor::Function {
                blueprint_id: blueprint,
                ..
            }
            | Actor::VirtualLazyLoad {
                blueprint_id: blueprint,
                ..
            } => {
                // FIXME: combine this with reference check of invocation
                self.current_frame
                    .get_node_visibility(blueprint.package_address.as_node_id())
                    .can_be_invoked(false)
            }
            Actor::Root => true,
        };
        if !can_be_invoked {
            return Err(RuntimeError::KernelError(KernelError::InvalidInvokeAccess));
        }

        // Before push call frame
        let mut message = Message::from_indexed_scrypto_value(&invocation.args);
        let actor = invocation.actor;
        let args = &invocation.args;
        M::before_push_frame(&actor, &mut message, &args, self)?;

        // Push call frame
        {
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
                .map_err(CallFrameError::CloseSubstateError)
                .map_err(KernelError::CallFrameError)?;

            // Run
            let output = M::invoke_upstream(args, self)?;
            let mut message = Message::from_indexed_scrypto_value(&output);

            // Auto-drop locks again in case module forgot to drop
            self.current_frame
                .drop_all_locks(&mut self.heap, self.store)
                .map_err(CallFrameError::CloseSubstateError)
                .map_err(KernelError::CallFrameError)?;

            // Handle execution finish
            M::on_execution_finish(&mut message, self)?;

            (output, message)
        };

        // Move
        {
            let parent = self.prev_frame_stack.last_mut().unwrap();

            // Move resource
            CallFrame::pass_message(&mut self.current_frame, parent, message)
                .map_err(CallFrameError::PassMessageError)
                .map_err(KernelError::CallFrameError)?;

            // Auto-drop
            let owned_nodes = self.current_frame.owned_nodes();
            M::auto_drop(owned_nodes, self)?;

            // Now, check if any own has been left!
            if let Some(node_id) = self.current_frame.owned_nodes().into_iter().next() {
                return Err(RuntimeError::KernelError(KernelError::NodeOrphaned(
                    node_id,
                )));
            }
        }

        // Pop call frame
        {
            let parent = self.prev_frame_stack.pop().unwrap();

            let dropped_frame = core::mem::replace(&mut self.current_frame, parent);

            M::after_pop_frame(self, dropped_frame.actor())?;
        }

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
            .drop_node(&mut self.heap, node_id)
            .map_err(CallFrameError::DropNodeError)
            .map_err(KernelError::CallFrameError)?;

        let total_substate_size = node
            .values()
            .map(|x| x.values().map(|x| x.len()).sum::<usize>())
            .sum::<usize>();

        M::after_drop_node(self, total_substate_size)?;

        Ok(node)
    }

    #[trace_resources(log=entity_type)]
    fn kernel_allocate_node_id(&mut self, entity_type: EntityType) -> Result<NodeId, RuntimeError> {
        M::on_allocate_node_id(entity_type, self)?;

        self.id_allocator.allocate_node_id(entity_type)
    }

    #[trace_resources(log=node_id.entity_type())]
    fn kernel_create_node(
        &mut self,
        node_id: NodeId,
        node_substates: NodeSubstates,
    ) -> Result<(), RuntimeError> {
        M::before_create_node(&node_id, &node_substates, self)?;

        let total_substate_size = node_substates
            .values()
            .map(|x| x.values().map(|x| x.len()).sum::<usize>())
            .sum::<usize>();

        let store_access = self
            .current_frame
            .create_node(
                node_id,
                node_substates,
                &mut self.heap,
                self.store,
                node_id.is_global(),
            )
            .map_err(CallFrameError::CreateNodeError)
            .map_err(KernelError::CallFrameError)?;

        M::after_create_node(&node_id, total_substate_size, &store_access, self)?;

        Ok(())
    }

    #[trace_resources]
    fn kernel_move_module(
        &mut self,
        src_node_id: &NodeId,
        src_partition_number: PartitionNumber,
        dest_node_id: &NodeId,
        dest_partition_number: PartitionNumber,
    ) -> Result<(), RuntimeError> {
        let store_access = self
            .current_frame
            .move_module(
                src_node_id,
                src_partition_number,
                dest_node_id,
                dest_partition_number,
                &mut self.heap,
                self.store,
            )
            .map_err(CallFrameError::MoveModuleError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)?;

        M::after_move_modules(src_node_id, dest_node_id, &store_access, self)?;

        Ok(())
    }
}

impl<'g, M, S> KernelInternalApi<M> for Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: SubstateStore,
{
    fn kernel_get_node_visibility(&self, node_id: &NodeId) -> NodeVisibility {
        self.current_frame.get_node_visibility(node_id)
    }

    fn kernel_get_current_depth(&self) -> usize {
        self.current_frame.depth()
    }

    fn kernel_get_system_state(&mut self) -> SystemState<'_, M> {
        let caller = match self.prev_frame_stack.last() {
            Some(call_frame) => call_frame.actor(),
            None => {
                // This will only occur on initialization
                self.current_frame.actor()
            }
        };
        SystemState {
            system: &mut self.callback,
            caller,
            current: self.current_frame.actor(),
        }
    }

    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot> {
        let (is_fungible_bucket, resource_address) = if let Some(substate) = self.heap.get_substate(
            &bucket_id,
            TYPE_INFO_FIELD_PARTITION,
            &TypeInfoField::TypeInfo.into(),
        ) {
            let type_info: TypeInfoSubstate = substate.as_typed().unwrap();
            match type_info {
                TypeInfoSubstate::Object(info)
                    if info.blueprint_id.package_address == RESOURCE_PACKAGE
                        && (info.blueprint_id.blueprint_name == FUNGIBLE_BUCKET_BLUEPRINT
                            || info.blueprint_id.blueprint_name
                                == NON_FUNGIBLE_BUCKET_BLUEPRINT) =>
                {
                    let is_fungible = info
                        .blueprint_id
                        .blueprint_name
                        .eq(FUNGIBLE_BUCKET_BLUEPRINT);
                    let parent = info.get_outer_object();
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
                    MAIN_BASE_PARTITION,
                    &FungibleBucketField::Liquid.into(),
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
                    MAIN_BASE_PARTITION,
                    &NonFungibleBucketField::Liquid.into(),
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
            TYPE_INFO_FIELD_PARTITION,
            &TypeInfoField::TypeInfo.into(),
        ) {
            let type_info: TypeInfoSubstate = substate.as_typed().unwrap();
            match type_info {
                TypeInfoSubstate::Object(ObjectInfo {
                    blueprint_id: blueprint,
                    ..
                }) if blueprint.package_address == RESOURCE_PACKAGE
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
                    TYPE_INFO_FIELD_PARTITION,
                    &TypeInfoField::TypeInfo.into(),
                )
                .unwrap();
            let info: TypeInfoSubstate = substate.as_typed().unwrap();
            let resource_address =
                ResourceAddress::new_or_panic(info.outer_object().unwrap().into());

            let substate = self
                .heap
                .get_substate(
                    proof_id,
                    MAIN_BASE_PARTITION,
                    &FungibleProofField::ProofRefs.into(),
                )
                .unwrap();
            let proof: FungibleProofSubstate = substate.as_typed().unwrap();

            Some(ProofSnapshot::Fungible {
                resource_address,
                total_locked: proof.amount(),
            })
        } else {
            let substate = self
                .heap
                .get_substate(
                    proof_id,
                    TYPE_INFO_FIELD_PARTITION,
                    &TypeInfoField::TypeInfo.into(),
                )
                .unwrap();
            let info: TypeInfoSubstate = substate.as_typed().unwrap();
            let resource_address =
                ResourceAddress::new_or_panic(info.outer_object().unwrap().into());

            let substate = self
                .heap
                .get_substate(
                    proof_id,
                    MAIN_BASE_PARTITION,
                    &NonFungibleProofField::ProofRefs.into(),
                )
                .unwrap();
            let proof: NonFungibleProofSubstate = substate.as_typed().unwrap();

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
    #[trace_resources(log=node_id.entity_type(), log=partition_num)]
    fn kernel_open_substate_with_default(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<fn() -> IndexedScryptoValue>,
        data: M::LockData,
    ) -> Result<LockHandle, RuntimeError> {
        M::before_open_substate(&node_id, &partition_num, substate_key, &flags, self)?;

        let maybe_lock_handle = self.current_frame.acquire_lock(
            &mut self.heap,
            self.store,
            node_id,
            partition_num,
            substate_key,
            flags,
            default,
            data,
        );

        let (lock_handle, value_size, store_access): (u32, usize, StoreAccessInfo) =
            match &maybe_lock_handle {
                Ok((lock_handle, value_size, store_access)) => {
                    (*lock_handle, *value_size, store_access.clone())
                }
                Err(OpenSubstateError::TrackError(track_err)) => {
                    if matches!(track_err.as_ref(), AcquireLockError::NotFound(..)) {
                        let retry = M::on_substate_lock_fault(
                            *node_id,
                            partition_num,
                            &substate_key,
                            self,
                        )?;

                        if retry {
                            self.current_frame
                                .acquire_lock(
                                    &mut self.heap,
                                    self.store,
                                    &node_id,
                                    partition_num,
                                    &substate_key,
                                    flags,
                                    None,
                                    M::LockData::default(),
                                )
                                .map_err(CallFrameError::OpenSubstateError)
                                .map_err(KernelError::CallFrameError)?
                        } else {
                            return maybe_lock_handle
                                .map(|(lock_handle, _, _)| lock_handle)
                                .map_err(CallFrameError::OpenSubstateError)
                                .map_err(KernelError::CallFrameError)
                                .map_err(RuntimeError::KernelError);
                        }
                    } else {
                        return Err(RuntimeError::KernelError(KernelError::CallFrameError(
                            CallFrameError::OpenSubstateError(OpenSubstateError::TrackError(
                                track_err.clone(),
                            )),
                        )));
                    }
                }
                Err(err) => {
                    return Err(RuntimeError::KernelError(KernelError::CallFrameError(
                        CallFrameError::OpenSubstateError(err.clone()),
                    )));
                }
            };

        M::after_open_substate(lock_handle, node_id, value_size, &store_access, self)?;

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
    fn kernel_close_substate(&mut self, lock_handle: LockHandle) -> Result<(), RuntimeError> {
        let store_access = self
            .current_frame
            .close_substate(&mut self.heap, self.store, lock_handle)
            .map_err(CallFrameError::CloseSubstateError)
            .map_err(KernelError::CallFrameError)?;

        M::on_close_substate(lock_handle, &store_access, self)?;

        Ok(())
    }

    #[trace_resources]
    fn kernel_read_substate(
        &mut self,
        lock_handle: LockHandle,
    ) -> Result<&IndexedScryptoValue, RuntimeError> {
        let (value, store_access) = self
            .current_frame
            .read_substate(&mut self.heap, self.store, lock_handle)
            .map_err(CallFrameError::ReadSubstateError)
            .map_err(KernelError::CallFrameError)?;
        let value_size = value.len();

        M::on_read_substate(lock_handle, value_size, &store_access, self)?;

        // Double read due to borrow chacker of self.
        Ok(self
            .current_frame
            .read_substate(&mut self.heap, self.store, lock_handle)
            .unwrap()
            .0)
    }

    #[trace_resources]
    fn kernel_write_substate(
        &mut self,
        lock_handle: LockHandle,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        let value_size = value.len();

        let store_access = self
            .current_frame
            .write_substate(&mut self.heap, self.store, lock_handle, value)
            .map_err(CallFrameError::WriteSubstateError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)?;

        M::on_write_substate(lock_handle, value_size, &store_access, self)
    }

    #[trace_resources]
    fn kernel_set_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        let value_size = value.len();
        let store_access = self
            .current_frame
            .set_substate(
                node_id,
                partition_num,
                substate_key,
                value,
                &mut self.heap,
                self.store,
            )
            .map_err(CallFrameError::SetSubstatesError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)?;

        M::on_set_substate(value_size, &store_access, self)?;

        Ok(())
    }

    #[trace_resources]
    fn kernel_remove_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<Option<IndexedScryptoValue>, RuntimeError> {
        let (substate, store_access) = self
            .current_frame
            .remove_substate(
                node_id,
                partition_num,
                &substate_key,
                &mut self.heap,
                self.store,
            )
            .map_err(CallFrameError::RemoveSubstatesError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)?;

        M::on_take_substates(&store_access, self)?;

        Ok(substate)
    }

    #[trace_resources]
    fn kernel_scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError> {
        let (substates, store_access) = self
            .current_frame
            .scan_sorted(node_id, partition_num, count, &mut self.heap, self.store)
            .map_err(CallFrameError::ScanSortedSubstatesError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)?;

        M::on_scan_substates(&store_access, self)?;

        Ok(substates)
    }

    #[trace_resources]
    fn kernel_scan_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError> {
        let (substeates, store_access) = self
            .current_frame
            .scan_substates(node_id, partition_num, count, &mut self.heap, self.store)
            .map_err(CallFrameError::ScanSubstatesError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)?;

        M::on_scan_substates(&store_access, self)?;

        Ok(substeates)
    }

    #[trace_resources]
    fn kernel_take_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        count: u32,
    ) -> Result<Vec<IndexedScryptoValue>, RuntimeError> {
        let (substeates, store_access) = self
            .current_frame
            .take_substates(node_id, partition_num, count, &mut self.heap, self.store)
            .map_err(CallFrameError::TakeSubstatesError)
            .map_err(KernelError::CallFrameError)
            .map_err(RuntimeError::KernelError)?;

        M::on_take_substates(&store_access, self)?;

        Ok(substeates)
    }
}

impl<'g, M, S> KernelInvokeApi for Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: SubstateStore,
{
    #[trace_resources]
    fn kernel_invoke(
        &mut self,
        invocation: Box<KernelInvocation>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        M::before_invoke(invocation.as_ref(), self)?;

        let rtn = self.invoke(invocation)?;

        M::after_invoke(rtn.len(), self)?;

        Ok(rtn)
    }
}

impl<'g, M, S> KernelApi<M> for Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: SubstateStore,
{
}
