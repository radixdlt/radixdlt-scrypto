use super::heap::Heap;
use super::id_allocator::IdAllocator;
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::call_frame::*;
use crate::kernel::kernel_api::*;
use crate::kernel::kernel_callback_api::*;
use crate::kernel::substate_io::{SubstateDevice, SubstateIO};
use crate::kernel::substate_locks::SubstateLocks;
use crate::track::interface::*;
use crate::track::Track;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_profiling_derive::trace_resources;
use radix_substate_store_interface::db_key_mapper::{SpreadPrefixKeyMapper, SubstateKeyContent};
use radix_substate_store_interface::interface::SubstateDatabase;
use sbor::rust::mem;

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

pub const BOOT_LOADER_KERNEL_BOOT_FIELD_KEY: FieldKey = 0u8;

pub type KernelBootSubstate = KernelBoot;

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum KernelBoot {
    V1,
}

impl KernelBoot {
    pub fn babylon() -> Self {
        Self::V1
    }
}

/// Organizes the radix engine stack to make a function entrypoint available for execution
pub struct BootLoader<'h, M: KernelTransactionCallbackObject, S: SubstateDatabase> {
    pub id_allocator: IdAllocator,
    pub track: Track<'h, S, SpreadPrefixKeyMapper>,
    pub init: M::Init,
    pub phantom: PhantomData<M>,
}

impl<'h, M: KernelTransactionCallbackObject, S: SubstateDatabase> BootLoader<'h, M, S> {
    /// Executes a transaction
    pub fn execute(self, executable: M::Executable) -> M::Receipt {
        // Start hardware resource usage tracker
        #[cfg(all(target_os = "linux", feature = "std", feature = "cpu_ram_metrics"))]
        let mut resources_tracker =
            crate::kernel::resources_tracker::ResourcesTracker::start_measurement();

        #[cfg(not(all(target_os = "linux", feature = "std", feature = "cpu_ram_metrics")))]
        {
            self.execute_internal(executable)
        }

        #[cfg(all(target_os = "linux", feature = "std", feature = "cpu_ram_metrics"))]
        {
            let mut receipt = self.execute_internal(executable);

            // Stop hardware resource usage tracker
            receipt.set_resource_usage(resources_tracker.end_measurement());

            receipt
        }
    }

    fn execute_internal(mut self, executable: M::Executable) -> M::Receipt {
        #[cfg(feature = "resource_tracker")]
        radix_engine_profiling::QEMU_PLUGIN_CALIBRATOR.with(|v| {
            v.borrow_mut();
        });

        // Read kernel boot configuration
        // Unused for now
        let _kernel_boot: KernelBoot = self
            .track
            .read_boot_substate(
                TRANSACTION_TRACKER.as_node_id(),
                BOOT_LOADER_PARTITION,
                &SubstateKey::Field(BOOT_LOADER_KERNEL_BOOT_FIELD_KEY),
            )
            .map(|v| scrypto_decode(v.as_slice()).unwrap())
            .unwrap_or(KernelBoot::babylon());

        // Upper Layer Initialization
        let system_init_result = M::init(&mut self.track, &executable, self.init.clone());

        let (mut system, call_frame_inits) = match system_init_result {
            Ok(success) => success,
            Err(receipt) => return receipt,
        };

        // Kernel Initialization
        let mut kernel = Kernel::new(
            &mut self.track,
            &mut self.id_allocator,
            &mut system,
            // TODO: Fix to take call frame inits for each intent
            {
                let mut call_frame_inits = call_frame_inits;
                let first_call_frame_init = call_frame_inits.drain(..).next().unwrap();
                first_call_frame_init
            },
        );

        // Execution
        let result = || -> Result<M::ExecutionOutput, RuntimeError> {
            // Invoke transaction processor
            let output = M::start(&mut kernel, executable)?;

            // Sanity check call frame
            assert!(kernel.prev_frame_stack.is_empty());

            // Sanity check heap
            assert!(kernel.substate_io.heap.is_empty());

            // Finalize state updates based on what has occurred
            let commit_info = kernel.substate_io.store.get_commit_info();
            kernel.callback.finish(commit_info)?;

            Ok(output)
        }()
        .map_err(|e| TransactionExecutionError::RuntimeError(e));

        // Create receipt representing the result of a transaction
        system.create_receipt(self.track, result)
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

impl<
        'g,
        M: KernelCallbackObject<CallFrameData: Default>,
        S: CommitableSubstateStore + BootStore,
    > Kernel<'g, M, S>
{
    pub fn new_no_refs(
        store: &'g mut S,
        id_allocator: &'g mut IdAllocator,
        callback: &'g mut M,
    ) -> Self {
        Self::new(store, id_allocator, callback, Default::default())
    }
}

impl<'g, M: KernelCallbackObject, S: CommitableSubstateStore + BootStore> Kernel<'g, M, S> {
    pub fn new(
        store: &'g mut S,
        id_allocator: &'g mut IdAllocator,
        callback: &'g mut M,
        call_frame_init: CallFrameInit<M::CallFrameData>,
    ) -> Self {
        Kernel {
            substate_io: SubstateIO {
                heap: Heap::new(),
                store,
                non_global_node_refs: NonGlobalNodeRefs::new(),
                substate_locks: SubstateLocks::new(),
                heap_transient_substates: TransientSubstates::new(),
                pinned_to_heap: BTreeSet::new(),
            },
            id_allocator,
            current_frame: CallFrame::new_root(call_frame_init),
            prev_frame_stack: vec![],
            callback,
        }
    }
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
            ReadSubstateEvent::OnRead {
                handle,
                value,
                device,
            },
            &mut read_only,
        )
    }
}

impl<'g, M, S> KernelNodeApi for Kernel<'g, M, S>
where
    M: KernelCallbackObject,
    S: CommitableSubstateStore,
{
    #[trace_resources]
    fn kernel_pin_node(&mut self, node_id: NodeId) -> Result<(), RuntimeError> {
        M::on_pin_node(&node_id, &mut as_read_only!(self))?;

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
            CreateNodeEvent::Start(&node_id, &node_substates),
            &mut read_only,
        )?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                M::on_create_node(CreateNodeEvent::IOAccess(&io_access), api)
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
        M::on_create_node(CreateNodeEvent::End(&node_id), &mut read_only)?;

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
                CreateNodeEvent::Start(&node_id, &node_substates),
                &mut read_only,
            )?;

            let mut handler = KernelHandler {
                callback: self.callback,
                prev_frame: self.prev_frame_stack.last(),
                on_io_access: |api, io_access| {
                    M::on_create_node(CreateNodeEvent::IOAccess(&io_access), api)
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
            M::on_create_node(CreateNodeEvent::End(&node_id), &mut read_only)?;
        }

        {
            let mut handler = KernelHandler {
                callback: self.callback,
                prev_frame: self.prev_frame_stack.last(),
                on_io_access: |api, io_access| {
                    M::on_move_module(MoveModuleEvent::IOAccess(&io_access), api)
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
        M::on_drop_node(DropNodeEvent::Start(node_id), &mut read_only)?;

        M::on_drop_node_mut(node_id, self)?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                M::on_drop_node(DropNodeEvent::IOAccess(&io_access), api)
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
            DropNodeEvent::End(node_id, &dropped_node.substates),
            &mut read_only,
        )?;

        Ok(dropped_node)
    }
}

#[deprecated = "Deprecated as a reminder to remove this when threads are implemented"]
fn single_intent_index() -> usize {
    0
}

// TODO: Remove
impl<'g, M: KernelCallbackObject, S: CommitableSubstateStore> KernelInternalApi
    for Kernel<'g, M, S>
{
    type System = M;

    fn kernel_get_node_visibility(&self, node_id: &NodeId) -> NodeVisibility {
        self.current_frame.get_node_visibility(node_id)
    }

    fn kernel_get_thread_id(&self) -> usize {
        // TODO - fix when threading is implemented!
        single_intent_index()
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

    fn kernel_read_substate_uncosted(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Option<&IndexedScryptoValue> {
        self.substate_io
            .heap
            .get_substate(node_id, partition_num, substate_key)
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

impl<'g, M: KernelCallbackObject> KernelInternalApi for KernelReadOnly<'g, M> {
    type System = M;

    fn kernel_get_node_visibility(&self, node_id: &NodeId) -> NodeVisibility {
        self.current_frame.get_node_visibility(node_id)
    }

    fn kernel_get_thread_id(&self) -> usize {
        // TODO - fix when threading is implemented!
        single_intent_index()
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

    fn kernel_read_substate_uncosted(
        &self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        substate_key: &SubstateKey,
    ) -> Option<&IndexedScryptoValue> {
        self.heap.get_substate(node_id, partition_num, substate_key)
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
        M::on_mark_substate_as_transient(&node_id, &partition_num, &key, &mut as_read_only!(self))?;

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
        M::on_open_substate(
            OpenSubstateEvent::Start {
                node_id: &node_id,
                partition_num: &partition_num,
                substate_key,
                flags: &flags,
            },
            &mut as_read_only!(self),
        )?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                M::on_open_substate(OpenSubstateEvent::IOAccess(&io_access), api)
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
                            M::on_open_substate(OpenSubstateEvent::IOAccess(&io_access), api)
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
            OpenSubstateEvent::End {
                handle: lock_handle,
                node_id: &node_id,
                size: value_size,
            },
            &mut read_only,
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
                M::on_read_substate(ReadSubstateEvent::IOAccess(&io_access), api)
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
            WriteSubstateEvent::Start {
                handle: lock_handle,
                value: &value,
            },
            &mut read_only,
        )?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                M::on_write_substate(WriteSubstateEvent::IOAccess(&io_access), api)
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
        M::on_close_substate(CloseSubstateEvent::Start(lock_handle), &mut read_only)?;

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
        M::on_set_substate(
            SetSubstateEvent::Start(node_id, &partition_num, &substate_key, &value),
            &mut as_read_only!(self),
        )?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                M::on_set_substate(SetSubstateEvent::IOAccess(&io_access), api)
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
        M::on_remove_substate(
            RemoveSubstateEvent::Start(node_id, &partition_num, substate_key),
            &mut as_read_only!(self),
        )?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                M::on_remove_substate(RemoveSubstateEvent::IOAccess(&io_access), api)
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
        M::on_scan_keys(ScanKeysEvent::Start, &mut as_read_only!(self))?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                M::on_scan_keys(ScanKeysEvent::IOAccess(&io_access), api)
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
        M::on_drain_substates(DrainSubstatesEvent::Start(limit), &mut as_read_only!(self))?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                M::on_drain_substates(DrainSubstatesEvent::IOAccess(&io_access), api)
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
        M::on_scan_sorted_substates(ScanSortedSubstatesEvent::Start, &mut as_read_only!(self))?;

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame: self.prev_frame_stack.last(),
            on_io_access: |api, io_access| {
                M::on_scan_sorted_substates(ScanSortedSubstatesEvent::IOAccess(&io_access), api)
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
                M::on_close_substate(CloseSubstateEvent::Start(handle), self)?;
            }
            self.current_frame
                .close_all_substates(&mut self.substate_io);

            // Run
            let output = M::invoke_upstream(args, self)?;
            let message = CallFrameMessage::from_output(&output);

            // Auto-drop locks again in case module forgot to drop
            for handle in self.current_frame.open_substates() {
                M::on_close_substate(CloseSubstateEvent::Start(handle), self)?;
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

impl<'g, M: KernelCallbackObject, S: CommitableSubstateStore> KernelThreadApi for Kernel<'g, M, S> {
    type CallFrameData = M::CallFrameData;
    fn kernel_set_call_frame_data(&mut self, data: M::CallFrameData) -> Result<(), RuntimeError> {
        *self.current_frame.data_mut() = data;
        Ok(())
    }

    fn kernel_get_owned_nodes(&mut self) -> Result<Vec<NodeId>, RuntimeError> {
        Ok(self.current_frame.owned_nodes())
    }
}

impl<'g, M: KernelCallbackObject, S: CommitableSubstateStore> KernelApi for Kernel<'g, M, S> {
    type CallbackObject = M;
}

#[cfg(feature = "radix_engine_tests")]
impl<'g, M: KernelCallbackObject, S: CommitableSubstateStore> Kernel<'g, M, S> {
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
