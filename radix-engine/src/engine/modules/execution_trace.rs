use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;
use scrypto::core::{FnIdent, MethodIdent, ReceiverMethodIdent};

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
    LockFee(Decimal),
    LockContingentFee(Decimal),
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum ExecutionTraceError {}

#[derive(Debug)]
pub struct ExecutionTrace {}

impl<R: FeeReserve> Module<R> for ExecutionTrace {
    fn pre_sys_call(
        &mut self,
        track: &mut Track<R>,
        call_frames: &mut Vec<CallFrame>,
        input: SysCallInput,
    ) -> Result<(), ModuleError> {
        Self::trace_invoke_method(track, call_frames, input);
        Ok(())
    }

    fn post_sys_call(
        &mut self,
        _track: &mut Track<R>,
        _call_frames: &mut Vec<CallFrame>,
        _output: SysCallOutput,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_wasm_instantiation(
        &mut self,
        _track: &mut Track<R>,
        _call_frames: &mut Vec<CallFrame>,
        _code: &[u8],
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_wasm_costing(
        &mut self,
        _track: &mut Track<R>,
        _call_frames: &mut Vec<CallFrame>,
        _units: u32,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_lock_fee(
        &mut self,
        _track: &mut Track<R>,
        _call_frames: &mut Vec<CallFrame>,
        _vault_id: VaultId,
        fee: Resource,
        _contingent: bool,
    ) -> Result<Resource, ModuleError> {
        Ok(fee)
    }
}

impl ExecutionTrace {
    pub fn new() -> ExecutionTrace {
        Self {}
    }

    fn trace_invoke_method<'s, R: FeeReserve>(
        track: &mut Track<'s, R>,
        call_frames: &Vec<CallFrame>,
        sys_input: SysCallInput,
    ) {
        let actor = &call_frames
            .last()
            .expect("Current call frame not found")
            .actor;

        if let SysCallInput::Invoke {
            fn_ident, input, ..
        } = sys_input
        {
            if let FnIdent::Method(ReceiverMethodIdent {
                receiver,
                method_ident,
            }) = fn_ident
            {
                match (receiver, method_ident) {
                    (
                        Receiver::Ref(RENodeId::Vault(vault_id)),
                        MethodIdent::Native(NativeMethod::Vault(VaultMethod::Put)),
                    ) => {
                        Self::handle_vault_put(track, actor, vault_id, input, call_frames);
                    }
                    (
                        Receiver::Ref(RENodeId::Vault(vault_id)),
                        MethodIdent::Native(NativeMethod::Vault(VaultMethod::Take)),
                    ) => {
                        Self::handle_vault_take(track, actor, vault_id, input);
                    }
                    (
                        Receiver::Ref(RENodeId::Vault(vault_id)),
                        MethodIdent::Native(NativeMethod::Vault(VaultMethod::LockFee)),
                    ) => {
                        Self::handle_vault_lock_fee(track, actor, vault_id, input);
                    }
                    (
                        Receiver::Ref(RENodeId::Vault(vault_id)),
                        MethodIdent::Native(NativeMethod::Vault(VaultMethod::LockContingentFee)),
                    ) => {
                        Self::handle_vault_lock_contingent_fee(track, actor, vault_id, input);
                    }
                    _ => {}
                };
            }
        }
    }

    fn handle_vault_put<'s, R: FeeReserve>(
        track: &mut Track<'s, R>,
        actor: &REActor,
        vault_id: &VaultId,
        input: &ScryptoValue,
        call_frames: &Vec<CallFrame>,
    ) {
        if let Ok(call_data) = scrypto_decode::<VaultPutInput>(&input.raw) {
            let bucket_id = call_data.bucket.0;

            let frame = call_frames.last().expect("Current call frame not found");

            if let Some(tree) = frame.owned_heap_nodes.get(&RENodeId::Bucket(bucket_id)) {
                if let HeapRENode::Bucket(bucket_node) = &tree.root {
                    track.vault_ops.push((
                        actor.clone(),
                        vault_id.clone(),
                        VaultOp::Put(bucket_node.total_amount()),
                    ));
                }
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
        input: &ScryptoValue,
    ) {
        if let Ok(call_data) = scrypto_decode::<VaultLockFeeInput>(&input.raw) {
            track.vault_ops.push((
                actor.clone(),
                vault_id.clone(),
                VaultOp::LockFee(call_data.amount),
            ));
        }
    }

    fn handle_vault_lock_contingent_fee<'s, R: FeeReserve>(
        track: &mut Track<'s, R>,
        actor: &REActor,
        vault_id: &VaultId,
        input: &ScryptoValue,
    ) {
        if let Ok(call_data) = scrypto_decode::<VaultLockFeeInput>(&input.raw) {
            track.vault_ops.push((
                actor.clone(),
                vault_id.clone(),
                VaultOp::LockContingentFee(call_data.amount),
            ));
        }
    }
}

impl ExecutionTraceReceipt {
    // TODO: is it better to derive resource change from substate diff, instead of execution trace?

    pub fn new(
        ops: Vec<(REActor, VaultId, VaultOp)>,
        actual_fee_payments: HashMap<VaultId, Decimal>,
        state_track: &mut AppStateTrack,
    ) -> Self {
        let mut component_vault_changes = HashMap::<ComponentId, HashMap<VaultId, Decimal>>::new();
        let mut component_vault_fee_locking = HashMap::<ComponentId, VaultId>::new();
        for op in ops {
            if let REActor::Method(FullyQualifiedReceiverMethod { receiver, .. }) = op.0 {
                if let Receiver::Ref(RENodeId::Component(component_id)) = receiver {
                    let vault_id = op.1;
                    match op.2 {
                        VaultOp::Create(_) => todo!("Not supported yet!"),
                        VaultOp::Put(amount) => {
                            *component_vault_changes
                                .entry(component_id)
                                .or_default()
                                .entry(vault_id)
                                .or_default() += amount;
                        }
                        VaultOp::Take(amount) => {
                            *component_vault_changes
                                .entry(component_id)
                                .or_default()
                                .entry(vault_id)
                                .or_default() -= amount;
                        }
                        VaultOp::LockFee(_) | VaultOp::LockContingentFee(_) => {
                            component_vault_fee_locking.insert(component_id, vault_id);
                        }
                    };
                }
            }
        }

        for (vault_id, amount) in actual_fee_payments {
            let component_id = component_vault_fee_locking
                .get(&vault_id)
                .expect("Failed to find component ID for a fee payment vault")
                .clone();
            *component_vault_changes
                .entry(component_id)
                .or_default()
                .entry(vault_id)
                .or_default() -= amount;
        }

        let mut resource_changes = Vec::<ResourceChange>::new();
        for (component_id, map) in component_vault_changes {
            for (vault_id, amount) in map {
                let resource_address = state_track
                    .get_substate(&SubstateId(
                        RENodeId::Vault(vault_id),
                        SubstateOffset::Vault(VaultOffset::Vault),
                    ))
                    .expect("Failed to find the vault substate")
                    .vault()
                    .0
                    .resource_address();
                resource_changes.push(ResourceChange {
                    resource_address,
                    component_id,
                    vault_id,
                    amount,
                });
            }
        }

        ExecutionTraceReceipt { resource_changes }
    }
}
