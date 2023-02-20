use crate::errors::*;
use crate::kernel::actor::{ResolvedActor, ResolvedReceiver};
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::event::TrackedEvent;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_substates::PersistedSubstate;
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::*;
use sbor::rust::fmt::Debug;

//===================================================================================
// Note: ExecutionTrace must not produce any error or transactional side effect!
//===================================================================================

#[derive(Debug, Clone)]
pub struct ExecutionTraceModule {
    /// Maximum depth up to which kernel calls are being traced.
    max_kernel_call_depth_traced: usize,

    /// Current transaction index
    current_transaction_index: usize,

    /// Current kernel calls depth. Note that this doesn't necessarily correspond to the
    /// call frame depth, as there can be nested kernel calls within a single call frame
    /// (e.g. lock_substate call inside drop_node).
    current_kernel_call_depth: usize,

    /// A stack of traced kernel call inputs, their origin, and the instruction index.
    traced_kernel_call_inputs_stack: Vec<(ResourceSummary, KernelCallOrigin, usize)>,

    /// A mapping of complete KernelCallTrace stacks (\w both inputs and outputs), indexed by depth.
    kernel_call_traces_stacks: HashMap<usize, Vec<KernelCallTrace>>,

    /// Vault operations: (Caller, Vault ID, operation)
    vault_ops: Vec<(ResolvedActor, VaultId, VaultOp)>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
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

#[derive(Debug, Clone)]
pub enum VaultOp {
    Create(Decimal), // TODO: add trace of vault creation
    Put(Decimal),    // TODO: add non-fungible support
    Take(Decimal),
    LockFee,
}

#[derive(Clone, Debug, PartialEq, Eq, ScryptoSbor)]
pub enum BucketSnapshot {
    Fungible {
        resource_address: ResourceAddress,
        restricted: bool,
        liquid_amount: Decimal,
    },
    NonFungible {
        resource_address: ResourceAddress,
        restricted: bool,
        liquid_ids: BTreeSet<NonFungibleLocalId>,
    },
}

impl BucketSnapshot {
    pub fn amount(&self) -> Decimal {
        match self {
            BucketSnapshot::Fungible {
                resource_address,
                restricted,
                liquid_amount,
            } => liquid_amount.clone(),
            BucketSnapshot::NonFungible {
                resource_address,
                restricted,
                liquid_ids,
            } => liquid_ids.len().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, ScryptoSbor)]
pub enum ProofSnapshot {
    Fungible {
        resource_address: ResourceAddress,
        restricted: bool,
        total_locked: Decimal,
    },
    NonFungible {
        resource_address: ResourceAddress,
        restricted: bool,
        total_locked: BTreeSet<NonFungibleLocalId>,
    },
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct ResourceSummary {
    pub buckets: HashMap<BucketId, BucketSnapshot>,
    pub proofs: HashMap<ProofId, ProofSnapshot>,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct KernelCallTrace {
    pub origin: KernelCallOrigin,
    pub kernel_call_depth: usize,
    pub current_frame_actor: ResolvedActor,
    pub current_frame_depth: usize,
    pub instruction_index: usize,
    pub input: ResourceSummary,
    pub output: ResourceSummary,
    pub children: Vec<KernelCallTrace>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum KernelCallOrigin {
    ScryptoFunction(ScryptoFnIdentifier),
    ScryptoMethod(ScryptoFnIdentifier),
    NativeFn(NativeFn),
    CreateNode,
    DropNode,
}

impl ResourceSummary {
    pub fn new_empty() -> Self {
        Self {
            buckets: HashMap::new(),
            proofs: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buckets.is_empty() && self.proofs.is_empty()
    }

    pub fn from_call_frame_update<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        call_frame_update: &CallFrameUpdate,
    ) -> Self {
        let mut buckets = HashMap::new();
        let mut proofs = HashMap::new();
        for node_id in &call_frame_update.nodes_to_move {
            match &node_id {
                RENodeId::Bucket(bucket_id) => {
                    if let Some(x) = api.kernel_read_bucket(*bucket_id) {
                        buckets.insert(*bucket_id, x);
                    }
                }
                RENodeId::Proof(proof_id) => {
                    if let Some(x) = api.kernel_read_proof(*proof_id) {
                        proofs.insert(*proof_id, x);
                    }
                }
                _ => {}
            }
        }
        Self { buckets, proofs }
    }

    pub fn from_node_id<Y: KernelModuleApi<RuntimeError>>(api: &mut Y, node_id: &RENodeId) -> Self {
        let mut buckets = HashMap::new();
        let mut proofs = HashMap::new();
        match node_id {
            RENodeId::Bucket(bucket_id) => {
                if let Some(x) = api.kernel_read_bucket(*bucket_id) {
                    buckets.insert(*bucket_id, x);
                }
            }
            RENodeId::Proof(proof_id) => {
                if let Some(x) = api.kernel_read_proof(*proof_id) {
                    proofs.insert(*proof_id, x);
                }
            }
            _ => {}
        }
        Self { buckets, proofs }
    }
}

impl KernelModule for ExecutionTraceModule {
    fn before_create_node<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _node_id: &RENodeId,
        _node_init: &RENodeInit,
        _node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_module_state()
            .execution_trace
            .handle_before_create_node();
        Ok(())
    }

    fn after_create_node<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        let current_actor = api.kernel_get_current_actor();
        let current_depth = api.kernel_get_current_depth();
        let resource_summary = ResourceSummary::from_node_id(api, node_id);
        api.kernel_get_module_state()
            .execution_trace
            .handle_after_create_node(current_actor, current_depth, resource_summary);
        Ok(())
    }

    fn before_drop_node<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        node_id: &RENodeId,
    ) -> Result<(), RuntimeError> {
        let resource_summary = ResourceSummary::from_node_id(api, node_id);
        api.kernel_get_module_state()
            .execution_trace
            .handle_before_drop_node(resource_summary);
        Ok(())
    }

    fn after_drop_node<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let current_actor = api.kernel_get_current_actor();
        let current_depth = api.kernel_get_current_depth();
        api.kernel_get_module_state()
            .execution_trace
            .handle_after_drop_node(current_actor, current_depth);
        Ok(())
    }

    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        callee: &ResolvedActor,
        update: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        let current_actor = api.kernel_get_current_actor();
        let resource_summary = ResourceSummary::from_call_frame_update(api, update);
        api.kernel_get_module_state()
            .execution_trace
            .handle_before_push_frame(current_actor, callee, resource_summary);
        Ok(())
    }

    fn on_execution_finish<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        caller: &ResolvedActor,
        update: &CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        let current_actor = api.kernel_get_current_actor();
        let current_depth = api.kernel_get_current_depth();
        let resource_summary = ResourceSummary::from_call_frame_update(api, update);
        api.kernel_get_module_state()
            .execution_trace
            .handle_on_execution_finish(current_actor, current_depth, caller, resource_summary);
        Ok(())
    }

    fn on_update_instruction_index<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        new_index: usize,
    ) -> Result<(), RuntimeError> {
        api.kernel_get_module_state()
            .execution_trace
            .current_transaction_index = new_index;
        Ok(())
    }
}

impl ExecutionTraceModule {
    pub fn new(max_kernel_call_depth_traced: usize) -> ExecutionTraceModule {
        Self {
            max_kernel_call_depth_traced,
            current_transaction_index: 0,
            current_kernel_call_depth: 0,
            traced_kernel_call_inputs_stack: vec![],
            kernel_call_traces_stacks: HashMap::new(),
            vault_ops: Vec::new(),
        }
    }

    fn handle_before_create_node(&mut self) {
        if self.current_kernel_call_depth <= self.max_kernel_call_depth_traced {
            let instruction_index = self.instruction_index();

            let traced_input = (
                ResourceSummary::new_empty(),
                KernelCallOrigin::CreateNode,
                instruction_index,
            );
            self.traced_kernel_call_inputs_stack.push(traced_input);
        }

        self.current_kernel_call_depth += 1;
    }

    fn handle_after_create_node(
        &mut self,
        current_actor: ResolvedActor,
        current_depth: usize,
        resource_summary: ResourceSummary,
    ) {
        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth -= 1;

        if self.current_kernel_call_depth > self.max_kernel_call_depth_traced {
            // Nothing to trace at this depth, exit.
            return;
        }

        self.finalize_kernel_call_trace(resource_summary, current_actor, current_depth)
    }

    fn handle_before_drop_node(&mut self, resource_summary: ResourceSummary) {
        if self.current_kernel_call_depth <= self.max_kernel_call_depth_traced {
            let instruction_index = self.instruction_index();

            let traced_input = (
                resource_summary,
                KernelCallOrigin::DropNode,
                instruction_index,
            );
            self.traced_kernel_call_inputs_stack.push(traced_input);
        }

        self.current_kernel_call_depth += 1;
    }

    fn handle_after_drop_node(&mut self, current_actor: ResolvedActor, current_depth: usize) {
        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth -= 1;

        if self.current_kernel_call_depth > self.max_kernel_call_depth_traced {
            // Nothing to trace at this depth, exit.
            return;
        }

        let traced_output = ResourceSummary::new_empty();

        self.finalize_kernel_call_trace(traced_output, current_actor, current_depth)
    }

    fn handle_before_push_frame(
        &mut self,
        current_actor: ResolvedActor,
        callee: &ResolvedActor,
        resource_summary: ResourceSummary,
    ) {
        if self.current_kernel_call_depth <= self.max_kernel_call_depth_traced {
            let origin = match &callee.identifier {
                FnIdentifier::Scrypto(scrypto_fn) => {
                    if callee.receiver.is_some() {
                        KernelCallOrigin::ScryptoMethod(scrypto_fn.clone())
                    } else {
                        KernelCallOrigin::ScryptoFunction(scrypto_fn.clone())
                    }
                }
                FnIdentifier::Native(native_fn) => KernelCallOrigin::NativeFn(native_fn.clone()),
            };

            let instruction_index = self.instruction_index();

            self.traced_kernel_call_inputs_stack.push((
                resource_summary.clone(),
                origin,
                instruction_index,
            ));
        }

        self.current_kernel_call_depth += 1;

        match &callee {
            ResolvedActor {
                identifier:
                    FnIdentifier::Scrypto(ScryptoFnIdentifier {
                        package_address,
                        blueprint_name,
                        ident,
                    }),
                receiver:
                    Some(ResolvedReceiver {
                        receiver: RENodeId::Vault(vault_id),
                        ..
                    }),
            } if package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                && blueprint_name.eq(VAULT_BLUEPRINT)
                && ident.eq(VAULT_PUT_IDENT) =>
            {
                self.handle_vault_put_input(&resource_summary, &current_actor, vault_id)
            }
            ResolvedActor {
                identifier:
                    FnIdentifier::Scrypto(ScryptoFnIdentifier {
                        package_address,
                        blueprint_name,
                        ident,
                    }),
                receiver:
                    Some(ResolvedReceiver {
                        receiver: RENodeId::Vault(vault_id),
                        ..
                    }),
            } if package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                && blueprint_name.eq(VAULT_BLUEPRINT)
                && ident.eq(VAULT_LOCK_FEE_IDENT) =>
            {
                self.handle_vault_lock_fee_input(&current_actor, vault_id)
            }
            _ => {}
        }
    }

    fn handle_on_execution_finish(
        &mut self,
        current_actor: ResolvedActor,
        current_depth: usize,
        caller: &ResolvedActor,
        resource_summary: ResourceSummary,
    ) {
        match &current_actor {
            ResolvedActor {
                identifier:
                    FnIdentifier::Scrypto(ScryptoFnIdentifier {
                        package_address,
                        blueprint_name,
                        ident,
                    }),
                receiver:
                    Some(ResolvedReceiver {
                        receiver: RENodeId::Vault(vault_id),
                        ..
                    }),
            } if package_address.eq(&RESOURCE_MANAGER_PACKAGE)
                && blueprint_name.eq(VAULT_BLUEPRINT)
                && ident.eq(VAULT_TAKE_IDENT) =>
            {
                self.handle_vault_take_output(&resource_summary, caller, vault_id)
            }
            _ => {}
        }

        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth -= 1;

        if self.current_kernel_call_depth > self.max_kernel_call_depth_traced {
            // Nothing to trace at this depth, exit.
            return;
        }

        self.finalize_kernel_call_trace(resource_summary, current_actor, current_depth)
    }

    fn finalize_kernel_call_trace(
        &mut self,
        traced_output: ResourceSummary,
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

    pub fn collect_events(mut self) -> (Vec<(ResolvedActor, VaultId, VaultOp)>, Vec<TrackedEvent>) {
        let mut events = Vec::new();
        for (_, traces) in self.kernel_call_traces_stacks.drain() {
            // Emit an output event for each "root" kernel call trace
            for trace in traces {
                events.push(TrackedEvent::KernelCallTrace(trace));
            }
        }

        (self.vault_ops, events)
    }

    fn instruction_index(&self) -> usize {
        self.current_transaction_index
    }

    fn handle_vault_put_input<'s>(
        &mut self,
        resource_summary: &ResourceSummary,
        caller: &ResolvedActor,
        vault_id: &VaultId,
    ) {
        for (_, resource) in &resource_summary.buckets {
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
        resource_summary: &ResourceSummary,
        caller: &ResolvedActor,
        vault_id: &VaultId,
    ) {
        for (_, resource) in &resource_summary.buckets {
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
