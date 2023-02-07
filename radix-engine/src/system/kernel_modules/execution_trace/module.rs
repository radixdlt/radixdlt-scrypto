use crate::errors::*;
use crate::kernel::*;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_substates::PersistedSubstate;
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::*;
use sbor::rust::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceChange {
    pub resource_address: ResourceAddress,
    pub component_id: ComponentId, // TODO: support non component actor
    pub vault_id: VaultId,
    pub amount: Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionTraceReceipt {
    pub resource_changes: Vec<ResourceChange>,
}

#[derive(Debug)]
pub enum VaultOp {
    Create(Decimal), // TODO: add trace of vault creation
    Put(Decimal),    // TODO: add non-fungible support
    Take(Decimal),
    LockFee,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ExecutionTraceError {
    CallFrameError(CallFrameError),
}

pub struct ExecutionTraceModule {
    /// Maximum depth up to which kernel calls are being traced.
    max_kernel_call_depth_traced: usize,

    /// Current kernel calls depth. Note that this doesn't necessarily correspond to the
    /// call frame depth, as there can be nested kernel calls within a single call frame
    /// (e.g. lock_substate call inside drop_node).
    current_kernel_call_depth: usize,

    /// A stack of traced kernel call inputs, their origin, and the instruction index.
    traced_kernel_call_inputs_stack:
        Vec<(TracedKernelCallData, KernelCallTraceOrigin, Option<u32>)>,

    /// A mapping of complete KernelCallTrace stacks (\w both inputs and outputs), indexed by depth.
    kernel_call_traces_stacks: HashMap<usize, Vec<KernelCallTrace>>,
}

#[derive(Clone, Debug, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ProofSnapshot {
    pub resource_address: ResourceAddress,
    pub resource_type: ResourceType,
    pub restricted: bool,
    pub total_locked: LockedAmountOrIds,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TracedKernelCallData {
    pub buckets: HashMap<BucketId, Resource>,
    pub proofs: HashMap<ProofId, ProofSnapshot>,
}

impl TracedKernelCallData {
    pub fn new_empty() -> Self {
        Self {
            buckets: HashMap::new(),
            proofs: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buckets.is_empty() && self.proofs.is_empty()
    }
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct KernelCallTrace {
    pub origin: KernelCallTraceOrigin,
    pub kernel_call_depth: usize,
    pub call_frame_actor: ResolvedActor,
    pub call_frame_depth: usize,
    pub instruction_index: Option<u32>,
    pub input: TracedKernelCallData,
    pub output: TracedKernelCallData,
    pub children: Vec<KernelCallTrace>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum KernelCallTraceOrigin {
    ScryptoFunction(ScryptoFnIdentifier),
    ScryptoMethod(ScryptoFnIdentifier),
    NativeFn(NativeFn),
    CreateNode,
    DropNode,
}

impl KernelModule for ExecutionTraceModule {
    fn pre_create_node(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        _node_id: &RENodeId,
        _node_init: &RENodeInit,
        _node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), ModuleError> {
        self.handle_pre_create_node(current_frame, heap)
    }

    fn post_create_node(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        node_id: &RENodeId,
    ) -> Result<(), ModuleError> {
        self.handle_post_create_node(current_frame, heap, node_id)
    }

    fn pre_drop_node(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        node_id: &RENodeId,
    ) -> Result<(), ModuleError> {
        self.handle_pre_drop_node(current_frame, heap, node_id)
    }

    fn post_drop_node(
        &mut self,
        current_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track,
    ) -> Result<(), ModuleError> {
        self.handle_post_drop_node(current_frame)
    }

    fn pre_kernel_execute(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        callee: &ResolvedActor,
        update: &CallFrameUpdate,
    ) -> Result<(), ModuleError> {
        if self.current_kernel_call_depth <= self.max_kernel_call_depth_traced {
            let origin = match &callee.identifier {
                FnIdentifier::Scrypto(scrypto_fn) => {
                    if callee.receiver.is_some() {
                        KernelCallTraceOrigin::ScryptoMethod(scrypto_fn.clone())
                    } else {
                        KernelCallTraceOrigin::ScryptoFunction(scrypto_fn.clone())
                    }
                }
                FnIdentifier::Native(native_fn) => {
                    KernelCallTraceOrigin::NativeFn(native_fn.clone())
                }
            };

            let instruction_index = Self::get_instruction_index(call_frame, heap);

            let trace_data = Self::extract_trace_data(update, heap)?;
            self.traced_kernel_call_inputs_stack
                .push((trace_data, origin, instruction_index));
        }

        self.current_kernel_call_depth += 1;

        match &callee {
            ResolvedActor {
                identifier: FnIdentifier::Native(NativeFn::Vault(VaultFn::Put)),
                receiver:
                    Some(ResolvedReceiver {
                        receiver: RENodeId::Vault(vault_id),
                        ..
                    }),
            } => Self::handle_vault_put_input(update, heap, track, &call_frame.actor, vault_id),
            ResolvedActor {
                identifier: FnIdentifier::Native(NativeFn::Vault(VaultFn::LockFee)),
                receiver:
                    Some(ResolvedReceiver {
                        receiver: RENodeId::Vault(vault_id),
                        ..
                    }),
            } => Self::handle_vault_lock_fee_input(track, &call_frame.actor, vault_id),
            _ => {}
        }

        Ok(())
    }

    fn post_kernel_execute(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track,
        update: &CallFrameUpdate,
    ) -> Result<(), ModuleError> {
        match &call_frame.actor {
            ResolvedActor {
                identifier: FnIdentifier::Native(NativeFn::Vault(VaultFn::Take)),
                receiver:
                    Some(ResolvedReceiver {
                        receiver: RENodeId::Vault(vault_id),
                        ..
                    }),
            } => Self::handle_vault_take_output(update, heap, track, &call_frame.actor, vault_id),
            _ => {}
        }

        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth -= 1;

        if self.current_kernel_call_depth > self.max_kernel_call_depth_traced {
            // Nothing to trace at this depth, exit.
            return Ok(());
        }

        let traced_output = Self::extract_trace_data(update, heap)?;
        self.finalize_kernel_call_trace(call_frame, traced_output)
    }
}

impl ExecutionTraceModule {
    pub fn new(max_kernel_call_depth_traced: usize) -> ExecutionTraceModule {
        Self {
            max_kernel_call_depth_traced,
            current_kernel_call_depth: 0,
            traced_kernel_call_inputs_stack: vec![],
            kernel_call_traces_stacks: HashMap::new(),
        }
    }

    fn get_instruction_index(call_frame: &CallFrame, heap: &mut Heap) -> Option<u32> {
        if call_frame
            .get_node_visibility(RENodeId::TransactionRuntime)
            .is_ok()
        {
            let substate_ref = heap
                .get_substate(
                    RENodeId::TransactionRuntime,
                    NodeModuleId::SELF,
                    &SubstateOffset::TransactionRuntime(
                        TransactionRuntimeOffset::TransactionRuntime,
                    ),
                )
                .unwrap();
            Some(substate_ref.transaction_runtime().instruction_index)
        } else {
            None
        }
    }

    fn handle_pre_create_node(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
    ) -> Result<(), ModuleError> {
        if self.current_kernel_call_depth <= self.max_kernel_call_depth_traced {
            let instruction_index = Self::get_instruction_index(call_frame, heap);

            let traced_input = (
                TracedKernelCallData::new_empty(),
                KernelCallTraceOrigin::CreateNode,
                instruction_index,
            );
            self.traced_kernel_call_inputs_stack.push(traced_input);
        }

        self.current_kernel_call_depth += 1;
        Ok(())
    }

    fn handle_pre_drop_node(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        node_id: &RENodeId,
    ) -> Result<(), ModuleError> {
        if self.current_kernel_call_depth <= self.max_kernel_call_depth_traced {
            let instruction_index = Self::get_instruction_index(call_frame, heap);

            let traced_input = {
                // Buckets can't be dropped, so only tracking Proofs here
                let data = if let RENodeId::Proof(proof_id) = node_id {
                    let proof = Self::read_proof(heap, proof_id)?;
                    TracedKernelCallData {
                        buckets: HashMap::new(),
                        proofs: HashMap::from([(proof_id.clone(), proof)]),
                    }
                } else {
                    // Not a proof, so nothing to trace
                    TracedKernelCallData::new_empty()
                };

                (data, KernelCallTraceOrigin::DropNode, instruction_index)
            };
            self.traced_kernel_call_inputs_stack.push(traced_input);
        }

        self.current_kernel_call_depth += 1;
        Ok(())
    }

    fn handle_post_create_node(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        node_id: &RENodeId,
    ) -> Result<(), ModuleError> {
        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth -= 1;

        if self.current_kernel_call_depth > self.max_kernel_call_depth_traced {
            // Nothing to trace at this depth, exit.
            return Ok(());
        }

        let traced_output = match node_id {
            RENodeId::Bucket(bucket_id) => {
                let bucket_resource = Self::read_bucket_resource(heap, bucket_id)?;
                TracedKernelCallData {
                    buckets: HashMap::from([(bucket_id.clone(), bucket_resource)]),
                    proofs: HashMap::new(),
                }
            }
            RENodeId::Proof(proof_id) => {
                let proof = Self::read_proof(heap, proof_id)?;
                TracedKernelCallData {
                    buckets: HashMap::new(),
                    proofs: HashMap::from([(proof_id.clone(), proof)]),
                }
            }
            _ => TracedKernelCallData::new_empty(),
        };

        self.finalize_kernel_call_trace(call_frame, traced_output)
    }

    fn handle_post_drop_node(&mut self, call_frame: &CallFrame) -> Result<(), ModuleError> {
        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth -= 1;

        if self.current_kernel_call_depth > self.max_kernel_call_depth_traced {
            // Nothing to trace at this depth, exit.
            return Ok(());
        }

        let traced_output = TracedKernelCallData::new_empty();

        self.finalize_kernel_call_trace(call_frame, traced_output)
    }

    fn finalize_kernel_call_trace(
        &mut self,
        call_frame: &CallFrame,
        traced_output: TracedKernelCallData,
    ) -> Result<(), ModuleError> {
        let child_traces = self
            .kernel_call_traces_stacks
            .remove(&(self.current_kernel_call_depth + 1))
            .unwrap_or(vec![]);

        let (traced_input, origin, instruction_index) = self
            .traced_kernel_call_inputs_stack
            .pop()
            .expect("kernel call input stack underflow");

        // Only include the trace if:
        // * there's a non-empty traced input or output
        // * OR there are any child traces: they need a parent regardless of whether it traces any inputs/outputs.
        //   At some depth (up to the tracing limit) there must have been at least one traced input/output
        //   so we need to include the full path up to the root.
        if !traced_input.is_empty() || !traced_output.is_empty() || !child_traces.is_empty() {
            let trace = KernelCallTrace {
                origin,
                kernel_call_depth: self.current_kernel_call_depth,
                call_frame_actor: call_frame.actor.clone(),
                call_frame_depth: call_frame.depth,
                instruction_index,
                input: traced_input,
                output: traced_output,
                children: child_traces,
            };

            let siblings = self
                .kernel_call_traces_stacks
                .entry(self.current_kernel_call_depth)
                .or_insert(vec![]);
            siblings.push(trace);
        }

        Ok(())
    }

    pub fn collect_events(&mut self) -> Vec<TrackedEvent> {
        let mut events = Vec::new();
        for (_, traces) in self.kernel_call_traces_stacks.drain() {
            // Emit an output event for each "root" kernel call trace
            for trace in traces {
                events.push(TrackedEvent::KernelCallTrace(trace));
            }
        }

        events
    }

    fn extract_trace_data(
        call_frame_update: &CallFrameUpdate,
        heap: &mut Heap,
    ) -> Result<TracedKernelCallData, ModuleError> {
        let mut buckets: HashMap<BucketId, Resource> = HashMap::new();
        let mut proofs: HashMap<ProofId, ProofSnapshot> = HashMap::new();

        for node_id in &call_frame_update.nodes_to_move {
            match node_id {
                RENodeId::Bucket(bucket_id) => {
                    let bucket_resource = Self::read_bucket_resource(heap, &bucket_id)?;
                    buckets.insert(*bucket_id, bucket_resource);
                }
                RENodeId::Proof(proof_id) => {
                    let proof = Self::read_proof(heap, &proof_id)?;
                    proofs.insert(*proof_id, proof);
                }
                _ => {}
            }
        }

        Ok(TracedKernelCallData { buckets, proofs })
    }

    fn read_proof(heap: &mut Heap, proof_id: &ProofId) -> Result<ProofSnapshot, ModuleError> {
        let node_id = RENodeId::Proof(proof_id.clone());
        let substate_ref = heap
            .get_substate(
                node_id,
                NodeModuleId::SELF,
                &SubstateOffset::Proof(ProofOffset::Proof),
            )
            .map_err(|e| {
                ModuleError::ExecutionTraceError(ExecutionTraceError::CallFrameError(e))
            })?;
        Ok(substate_ref.proof().snapshot())
    }

    fn read_bucket_resource(
        heap: &mut Heap,
        bucket_id: &BucketId,
    ) -> Result<Resource, ModuleError> {
        let node_id = RENodeId::Bucket(bucket_id.clone());
        let substate_ref = heap
            .get_substate(
                node_id,
                NodeModuleId::SELF,
                &SubstateOffset::Bucket(BucketOffset::Bucket),
            )
            .map_err(|e| {
                ModuleError::ExecutionTraceError(ExecutionTraceError::CallFrameError(e))
            })?;
        Ok(substate_ref.bucket().peek_resource())
    }

    fn handle_vault_put_input<'s>(
        call_frame_update: &CallFrameUpdate,
        heap: &mut Heap,
        track: &mut Track<'s>,
        actor: &ResolvedActor,
        vault_id: &VaultId,
    ) {
        for node_id in &call_frame_update.nodes_to_move {
            match node_id {
                RENodeId::Bucket(bucket_id) => {
                    if let Ok(bucket_substate) = heap.get_substate(
                        RENodeId::Bucket(*bucket_id),
                        NodeModuleId::SELF,
                        &SubstateOffset::Bucket(BucketOffset::Bucket),
                    ) {
                        track.vault_ops.push((
                            actor.clone(),
                            vault_id.clone(),
                            VaultOp::Put(bucket_substate.bucket().total_amount()),
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_vault_lock_fee_input<'s>(
        track: &mut Track<'s>,
        actor: &ResolvedActor,
        vault_id: &VaultId,
    ) {
        track
            .vault_ops
            .push((actor.clone(), vault_id.clone(), VaultOp::LockFee));
    }

    fn handle_vault_take_output<'s>(
        update: &CallFrameUpdate,
        heap: &mut Heap,
        track: &mut Track<'s>,
        actor: &ResolvedActor,
        vault_id: &VaultId,
    ) {
        for node_id in &update.nodes_to_move {
            match node_id {
                RENodeId::Bucket(bucket_id) => {
                    if let Ok(bucket_substate) = heap.get_substate(
                        RENodeId::Bucket(*bucket_id),
                        NodeModuleId::SELF,
                        &SubstateOffset::Bucket(BucketOffset::Bucket),
                    ) {
                        track.vault_ops.push((
                            actor.clone(),
                            vault_id.clone(),
                            VaultOp::Take(bucket_substate.bucket().total_amount()),
                        ));
                    }
                }
                _ => {}
            }
        }
    }
}

impl ExecutionTraceReceipt {
    // TODO: is it better to derive resource changes from substate diff, instead of execution trace?
    // The current approach relies on various runtime invariants.

    pub fn new(
        ops: Vec<(ResolvedActor, VaultId, VaultOp)>,
        actual_fee_payments: &BTreeMap<VaultId, Decimal>,
        to_persist: &mut HashMap<SubstateId, (PersistedSubstate, Option<u32>)>,
        is_commit_success: bool,
    ) -> Self {
        // TODO: Might want to change the key from being a ComponentId to being an enum to
        //       accommodate for accounts
        let mut vault_changes = HashMap::<ComponentId, HashMap<VaultId, Decimal>>::new();
        let mut vault_locked_by = HashMap::<VaultId, ComponentId>::new();
        for (actor, vault_id, vault_op) in ops {
            if let Some(resolved_receiver) = actor.receiver {
                match resolved_receiver.receiver {
                    RENodeId::Component(component_id) | RENodeId::Account(component_id) => {
                        match vault_op {
                            VaultOp::Create(_) => todo!("Not supported yet!"),
                            VaultOp::Put(amount) => {
                                *vault_changes
                                    .entry(component_id)
                                    .or_default()
                                    .entry(vault_id)
                                    .or_default() += amount;
                            }
                            VaultOp::Take(amount) => {
                                *vault_changes
                                    .entry(component_id)
                                    .or_default()
                                    .entry(vault_id)
                                    .or_default() -= amount;
                            }
                            VaultOp::LockFee => {
                                *vault_changes
                                    .entry(component_id)
                                    .or_default()
                                    .entry(vault_id)
                                    .or_default() -= 0;

                                // Hack: Additional check to avoid second `lock_fee` attempts (runtime failure) from
                                // polluting the `vault_locked_by` index.
                                if !vault_locked_by.contains_key(&vault_id) {
                                    vault_locked_by.insert(vault_id, component_id);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut resource_changes = Vec::<ResourceChange>::new();
        for (component_id, map) in vault_changes {
            for (vault_id, delta) in map {
                // Amount = put/take amount - fee_amount
                let fee_amount = actual_fee_payments
                    .get(&vault_id)
                    .cloned()
                    .unwrap_or_default();
                let amount = if is_commit_success {
                    delta
                } else {
                    Decimal::zero()
                } - fee_amount;

                // Add a resource change log if non-zero
                if !amount.is_zero() {
                    let resource_address = Self::get_vault_resource_address(vault_id, to_persist);
                    resource_changes.push(ResourceChange {
                        resource_address,
                        component_id,
                        vault_id,
                        amount,
                    });
                }
            }
        }

        ExecutionTraceReceipt { resource_changes }
    }

    fn get_vault_resource_address(
        vault_id: VaultId,
        to_persist: &mut HashMap<SubstateId, (PersistedSubstate, Option<u32>)>,
    ) -> ResourceAddress {
        let (substate, _) = to_persist
            .get(&SubstateId(
                RENodeId::Vault(vault_id),
                NodeModuleId::SELF,
                SubstateOffset::Vault(VaultOffset::Vault),
            ))
            .expect("Failed to find the vault substate");
        substate.vault().resource_address()
    }
}
