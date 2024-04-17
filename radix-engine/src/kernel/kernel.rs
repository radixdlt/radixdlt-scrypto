use super::call_frame::{CallFrame, NodeVisibility, OpenSubstateError};
use super::heap::Heap;
use super::id_allocator::IdAllocator;
use crate::blueprints::resource::*;
use crate::errors::RuntimeError;
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::call_frame::{
    CallFrameIOAccessHandler, CallFrameMessage, CallFrameSubstateReadHandler, NonGlobalNodeRefs,
    TransientSubstates,
};
use crate::kernel::kernel_api::*;
use crate::kernel::kernel_callback_api::CallFrameReferences;
use crate::kernel::kernel_callback_api::{
    CloseSubstateEvent, CreateNodeEvent, DrainSubstatesEvent, DropNodeEvent, KernelCallbackObject,
    MoveModuleEvent, OpenSubstateEvent, ReadSubstateEvent, RemoveSubstateEvent, ScanKeysEvent,
    ScanSortedSubstatesEvent, SetSubstateEvent, WriteSubstateEvent,
};
use crate::kernel::substate_io::{SubstateDevice, SubstateIO};
use crate::kernel::substate_locks::SubstateLocks;
use crate::system::system_modules::execution_trace::{BucketSnapshot, ProofSnapshot};
use crate::system::type_info::TypeInfoSubstate;
use crate::track::interface::{CallbackError, CommitableSubstateStore, IOAccess, NodeSubstates};
use crate::track::{BootStore, Track};
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use radix_engine_profiling_derive::trace_resources;
use radix_substate_store_interface::db_key_mapper::{SpreadPrefixKeyMapper, SubstateKeyContent};
use radix_substate_store_interface::interface::SubstateDatabase;
use radix_transactions::prelude::{Executable, PreAllocatedAddress};
use sbor::rust::mem;
use crate::transaction::{CostingParameters, TransactionFeeDetails, TransactionFeeSummary, TransactionResult};

/// Organizes the radix engine stack to make a function entrypoint available for execution
pub struct BootLoader<'h, M: KernelCallbackObject, S: SubstateDatabase> {
    pub id_allocator: IdAllocator,
    pub callback: M,
    pub store: Track<'h, S, SpreadPrefixKeyMapper>,
}

impl<'h, M: KernelCallbackObject, S: SubstateDatabase> BootLoader<'h, M, S> {
    /// Creates a new kernel with data loaded from the substate store
    pub fn boot(&mut self) -> Result<Kernel<M, Track<'h, S, SpreadPrefixKeyMapper>>, BootloadingError> {
        let kernel = Kernel {
            substate_io: SubstateIO {
                heap: Heap::new(),
                store: &mut self.store,
                non_global_node_refs: NonGlobalNodeRefs::new(),
                substate_locks: SubstateLocks::new(),
                heap_transient_substates: TransientSubstates::new(),
                pinned_to_heap: BTreeSet::new(),
            },
            id_allocator: &mut self.id_allocator,
            current_frame: CallFrame::new_root(M::CallFrameData::root()),
            prev_frame_stack: vec![],
            callback: &mut self.callback,
        };

        Ok(kernel)
    }

    pub fn check_references(
        &mut self,
        references: &IndexSet<Reference>,
    ) -> Result<(IndexSet<GlobalAddress>, IndexSet<InternalAddress>), BootloadingError> {
        let mut global_addresses = indexset!();
        let mut direct_accesses = indexset!();
        for reference in references.iter() {
            let node_id = &reference.0;

            if ALWAYS_VISIBLE_GLOBAL_NODES.contains(node_id) {
                // Allow always visible node and do not add reference
                continue;
            }

            if node_id.is_global_virtual() {
                // Allow global virtual and add reference
                global_addresses.insert(GlobalAddress::new_or_panic(node_id.clone().into()));
                continue;
            }

            let substate_ref = self
                .store
                .read_substate(
                    node_id,
                    TYPE_INFO_FIELD_PARTITION,
                    &TypeInfoField::TypeInfo.into(),
                )
                .ok_or_else(|| BootloadingError::ReferencedNodeDoesNotExist(*node_id))?;
            let type_substate: TypeInfoSubstate = substate_ref.as_typed().unwrap();
            match &type_substate {
                TypeInfoSubstate::Object(
                    info @ ObjectInfo {
                        blueprint_info: BlueprintInfo { blueprint_id, .. },
                        ..
                    },
                ) => {
                    if info.is_global() {
                        global_addresses
                            .insert(GlobalAddress::new_or_panic(node_id.clone().into()));
                    } else if blueprint_id.package_address.eq(&RESOURCE_PACKAGE)
                        && (blueprint_id.blueprint_name.eq(FUNGIBLE_VAULT_BLUEPRINT)
                            || blueprint_id.blueprint_name.eq(NON_FUNGIBLE_VAULT_BLUEPRINT))
                    {
                        direct_accesses
                            .insert(InternalAddress::new_or_panic(node_id.clone().into()));
                    } else {
                        return Err(BootloadingError::ReferencedNodeDoesNotAllowDirectAccess(
                            node_id.clone(),
                        ));
                    }
                }
                _ => {
                    return Err(BootloadingError::ReferencedNodeIsNotAnObject(
                        node_id.clone(),
                    ));
                }
            }
        }

        Ok((global_addresses, direct_accesses))
    }

    /// Executes a transaction
    pub fn execute<'a>(
        mut self,
        executable: &Executable,
    ) -> (
        CostingParameters,
        TransactionFeeSummary,
        Option<TransactionFeeDetails>,
        TransactionResult,
    ) {
        #[cfg(feature = "resource_tracker")]
        radix_engine_profiling::QEMU_PLUGIN_CALIBRATOR.with(|v| {
            v.borrow_mut();
        });

        // Check reference
        let engine_references = match self.check_references(executable.references()) {
            Ok(engine_references) => engine_references,
            Err(e) => {
                return self.callback.on_teardown3(self.store, executable, Err(TransactionExecutionError::BootloadingError(e)));
            }
        };

        let mut kernel = Kernel {
            substate_io: SubstateIO {
                heap: Heap::new(),
                store: &mut self.store,
                non_global_node_refs: NonGlobalNodeRefs::new(),
                substate_locks: SubstateLocks::new(),
                heap_transient_substates: TransientSubstates::new(),
                pinned_to_heap: BTreeSet::new(),
            },
            id_allocator: &mut self.id_allocator,
            current_frame: CallFrame::new_root(M::CallFrameData::root()),
            prev_frame_stack: vec![],
            callback: &mut self.callback,
        };

        // Add visibility
        for global_ref in engine_references.0 {
            kernel.current_frame.add_global_reference(global_ref);
        }
        for direct_access in engine_references.1 {
            kernel
                .current_frame
                .add_direct_access_reference(direct_access);
        }

        let mut sys_exec = || -> Result<Vec<u8>, RuntimeError> {
            // Invoke transaction processor
            let rtn = M::start(
                &mut kernel,
                executable.encoded_instructions(),
                executable.pre_allocated_addresses(),
                executable.references(),
                executable.blobs(),
            )?;

            // Sanity check call frame
            assert!(kernel.prev_frame_stack.is_empty());

            // Sanity check heap
            assert!(kernel.substate_io.heap.is_empty());

            M::on_teardown(&mut kernel)?;

            let commit_info = kernel.substate_io.store.get_commit_info();

            kernel.callback.on_teardown2(commit_info)?;

            Ok(rtn)
        };

        let result = sys_exec();

        // Panic if an error is encountered in the system layer or below. The following code
        // is only enabled when compiling with the standard library since the panic catching
        // machinery and `SystemPanic` errors are only implemented in `std`.
        #[cfg(feature = "std")]
        if let Err(RuntimeError::SystemError(SystemError::SystemPanic(..))) = result
        {
            panic!("An error has occurred in the system layer or below and thus the transaction executor has panicked. Error: \"{result:?}\"")
        }

        let result = result
            .map(|rtn| {
                let output: Vec<InstructionOutput> = scrypto_decode(&rtn).unwrap();
                output
            })
            .map_err(|e| TransactionExecutionError::RuntimeError(e));

        self.callback.on_teardown3(self.store, executable, result)
    }
}

pub struct Kernel<
    'g, // Lifetime of values outliving all frames
    M,  // Upstream System layer
    S,  // Substate store
> where
    M: KernelCallbackObject,
    S: CommitableSubstateStore,
{
    /// Stack
    current_frame: CallFrame<M::CallFrameData, M::LockData>,
    // This stack could potentially be removed and just use the native stack
    // but keeping this call_frames stack may potentially prove useful if implementing
    // execution pause and/or for better debuggability
    prev_frame_stack: Vec<CallFrame<M::CallFrameData, M::LockData>>,

    substate_io: SubstateIO<'g, S>,

    /// ID allocator
    id_allocator: &'g mut IdAllocator,

    /// Upper system layer
    callback: &'g mut M,
}

struct KernelHandler<
    'a,
    M: KernelCallbackObject,
    F: FnMut(&mut KernelReadOnly<M>, IOAccess) -> Result<(), RuntimeError>,
> {
    callback: &'a mut M,
    prev_frame: Option<&'a CallFrame<M::CallFrameData, M::LockData>>,
    on_io_access: F,
}

impl<
        M: KernelCallbackObject,
        F: FnMut(&mut KernelReadOnly<M>, IOAccess) -> Result<(), RuntimeError>,
    > CallFrameIOAccessHandler<M::CallFrameData, M::LockData, RuntimeError>
    for KernelHandler<'_, M, F>
{
    fn on_io_access(
        &mut self,
        current_frame: &CallFrame<M::CallFrameData, M::LockData>,
        heap: &Heap,
        io_access: IOAccess,
    ) -> Result<(), RuntimeError> {
        let mut read_only = KernelReadOnly {
            current_frame,
            prev_frame: self.prev_frame,
            heap,
            callback: self.callback,
        };

        (self.on_io_access)(&mut read_only, io_access)
    }
}

impl<
        M: KernelCallbackObject,
        F: FnMut(&mut KernelReadOnly<M>, IOAccess) -> Result<(), RuntimeError>,
    > CallFrameSubstateReadHandler<M::CallFrameData, M::LockData> for KernelHandler<'_, M, F>
{
    type Error = RuntimeError;
    fn on_read_substate(
        &mut self,
        current_frame: &CallFrame<M::CallFrameData, M::LockData>,
        heap: &Heap,
        handle: SubstateHandle,
        value: &IndexedScryptoValue,
        device: SubstateDevice,
    ) -> Result<(), Self::Error> {
        let mut read_only = KernelReadOnly {
            current_frame,
            prev_frame: self.prev_frame,
            heap,
            callback: self.callback,
        };

        M::on_read_substate(
            &mut read_only,
            ReadSubstateEvent::OnRead {
                handle,
                value,
                device,
            },
        )
    }
}

macro_rules! as_read_only {
    ($kernel:expr) => {{
        KernelReadOnly {
            current_frame: &$kernel.current_frame,
            prev_frame: $kernel.prev_frame_stack.last(),
            heap: &$kernel.substate_io.heap,
            callback: $kernel.callback,
        }
    }};
}

impl<'g, M, S> KernelNodeApi for Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: CommitableSubstateStore,
{
    #[trace_resources]
    fn kernel_pin_node(&mut self, node_id: NodeId) -> Result<(), RuntimeError> {
        self.callback.on_pin_node(&node_id)?;

        self.current_frame
            .pin_node(&mut self.substate_io, node_id)
            .map_err(|e| {
                RuntimeError::KernelError(KernelError::CallFrameError(
                    CallFrameError::PinNodeError(e),
                ))
            })
    }

    #[trace_resources]
    fn kernel_allocate_node_id(&mut self, entity_type: EntityType) -> Result<NodeId, RuntimeError> {
        M::on_allocate_node_id(entity_type, self)?;

        self.id_allocator.allocate_node_id(entity_type)
    }

    #[trace_resources]
    fn kernel_create_node(
        &mut self,
        node_id: NodeId,
        node_substates: NodeSubstates,
    ) -> Result<(), RuntimeError> {
        let mut read_only = as_read_only!(self);
        M::on_create_node(
            &mut read_only,
            CreateNodeEvent::Start(&node_id, &node_substates),
        )?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                M::on_create_node(api, CreateNodeEvent::IOAccess(&io_access))
            },
        };

        self.current_frame
            .create_node(&mut self.substate_io, &mut handler, node_id, node_substates)
            .map_err(|e| match e {
                CallbackError::Error(e) => RuntimeError::KernelError(KernelError::CallFrameError(
                    CallFrameError::CreateNodeError(e),
                )),
                CallbackError::CallbackError(e) => e,
            })?;

        let mut read_only = as_read_only!(self);
        M::on_create_node(&mut read_only, CreateNodeEvent::End(&node_id))?;

        Ok(())
    }

    #[trace_resources]
    fn kernel_create_node_from(
        &mut self,
        node_id: NodeId,
        partitions: BTreeMap<PartitionNumber, (NodeId, PartitionNumber)>,
    ) -> Result<(), RuntimeError> {
        {
            let node_substates = NodeSubstates::new();
            let mut read_only = as_read_only!(self);
            M::on_create_node(
                &mut read_only,
                CreateNodeEvent::Start(&node_id, &node_substates),
            )?;

            let mut handler = KernelHandler {
                callback: self.callback,
                prev_frame: self.prev_frame_stack.last(),
                on_io_access: |api, io_access| {
                    M::on_create_node(api, CreateNodeEvent::IOAccess(&io_access))
                },
            };

            self.current_frame
                .create_node(
                    &mut self.substate_io,
                    &mut handler,
                    node_id,
                    NodeSubstates::new(),
                )
                .map_err(|e| match e {
                    CallbackError::Error(e) => RuntimeError::KernelError(
                        KernelError::CallFrameError(CallFrameError::CreateNodeError(e)),
                    ),
                    CallbackError::CallbackError(e) => e,
                })?;

            let mut read_only = as_read_only!(self);
            M::on_create_node(&mut read_only, CreateNodeEvent::End(&node_id))?;
        }

        {
            let mut handler = KernelHandler {
                callback: self.callback,
                prev_frame: self.prev_frame_stack.last(),
                on_io_access: |api, io_access| {
                    M::on_move_module(api, MoveModuleEvent::IOAccess(&io_access))
                },
            };

            for (dest_partition_number, (src_node_id, src_partition_number)) in partitions {
                self.current_frame
                    .move_partition(
                        &mut self.substate_io,
                        &mut handler,
                        &src_node_id,
                        src_partition_number,
                        &node_id,
                        dest_partition_number,
                    )
                    .map_err(|e| match e {
                        CallbackError::Error(e) => RuntimeError::KernelError(
                            KernelError::CallFrameError(CallFrameError::MovePartitionError(e)),
                        ),
                        CallbackError::CallbackError(e) => e,
                    })?;
            }
        }

        Ok(())
    }

    #[trace_resources]
    fn kernel_drop_node(&mut self, node_id: &NodeId) -> Result<DroppedNode, RuntimeError> {
        let mut read_only = as_read_only!(self);
        M::on_drop_node(&mut read_only, DropNodeEvent::Start(node_id))?;

        M::on_drop_node_mut(node_id, self)?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                M::on_drop_node(api, DropNodeEvent::IOAccess(&io_access))
            },
        };
        let dropped_node = self
            .current_frame
            .drop_node(&mut self.substate_io, node_id, &mut handler)
            .map_err(|e| match e {
                CallbackError::Error(e) => RuntimeError::KernelError(KernelError::CallFrameError(
                    CallFrameError::DropNodeError(e),
                )),
                CallbackError::CallbackError(e) => e,
            })?;

        let mut read_only = as_read_only!(self);
        M::on_drop_node(
            &mut read_only,
            DropNodeEvent::End(node_id, &dropped_node.substates),
        )?;

        Ok(dropped_node)
    }
}

// TODO: Remove
impl<'g, M, S> KernelInternalApi<M> for Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: CommitableSubstateStore,
{
    fn kernel_get_node_visibility(&self, node_id: &NodeId) -> NodeVisibility {
        self.current_frame.get_node_visibility(node_id)
    }

    fn kernel_get_current_depth(&self) -> usize {
        self.current_frame.depth()
    }

    fn kernel_get_system_state(&mut self) -> SystemState<'_, M> {
        let caller_actor = match self.prev_frame_stack.last() {
            Some(call_frame) => call_frame.data(),
            None => {
                // This will only occur on initialization
                self.current_frame.data()
            }
        };
        SystemState {
            system: &mut self.callback,
            current_call_frame: self.current_frame.data(),
            caller_call_frame: caller_actor,
        }
    }

    fn kernel_read_bucket(&mut self, bucket_id: &NodeId) -> Option<BucketSnapshot> {
        let mut read_only = as_read_only!(self);
        read_only.kernel_read_bucket(bucket_id)
    }

    fn kernel_read_proof(&mut self, proof_id: &NodeId) -> Option<ProofSnapshot> {
        let mut read_only = as_read_only!(self);
        read_only.kernel_read_proof(proof_id)
    }
}

struct KernelReadOnly<'g, M>
where
    M: KernelCallbackObject,
{
    current_frame: &'g CallFrame<M::CallFrameData, M::LockData>,
    prev_frame: Option<&'g CallFrame<M::CallFrameData, M::LockData>>,
    heap: &'g Heap,
    callback: &'g mut M,
}

impl<'g, M> KernelInternalApi<M> for KernelReadOnly<'g, M>
where
    M: KernelCallbackObject,
{
    fn kernel_get_node_visibility(&self, node_id: &NodeId) -> NodeVisibility {
        self.current_frame.get_node_visibility(node_id)
    }

    fn kernel_get_current_depth(&self) -> usize {
        self.current_frame.depth()
    }

    fn kernel_get_system_state(&mut self) -> SystemState<'_, M> {
        let caller_call_frame = match self.prev_frame {
            Some(call_frame) => call_frame.data(),
            None => {
                // This will only occur on initialization
                self.current_frame.data()
            }
        };
        SystemState {
            system: self.callback,
            current_call_frame: self.current_frame.data(),
            caller_call_frame,
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
                    if info.blueprint_info.blueprint_id.package_address == RESOURCE_PACKAGE
                        && (info.blueprint_info.blueprint_id.blueprint_name
                            == FUNGIBLE_BUCKET_BLUEPRINT
                            || info.blueprint_info.blueprint_id.blueprint_name
                                == NON_FUNGIBLE_BUCKET_BLUEPRINT) =>
                {
                    let is_fungible = info
                        .blueprint_info
                        .blueprint_id
                        .blueprint_name
                        .eq(FUNGIBLE_BUCKET_BLUEPRINT);
                    let parent = info.get_outer_object();
                    let resource_address: ResourceAddress =
                        ResourceAddress::new_or_panic(parent.as_ref().try_into().unwrap());
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
            let liquid: FieldSubstate<LiquidFungibleResource> = substate.as_typed().unwrap();

            Some(BucketSnapshot::Fungible {
                resource_address,
                liquid: liquid.into_payload().amount(),
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
            let liquid: FieldSubstate<LiquidNonFungibleResource> = substate.as_typed().unwrap();

            Some(BucketSnapshot::NonFungible {
                resource_address,
                liquid: liquid.into_payload().ids().clone(),
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
                    blueprint_info: BlueprintInfo { blueprint_id, .. },
                    ..
                }) if blueprint_id.package_address == RESOURCE_PACKAGE
                    && (blueprint_id.blueprint_name == NON_FUNGIBLE_PROOF_BLUEPRINT
                        || blueprint_id.blueprint_name == FUNGIBLE_PROOF_BLUEPRINT) =>
                {
                    blueprint_id.blueprint_name.eq(FUNGIBLE_PROOF_BLUEPRINT)
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
            let proof: FieldSubstate<FungibleProofSubstate> = substate.as_typed().unwrap();

            Some(ProofSnapshot::Fungible {
                resource_address,
                total_locked: proof.into_payload().amount(),
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
            let proof: FieldSubstate<NonFungibleProofSubstate> = substate.as_typed().unwrap();

            Some(ProofSnapshot::NonFungible {
                resource_address,
                total_locked: proof.into_payload().non_fungible_local_ids().clone(),
            })
        }
    }
}

impl<'g, M, S> KernelSubstateApi<M::LockData> for Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: CommitableSubstateStore,
{
    #[trace_resources]
    fn kernel_mark_substate_as_transient(
        &mut self,
        node_id: NodeId,
        partition_num: PartitionNumber,
        key: SubstateKey,
    ) -> Result<(), RuntimeError> {
        self.callback
            .on_mark_substate_as_transient(&node_id, &partition_num, &key)?;

        self.current_frame
            .mark_substate_as_transient(&mut self.substate_io, node_id, partition_num, key)
            .map_err(|e| {
                RuntimeError::KernelError(KernelError::CallFrameError(
                    CallFrameError::MarkTransientSubstateError(e),
                ))
            })
    }

    #[trace_resources]
    fn kernel_open_substate_with_default<F: FnOnce() -> IndexedScryptoValue>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
        flags: LockFlags,
        default: Option<F>,
        data: M::LockData,
    ) -> Result<SubstateHandle, RuntimeError> {
        let mut read_only = as_read_only!(self);
        M::on_open_substate(
            &mut read_only,
            OpenSubstateEvent::Start {
                node_id: &node_id,
                partition_num: &partition_num,
                substate_key,
                flags: &flags,
            },
        )?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                M::on_open_substate(api, OpenSubstateEvent::IOAccess(&io_access))
            },
        };

        let maybe_lock_handle = self.current_frame.open_substate(
            &mut self.substate_io,
            node_id,
            partition_num,
            substate_key,
            flags,
            default,
            data,
            &mut handler,
        );

        let (lock_handle, value_size): (u32, usize) = match &maybe_lock_handle {
            Ok((lock_handle, value_size)) => (*lock_handle, *value_size),
            Err(CallbackError::CallbackError(e)) => return Err(e.clone()),
            Err(CallbackError::Error(OpenSubstateError::SubstateFault)) => {
                let retry =
                    M::on_substate_lock_fault(*node_id, partition_num, &substate_key, self)?;

                if retry {
                    let mut handler = KernelHandler {
                        callback: self.callback,
                        prev_frame: self.prev_frame_stack.last(),
                        on_io_access: |api, io_access| {
                            M::on_open_substate(api, OpenSubstateEvent::IOAccess(&io_access))
                        },
                    };

                    self.current_frame
                        .open_substate(
                            &mut self.substate_io,
                            &node_id,
                            partition_num,
                            &substate_key,
                            flags,
                            None::<fn() -> IndexedScryptoValue>,
                            M::LockData::default(),
                            &mut handler,
                        )
                        .map_err(|e| match e {
                            CallbackError::Error(e) => RuntimeError::KernelError(
                                KernelError::CallFrameError(CallFrameError::OpenSubstateError(e)),
                            ),
                            CallbackError::CallbackError(e) => e,
                        })?
                } else {
                    return maybe_lock_handle
                        .map(|(lock_handle, _)| lock_handle)
                        .map_err(|e| match e {
                            CallbackError::Error(e) => RuntimeError::KernelError(
                                KernelError::CallFrameError(CallFrameError::OpenSubstateError(e)),
                            ),
                            CallbackError::CallbackError(e) => e,
                        });
                }
            }
            Err(err) => {
                let runtime_error = match err {
                    CallbackError::Error(e) => RuntimeError::KernelError(
                        KernelError::CallFrameError(CallFrameError::OpenSubstateError(e.clone())),
                    ),
                    CallbackError::CallbackError(e) => e.clone(),
                };
                return Err(runtime_error);
            }
        };

        let mut read_only = as_read_only!(self);
        M::on_open_substate(
            &mut read_only,
            OpenSubstateEvent::End {
                handle: lock_handle,
                node_id: &node_id,
                size: value_size,
            },
        )?;

        Ok(lock_handle)
    }

    #[trace_resources]
    fn kernel_get_lock_data(
        &mut self,
        lock_handle: SubstateHandle,
    ) -> Result<M::LockData, RuntimeError> {
        self.current_frame
            .get_handle_info(lock_handle)
            .ok_or(RuntimeError::KernelError(
                KernelError::SubstateHandleDoesNotExist(lock_handle),
            ))
    }

    #[trace_resources]
    fn kernel_read_substate(
        &mut self,
        lock_handle: SubstateHandle,
    ) -> Result<&IndexedScryptoValue, RuntimeError> {
        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                M::on_read_substate(api, ReadSubstateEvent::IOAccess(&io_access))
            },
        };

        let value = self
            .current_frame
            .read_substate(&mut self.substate_io, lock_handle, &mut handler)
            .map_err(|e| match e {
                CallbackError::Error(e) => RuntimeError::KernelError(KernelError::CallFrameError(
                    CallFrameError::ReadSubstateError(e),
                )),
                CallbackError::CallbackError(e) => e,
            })?;

        Ok(value)
    }

    #[trace_resources]
    fn kernel_write_substate(
        &mut self,
        lock_handle: SubstateHandle,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        let mut read_only = as_read_only!(self);
        M::on_write_substate(
            &mut read_only,
            WriteSubstateEvent::Start {
                handle: lock_handle,
                value: &value,
            },
        )?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                M::on_write_substate(api, WriteSubstateEvent::IOAccess(&io_access))
            },
        };

        self.current_frame
            .write_substate(&mut self.substate_io, lock_handle, value, &mut handler)
            .map_err(|e| match e {
                CallbackError::Error(e) => RuntimeError::KernelError(KernelError::CallFrameError(
                    CallFrameError::WriteSubstateError(e),
                )),
                CallbackError::CallbackError(e) => e,
            })?;

        Ok(())
    }

    #[trace_resources]
    fn kernel_close_substate(&mut self, lock_handle: SubstateHandle) -> Result<(), RuntimeError> {
        // Note: It is very important that this occurs before the actual call to close_substate
        // as we want to check limits/costing before doing the actual action. Otherwise,
        // certain invariants might break such as a costing error occurring after a vault
        // lock_fee has been force committed.
        let mut read_only = as_read_only!(self);
        M::on_close_substate(&mut read_only, CloseSubstateEvent::Start(lock_handle))?;

        self.current_frame
            .close_substate(&mut self.substate_io, lock_handle)
            .map_err(|e| {
                RuntimeError::KernelError(KernelError::CallFrameError(
                    CallFrameError::CloseSubstateError(e),
                ))
            })?;

        Ok(())
    }

    #[trace_resources]
    fn kernel_set_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: SubstateKey,
        value: IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        self.callback.on_set_substate(SetSubstateEvent::Start(
            node_id,
            &partition_num,
            &substate_key,
            &value,
        ))?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                api.callback
                    .on_set_substate(SetSubstateEvent::IOAccess(&io_access))
            },
        };

        self.current_frame
            .set_substate(
                &mut self.substate_io,
                node_id,
                partition_num,
                substate_key,
                value,
                &mut handler,
            )
            .map_err(|e| match e {
                CallbackError::Error(e) => RuntimeError::KernelError(KernelError::CallFrameError(
                    CallFrameError::SetSubstatesError(e),
                )),
                CallbackError::CallbackError(e) => e,
            })?;

        Ok(())
    }

    #[trace_resources]
    fn kernel_remove_substate(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Result<Option<IndexedScryptoValue>, RuntimeError> {
        self.callback
            .on_remove_substate(RemoveSubstateEvent::Start(
                node_id,
                &partition_num,
                substate_key,
            ))?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                api.callback
                    .on_remove_substate(RemoveSubstateEvent::IOAccess(&io_access))
            },
        };

        let substate = self
            .current_frame
            .remove_substate(
                &mut self.substate_io,
                node_id,
                partition_num,
                &substate_key,
                &mut handler,
            )
            .map_err(|e| match e {
                CallbackError::Error(e) => RuntimeError::KernelError(KernelError::CallFrameError(
                    CallFrameError::RemoveSubstatesError(e),
                )),
                CallbackError::CallbackError(e) => e,
            })?;

        Ok(substate)
    }

    #[trace_resources]
    fn kernel_scan_keys<K: SubstateKeyContent + 'static>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        limit: u32,
    ) -> Result<Vec<SubstateKey>, RuntimeError> {
        self.callback.on_scan_keys(ScanKeysEvent::Start)?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                api.callback
                    .on_scan_keys(ScanKeysEvent::IOAccess(&io_access))
            },
        };

        let keys = self
            .current_frame
            .scan_keys::<K, _, _>(
                &mut self.substate_io,
                node_id,
                partition_num,
                limit,
                &mut handler,
            )
            .map_err(|e| match e {
                CallbackError::Error(e) => RuntimeError::KernelError(KernelError::CallFrameError(
                    CallFrameError::ScanSubstatesError(e),
                )),
                CallbackError::CallbackError(e) => e,
            })?;

        Ok(keys)
    }

    #[trace_resources(log=limit)]
    fn kernel_drain_substates<K: SubstateKeyContent + 'static>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        limit: u32,
    ) -> Result<Vec<(SubstateKey, IndexedScryptoValue)>, RuntimeError> {
        self.callback
            .on_drain_substates(DrainSubstatesEvent::Start(limit))?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                api.callback
                    .on_drain_substates(DrainSubstatesEvent::IOAccess(&io_access))
            },
        };

        let substates = self
            .current_frame
            .drain_substates::<K, _, _>(
                &mut self.substate_io,
                node_id,
                partition_num,
                limit,
                &mut handler,
            )
            .map_err(|e| match e {
                CallbackError::CallbackError(e) => e,
                CallbackError::Error(e) => RuntimeError::KernelError(KernelError::CallFrameError(
                    CallFrameError::DrainSubstatesError(e),
                )),
            })?;

        Ok(substates)
    }

    #[trace_resources]
    fn kernel_scan_sorted_substates(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        limit: u32,
    ) -> Result<Vec<(SortedKey, IndexedScryptoValue)>, RuntimeError> {
        self.callback
            .on_scan_sorted_substates(ScanSortedSubstatesEvent::Start)?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                api.callback
                    .on_scan_sorted_substates(ScanSortedSubstatesEvent::IOAccess(&io_access))
            },
        };

        let substates = self
            .current_frame
            .scan_sorted(
                &mut self.substate_io,
                node_id,
                partition_num,
                limit,
                &mut handler,
            )
            .map_err(|e| match e {
                CallbackError::Error(e) => RuntimeError::KernelError(KernelError::CallFrameError(
                    CallFrameError::ScanSortedSubstatesError(e),
                )),
                CallbackError::CallbackError(e) => e,
            })?;

        Ok(substates)
    }
}

impl<'g, M, S> KernelInvokeApi<M::CallFrameData> for Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: CommitableSubstateStore,
{
    #[trace_resources]
    fn kernel_invoke(
        &mut self,
        invocation: Box<KernelInvocation<M::CallFrameData>>,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        M::before_invoke(invocation.as_ref(), self)?;

        // Before push call frame
        let callee = invocation.call_frame_data;
        let args = &invocation.args;
        let message = CallFrameMessage::from_input(&args, &callee);

        // Push call frame
        {
            let frame = CallFrame::new_child_from_parent(
                &self.substate_io,
                &mut self.current_frame,
                callee,
                message,
            )
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
            for handle in self.current_frame.open_substates() {
                M::on_close_substate(self, CloseSubstateEvent::Start(handle))?;
            }
            self.current_frame
                .close_all_substates(&mut self.substate_io);

            // Run
            let output = M::invoke_upstream(args, self)?;
            let message = CallFrameMessage::from_output(&output);

            // Auto-drop locks again in case module forgot to drop
            for handle in self.current_frame.open_substates() {
                M::on_close_substate(self, CloseSubstateEvent::Start(handle))?;
            }
            self.current_frame
                .close_all_substates(&mut self.substate_io);

            // Handle execution finish
            M::on_execution_finish(&message, self)?;

            (output, message)
        };

        // Move
        {
            let parent = self.prev_frame_stack.last_mut().unwrap();

            // Move resource
            CallFrame::pass_message(
                &self.substate_io,
                &mut self.current_frame,
                parent,
                message.clone(),
            )
            .map_err(CallFrameError::PassMessageError)
            .map_err(KernelError::CallFrameError)?;

            // Auto-drop
            let owned_nodes = self.current_frame.owned_nodes();
            M::auto_drop(owned_nodes, self)?;

            // Now, check if any own has been left!
            let owned_nodes = self.current_frame.owned_nodes();
            if !owned_nodes.is_empty() {
                return Err(RuntimeError::KernelError(KernelError::OrphanedNodes(
                    owned_nodes,
                )));
            }
        }

        // Pop call frame
        {
            let parent = self.prev_frame_stack.pop().unwrap();
            let _ = core::mem::replace(&mut self.current_frame, parent);
        }

        M::after_invoke(&output, self)?;

        Ok(output)
    }
}

impl<'g, M, S> KernelApi<M> for Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: CommitableSubstateStore,
{
}

#[cfg(feature = "radix_engine_tests")]
impl<'g, M, S> Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: CommitableSubstateStore,
{
    pub fn kernel_create_kernel_for_testing(
        substate_io: SubstateIO<'g, S>,
        id_allocator: &'g mut IdAllocator,
        current_frame: CallFrame<M::CallFrameData, M::LockData>,
        prev_frame_stack: Vec<CallFrame<M::CallFrameData, M::LockData>>,
        callback: &'g mut M,
    ) -> Kernel<'g, M, S> {
        Self {
            current_frame,
            prev_frame_stack,
            substate_io,
            id_allocator,
            callback,
        }
    }

    pub fn kernel_current_frame(
        &self,
    ) -> &CallFrame<<M as KernelCallbackObject>::CallFrameData, <M as KernelCallbackObject>::LockData>
    {
        &self.current_frame
    }

    pub fn kernel_current_frame_mut(
        &mut self,
    ) -> (
        &SubstateIO<S>,
        &mut CallFrame<
            <M as KernelCallbackObject>::CallFrameData,
            <M as KernelCallbackObject>::LockData,
        >,
    ) {
        (&self.substate_io, &mut self.current_frame)
    }

    pub fn kernel_prev_frame_stack(
        &self,
    ) -> &Vec<
        CallFrame<
            <M as KernelCallbackObject>::CallFrameData,
            <M as KernelCallbackObject>::LockData,
        >,
    > {
        &self.prev_frame_stack
    }

    pub fn kernel_prev_frame_stack_mut(
        &mut self,
    ) -> &mut Vec<
        CallFrame<
            <M as KernelCallbackObject>::CallFrameData,
            <M as KernelCallbackObject>::LockData,
        >,
    > {
        &mut self.prev_frame_stack
    }

    pub fn kernel_substate_io(&self) -> &SubstateIO<'g, S> {
        &self.substate_io
    }

    pub fn kernel_substate_io_mut(&mut self) -> &mut SubstateIO<'g, S> {
        &mut self.substate_io
    }

    pub fn kernel_id_allocator(&self) -> &IdAllocator {
        &self.id_allocator
    }

    pub fn kernel_id_allocator_mut(&mut self) -> &mut &'g mut IdAllocator {
        &mut self.id_allocator
    }

    pub fn kernel_callback(&self) -> &M {
        &self.callback
    }

    pub fn kernel_callback_mut(&mut self) -> &mut M {
        &mut self.callback
    }
}
