use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use radix_engine_interface::api::types::{
    BucketOffset, ComponentId, RENodeId, SubstateId, SubstateOffset, VaultFn, VaultId, VaultOffset,
};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
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
    /// Maximum depth up to which sys calls are being traced.
    max_sys_call_trace_depth: usize,

    /// Current sys calls depth. Note that this doesn't necessarily correspond to the
    /// call frame depth, as there can be nested sys calls within a single call frame
    /// (e.g. lock_substate call inside drop_node).
    current_sys_call_depth: usize,

    /// A stack of traced sys call inputs, their origin, and the instruction index.
    traced_sys_call_inputs_stack: Vec<(TracedSysCallData, SysCallTraceOrigin, Option<u32>)>,

    /// A mapping of complete SysCallTrace stacks (\w both inputs and outputs), indexed by depth.
    sys_call_traces_stacks: HashMap<usize, Vec<SysCallTrace>>,
}

#[derive(Clone, Debug, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ProofSnapshot {
    pub resource_address: ResourceAddress,
    pub resource_type: ResourceType,
    pub restricted: bool,
    pub total_locked: LockedAmountOrIds,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct TracedSysCallData {
    pub buckets: HashMap<BucketId, Resource>,
    pub proofs: HashMap<ProofId, ProofSnapshot>,
}

impl TracedSysCallData {
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
pub struct SysCallTrace {
    pub origin: SysCallTraceOrigin,
    pub sys_call_depth: usize,
    pub call_frame_actor: ResolvedActor,
    pub call_frame_depth: usize,
    pub instruction_index: Option<u32>,
    pub input: TracedSysCallData,
    pub output: TracedSysCallData,
    pub children: Vec<SysCallTrace>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum SysCallTraceOrigin {
    ScryptoFunction(ScryptoFnIdentifier),
    ScryptoMethod(ScryptoFnIdentifier),
    NativeFn(NativeFn),
    CreateNode,
    DropNode,
    /// Anything else that isn't traced on its own, but the trace exists for its children
    Opaque,
}

impl<R: FeeReserve> BaseModule<R> for ExecutionTraceModule {
    fn pre_sys_call(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track<R>,
        input: SysCallInput,
    ) -> Result<(), ModuleError> {
        self.handle_pre_sys_call(call_frame, heap, input)
    }

    fn post_sys_call(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track<R>,
        output: SysCallOutput,
    ) -> Result<(), ModuleError> {
        self.handle_post_sys_call(call_frame, heap, output)
    }

    fn pre_execute_invocation(
        &mut self,
        actor: &ResolvedActor,
        update: &CallFrameUpdate,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        if self.current_sys_call_depth <= self.max_sys_call_trace_depth {
            let origin = match &actor.identifier {
                FnIdentifier::Scrypto(scrypto_fn) => {
                    if actor.receiver.is_some() {
                        SysCallTraceOrigin::ScryptoMethod(scrypto_fn.clone())
                    } else {
                        SysCallTraceOrigin::ScryptoFunction(scrypto_fn.clone())
                    }
                }
                FnIdentifier::Native(native_fn) => SysCallTraceOrigin::NativeFn(native_fn.clone()),
            };

            let instruction_index = Self::get_instruction_index(call_frame, heap);

            let trace_data = Self::extract_trace_data(update, heap)?;
            self.traced_sys_call_inputs_stack
                .push((trace_data, origin, instruction_index));
        }

        self.current_sys_call_depth += 1;

        match &actor {
            ResolvedActor {
                identifier: FnIdentifier::Native(NativeFn::Vault(VaultFn::Put)),
                receiver:
                    Some(ResolvedReceiver {
                        receiver: RENodeId::Vault(vault_id),
                        ..
                    }),
            } => Self::handle_vault_put(update, heap, track, &call_frame.actor, vault_id),
            ResolvedActor {
                identifier: FnIdentifier::Native(NativeFn::Vault(VaultFn::LockFee)),
                receiver:
                    Some(ResolvedReceiver {
                        receiver: RENodeId::Vault(vault_id),
                        ..
                    }),
            } => Self::handle_vault_lock_fee(track, &call_frame.actor, vault_id),
            _ => {}
        }

        Ok(())
    }

    fn post_execute_invocation(
        &mut self,
        caller: &ResolvedActor,
        update: &CallFrameUpdate,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        match &call_frame.actor {
            ResolvedActor {
                identifier: FnIdentifier::Native(NativeFn::Vault(VaultFn::Take)),
                receiver:
                    Some(ResolvedReceiver {
                        receiver: RENodeId::Vault(vault_id),
                        ..
                    }),
            } => Self::handle_vault_take(update, heap, track, caller, vault_id),
            _ => {}
        }

        // Important to always update the counter (even if we're over the depth limit).
        self.current_sys_call_depth -= 1;

        if self.current_sys_call_depth > self.max_sys_call_trace_depth {
            // Nothing to trace at this depth, exit.
            return Ok(());
        }

        let traced_output = Self::extract_trace_data(update, heap)?;
        self.finalize_sys_call_trace(call_frame, traced_output)
    }

    fn on_wasm_instantiation(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _code: &[u8],
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_wasm_costing(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _units: u32,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_lock_fee(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _vault_id: VaultId,
        fee: Resource,
        _contingent: bool,
    ) -> Result<Resource, ModuleError> {
        Ok(fee)
    }
}

impl ExecutionTraceModule {
    pub fn new(max_sys_call_trace_depth: usize) -> ExecutionTraceModule {
        Self {
            max_sys_call_trace_depth,
            current_sys_call_depth: 0,
            traced_sys_call_inputs_stack: vec![],
            sys_call_traces_stacks: HashMap::new(),
        }
    }

    fn get_instruction_index(call_frame: &CallFrame, heap: &mut Heap) -> Option<u32> {
        let maybe_runtime_id = call_frame
            .get_visible_nodes()
            .into_iter()
            .find(|e| matches!(e, RENodeId::TransactionRuntime(..)));
        maybe_runtime_id.map(|runtime_id| {
            let substate_ref = heap
                .get_substate(
                    runtime_id,
                    &SubstateOffset::TransactionRuntime(
                        TransactionRuntimeOffset::TransactionRuntime,
                    ),
                )
                .unwrap();
            substate_ref.transaction_runtime().instruction_index
        })
    }

    fn handle_pre_sys_call(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        input: SysCallInput,
    ) -> Result<(), ModuleError> {
        if let SysCallInput::Invoke { .. } = input {
            // Invoke calls are handled separately in pre_execute_invocation
            return Ok(());
        }

        if self.current_sys_call_depth <= self.max_sys_call_trace_depth {
            let instruction_index = Self::get_instruction_index(call_frame, heap);

            let traced_input = match input {
                SysCallInput::DropNode { node_id } => {
                    // Buckets can't be dropped, so only tracking Proofs here
                    let data = if let RENodeId::Proof(proof_id) = node_id {
                        let proof = Self::read_proof(heap, proof_id)?;
                        TracedSysCallData {
                            buckets: HashMap::new(),
                            proofs: HashMap::from([(proof_id.clone(), proof)]),
                        }
                    } else {
                        // Not a proof, so nothing to trace
                        TracedSysCallData::new_empty()
                    };

                    (data, SysCallTraceOrigin::DropNode, instruction_index)
                }
                SysCallInput::CreateNode { .. } => (
                    TracedSysCallData::new_empty(),
                    SysCallTraceOrigin::CreateNode,
                    instruction_index,
                ),
                _ => (
                    TracedSysCallData::new_empty(),
                    SysCallTraceOrigin::Opaque,
                    instruction_index,
                ),
            };
            self.traced_sys_call_inputs_stack.push(traced_input);
        }

        self.current_sys_call_depth += 1;
        Ok(())
    }

    fn handle_post_sys_call(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        output: SysCallOutput,
    ) -> Result<(), ModuleError> {
        if let SysCallOutput::Invoke { .. } = output {
            // Invoke calls are handled separately in post_execute_invocation
            return Ok(());
        }

        // Important to always update the counter (even if we're over the depth limit).
        self.current_sys_call_depth -= 1;

        if self.current_sys_call_depth > self.max_sys_call_trace_depth {
            // Nothing to trace at this depth, exit.
            return Ok(());
        }

        let traced_output = match output {
            SysCallOutput::CreateNode { node_id } => match node_id {
                RENodeId::Bucket(bucket_id) => {
                    let bucket_resource = Self::read_bucket_resource(heap, bucket_id)?;
                    TracedSysCallData {
                        buckets: HashMap::from([(bucket_id.clone(), bucket_resource)]),
                        proofs: HashMap::new(),
                    }
                }
                RENodeId::Proof(proof_id) => {
                    let proof = Self::read_proof(heap, proof_id)?;
                    TracedSysCallData {
                        buckets: HashMap::new(),
                        proofs: HashMap::from([(proof_id.clone(), proof)]),
                    }
                }
                _ => TracedSysCallData::new_empty(),
            },
            _ => TracedSysCallData::new_empty(),
        };

        self.finalize_sys_call_trace(call_frame, traced_output)
    }

    fn finalize_sys_call_trace(
        &mut self,
        call_frame: &CallFrame,
        traced_output: TracedSysCallData,
    ) -> Result<(), ModuleError> {
        let child_traces = self
            .sys_call_traces_stacks
            .remove(&(self.current_sys_call_depth + 1))
            .unwrap_or(vec![]);

        let (traced_input, origin, instruction_index) = self
            .traced_sys_call_inputs_stack
            .pop()
            .expect("Sys call input stack underflow");

        // Only include the trace if:
        // * there's a non-empty traced input or output
        // * OR there are any child traces: they need a parent regardless of whether it traces any inputs/outputs.
        //   At some depth (up to the tracing limit) there must have been at least one traced input/output
        //   so we need to include the full path up to the root.
        if !traced_input.is_empty() || !traced_output.is_empty() || !child_traces.is_empty() {
            let trace = SysCallTrace {
                origin,
                sys_call_depth: self.current_sys_call_depth,
                call_frame_actor: call_frame.actor.clone(),
                call_frame_depth: call_frame.depth,
                instruction_index,
                input: traced_input,
                output: traced_output,
                children: child_traces,
            };

            let siblings = self
                .sys_call_traces_stacks
                .entry(self.current_sys_call_depth)
                .or_insert(vec![]);
            siblings.push(trace);
        }

        Ok(())
    }

    pub fn collect_events(&mut self) -> Vec<TrackedEvent> {
        let mut events = Vec::new();
        for (_, traces) in self.sys_call_traces_stacks.drain() {
            // Emit an output event for each "root" sys call trace
            for trace in traces {
                events.push(TrackedEvent::SysCallTrace(trace));
            }
        }

        events
    }

    fn extract_trace_data(
        call_frame_update: &CallFrameUpdate,
        heap: &mut Heap,
    ) -> Result<TracedSysCallData, ModuleError> {
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

        Ok(TracedSysCallData { buckets, proofs })
    }

    fn read_proof(heap: &mut Heap, proof_id: &ProofId) -> Result<ProofSnapshot, ModuleError> {
        let node_id = RENodeId::Proof(proof_id.clone());
        let substate_ref = heap
            .get_substate(node_id, &SubstateOffset::Proof(ProofOffset::Proof))
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
            .get_substate(node_id, &SubstateOffset::Bucket(BucketOffset::Bucket))
            .map_err(|e| {
                ModuleError::ExecutionTraceError(ExecutionTraceError::CallFrameError(e))
            })?;
        Ok(substate_ref.bucket().peek_resource())
    }

    fn handle_vault_put<'s, R: FeeReserve>(
        call_frame_update: &CallFrameUpdate,
        heap: &mut Heap,
        track: &mut Track<'s, R>,
        actor: &ResolvedActor,
        vault_id: &VaultId,
    ) {
        for node_id in &call_frame_update.nodes_to_move {
            match node_id {
                RENodeId::Bucket(bucket_id) => {
                    if let Ok(bucket_substate) = heap.get_substate(
                        RENodeId::Bucket(*bucket_id),
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

    fn handle_vault_take<'s, R: FeeReserve>(
        update: &CallFrameUpdate,
        heap: &mut Heap,
        track: &mut Track<'s, R>,
        actor: &ResolvedActor,
        vault_id: &VaultId,
    ) {
        for node_id in &update.nodes_to_move {
            match node_id {
                RENodeId::Bucket(bucket_id) => {
                    if let Ok(bucket_substate) = heap.get_substate(
                        RENodeId::Bucket(*bucket_id),
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

    fn handle_vault_lock_fee<'s, R: FeeReserve>(
        track: &mut Track<'s, R>,
        actor: &ResolvedActor,
        vault_id: &VaultId,
    ) {
        track
            .vault_ops
            .push((actor.clone(), vault_id.clone(), VaultOp::LockFee));
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
        let mut vault_changes = HashMap::<ComponentId, HashMap<VaultId, Decimal>>::new();
        let mut vault_locked_by = HashMap::<VaultId, ComponentId>::new();
        for (actor, vault_id, vault_op) in ops {
            if let Some(resolved_receiver) = actor.receiver {
                if let RENodeId::Component(component_id) = resolved_receiver.receiver {
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
                    };
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
                SubstateOffset::Vault(VaultOffset::Vault),
            ))
            .expect("Failed to find the vault substate");
        substate.vault().resource_address()
    }
}
