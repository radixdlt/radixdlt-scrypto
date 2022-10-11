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

#[derive(Debug)]
pub struct ExecutionTrace {
    /// Stores resource changes that resulted from vault's operations.
    pub vault_ops: Vec<(REActor, VaultId, VaultOp)>,
}

impl<R: FeeReserve> Module<R> for ExecutionTrace {
    fn pre_sys_call(
        &mut self,
        track: &mut Track<R>,
        call_frames: &mut Vec<CallFrame>,
        input: SysCallInput,
    ) -> Result<(), ModuleError> {
        self.trace_invoke_method(track, call_frames, input)
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
        Self {
            vault_ops: Vec::new(),
        }
    }

    fn trace_invoke_method<'s, R: FeeReserve>(
        &mut self,
        track: &mut Track<'s, R>,
        call_frames: &Vec<CallFrame>,
        sys_input: SysCallInput,
    ) -> Result<(), ModuleError> {
        let actor = &call_frames
            .last()
            .expect("Current call frame not found")
            .actor;

        if let SysCallInput::Invoke {
            fn_ident,
            input,
            depth,
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
                        self.handle_vault_put(actor, vault_id, input)?;
                    }
                    (
                        Receiver::Ref(RENodeId::Vault(vault_id)),
                        MethodIdent::Native(NativeMethod::Vault(VaultMethod::Take)),
                    ) => {
                        self.handle_vault_take(actor, vault_id, input)?;
                    }
                    (
                        Receiver::Ref(RENodeId::Vault(vault_id)),
                        MethodIdent::Native(NativeMethod::Vault(VaultMethod::LockFee)),
                    ) => {
                        self.handle_vault_lock_fee(actor, vault_id, input)?;
                    }
                    (
                        Receiver::Ref(RENodeId::Vault(vault_id)),
                        MethodIdent::Native(NativeMethod::Vault(VaultMethod::LockContingentFee)),
                    ) => {
                        self.handle_vault_lock_contingent_fee(actor, vault_id, input)?;
                    }
                    _ => {}
                };
            }
        }

        Ok(())
    }

    fn handle_vault_put(
        &mut self,
        actor: &REActor,
        vault_id: &VaultId,
        input: &ScryptoValue,
    ) -> Result<(), ModuleError> {
        let bucket_id = input.bucket.0;
        let bucket_node_id = RENodeId::Bucket(bucket_id);

        let bucket_node =
            next_owned_values
                .get(&bucket_node_id)
                .ok_or(RuntimeError::KernelError(KernelError::RENodeNotFound(
                    bucket_node_id,
                )))?;

        if let HeapRENode::Bucket(bucket) = &bucket_node.root {
            if let ResourceType::Fungible { divisibility: _ } = bucket.resource_type() {
                self.record_resource_change(
                    &bucket.resource_address(),
                    component_id,
                    vault_id,
                    bucket.total_amount(),
                );
                Ok(())
            } else {
                /* TODO: Also handle non-fungible resource changes */
                Ok(())
            }
        } else {
            Err(RuntimeError::KernelError(KernelError::BucketNotFound(
                bucket_id,
            )))
        }
    }

    fn handle_vault_take(
        &mut self,
        actor: &REActor,
        vault_id: &VaultId,
        input: &ScryptoValue,
    ) -> Result<(), ModuleError> {
        self.record_resource_change(resource_address, component_id, vault_id, -input.amount);
        Ok(())
    }

    fn handle_vault_lock_fee(
        &mut self,
        actor: &REActor,
        vault_id: &VaultId,
        input: &ScryptoValue,
    ) -> Result<(), ModuleError> {
        self.fee_vaults_components
            .insert(vault_id.clone(), component_id.clone());
        Ok(())
    }

    fn handle_vault_lock_contingent_fee(
        &mut self,
        actor: &REActor,
        vault_id: &VaultId,
        input: &ScryptoValue,
    ) -> Result<(), ModuleError> {
        self.fee_vaults_components
            .insert(vault_id.clone(), component_id.clone());
        Ok(())
    }

    pub fn to_receipt(
        mut self,
        fee_payments: HashMap<VaultId, (ResourceAddress, Decimal)>,
    ) -> ExecutionTraceReceipt {
        // Add fee payments resource changes
        for (vault_id, (resource_address, amount)) in fee_payments {
            let component_id = self
                .fee_vaults_components
                .get(&vault_id)
                .expect("Failed to find component ID for a fee payment vault")
                .clone();
            self.record_resource_change(&resource_address, &component_id, &vault_id, -amount);
        }

        let resource_changes: Vec<ResourceChange> = self
            .resource_changes
            .into_iter()
            .flat_map(|(component_id, v)| {
                v.into_iter().map(
                    move |(vault_id, (resource_address, amount))| ResourceChange {
                        resource_address,
                        component_id,
                        vault_id,
                        amount,
                    },
                )
            })
            .filter(|el| !el.amount.is_zero())
            .collect();

        ExecutionTraceReceipt { resource_changes }
    }
}
