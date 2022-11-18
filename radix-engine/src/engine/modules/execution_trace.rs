use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use radix_engine_interface::api::types::{
    BucketOffset, ComponentId, NativeMethod, RENodeId, ScryptoFunctionIdent, ScryptoMethodIdent,
    SubstateId, SubstateOffset, VaultId, VaultMethod, VaultOffset,
};
use radix_engine_interface::data::IndexedScryptoValue;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ExecutionTraceError {
    InvalidState(String),
    CallFrameError(CallFrameError),
}

pub struct ExecutionTraceModule {
    /// Maximum depth up to which sys calls are being traced.
    max_sys_call_trace_depth: usize,

    /// Current sys calls depth. Note that this doesn't necessarily correspond to the
    /// call frame depth, as there can be nested sys calls within a single call frame
    /// (e.g. lock_substate call inside drop_node).
    current_sys_call_depth: usize,

    /// The index of the manifest instruction currently being executed.
    /// None for any sys calls that happen before transaction processor
    /// starts processing instructions.
    current_instruction_index: Option<usize>,

    /// A stack of traced sys call inputs, their origin, and the instruction index.
    traced_sys_call_inputs_stack: Vec<(TracedSysCallData, SysCallTraceOrigin, Option<usize>)>,

    /// A mapping of complete SysCallTrace stacks (\w both inputs and outputs), indexed by depth.
    sys_call_traces_stacks: HashMap<usize, Vec<SysCallTrace>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ProofSnapshot {
    pub resource_address: ResourceAddress,
    pub resource_type: ResourceType,
    pub restricted: bool,
    pub total_locked: LockedAmountOrIds,
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
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

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct SysCallTrace {
    pub origin: SysCallTraceOrigin,
    pub sys_call_depth: usize,
    pub call_frame_actor: REActor,
    pub call_frame_depth: usize,
    pub instruction_index: Option<usize>,
    pub input: TracedSysCallData,
    pub output: TracedSysCallData,
    pub children: Vec<SysCallTrace>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum SysCallTraceOrigin {
    ScryptoFunction(ScryptoFunctionIdent),
    ScryptoMethod(ScryptoMethodIdent),
    NativeFunction(NativeFunction),
    NativeMethod(NativeMethod),
    CreateNode,
    DropNode,
    /// Anything else that isn't traced on its own, but the trace exists for its children
    Opaque,
}

pub trait Traceable: Debug {
    fn buckets(&self) -> Vec<BucketId>;
    fn proofs(&self) -> Vec<ProofId>;
}

impl Traceable for IndexedScryptoValue {
    fn buckets(&self) -> Vec<BucketId> {
        self.bucket_ids.keys().map(|k| k.clone()).collect()
    }

    fn proofs(&self) -> Vec<ProofId> {
        self.proof_ids.keys().map(|k| k.clone()).collect()
    }
}

impl Traceable for Bucket {
    fn buckets(&self) -> Vec<BucketId> {
        vec![self.0]
    }

    fn proofs(&self) -> Vec<ProofId> {
        vec![]
    }
}

impl Traceable for Proof {
    fn buckets(&self) -> Vec<BucketId> {
        vec![]
    }

    fn proofs(&self) -> Vec<ProofId> {
        vec![self.0]
    }
}

impl Traceable for (ResourceAddress, Option<Bucket>) {
    fn buckets(&self) -> Vec<BucketId> {
        match &self.1 {
            Some(bucket) => vec![bucket.0.clone()],
            None => vec![],
        }
    }

    fn proofs(&self) -> Vec<ProofId> {
        vec![]
    }
}

impl Traceable for CallFrameUpdate {
    fn buckets(&self) -> Vec<BucketId> {
        self.nodes_to_move
            .iter()
            .filter_map(|node_id| match node_id {
                RENodeId::Bucket(bucket_id) => Some(bucket_id.clone()),
                _ => None,
            })
            .collect()
    }

    fn proofs(&self) -> Vec<ProofId> {
        self.nodes_to_move
            .iter()
            .filter_map(|node_id| match node_id {
                RENodeId::Proof(proof_id) => Some(proof_id.clone()),
                _ => None,
            })
            .collect()
    }
}

impl<A: Traceable> Traceable for Vec<A> {
    fn buckets(&self) -> Vec<BucketId> {
        self.into_iter().flat_map(|t| t.buckets()).collect()
    }

    fn proofs(&self) -> Vec<ProofId> {
        self.into_iter().flat_map(|t| t.proofs()).collect()
    }
}

macro_rules! impl_empty_traceable {
    (for $($t:ty),+) => {
        $(impl Traceable for $t {
            fn buckets(&self) -> Vec<BucketId> {
                vec![]
            }
            fn proofs(&self) -> Vec<ProofId> {
                vec![]
            }
        })*
    }
}

// Remaining Traceable implementations for invocations that don't involve proofs/buckets
impl_empty_traceable!(for
    (),
    bool,
    u8,
    u64,
    [Vec<u8>; 2],
    (String, String),
    HashMap<String, String>,
    Decimal,
    Vault,
    VaultId,
    ResourceType,
    BTreeSet<NonFungibleId>,
    PackageAddress,
    ResourceAddress,
    SystemAddress
);

impl<R: FeeReserve> Module<R> for ExecutionTraceModule {
    fn pre_sys_call(
        &mut self,
        _call_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track<R>,
        input: SysCallInput,
    ) -> Result<(), ModuleError> {
        self.handle_pre_sys_call(heap, input)
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

    fn on_run(
        &mut self,
        actor: &REActor,
        input: &IndexedScryptoValue,
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        Self::trace_run(call_frame, heap, track, actor, input);
        Ok(())
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

    fn on_finished_processing(
        &mut self,
        _heap: &mut Heap,
        track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        self.handle_processing_completed(track)
    }
}

impl ExecutionTraceModule {
    pub fn new(max_sys_call_trace_depth: usize) -> ExecutionTraceModule {
        Self {
            max_sys_call_trace_depth,
            current_sys_call_depth: 0,
            current_instruction_index: None,
            traced_sys_call_inputs_stack: vec![],
            sys_call_traces_stacks: HashMap::new(),
        }
    }

    fn handle_pre_sys_call(
        &mut self,
        heap: &mut Heap,
        input: SysCallInput,
    ) -> Result<(), ModuleError> {
        if self.current_sys_call_depth > self.max_sys_call_trace_depth {
            // It's important to update the depth counter,
            // even if we don't trace at this depth anymore.
            self.current_sys_call_depth += 1;
            return Ok(());
        }

        // Handle transaction processor events
        match input {
            SysCallInput::EmitEvent {
                event:
                    Event::Runtime(RuntimeEvent::PreExecuteInstruction {
                        instruction_index, ..
                    }),
            } => {
                self.current_instruction_index = Some(instruction_index.clone());
            }
            SysCallInput::EmitEvent {
                event: Event::Runtime(RuntimeEvent::PostExecuteManifest),
            } => {
                self.current_instruction_index = None;
            }
            _ => {}
        };

        let traced_input = match input {
            SysCallInput::Invoke { info, .. } => {
                let (origin, trace_data) = match info {
                    InvocationInfo::Scrypto(ScryptoInvocation::Function(fn_ident, value)) => (
                        SysCallTraceOrigin::ScryptoFunction(fn_ident.clone()),
                        Self::extract_trace_data(heap, value)?,
                    ),
                    InvocationInfo::Scrypto(ScryptoInvocation::Method(method_ident, value)) => (
                        SysCallTraceOrigin::ScryptoMethod(method_ident.clone()),
                        Self::extract_trace_data(heap, value)?,
                    ),
                    InvocationInfo::Native(NativeInvocationInfo::Function(native_fn, value)) => (
                        SysCallTraceOrigin::NativeFunction(native_fn.clone()),
                        Self::extract_trace_data(heap, value)?,
                    ),
                    InvocationInfo::Native(NativeInvocationInfo::Method(
                        native_method,
                        _,
                        value,
                    )) => (
                        SysCallTraceOrigin::NativeMethod(native_method.clone()),
                        Self::extract_trace_data(heap, value)?,
                    ),
                };
                (trace_data, origin, self.current_instruction_index)
            }
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

                (
                    data,
                    SysCallTraceOrigin::DropNode,
                    self.current_instruction_index,
                )
            }
            SysCallInput::CreateNode { .. } => (
                TracedSysCallData::new_empty(),
                SysCallTraceOrigin::CreateNode,
                self.current_instruction_index,
            ),
            _ => (
                TracedSysCallData::new_empty(),
                SysCallTraceOrigin::Opaque,
                self.current_instruction_index,
            ),
        };

        self.traced_sys_call_inputs_stack.push(traced_input);
        self.current_sys_call_depth += 1;
        Ok(())
    }

    fn handle_post_sys_call(
        &mut self,
        call_frame: &CallFrame,
        heap: &mut Heap,
        output: SysCallOutput,
    ) -> Result<(), ModuleError> {
        // Important to always update the counter (even if we're over the depth limit).
        self.current_sys_call_depth -= 1;

        if self.current_sys_call_depth > self.max_sys_call_trace_depth {
            // Nothing to trace at this depth, exit.
            return Ok(());
        }

        let child_traces = self
            .sys_call_traces_stacks
            .remove(&(self.current_sys_call_depth + 1))
            .unwrap_or(vec![]);

        let (traced_input, origin, instruction_index) = self
            .traced_sys_call_inputs_stack
            .pop()
            .expect("Sys call input stack underflow");

        let traced_output = match output {
            SysCallOutput::Invoke { rtn } => Self::extract_trace_data(heap, rtn)?,
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

    fn handle_processing_completed<R: FeeReserve>(
        &mut self,
        track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        // Some sanity checks
        // No leftover inputs
        if !self.traced_sys_call_inputs_stack.is_empty() {
            return Err(ModuleError::ExecutionTraceError(
                ExecutionTraceError::InvalidState("Leftover sys call inputs on stack".to_string()),
            ));
        }
        // At most one entry in call traces mapping (the root level)
        if self.sys_call_traces_stacks.len() > 1 {
            return Err(ModuleError::ExecutionTraceError(
                ExecutionTraceError::InvalidState("Leftover sys call traces on stack".to_string()),
            ));
        }

        for (_, traces) in self.sys_call_traces_stacks.drain() {
            // Emit an output event for each "root" sys call trace
            for trace in traces {
                track.add_event(TrackedEvent::Native(NativeEvent::SysCallTrace(trace)));
            }
        }

        Ok(())
    }

    fn extract_trace_data(
        heap: &mut Heap,
        traceable: &dyn Traceable,
    ) -> Result<TracedSysCallData, ModuleError> {
        let mut buckets: HashMap<BucketId, Resource> = HashMap::new();
        for bucket_id in traceable.buckets() {
            let bucket_resource = Self::read_bucket_resource(heap, &bucket_id)?;
            buckets.insert(bucket_id, bucket_resource);
        }

        let mut proofs: HashMap<ProofId, ProofSnapshot> = HashMap::new();
        for proof_id in traceable.proofs() {
            let proof = Self::read_proof(heap, &proof_id)?;
            proofs.insert(proof_id, proof);
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

    fn trace_run<'s, R: FeeReserve>(
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<'s, R>,
        actor: &REActor,
        input: &IndexedScryptoValue,
    ) {
        if let REActor::Method(ResolvedMethod::Native(native_method), resolved_receiver) = actor {
            let caller = &call_frame.actor;

            match (native_method, resolved_receiver.receiver) {
                (NativeMethod::Vault(VaultMethod::Put), RENodeId::Vault(vault_id)) => {
                    Self::handle_vault_put(heap, track, caller, &vault_id, input)
                }
                (NativeMethod::Vault(VaultMethod::Take), RENodeId::Vault(vault_id)) => {
                    Self::handle_vault_take(track, caller, &vault_id, input)
                }
                (NativeMethod::Vault(VaultMethod::LockFee), RENodeId::Vault(vault_id)) => {
                    Self::handle_vault_lock_fee(track, caller, &vault_id)
                }
                _ => {}
            }
        }
    }

    fn handle_vault_put<'s, R: FeeReserve>(
        heap: &mut Heap,
        track: &mut Track<'s, R>,
        actor: &REActor,
        vault_id: &VaultId,
        input: &IndexedScryptoValue,
    ) {
        if let Ok(call_data) = scrypto_decode::<VaultPutInvocation>(&input.raw) {
            let bucket_id = call_data.bucket.0;
            if let Ok(bucket_substate) = heap.get_substate(
                RENodeId::Bucket(bucket_id),
                &SubstateOffset::Bucket(BucketOffset::Bucket),
            ) {
                track.vault_ops.push((
                    actor.clone(),
                    vault_id.clone(),
                    VaultOp::Put(bucket_substate.bucket().total_amount()),
                ));
            }
        }
    }

    fn handle_vault_take<'s, R: FeeReserve>(
        track: &mut Track<'s, R>,
        actor: &REActor,
        vault_id: &VaultId,
        input: &IndexedScryptoValue,
    ) {
        if let Ok(call_data) = scrypto_decode::<VaultTakeInvocation>(&input.raw) {
            track.vault_ops.push((
                actor.clone(),
                vault_id.clone(),
                VaultOp::Take(call_data.amount),
            ));
        }
    }

    fn handle_vault_lock_fee<'s, R: FeeReserve>(
        track: &mut Track<'s, R>,
        actor: &REActor,
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
        ops: Vec<(REActor, VaultId, VaultOp)>,
        actual_fee_payments: HashMap<VaultId, Decimal>,
        to_persist: &mut HashMap<SubstateId, (PersistedSubstate, Option<u32>)>,
        is_commit_success: bool,
    ) -> Self {
        let mut vault_changes = HashMap::<ComponentId, HashMap<VaultId, Decimal>>::new();
        let mut vault_locked_by = HashMap::<VaultId, ComponentId>::new();
        for (actor, vault_id, vault_op) in ops {
            if let REActor::Method(_, resolved_receiver) = actor {
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
