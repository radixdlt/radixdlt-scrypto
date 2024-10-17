use crate::blueprints::resource::*;
use crate::errors::*;
use crate::internal_prelude::*;
use crate::kernel::call_frame::CallFrameMessage;
use crate::kernel::kernel_api::*;
use crate::kernel::kernel_callback_api::*;
use crate::system::actor::{Actor, FunctionActor, MethodActor};
use crate::system::module::*;
use crate::system::system_callback::*;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::system::type_info::TypeInfoSubstate;
use crate::transaction::{FeeLocks, TransactionExecutionTrace};
use radix_common::math::Decimal;
use radix_engine_interface::blueprints::resource::*;
use sbor::rust::collections::*;
use sbor::rust::fmt::Debug;

//===================================================================================
// Note: ExecutionTrace must not produce any error or transactional side effect!
//===================================================================================

// TODO: Handle potential Decimal arithmetic operation (checked_add, checked_sub) errors instead of panicking.
// ATM, ExecutionTrace cannot return any errors (as stated above), so it shall be thoroughly
// designed.

#[derive(Debug, Clone)]
pub struct ExecutionTraceModule {
    /// Maximum depth up to which kernel calls are being traced.
    max_kernel_call_depth_traced: usize,

    /// Current transaction index
    current_instruction_index: usize,

    /// Current kernel calls depth. Note that this doesn't necessarily correspond to the
    /// call frame depth, as there can be nested kernel calls within a single call frame
    /// (e.g. open_substate call inside drop_node).
    current_kernel_call_depth: usize,

    /// A stack of traced kernel call inputs, their origin, and the instruction index.
    traced_kernel_call_inputs_stack: Vec<(ResourceSummary, TraceOrigin, usize)>,

    /// A mapping of complete KernelCallTrace stacks (\w both inputs and outputs), indexed by depth.
    kernel_call_traces_stacks: IndexMap<usize, Vec<ExecutionTrace>>,

    /// Vault operations: (Caller, Vault ID, operation, instruction index)
    vault_ops: Vec<(TraceActor, NodeId, VaultOp, usize)>,
}

impl ExecutionTraceModule {
    pub fn update_instruction_index(&mut self, new_index: usize) {
        self.current_instruction_index = new_index;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ResourceChange {
    pub node_id: NodeId,
    pub vault_id: NodeId,
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum WorktopChange {
    Take(ResourceSpecifier),
    Put(ResourceSpecifier),
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ResourceSpecifier {
    Amount(ResourceAddress, Decimal),
    Ids(ResourceAddress, IndexSet<NonFungibleLocalId>),
}

impl From<&BucketSnapshot> for ResourceSpecifier {
    fn from(value: &BucketSnapshot) -> Self {
        match value {
            BucketSnapshot::Fungible {
                resource_address,
                liquid,
                ..
            } => Self::Amount(*resource_address, *liquid),
            BucketSnapshot::NonFungible {
                resource_address,
                liquid,
                ..
            } => Self::Ids(*resource_address, liquid.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum VaultOp {
    Put(ResourceAddress, Decimal), // TODO: add non-fungible support
    Take(ResourceAddress, Decimal),
    TakeAdvanced(ResourceAddress, Decimal),
    Recall(ResourceAddress, Decimal),
    LockFee(Decimal, bool),
}

trait SystemModuleApiResourceSnapshotExtension {
    fn read_bucket_uncosted(&self, bucket_id: &NodeId) -> Option<BucketSnapshot>;
    fn read_proof_uncosted(&self, proof_id: &NodeId) -> Option<ProofSnapshot>;
}

impl<'a, V: SystemCallbackObject, K: KernelInternalApi<System = System<V>>>
    SystemModuleApiResourceSnapshotExtension for SystemModuleApiImpl<'a, K>
{
    fn read_bucket_uncosted(&self, bucket_id: &NodeId) -> Option<BucketSnapshot> {
        let (is_fungible_bucket, resource_address) = if let Some(substate) =
            self.api_ref().kernel_read_substate_uncosted(
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
                        ResourceAddress::new_or_panic(parent.as_bytes().try_into().unwrap());
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
                .api_ref()
                .kernel_read_substate_uncosted(
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
                .api_ref()
                .kernel_read_substate_uncosted(
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

    fn read_proof_uncosted(&self, proof_id: &NodeId) -> Option<ProofSnapshot> {
        let is_fungible = if let Some(substate) = self.api_ref().kernel_read_substate_uncosted(
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
                .api_ref()
                .kernel_read_substate_uncosted(
                    proof_id,
                    TYPE_INFO_FIELD_PARTITION,
                    &TypeInfoField::TypeInfo.into(),
                )
                .unwrap();
            let info: TypeInfoSubstate = substate.as_typed().unwrap();
            let resource_address =
                ResourceAddress::new_or_panic(info.outer_object().unwrap().into());

            let substate = self
                .api_ref()
                .kernel_read_substate_uncosted(
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
                .api_ref()
                .kernel_read_substate_uncosted(
                    proof_id,
                    TYPE_INFO_FIELD_PARTITION,
                    &TypeInfoField::TypeInfo.into(),
                )
                .unwrap();
            let info: TypeInfoSubstate = substate.as_typed().unwrap();
            let resource_address =
                ResourceAddress::new_or_panic(info.outer_object().unwrap().into());

            let substate = self
                .api_ref()
                .kernel_read_substate_uncosted(
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

#[derive(Clone, Debug, PartialEq, Eq, ScryptoSbor)]
pub enum BucketSnapshot {
    Fungible {
        resource_address: ResourceAddress,
        liquid: Decimal,
    },
    NonFungible {
        resource_address: ResourceAddress,
        liquid: IndexSet<NonFungibleLocalId>,
    },
}

impl BucketSnapshot {
    pub fn resource_address(&self) -> ResourceAddress {
        match self {
            BucketSnapshot::Fungible {
                resource_address, ..
            } => resource_address.clone(),
            BucketSnapshot::NonFungible {
                resource_address, ..
            } => resource_address.clone(),
        }
    }
    pub fn amount(&self) -> Decimal {
        match self {
            BucketSnapshot::Fungible { liquid, .. } => liquid.clone(),
            BucketSnapshot::NonFungible { liquid, .. } => liquid.len().into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, ScryptoSbor)]
pub enum ProofSnapshot {
    Fungible {
        resource_address: ResourceAddress,
        total_locked: Decimal,
    },
    NonFungible {
        resource_address: ResourceAddress,
        total_locked: IndexSet<NonFungibleLocalId>,
    },
}

impl ProofSnapshot {
    pub fn resource_address(&self) -> ResourceAddress {
        match self {
            ProofSnapshot::Fungible {
                resource_address, ..
            } => resource_address.clone(),
            ProofSnapshot::NonFungible {
                resource_address, ..
            } => resource_address.clone(),
        }
    }
    pub fn amount(&self) -> Decimal {
        match self {
            ProofSnapshot::Fungible { total_locked, .. } => total_locked.clone(),
            ProofSnapshot::NonFungible { total_locked, .. } => total_locked.len().into(),
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct ResourceSummary {
    pub buckets: IndexMap<NodeId, BucketSnapshot>,
    pub proofs: IndexMap<NodeId, ProofSnapshot>,
}

// TODO: Clean up
#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub enum TraceActor {
    Method(NodeId),
    NonMethod,
}

impl TraceActor {
    pub fn from_actor(actor: &Actor) -> TraceActor {
        match actor {
            Actor::Method(MethodActor { node_id, .. }) => TraceActor::Method(node_id.clone()),
            _ => TraceActor::NonMethod,
        }
    }
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq)]
pub struct ExecutionTrace {
    pub origin: TraceOrigin,
    pub kernel_call_depth: usize,
    pub current_frame_actor: TraceActor,
    pub current_frame_depth: usize,
    pub instruction_index: usize,
    pub input: ResourceSummary,
    pub output: ResourceSummary,
    pub children: Vec<ExecutionTrace>,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ApplicationFnIdentifier {
    pub blueprint_id: BlueprintId,
    pub ident: String,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum TraceOrigin {
    ScryptoFunction(ApplicationFnIdentifier),
    ScryptoMethod(ApplicationFnIdentifier),
    CreateNode,
    DropNode,
}

impl ExecutionTrace {
    pub fn worktop_changes(
        &self,
        worktop_changes_aggregator: &mut IndexMap<usize, Vec<WorktopChange>>,
    ) {
        if let TraceOrigin::ScryptoMethod(fn_identifier) = &self.origin {
            if fn_identifier.blueprint_id == BlueprintId::new(&RESOURCE_PACKAGE, WORKTOP_BLUEPRINT)
            {
                if fn_identifier.ident == WORKTOP_PUT_IDENT {
                    for (_, bucket_snapshot) in self.input.buckets.iter() {
                        worktop_changes_aggregator
                            .entry(self.instruction_index)
                            .or_default()
                            .push(WorktopChange::Put(bucket_snapshot.into()))
                    }
                } else if fn_identifier.ident == WORKTOP_TAKE_IDENT
                    || fn_identifier.ident == WORKTOP_TAKE_ALL_IDENT
                    || fn_identifier.ident == WORKTOP_TAKE_NON_FUNGIBLES_IDENT
                    || fn_identifier.ident == WORKTOP_DRAIN_IDENT
                {
                    for (_, bucket_snapshot) in self.output.buckets.iter() {
                        worktop_changes_aggregator
                            .entry(self.instruction_index)
                            .or_default()
                            .push(WorktopChange::Take(bucket_snapshot.into()))
                    }
                }
            }
        }

        // Aggregate the worktop changes for all children traces
        for child in self.children.iter() {
            child.worktop_changes(worktop_changes_aggregator)
        }
    }
}

impl ResourceSummary {
    pub fn default() -> Self {
        Self {
            buckets: index_map_new(),
            proofs: index_map_new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buckets.is_empty() && self.proofs.is_empty()
    }

    fn from_message(
        api: &mut impl SystemModuleApiResourceSnapshotExtension,
        message: &CallFrameMessage,
    ) -> Self {
        let mut buckets = index_map_new();
        let mut proofs = index_map_new();
        for node_id in &message.move_nodes {
            if let Some(x) = api.read_bucket_uncosted(node_id) {
                buckets.insert(*node_id, x);
            }
            if let Some(x) = api.read_proof_uncosted(node_id) {
                proofs.insert(*node_id, x);
            }
        }
        Self { buckets, proofs }
    }

    fn from_node_id(
        api: &mut impl SystemModuleApiResourceSnapshotExtension,
        node_id: &NodeId,
    ) -> Self {
        let mut buckets = index_map_new();
        let mut proofs = index_map_new();
        if let Some(x) = api.read_bucket_uncosted(node_id) {
            buckets.insert(*node_id, x);
        }
        if let Some(x) = api.read_proof_uncosted(node_id) {
            proofs.insert(*node_id, x);
        }
        Self { buckets, proofs }
    }
}

impl InitSystemModule for ExecutionTraceModule {}
impl ResolvableSystemModule for ExecutionTraceModule {
    #[inline]
    fn resolve_from_system(system: &mut impl HasModules) -> &mut Self {
        &mut system.modules_mut().execution_trace
    }
}
impl PrivilegedSystemModule for ExecutionTraceModule {}

impl<ModuleApi: SystemModuleApiFor<Self> + SystemModuleApiResourceSnapshotExtension>
    SystemModule<ModuleApi> for ExecutionTraceModule
{
    fn on_create_node(api: &mut ModuleApi, event: &CreateNodeEvent) -> Result<(), RuntimeError> {
        if api.current_stack_id_uncosted() != 0 {
            return Ok(());
        }
        match event {
            CreateNodeEvent::Start(..) => {
                api.module().handle_before_create_node();
            }
            CreateNodeEvent::IOAccess(..) => {}
            CreateNodeEvent::End(node_id) => {
                let current_depth = api.current_stack_depth_uncosted();
                let resource_summary = ResourceSummary::from_node_id(api, node_id);
                let system_state = api.system_state();
                Self::resolve_from_system(system_state.system).handle_after_create_node(
                    system_state.current_call_frame,
                    current_depth,
                    resource_summary,
                );
            }
        }

        Ok(())
    }

    fn on_drop_node(api: &mut ModuleApi, event: &DropNodeEvent) -> Result<(), RuntimeError> {
        if api.current_stack_id_uncosted() != 0 {
            return Ok(());
        }

        match event {
            DropNodeEvent::Start(node_id) => {
                let resource_summary = ResourceSummary::from_node_id(api, node_id);
                api.module().handle_before_drop_node(resource_summary);
            }
            DropNodeEvent::End(..) => {
                let current_depth = api.current_stack_depth_uncosted();
                let system_state = api.system_state();
                system_state
                    .system
                    .modules
                    .execution_trace
                    .handle_after_drop_node(system_state.current_call_frame, current_depth);
            }
            DropNodeEvent::IOAccess(_) => {}
        }

        Ok(())
    }

    fn before_invoke(
        api: &mut ModuleApi,
        invocation: &KernelInvocation<Actor>,
    ) -> Result<(), RuntimeError> {
        if api.current_stack_id_uncosted() != 0 {
            return Ok(());
        }

        let message = CallFrameMessage::from_input(&invocation.args, &invocation.call_frame_data);
        let resource_summary = ResourceSummary::from_message(api, &message);
        let callee = &invocation.call_frame_data;
        let args = &invocation.args;
        let system_state = api.system_state();
        system_state
            .system
            .modules
            .execution_trace
            .handle_before_invoke(
                system_state.current_call_frame,
                callee,
                resource_summary,
                args,
            );
        Ok(())
    }

    fn on_execution_finish(
        api: &mut ModuleApi,
        message: &CallFrameMessage,
    ) -> Result<(), RuntimeError> {
        if api.current_stack_id_uncosted() != 0 {
            return Ok(());
        }

        let current_depth = api.current_stack_depth_uncosted();
        let resource_summary = ResourceSummary::from_message(api, message);

        let system_state = api.system_state();

        let caller = TraceActor::from_actor(system_state.caller_call_frame);

        system_state
            .system
            .modules
            .execution_trace
            .handle_on_execution_finish(
                system_state.current_call_frame,
                current_depth,
                &caller,
                resource_summary,
            );

        Ok(())
    }
}

impl ExecutionTraceModule {
    pub fn new(max_kernel_call_depth_traced: usize) -> ExecutionTraceModule {
        Self {
            max_kernel_call_depth_traced,
            current_instruction_index: 0,
            current_kernel_call_depth: 0,
            traced_kernel_call_inputs_stack: vec![],
            kernel_call_traces_stacks: index_map_new(),
            vault_ops: Vec::new(),
        }
    }

    fn handle_before_create_node(&mut self) {
        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth += 1;
        if self.current_kernel_call_depth - 1 > self.max_kernel_call_depth_traced {
            return;
        }

        let instruction_index = self.instruction_index();
        let traced_input = (
            ResourceSummary::default(),
            TraceOrigin::CreateNode,
            instruction_index,
        );
        self.traced_kernel_call_inputs_stack.push(traced_input);
    }

    fn handle_after_create_node(
        &mut self,
        current_actor: &Actor,
        current_depth: usize,
        resource_summary: ResourceSummary,
    ) {
        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth -= 1;
        if self.current_kernel_call_depth > self.max_kernel_call_depth_traced {
            return;
        }

        let current_actor = TraceActor::from_actor(current_actor);
        self.finalize_kernel_call_trace(resource_summary, current_actor, current_depth)
    }

    fn handle_before_drop_node(&mut self, resource_summary: ResourceSummary) {
        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth += 1;
        if self.current_kernel_call_depth - 1 > self.max_kernel_call_depth_traced {
            return;
        }

        let instruction_index = self.instruction_index();
        let traced_input = (resource_summary, TraceOrigin::DropNode, instruction_index);
        self.traced_kernel_call_inputs_stack.push(traced_input);
    }

    fn handle_after_drop_node(&mut self, current_actor: &Actor, current_depth: usize) {
        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth -= 1;
        if self.current_kernel_call_depth > self.max_kernel_call_depth_traced {
            return;
        }

        let traced_output = ResourceSummary::default();
        let current_actor = TraceActor::from_actor(current_actor);
        self.finalize_kernel_call_trace(traced_output, current_actor, current_depth)
    }

    fn handle_before_invoke(
        &mut self,
        current_actor: &Actor,
        callee: &Actor,
        resource_summary: ResourceSummary,
        args: &IndexedScryptoValue,
    ) {
        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth += 1;
        if self.current_kernel_call_depth - 1 > self.max_kernel_call_depth_traced {
            return;
        }

        let origin = match &callee {
            Actor::Method(actor @ MethodActor { ident, .. }) => {
                TraceOrigin::ScryptoMethod(ApplicationFnIdentifier {
                    blueprint_id: actor.get_blueprint_id(),
                    ident: ident.clone(),
                })
            }
            Actor::Function(FunctionActor {
                blueprint_id,
                ident,
                ..
            }) => TraceOrigin::ScryptoFunction(ApplicationFnIdentifier {
                blueprint_id: blueprint_id.clone(),
                ident: ident.clone(),
            }),
            Actor::BlueprintHook(..) | Actor::Root => {
                return;
            }
        };
        let instruction_index = self.instruction_index();
        self.traced_kernel_call_inputs_stack.push((
            resource_summary.clone(),
            origin,
            instruction_index,
        ));

        match &callee {
            Actor::Method(actor @ MethodActor { node_id, ident, .. })
                if VaultUtil::is_vault_blueprint(&actor.get_blueprint_id()) =>
            {
                match ident.as_str() {
                    VAULT_PUT_IDENT => {
                        self.handle_vault_put_input(&resource_summary, current_actor, node_id)
                    }
                    FUNGIBLE_VAULT_LOCK_FEE_IDENT => {
                        self.handle_vault_lock_fee_input(current_actor, node_id, args)
                    }
                    VAULT_BURN_IDENT
                    | VAULT_TAKE_IDENT
                    | VAULT_TAKE_ADVANCED_IDENT
                    | VAULT_RECALL_IDENT
                    | VAULT_GET_AMOUNT_IDENT
                    | VAULT_FREEZE_IDENT
                    | VAULT_UNFREEZE_IDENT => { /* no-op */ }
                    FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT
                    | FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT
                    | FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_IDENT => { /* no-op */ }
                    NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT
                    | NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT
                    | NON_FUNGIBLE_VAULT_CONTAINS_NON_FUNGIBLE_IDENT
                    | NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT
                    | NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT
                    | NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT
                    | NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT
                    | NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT => { /* no-op */ }
                    _ => panic!("Unhandled vault method"),
                }
            }
            _ => { /* no-op */ }
        }
    }

    fn handle_on_execution_finish(
        &mut self,
        current_actor: &Actor,
        current_depth: usize,
        caller: &TraceActor,
        resource_summary: ResourceSummary,
    ) {
        // Important to always update the counter (even if we're over the depth limit).
        self.current_kernel_call_depth -= 1;
        if self.current_kernel_call_depth > self.max_kernel_call_depth_traced {
            return;
        }

        match current_actor {
            Actor::Method(actor @ MethodActor { node_id, ident, .. }) => {
                if VaultUtil::is_vault_blueprint(&actor.get_blueprint_id()) {
                    match ident.as_str() {
                        VAULT_TAKE_IDENT | VAULT_TAKE_ADVANCED_IDENT | VAULT_RECALL_IDENT => {
                            for (_, resource) in &resource_summary.buckets {
                                let op = if ident == VAULT_TAKE_IDENT {
                                    VaultOp::Take(resource.resource_address(), resource.amount())
                                } else if ident == VAULT_TAKE_ADVANCED_IDENT {
                                    VaultOp::TakeAdvanced(
                                        resource.resource_address(),
                                        resource.amount(),
                                    )
                                } else if ident == VAULT_RECALL_IDENT {
                                    VaultOp::Recall(resource.resource_address(), resource.amount())
                                } else {
                                    panic!("Unhandled vault method")
                                };
                                self.vault_ops.push((
                                    caller.clone(),
                                    node_id.clone(),
                                    op,
                                    self.instruction_index(),
                                ));
                            }
                        }
                        VAULT_PUT_IDENT
                        | VAULT_GET_AMOUNT_IDENT
                        | VAULT_FREEZE_IDENT
                        | VAULT_UNFREEZE_IDENT
                        | VAULT_BURN_IDENT => { /* no-op */ }
                        FUNGIBLE_VAULT_LOCK_FEE_IDENT
                        | FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT
                        | FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT
                        | FUNGIBLE_VAULT_CREATE_PROOF_OF_AMOUNT_IDENT => { /* no-op */ }
                        NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT
                        | NON_FUNGIBLE_VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT
                        | NON_FUNGIBLE_VAULT_CONTAINS_NON_FUNGIBLE_IDENT
                        | NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT
                        | NON_FUNGIBLE_VAULT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT
                        | NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT
                        | NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT
                        | NON_FUNGIBLE_VAULT_BURN_NON_FUNGIBLES_IDENT => { /* no-op */ }
                        _ => panic!("Unhandled vault method"),
                    }
                }
            }
            Actor::Function(_) => {}
            Actor::BlueprintHook(..) | Actor::Root => return,
        }

        let current_actor = TraceActor::from_actor(current_actor);
        self.finalize_kernel_call_trace(resource_summary, current_actor, current_depth)
    }

    fn finalize_kernel_call_trace(
        &mut self,
        traced_output: ResourceSummary,
        current_actor: TraceActor,
        current_depth: usize,
    ) {
        let child_traces = self
            .kernel_call_traces_stacks
            .swap_remove(&(self.current_kernel_call_depth + 1))
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
            let trace = ExecutionTrace {
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

    pub fn finalize(
        mut self,
        fee_payments: &IndexMap<NodeId, Decimal>,
        is_success: bool,
    ) -> TransactionExecutionTrace {
        let mut execution_traces = Vec::new();
        for (_, traces) in self.kernel_call_traces_stacks.drain(..) {
            execution_traces.extend(traces);
        }

        let fee_locks = calculate_fee_locks(&self.vault_ops);
        let resource_changes = calculate_resource_changes(self.vault_ops, fee_payments, is_success);

        TransactionExecutionTrace {
            execution_traces,
            resource_changes,
            fee_locks,
        }
    }

    fn instruction_index(&self) -> usize {
        self.current_instruction_index
    }

    fn handle_vault_put_input<'s>(
        &mut self,
        resource_summary: &ResourceSummary,
        caller: &Actor,
        vault_id: &NodeId,
    ) {
        let actor = TraceActor::from_actor(caller);
        for (_, resource) in &resource_summary.buckets {
            self.vault_ops.push((
                actor.clone(),
                vault_id.clone(),
                VaultOp::Put(resource.resource_address(), resource.amount()),
                self.instruction_index(),
            ));
        }
    }

    fn handle_vault_lock_fee_input<'s>(
        &mut self,
        caller: &Actor,
        vault_id: &NodeId,
        args: &IndexedScryptoValue,
    ) {
        let actor = TraceActor::from_actor(caller);
        let FungibleVaultLockFeeInput { amount, contingent } = args.as_typed().unwrap();
        self.vault_ops.push((
            actor,
            vault_id.clone(),
            VaultOp::LockFee(amount, contingent),
            self.instruction_index(),
        ));
    }
}

pub fn calculate_resource_changes(
    mut vault_ops: Vec<(TraceActor, NodeId, VaultOp, usize)>,
    fee_payments: &IndexMap<NodeId, Decimal>,
    is_commit_success: bool,
) -> IndexMap<usize, Vec<ResourceChange>> {
    // Retain lock fee only if the transaction fails.
    if !is_commit_success {
        vault_ops.retain(|x| matches!(x.2, VaultOp::LockFee(..)));
    }

    // Calculate per instruction index, actor, vault resource changes.
    let mut vault_changes =
        index_map_new::<usize, IndexMap<NodeId, IndexMap<NodeId, (ResourceAddress, Decimal)>>>();
    for (actor, vault_id, vault_op, instruction_index) in vault_ops {
        if let TraceActor::Method(node_id) = actor {
            match vault_op {
                VaultOp::Put(resource_address, amount) => {
                    let entry = &mut vault_changes
                        .entry(instruction_index)
                        .or_default()
                        .entry(node_id)
                        .or_default()
                        .entry(vault_id)
                        .or_insert((resource_address, Decimal::zero()))
                        .1;
                    *entry = entry.checked_add(amount).unwrap();
                }
                VaultOp::Take(resource_address, amount)
                | VaultOp::TakeAdvanced(resource_address, amount)
                | VaultOp::Recall(resource_address, amount) => {
                    let entry = &mut vault_changes
                        .entry(instruction_index)
                        .or_default()
                        .entry(node_id)
                        .or_default()
                        .entry(vault_id)
                        .or_insert((resource_address, Decimal::zero()))
                        .1;
                    *entry = entry.checked_sub(amount).unwrap();
                }
                VaultOp::LockFee(..) => {
                    let entry = &mut vault_changes
                        .entry(instruction_index)
                        .or_default()
                        .entry(node_id)
                        .or_default()
                        .entry(vault_id)
                        .or_insert((XRD, Decimal::zero()))
                        .1;
                    *entry = entry
                        .checked_sub(fee_payments.get(&vault_id).cloned().unwrap_or_default())
                        .unwrap();
                }
            }
        }
    }

    // Convert into a vec for ease of consumption.
    let mut resource_changes = index_map_new::<usize, Vec<ResourceChange>>();
    for (instruction_index, instruction_resource_changes) in vault_changes {
        for (node_id, map) in instruction_resource_changes {
            for (vault_id, (resource_address, delta)) in map {
                // Add a resource change log if non-zero
                if !delta.is_zero() {
                    resource_changes
                        .entry(instruction_index)
                        .or_default()
                        .push(ResourceChange {
                            resource_address,
                            node_id,
                            vault_id,
                            amount: delta,
                        });
                }
            }
        }
    }

    resource_changes
}

pub fn calculate_fee_locks(vault_ops: &Vec<(TraceActor, NodeId, VaultOp, usize)>) -> FeeLocks {
    let mut fee_locks = FeeLocks {
        lock: Decimal::ZERO,
        contingent_lock: Decimal::ZERO,
    };
    for (_, _, vault_op, _) in vault_ops {
        if let VaultOp::LockFee(amount, is_contingent) = vault_op {
            if !is_contingent {
                fee_locks.lock = fee_locks.lock.checked_add(*amount).unwrap()
            } else {
                fee_locks.contingent_lock = fee_locks.contingent_lock.checked_add(*amount).unwrap()
            }
        };
    }
    fee_locks
}
