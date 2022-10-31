use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use transaction::model::Instruction;

#[derive(Debug, Clone, PartialEq, TypeId, Encode, Decode)]
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
    LockContingentFee,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum ExecutionTraceError {
    InvalidState(String),
}

pub struct ExecutionTraceModule {
    snapshot_pre_current_instruction: Option<TraceHeapSnapshot>,
}

#[derive(Clone, Debug, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct ProofSnapshot {
    pub resource_address: ResourceAddress,
    pub resource_type: ResourceType,
    pub restricted: bool,
    pub total_locked: LockedAmountOrIds,
}

impl From<&ProofSubstate> for ProofSnapshot {
    fn from(proof: &ProofSubstate) -> ProofSnapshot {
        ProofSnapshot {
            resource_address: proof.resource_address,
            resource_type: proof.resource_type,
            restricted: proof.restricted,
            total_locked: proof.total_locked.clone(),
        }
    }
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct TraceHeapSnapshot {
    pub owned_buckets: HashMap<BucketId, Resource>,
    pub owned_proofs: HashMap<ProofId, ProofSnapshot>,
    pub auth_zone_proofs: Vec<ProofSnapshot>,
    pub worktop_resources: HashMap<ResourceAddress, Resource>,
}

impl<R: FeeReserve> Module<R> for ExecutionTraceModule {
    fn pre_sys_call(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _input: SysCallInput,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn post_sys_call(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _output: SysCallOutput,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_run(
        &mut self,
        actor: &REActor,
        input: &ScryptoValue,
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


    fn on_application_event(
        &mut self,
        _call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<R>,
        event: &ApplicationEvent,
    ) -> Result<(), ModuleError> {
        match event {
            ApplicationEvent::PreExecuteInstruction { .. } =>
                self.pre_instruction_trace(heap),
            ApplicationEvent::PostExecuteInstruction { instruction } =>
                self.post_instruction_trace(heap, track, instruction)
        }
    }
}

impl ExecutionTraceModule {
    pub fn new() -> ExecutionTraceModule {
        Self {
            snapshot_pre_current_instruction: None,
        }
    }

    fn pre_instruction_trace(&mut self, heap: &mut Heap) -> Result<(), ModuleError> {
        if self.snapshot_pre_current_instruction.is_none() {
            let pre_snapshot = ExecutionTraceModule::heap_snapshot(heap)?;
            self.snapshot_pre_current_instruction = Some(pre_snapshot);
            Ok(())
        } else {
            Err(ModuleError::ExecutionTraceError(
                ExecutionTraceError::InvalidState("Unexpected \"pre\" trace".to_string()),
            ))
        }
    }

    fn post_instruction_trace<'s, R: FeeReserve>(
        &mut self,
        heap: &mut Heap,
        track: &mut Track<'s, R>,
        instruction: &Instruction,
    ) -> Result<(), ModuleError> {
        match self.snapshot_pre_current_instruction.take() {
            Some(pre_snapshot) => {
                let post_snapshot = ExecutionTraceModule::heap_snapshot(heap)?;
                track
                    .instruction_traces
                    .push((instruction.clone(), pre_snapshot, post_snapshot));
                Ok(())
            }
            None => Err(ModuleError::ExecutionTraceError(
                ExecutionTraceError::InvalidState("Missing \"pre\" trace".to_string()),
            )),
        }
    }

    fn heap_snapshot(heap: &mut Heap) -> Result<TraceHeapSnapshot, ModuleError> {
        // Trace worktop resources
        let worktop_substate_ref = heap
            .get_substate(
                RENodeId::Worktop,
                &SubstateOffset::Worktop(WorktopOffset::Worktop),
            )
            .expect("Worktop does not exist");

        let worktop_resources = worktop_substate_ref.worktop().peek_resources();

        // Trace proofs in AuthZone
        let auth_zone_node = heap
            .nodes()
            .iter()
            .find(|(node_id, _)| matches!(node_id, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist")
            .1;

        let auth_zone_substate_ref = auth_zone_node
            .substates
            .get(&SubstateOffset::AuthZone(AuthZoneOffset::AuthZone))
            .expect("AuthZone node does not contain an AuthZone substate")
            .to_ref();

        let auth_zone = auth_zone_substate_ref.auth_zone().cur_auth_zone();

        let auth_zone_proofs: Vec<ProofSnapshot> = auth_zone
            .proofs()
            .iter()
            .map(|proof| proof.into())
            .collect();

        // Trace owned Bucket and Proof nodes
        let mut owned_buckets = HashMap::new();
        let mut owned_proofs: HashMap<ProofId, ProofSnapshot> = HashMap::new();
        for (owned_node_id, owned_node) in heap.nodes() {
            match owned_node_id {
                RENodeId::Bucket(bucket_id) => {
                    let bucket_substate_ref = owned_node
                        .substates
                        .get(&SubstateOffset::Bucket(BucketOffset::Bucket))
                        .expect("Bucket node does not contain a Bucket substate")
                        .to_ref();
                    let resource = bucket_substate_ref.bucket().peek_resource();
                    owned_buckets.insert(bucket_id.clone(), resource);
                }
                RENodeId::Proof(proof_id) => {
                    let proof_substate_ref = owned_node
                        .substates
                        .get(&SubstateOffset::Proof(ProofOffset::Proof))
                        .expect("Proof node does not contain a Proof substate")
                        .to_ref();
                    let proof_trace: ProofSnapshot = proof_substate_ref.proof().into();
                    owned_proofs.insert(proof_id.clone(), proof_trace);
                }
                _ => (), // no-op
            }
        }

        Ok(TraceHeapSnapshot {
            owned_buckets,
            owned_proofs,
            auth_zone_proofs,
            worktop_resources,
        })
    }

    fn trace_run<'s, R: FeeReserve>(
        call_frame: &CallFrame,
        heap: &mut Heap,
        track: &mut Track<'s, R>,
        actor: &REActor,
        input: &ScryptoValue,
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
                (
                    NativeMethod::Vault(VaultMethod::LockContingentFee),
                    RENodeId::Vault(vault_id),
                ) => Self::handle_vault_lock_contingent_fee(track, caller, &vault_id),
                _ => {}
            }
        }
    }

    fn handle_vault_put<'s, R: FeeReserve>(
        heap: &mut Heap,
        track: &mut Track<'s, R>,
        actor: &REActor,
        vault_id: &VaultId,
        input: &ScryptoValue,
    ) {
        if let Ok(call_data) = scrypto_decode::<VaultPutInput>(&input.raw) {
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
        input: &ScryptoValue,
    ) {
        if let Ok(call_data) = scrypto_decode::<VaultTakeInput>(&input.raw) {
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

    fn handle_vault_lock_contingent_fee<'s, R: FeeReserve>(
        track: &mut Track<'s, R>,
        actor: &REActor,
        vault_id: &VaultId,
    ) {
        track
            .vault_ops
            .push((actor.clone(), vault_id.clone(), VaultOp::LockContingentFee));
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
                        VaultOp::LockFee | VaultOp::LockContingentFee => {
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
