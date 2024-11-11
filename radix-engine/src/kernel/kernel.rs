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
use radix_substate_store_interface::db_key_mapper::SubstateKeyContent;
use radix_substate_store_interface::interface::SubstateDatabase;
use sbor::rust::mem;

macro_rules! as_read_only {
    ($kernel:expr) => {{
        let (current_frame, previous_frame) = $kernel.stacks.current_frame_and_previous_frame();
        KernelReadOnly {
            current_frame,
            previous_frame,
            heap: &$kernel.substate_io.heap,
            callback: $kernel.callback,
        }
    }};
}

pub type KernelBootSubstate = KernelBoot;

#[derive(Debug, Clone, PartialEq, Eq, Sbor, ScryptoSborAssertion)]
#[sbor_assert(backwards_compatible(
    cuttlefish = "FILE:kernel_boot_substate_cuttlefish_schema.bin",
))]
pub enum KernelBoot {
    V1,
    V2 {
        global_nodes_version: AlwaysVisibleGlobalNodesVersion,
    },
}

impl KernelBoot {
    /// Loads kernel boot from the database, or resolves a fallback.
    pub fn load(substate_db: &impl SubstateDatabase) -> Self {
        substate_db
            .get_substate(
                TRANSACTION_TRACKER,
                BOOT_LOADER_PARTITION,
                BootLoaderField::KernelBoot,
            )
            .unwrap_or_else(|| KernelBoot::babylon())
    }

    pub fn babylon() -> Self {
        Self::V1
    }

    pub fn cuttlefish() -> Self {
        Self::V2 {
            global_nodes_version: AlwaysVisibleGlobalNodesVersion::V2,
        }
    }

    pub fn always_visible_global_nodes_version(&self) -> AlwaysVisibleGlobalNodesVersion {
        match self {
            KernelBoot::V1 => AlwaysVisibleGlobalNodesVersion::V1,
            KernelBoot::V2 {
                global_nodes_version,
                ..
            } => *global_nodes_version,
        }
    }

    pub fn always_visible_global_nodes(&self) -> &'static IndexSet<NodeId> {
        always_visible_global_nodes(self.always_visible_global_nodes_version())
    }
}

pub struct KernelInit<
    's,
    S: SubstateDatabase,
    I: InitializationParameters<For: KernelTransactionExecutor<Init = I>>,
> {
    substate_db: &'s S,
    kernel_boot: KernelBoot,
    callback_init: I,
}

impl<
        's,
        S: SubstateDatabase,
        I: InitializationParameters<For: KernelTransactionExecutor<Init = I>>,
    > KernelInit<'s, S, I>
{
    pub fn load(substate_db: &'s S, callback_init: I) -> Self {
        let kernel_boot = KernelBoot::load(substate_db);
        Self {
            substate_db,
            kernel_boot,
            callback_init,
        }
    }

    /// Executes a transaction
    pub fn execute(
        self,
        executable: &<I::For as KernelTransactionExecutor>::Executable,
    ) -> <I::For as KernelTransactionExecutor>::Receipt {
        let boot_loader = BootLoader {
            id_allocator: IdAllocator::new(executable.unique_seed_for_id_allocator()),
            track: Track::new(self.substate_db),
        };

        #[cfg(not(all(target_os = "linux", feature = "std", feature = "cpu_ram_metrics")))]
        {
            boot_loader.execute::<I::For>(self.kernel_boot, self.callback_init, executable)
        }

        #[cfg(all(target_os = "linux", feature = "std", feature = "cpu_ram_metrics"))]
        {
            use crate::kernel::resources_tracker::ResourcesTracker;

            let mut resources_tracker = ResourcesTracker::start_measurement();
            let mut receipt =
                boot_loader.execute::<I::For>(self.kernel_boot, self.callback_init, executable);
            receipt.set_resource_usage(resources_tracker.end_measurement());
            receipt
        }
    }
}

/// Organizes the radix engine stack to make a function entrypoint available for execution
pub struct BootLoader<'h, S: SubstateDatabase> {
    id_allocator: IdAllocator,
    track: Track<'h, S>,
}

impl<'h, S: SubstateDatabase> BootLoader<'h, S> {
    fn execute<E: KernelTransactionExecutor>(
        mut self,
        kernel_boot: KernelBoot,
        callback_init: E::Init,
        executable: &E::Executable,
    ) -> E::Receipt {
        #[cfg(feature = "resource_tracker")]
        radix_engine_profiling::QEMU_PLUGIN_CALIBRATOR.with(|v| {
            v.borrow_mut();
        });

        // Upper Layer Initialization
        let system_init_result = E::init(
            &mut self.track,
            &executable,
            callback_init,
            kernel_boot.always_visible_global_nodes(),
        );

        let (mut system, call_frame_inits) = match system_init_result {
            Ok(success) => success,
            Err(receipt) => return receipt,
        };

        // Kernel Initialization
        let mut kernel = Kernel::new(
            &mut self.track,
            &mut self.id_allocator,
            &mut system,
            call_frame_inits,
        );

        // Execution
        let result = || -> Result<E::ExecutionOutput, RuntimeError> {
            // Invoke transaction processor
            let output = E::execute(&mut kernel, executable)?;

            // Sanity check call frame
            for stack in &kernel.stacks.stacks {
                assert!(stack.prev_frames.is_empty());
            }

            // Sanity check heap
            assert!(kernel.substate_io.heap.is_empty());

            // Finalize state updates based on what has occurred
            let commit_info = kernel.substate_io.store.get_commit_info();
            kernel.callback.finalize(executable, commit_info)?;

            Ok(output)
        }()
        .map_err(|e| TransactionExecutionError::RuntimeError(e));

        // Create receipt representing the result of a transaction
        system.create_receipt(self.track, result)
    }
}

pub struct KernelStack<M: KernelCallbackObject> {
    current_frame: CallFrame<M::CallFrameData, M::LockData>,
    prev_frames: Vec<CallFrame<M::CallFrameData, M::LockData>>,
}

impl<M: KernelCallbackObject> KernelStack<M> {
    pub fn new(init: CallFrameInit<M::CallFrameData>) -> Self {
        Self {
            current_frame: CallFrame::new_root(init),
            prev_frames: vec![],
        }
    }
}

/// The kernel manages multiple call frame stacks. There will always be a single
/// "current" stack (and call frame) in context.
pub struct KernelStacks<M: KernelCallbackObject> {
    current_stack_index: usize,
    stacks: Vec<KernelStack<M>>,
}

impl<M: KernelCallbackObject> KernelStacks<M> {
    pub fn new(call_frames: Vec<CallFrameInit<M::CallFrameData>>) -> Self {
        let stacks = call_frames
            .into_iter()
            .map(|call_frame| KernelStack::new(call_frame))
            .collect();
        Self {
            current_stack_index: 0usize,
            stacks,
        }
    }

    fn current_stack_mut(&mut self) -> &mut KernelStack<M> {
        self.stacks.get_mut(self.current_stack_index).unwrap()
    }

    fn current_stack(&self) -> &KernelStack<M> {
        self.stacks.get(self.current_stack_index).unwrap()
    }

    /// Pushes a new call frame on the current stack
    pub fn push_frame(&mut self, frame: CallFrame<M::CallFrameData, M::LockData>) {
        let stack = self.current_stack_mut();
        let parent = mem::replace(&mut stack.current_frame, frame);
        stack.prev_frames.push(parent);
    }

    /// Pushes a call frame from the current stack
    pub fn pop_frame(&mut self) {
        let stack = self.current_stack_mut();
        let parent = stack.prev_frames.pop().unwrap();
        let _ = core::mem::replace(&mut stack.current_frame, parent);
    }

    /// Switches the current stack
    pub fn switch_stack(&mut self, stack_index: usize) -> Result<(), RuntimeError> {
        if stack_index >= self.stacks.len() {
            return Err(RuntimeError::KernelError(KernelError::StackError(
                StackError::InvalidStackId,
            )));
        }
        self.current_stack_index = stack_index;

        Ok(())
    }

    pub fn current_frame_mut_in_this_and_other_stack(
        &mut self,
        other_stack: usize,
    ) -> (
        &mut CallFrame<M::CallFrameData, M::LockData>,
        &mut CallFrame<M::CallFrameData, M::LockData>,
    ) {
        let mut mut_stacks: Vec<_> = self
            .stacks
            .iter_mut()
            .enumerate()
            .filter(|(id, _)| (*id).eq(&self.current_stack_index) || (*id).eq(&other_stack))
            .map(|stack| Some(stack))
            .collect();

        let (id0, stack0) = mut_stacks[0].take().unwrap();
        let (_id1, stack1) = mut_stacks[1].take().unwrap();
        if id0.eq(&self.current_stack_index) {
            (&mut stack0.current_frame, &mut stack1.current_frame)
        } else {
            (&mut stack1.current_frame, &mut stack0.current_frame)
        }
    }

    pub fn current_frame_and_previous_frame(
        &self,
    ) -> (
        &CallFrame<M::CallFrameData, M::LockData>,
        Option<&CallFrame<M::CallFrameData, M::LockData>>,
    ) {
        let stack = self.current_stack();
        (&stack.current_frame, stack.prev_frames.last())
    }

    pub fn mut_current_frame_and_previous_frame(
        &mut self,
    ) -> (
        &mut CallFrame<M::CallFrameData, M::LockData>,
        Option<&CallFrame<M::CallFrameData, M::LockData>>,
    ) {
        let stack = self.current_stack_mut();
        (&mut stack.current_frame, stack.prev_frames.last())
    }

    pub fn mut_current_frame_and_mut_previous_frame(
        &mut self,
    ) -> (
        &mut CallFrame<M::CallFrameData, M::LockData>,
        Option<&mut CallFrame<M::CallFrameData, M::LockData>>,
    ) {
        let stack = self.current_stack_mut();
        (&mut stack.current_frame, stack.prev_frames.last_mut())
    }

    pub fn current_frame(&self) -> &CallFrame<M::CallFrameData, M::LockData> {
        &self.current_stack().current_frame
    }

    pub fn current_frame_mut(&mut self) -> &mut CallFrame<M::CallFrameData, M::LockData> {
        &mut self.current_stack_mut().current_frame
    }

    #[cfg(feature = "radix_engine_tests")]
    pub fn previous_frames_mut(&mut self) -> &mut Vec<CallFrame<M::CallFrameData, M::LockData>> {
        &mut self.current_stack_mut().prev_frames
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
    stacks: KernelStacks<M>,

    substate_io: SubstateIO<'g, S>,

    /// ID allocator
    id_allocator: &'g mut IdAllocator,

    /// Upper system layer
    callback: &'g mut M,
}

#[cfg(feature = "radix_engine_tests")]
impl<'g, M: KernelCallbackObject<CallFrameData: Default>, S: CommitableSubstateStore>
    Kernel<'g, M, S>
{
    pub fn new_no_refs(
        store: &'g mut S,
        id_allocator: &'g mut IdAllocator,
        callback: &'g mut M,
    ) -> Self {
        Self::new(
            store,
            id_allocator,
            callback,
            vec![CallFrameInit {
                data: M::CallFrameData::default(),
                direct_accesses: Default::default(),
                global_addresses: Default::default(),
                always_visible_global_nodes: always_visible_global_nodes(
                    AlwaysVisibleGlobalNodesVersion::latest(),
                ),
                stack_id: 0,
            }],
        )
    }
}

impl<'g, M: KernelCallbackObject, S: CommitableSubstateStore> Kernel<'g, M, S> {
    pub fn new(
        store: &'g mut S,
        id_allocator: &'g mut IdAllocator,
        callback: &'g mut M,
        call_frame_inits: Vec<CallFrameInit<M::CallFrameData>>,
    ) -> Self {
        Kernel {
            stacks: KernelStacks::new(call_frame_inits),
            substate_io: SubstateIO {
                heap: Heap::new(),
                store,
                non_global_node_refs: NonGlobalNodeRefs::new(),
                substate_locks: SubstateLocks::new(),
                heap_transient_substates: TransientSubstates::new(),
                pinned_to_heap: BTreeSet::new(),
            },
            id_allocator,
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
            previous_frame: self.prev_frame,
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
            previous_frame: self.prev_frame,
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

        self.stacks
            .current_frame_mut()
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

        let (cur_frame, prev_frame) = self.stacks.mut_current_frame_and_previous_frame();

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame,
            on_io_access: |api, io_access| {
                M::on_create_node(CreateNodeEvent::IOAccess(&io_access), api)
            },
        };

        cur_frame
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

            let (cur_frame, prev_frame) = self.stacks.mut_current_frame_and_previous_frame();

            let mut handler = KernelHandler {
                callback: self.callback,
                prev_frame,
                on_io_access: |api, io_access| {
                    M::on_create_node(CreateNodeEvent::IOAccess(&io_access), api)
                },
            };

            cur_frame
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
            let (cur_frame, prev_frame) = self.stacks.mut_current_frame_and_previous_frame();

            let mut handler = KernelHandler {
                callback: self.callback,
                prev_frame,
                on_io_access: |api, io_access| {
                    M::on_move_module(MoveModuleEvent::IOAccess(&io_access), api)
                },
            };

            for (dest_partition_number, (src_node_id, src_partition_number)) in partitions {
                cur_frame
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

        let (cur_frame, prev_frame) = self.stacks.mut_current_frame_and_previous_frame();

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame,
            on_io_access: |api, io_access| {
                M::on_drop_node(DropNodeEvent::IOAccess(&io_access), api)
            },
        };
        let dropped_node = cur_frame
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

// TODO: Remove
impl<'g, M: KernelCallbackObject, S: CommitableSubstateStore> KernelInternalApi
    for Kernel<'g, M, S>
{
    type System = M;

    fn kernel_get_node_visibility_uncosted(&self, node_id: &NodeId) -> NodeVisibility {
        self.stacks.current_frame().get_node_visibility(node_id)
    }

    fn kernel_get_current_stack_depth_uncosted(&self) -> usize {
        self.stacks.current_frame().depth()
    }

    fn kernel_get_current_stack_id_uncosted(&self) -> usize {
        self.stacks.current_stack_index
    }

    fn kernel_get_system_state(&mut self) -> SystemState<'_, M> {
        let (cur, prev) = self.stacks.current_frame_and_previous_frame();
        let caller_actor = match prev {
            Some(call_frame) => call_frame.data(),
            None => {
                // This will only occur on initialization
                cur.data()
            }
        };
        SystemState {
            system: &mut self.callback,
            current_call_frame: cur.data(),
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
    previous_frame: Option<&'g CallFrame<M::CallFrameData, M::LockData>>,
    heap: &'g Heap,
    callback: &'g mut M,
}

impl<'g, M: KernelCallbackObject> KernelInternalApi for KernelReadOnly<'g, M> {
    type System = M;

    fn kernel_get_node_visibility_uncosted(&self, node_id: &NodeId) -> NodeVisibility {
        self.current_frame.get_node_visibility(node_id)
    }

    fn kernel_get_current_stack_depth_uncosted(&self) -> usize {
        self.current_frame.depth()
    }

    fn kernel_get_current_stack_id_uncosted(&self) -> usize {
        self.current_frame.stack_id()
    }

    fn kernel_get_system_state(&mut self) -> SystemState<'_, M> {
        let caller_call_frame = match self.previous_frame {
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

        self.stacks
            .current_frame_mut()
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

        let (cur_frame, prev_frame) = self.stacks.mut_current_frame_and_previous_frame();

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame,
            on_io_access: |api, io_access| {
                M::on_open_substate(OpenSubstateEvent::IOAccess(&io_access), api)
            },
        };

        let maybe_lock_handle = cur_frame.open_substate(
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
                    let (cur_frame, prev_frame) =
                        self.stacks.mut_current_frame_and_previous_frame();

                    let mut handler = KernelHandler {
                        callback: self.callback,
                        prev_frame,
                        on_io_access: |api, io_access| {
                            M::on_open_substate(OpenSubstateEvent::IOAccess(&io_access), api)
                        },
                    };

                    cur_frame
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
        self.stacks
            .current_frame()
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
        let (cur_frame, prev_frame) = self.stacks.mut_current_frame_and_previous_frame();
        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame,
            on_io_access: |api, io_access| {
                M::on_read_substate(ReadSubstateEvent::IOAccess(&io_access), api)
            },
        };

        let value = cur_frame
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

        let (cur_frame, prev_frame) = self.stacks.mut_current_frame_and_previous_frame();

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame,
            on_io_access: |api, io_access| {
                M::on_write_substate(WriteSubstateEvent::IOAccess(&io_access), api)
            },
        };

        cur_frame
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

        self.stacks
            .current_frame_mut()
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

        let (cur_frame, prev_frame) = self.stacks.mut_current_frame_and_previous_frame();

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame,
            on_io_access: |api, io_access| {
                M::on_set_substate(SetSubstateEvent::IOAccess(&io_access), api)
            },
        };

        cur_frame
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

        let (cur_frame, prev_frame) = self.stacks.mut_current_frame_and_previous_frame();

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame,
            on_io_access: |api, io_access| {
                M::on_remove_substate(RemoveSubstateEvent::IOAccess(&io_access), api)
            },
        };

        let substate = cur_frame
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
    fn kernel_scan_keys<K: SubstateKeyContent>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        limit: u32,
    ) -> Result<Vec<SubstateKey>, RuntimeError> {
        M::on_scan_keys(ScanKeysEvent::Start, &mut as_read_only!(self))?;

        let (cur_frame, prev_frame) = self.stacks.mut_current_frame_and_previous_frame();

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame,
            on_io_access: |api, io_access| {
                M::on_scan_keys(ScanKeysEvent::IOAccess(&io_access), api)
            },
        };

        let keys = cur_frame
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
    fn kernel_drain_substates<K: SubstateKeyContent>(
        &mut self,
        node_id: &NodeId,
        partition_num: PartitionNumber,
        limit: u32,
    ) -> Result<Vec<(SubstateKey, IndexedScryptoValue)>, RuntimeError> {
        M::on_drain_substates(DrainSubstatesEvent::Start(limit), &mut as_read_only!(self))?;

        let (cur_frame, prev_frame) = self.stacks.mut_current_frame_and_previous_frame();

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame,
            on_io_access: |api, io_access| {
                M::on_drain_substates(DrainSubstatesEvent::IOAccess(&io_access), api)
            },
        };

        let substates = cur_frame
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

        let (cur_frame, prev_frame) = self.stacks.mut_current_frame_and_previous_frame();

        let mut handler = KernelHandler {
            callback: self.callback,
            prev_frame,
            on_io_access: |api, io_access| {
                M::on_scan_sorted_substates(ScanSortedSubstatesEvent::IOAccess(&io_access), api)
            },
        };

        let substates = cur_frame
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
                self.stacks.current_frame_mut(),
                callee,
                message,
            )
            .map_err(CallFrameError::CreateFrameError)
            .map_err(KernelError::CallFrameError)?;

            self.stacks.push_frame(frame);
        }

        // Execute
        let (output, message) = {
            // Handle execution start
            M::on_execution_start(self)?;

            // Auto drop locks
            for handle in self.stacks.current_frame().open_substates() {
                M::on_close_substate(CloseSubstateEvent::Start(handle), self)?;
            }
            self.stacks
                .current_frame_mut()
                .close_all_substates(&mut self.substate_io);

            // Run
            let output = M::invoke_upstream(args, self)?;
            let message = CallFrameMessage::from_output(&output);

            // Auto-drop locks again in case module forgot to drop
            for handle in self.stacks.current_frame().open_substates() {
                M::on_close_substate(CloseSubstateEvent::Start(handle), self)?;
            }
            self.stacks
                .current_frame_mut()
                .close_all_substates(&mut self.substate_io);

            // Handle execution finish
            M::on_execution_finish(&message, self)?;

            (output, message)
        };

        // Move
        {
            let (cur_frame, prev_frame) = self.stacks.mut_current_frame_and_mut_previous_frame();

            // Move resource
            CallFrame::pass_message(
                &self.substate_io,
                cur_frame,
                prev_frame.unwrap(),
                message.clone(),
            )
            .map_err(CallFrameError::PassMessageError)
            .map_err(KernelError::CallFrameError)?;

            // Auto-drop
            let owned_nodes = cur_frame.owned_nodes();
            M::auto_drop(owned_nodes, self)?;

            // Now, check if any own has been left!
            let owned_nodes = self.stacks.current_frame().owned_nodes();
            if !owned_nodes.is_empty() {
                return Err(RuntimeError::KernelError(KernelError::OrphanedNodes(
                    owned_nodes.into_iter().map(|n| n.into()).collect(),
                )));
            }
        }

        // Pop call frame
        self.stacks.pop_frame();

        M::after_invoke(&output, self)?;

        Ok(output)
    }
}

impl<'g, M: KernelCallbackObject, S: CommitableSubstateStore> KernelStackApi for Kernel<'g, M, S> {
    type CallFrameData = M::CallFrameData;

    fn kernel_get_stack_id(&mut self) -> Result<usize, RuntimeError> {
        M::on_get_stack_id(&mut as_read_only!(self))?;

        Ok(self.stacks.current_stack_index)
    }

    fn kernel_switch_stack(&mut self, other_stack_index: usize) -> Result<(), RuntimeError> {
        M::on_switch_stack(&mut as_read_only!(self))?;

        self.stacks.switch_stack(other_stack_index)?;
        Ok(())
    }

    fn kernel_send_to_stack(
        &mut self,
        other_stack_index: usize,
        value: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        M::on_send_to_stack(value, &mut as_read_only!(self))?;

        let message = CallFrameMessage::from_output(value);

        let (cur, other) = self
            .stacks
            .current_frame_mut_in_this_and_other_stack(other_stack_index);

        CallFrame::pass_message(&self.substate_io, cur, other, message)
            .map_err(CallFrameError::PassMessageError)
            .map_err(KernelError::CallFrameError)?;

        Ok(())
    }

    fn kernel_set_call_frame_data(&mut self, data: M::CallFrameData) -> Result<(), RuntimeError> {
        M::on_set_call_frame_data(&data, &mut as_read_only!(self))?;

        *self.stacks.current_frame_mut().data_mut() = data;
        Ok(())
    }

    fn kernel_get_owned_nodes(&mut self) -> Result<Vec<NodeId>, RuntimeError> {
        M::on_get_owned_nodes(&mut as_read_only!(self))?;

        Ok(self.stacks.current_frame().owned_nodes())
    }
}

impl<'g, M: KernelCallbackObject, S: CommitableSubstateStore> KernelApi for Kernel<'g, M, S> {
    type CallbackObject = M;
}

#[cfg(feature = "radix_engine_tests")]
impl<'g, M, S> Kernel<'g, M, S>
where
    M: KernelCallbackObject<CallFrameData: Default>,
    S: CommitableSubstateStore,
{
    pub fn kernel_create_kernel_for_testing(
        substate_io: SubstateIO<'g, S>,
        id_allocator: &'g mut IdAllocator,
        callback: &'g mut M,
        always_visible_global_nodes: &'static IndexSet<NodeId>,
    ) -> Kernel<'g, M, S> {
        Self {
            stacks: KernelStacks::new(vec![CallFrameInit {
                data: M::CallFrameData::default(),
                direct_accesses: Default::default(),
                global_addresses: Default::default(),
                always_visible_global_nodes,
                stack_id: 0,
            }]),
            substate_io,
            id_allocator,
            callback,
        }
    }
}

#[cfg(feature = "radix_engine_tests")]
impl<'g, M: KernelCallbackObject, S: CommitableSubstateStore> Kernel<'g, M, S> {
    pub fn kernel_current_frame(
        &self,
    ) -> &CallFrame<<M as KernelCallbackObject>::CallFrameData, <M as KernelCallbackObject>::LockData>
    {
        self.stacks.current_frame()
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
        (&self.substate_io, self.stacks.current_frame_mut())
    }

    pub fn kernel_prev_frame_stack_mut(
        &mut self,
    ) -> &mut Vec<
        CallFrame<
            <M as KernelCallbackObject>::CallFrameData,
            <M as KernelCallbackObject>::LockData,
        >,
    > {
        self.stacks.previous_frames_mut()
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
