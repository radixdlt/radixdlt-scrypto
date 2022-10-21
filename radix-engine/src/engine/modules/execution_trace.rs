use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;

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
pub enum ExecutionTraceError {}

#[derive(Debug)]
pub struct ExecutionTraceModule {}

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
}

impl ExecutionTraceModule {
    pub fn new() -> ExecutionTraceModule {
        Self {}
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
                (
                    NativeMethod::Vault(VaultMethod::Put),
                    Receiver::Ref(RENodeId::Vault(vault_id)),
                ) => Self::handle_vault_put(heap, track, caller, &vault_id, input),
                (
                    NativeMethod::Vault(VaultMethod::Take),
                    Receiver::Ref(RENodeId::Vault(vault_id)),
                ) => Self::handle_vault_take(track, caller, &vault_id, input),
                (
                    NativeMethod::Vault(VaultMethod::LockFee),
                    Receiver::Ref(RENodeId::Vault(vault_id)),
                ) => Self::handle_vault_lock_fee(track, caller, &vault_id),
                (
                    NativeMethod::Vault(VaultMethod::LockContingentFee),
                    Receiver::Ref(RENodeId::Vault(vault_id)),
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
        state_track: &mut StateTrack,
        is_commit_success: bool,
    ) -> Self {
        let mut vault_changes = HashMap::<ComponentId, HashMap<VaultId, Decimal>>::new();
        let mut vault_locked_by = HashMap::<VaultId, ComponentId>::new();
        for (actor, vault_id, vault_op) in ops {
            if let REActor::Method(_, resolved_receiver) = actor {
                if let Receiver::Ref(RENodeId::Component(component_id)) = resolved_receiver.receiver
                {
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
                    let resource_address = Self::get_vault_resource_address(vault_id, state_track);
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
        state_track: &mut StateTrack,
    ) -> ResourceAddress {
        state_track
            .get_substate(&SubstateId(
                RENodeId::Vault(vault_id),
                SubstateOffset::Vault(VaultOffset::Vault),
            ))
            .expect("Failed to find the vault substate")
            .to_runtime()
            .vault()
            .resource_address()
    }
}
