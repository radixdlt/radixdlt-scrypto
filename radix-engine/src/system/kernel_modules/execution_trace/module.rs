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

#[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ExecutionTraceModule {
    /// Maximum depth up to which kernel calls are being traced.
    max_kernel_call_depth_traced: usize,

    /// Current kernel calls depth. Note that this doesn't necessarily correspond to the
    /// call frame depth, as there can be nested kernel calls within a single call frame
    /// (e.g. lock_substate call inside drop_node).
    current_kernel_call_depth: usize,

    /// A stack of traced kernel call inputs, their origin, and the instruction index.
    traced_kernel_call_inputs_stack: Vec<(ResourceMovement, KernelCallTraceOrigin, Option<u32>)>,

    /// A mapping of complete KernelCallTrace stacks (\w both inputs and outputs), indexed by depth.
    kernel_call_traces_stacks: HashMap<usize, Vec<KernelCallTrace>>,

    /// Vault operations: (Caller, Vault ID, operation)
    vault_ops: Vec<(ResolvedActor, VaultId, VaultOp)>,
}

impl KernelModuleState for ExecutionTraceModule {
    const ID: u8 = KernelModuleId::ExecutionTrace as u8;
}

#[derive(Clone, Debug, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ProofSnapshot {
    pub resource_address: ResourceAddress,
    pub resource_type: ResourceType,
    pub restricted: bool,
    pub total_locked: LockedAmountOrIds,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceMovement {
    pub buckets: HashMap<BucketId, Resource>,
    pub proofs: HashMap<ProofId, ProofSnapshot>,
}

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct KernelCallTrace {
    pub origin: KernelCallTraceOrigin,
    pub kernel_call_depth: usize,
    pub current_frame_actor: ResolvedActor,
    pub current_frame_depth: usize,
    pub instruction_index: Option<u32>,
    pub input: ResourceMovement,
    pub output: ResourceMovement,
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

//===================================================================================
// Note: execution trace should not produce error or any transactional side-effects!
//===================================================================================

impl ResourceMovement {
    pub fn new_empty() -> Self {
        Self {
            buckets: HashMap::new(),
            proofs: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buckets.is_empty() && self.proofs.is_empty()
    }

    pub fn from_call_frame_update<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        call_frame_update: &CallFrameUpdate,
    ) -> Self {
        let mut buckets = HashMap::new();
        let mut proofs = HashMap::new();
        for node_id in &call_frame_update.nodes_to_move {
            match &node_id {
                RENodeId::Bucket(bucket_id) => {
                    if let Some(x) = api.read_bucket(*bucket_id) {
                        buckets.insert(*bucket_id, x);
                    }
                }
                RENodeId::Proof(proof_id) => {
                    if let Some(x) = api.read_proof(*proof_id) {
                        proofs.insert(*proof_id, x);
                    }
                }
                _ => {}
            }
        }
        Self { buckets, proofs }
    }

    pub fn from_node_id<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        node_id: &RENodeId,
    ) -> Self {
        let mut buckets = HashMap::new();
        let mut proofs = HashMap::new();
        match node_id {
            RENodeId::Bucket(bucket_id) => {
                if let Some(x) = api.read_bucket(*bucket_id) {
                    buckets.insert(*bucket_id, x);
                }
            }
            RENodeId::Proof(proof_id) => {
                if let Some(x) = api.read_proof(*proof_id) {
                    proofs.insert(*proof_id, x);
                }
            }
            _ => {}
        }
        Self { buckets, proofs }
    }
}

impl KernelModule for ExecutionTraceModule {
    fn pre_create_node<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        _node_id: &RENodeId,
        _node_init: &RENodeInit,
        _node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        if let Some(state) = api.get_module_state::<ExecutionTraceModule>() {
            state.handle_pre_create_node();
        }
        Ok(())
    }

    fn post_create_node<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        let current_actor = api.get_current_actor();
        let current_depth = api.get_current_depth();
        let resource_movement = ResourceMovement::from_node_id(api, node_id);
        if let Some(state) = api.get_module_state::<ExecutionTraceModule>() {
            state.handle_post_create_node(current_actor, current_depth, resource_movement);
        }
        Ok(())
    }

    fn pre_drop_node<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        let resource_movement = ResourceMovement::from_node_id(api, node_id);
        if let Some(state) = api.get_module_state::<ExecutionTraceModule>() {
            state.handle_pre_drop_node(resource_movement);
        }
        Ok(())
    }

    fn post_drop_node<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let current_actor = api.get_current_actor();
        let current_depth = api.get_current_depth();
        if let Some(state) = api.get_module_state::<ExecutionTraceModule>() {
            state.handle_post_drop_node(current_actor, current_depth);
        }
        Ok(())
    }

    fn before_create_frame<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        callee: &ResolvedActor,
        update: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        let current_actor = api.get_current_actor();
        let current_depth = api.get_current_depth();
        let resource_movement = ResourceMovement::from_call_frame_update(api, update);
        if let Some(state) = api.get_module_state::<ExecutionTraceModule>() {
            state.handle_before_create_frame(
                current_actor,
                current_depth,
                callee,
                resource_movement,
            );
        }
        Ok(())
    }

    fn after_actor_run<Y: KernelNodeApi + KernelSubstateApi + KernelActorApi<RuntimeError>>(
        api: &mut Y,
        caller: &ResolvedActor,
        update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        let current_actor = api.get_current_actor();
        let current_depth = api.get_current_depth();
        let resource_movement = ResourceMovement::from_call_frame_update(api, update);
        if let Some(state) = api.get_module_state::<ExecutionTraceModule>() {
            state.handle_after_actor_run(current_actor, current_depth, caller, resource_movement);
        }
        Ok(())
    }
}

impl ExecutionTraceModule {
    pub fn new(max_kernel_call_depth_traced: usize) -> ExecutionTraceModule {
        Self {
            max_kernel_call_depth_traced,
            current_kernel_call_depth: 0,
            traced_kernel_call_inputs_stack: vec![],
            kernel_call_traces_stacks: HashMap::new(),
            vault_ops: Vec::new(),
        }
    }

    fn handle_pre_create_node(&mut self) {
        if self.current_kernel_call_depth <= self.max_kernel_call_depth_traced {
            let instruction_index = Self::read_instruction_index(current_frame, heap);

            let traced_input = (
                ResourceMovement::new_empty(),
                KernelCallTraceOrigin::CreateNode,
                instruction_index,
            );
            self.traced_kernel_call_inputs_stack.push(traced_input);
        }

        self.current_kernel_call_depth += 1;
    }

    fn handle_post_create_node(
        &mut self,
        current_actor: ResolvedActor,
        current_depth: usize,
        resource_movement: ResourceMovement,
    ) {
        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth -= 1;

        if self.current_kernel_call_depth > self.max_kernel_call_depth_traced {
            // Nothing to trace at this depth, exit.
            return;
        }

        self.finalize_kernel_call_trace(resource_movement, current_actor, current_depth)
    }

    fn handle_pre_drop_node(&mut self, resource_movement: ResourceMovement) {
        if self.current_kernel_call_depth <= self.max_kernel_call_depth_traced {
            let instruction_index = Self::read_instruction_index(current_frame, heap);

            let traced_input = (
                resource_movement,
                KernelCallTraceOrigin::DropNode,
                instruction_index,
            );
            self.traced_kernel_call_inputs_stack.push(traced_input);
        }

        self.current_kernel_call_depth += 1;
    }

    fn handle_post_drop_node(&mut self, current_actor: ResolvedActor, current_depth: usize) {
        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth -= 1;

        if self.current_kernel_call_depth > self.max_kernel_call_depth_traced {
            // Nothing to trace at this depth, exit.
            return;
        }

        let traced_output = ResourceMovement::new_empty();

        self.finalize_kernel_call_trace(traced_output, current_actor, current_depth)
    }

    fn handle_before_create_frame(
        &mut self,
        current_actor: ResolvedActor,
        current_depth: usize,
        callee: &ResolvedActor,
        resource_movement: ResourceMovement,
    ) -> Result<(), RuntimeError> {
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

            let instruction_index = Self::get_instruction_index(current_frame, heap);

            self.traced_kernel_call_inputs_stack.push((
                resource_movement,
                origin,
                instruction_index,
            ));
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
            } => self.handle_vault_put_input(&resource_movement, &current_actor, vault_id),
            ResolvedActor {
                identifier: FnIdentifier::Native(NativeFn::Vault(VaultFn::LockFee)),
                receiver:
                    Some(ResolvedReceiver {
                        receiver: RENodeId::Vault(vault_id),
                        ..
                    }),
            } => self.handle_vault_lock_fee_input(&current_actor, vault_id),
            _ => {}
        }

        Ok(())
    }

    fn handle_after_actor_run(
        &mut self,
        current_actor: ResolvedActor,
        current_depth: usize,
        return_to: &ResolvedActor,
        resource_movement: ResourceMovement,
    ) {
        match &current_actor {
            ResolvedActor {
                identifier: FnIdentifier::Native(NativeFn::Vault(VaultFn::Take)),
                receiver:
                    Some(ResolvedReceiver {
                        receiver: RENodeId::Vault(vault_id),
                        ..
                    }),
            } => self.handle_vault_take_output(&resource_movement, return_to, vault_id),
            _ => {}
        }

        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth -= 1;

        if self.current_kernel_call_depth > self.max_kernel_call_depth_traced {
            // Nothing to trace at this depth, exit.
            return;
        }

        self.finalize_kernel_call_trace(resource_movement, current_actor, current_depth)
    }

    fn finalize_kernel_call_trace(
        &mut self,
        traced_output: ResourceMovement,
        current_actor: ResolvedActor,
        current_depth: usize,
    ) {
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
                current_frame_actor: current_actor,
                current_frame_depth: current_depth,
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
    }

    pub fn destroy(mut self) -> (Vec<(ResolvedActor, VaultId, VaultOp)>, Vec<TrackedEvent>) {
        let mut events = Vec::new();
        for (_, traces) in self.kernel_call_traces_stacks.drain() {
            // Emit an output event for each "root" kernel call trace
            for trace in traces {
                events.push(TrackedEvent::KernelCallTrace(trace));
            }
        }

        (self.vault_ops, events)
    }

    fn read_instruction_index(current_frame: &CallFrame, heap: &mut Heap) -> Option<u32> {
        if current_frame
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

    fn handle_vault_put_input<'s>(
        &mut self,
        resource_movement: &ResourceMovement,
        caller: &ResolvedActor,
        vault_id: &VaultId,
    ) {
        for (bucket_id, resource) in resource_movement.buckets {
            self.vault_ops.push((
                caller.clone(),
                vault_id.clone(),
                VaultOp::Put(resource.amount()),
            ));
        }
    }

    fn handle_vault_lock_fee_input<'s>(&mut self, caller: &ResolvedActor, vault_id: &VaultId) {
        self.vault_ops
            .push((caller.clone(), vault_id.clone(), VaultOp::LockFee));
    }

    fn handle_vault_take_output<'s>(
        &mut self,
        resource_movement: &ResourceMovement,
        caller: &ResolvedActor,
        vault_id: &VaultId,
    ) {
        for (bucket_id, resource) in resource_movement.buckets {
            self.vault_ops.push((
                caller.clone(),
                vault_id.clone(),
                VaultOp::Take(resource.amount()),
            ));
        }
    }
}

impl ExecutionTraceReceipt {
    // TODO: is it better to derive resource changes from substate diff, instead of execution trace?
    // The current approach relies on various runtime invariants.

    pub fn new(
        ops: Vec<(ResolvedActor, VaultId, VaultOp)>,
        actual_fee_payments: &BTreeMap<VaultId, Decimal>,
        to_persist: &HashMap<SubstateId, (PersistedSubstate, Option<u32>)>,
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
        to_persist: &HashMap<SubstateId, (PersistedSubstate, Option<u32>)>,
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
