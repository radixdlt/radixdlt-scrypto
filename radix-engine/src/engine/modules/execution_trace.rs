use crate::engine::*;
use crate::fee::FeeReserve;
use crate::model::*;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, TypeId, Encode, Decode)]
pub struct ResourceChange {
    pub resource_address: ResourceAddress,
    pub component_address: ComponentAddress,
    pub vault_id: VaultId,
    pub amount: Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionTraceReceipt {
    pub resource_changes: Vec<ResourceChange>,
}

#[derive(Debug)]
pub struct ExecutionTrace {
    pub resource_changes: HashMap<ComponentAddress, HashMap<VaultId, (ResourceAddress, Decimal)>>,
}

impl ExecutionTrace {
    pub fn new() -> ExecutionTrace {
        Self {
            resource_changes: HashMap::new(),
        }
    }

    pub fn trace_invoke_method<'s, R: FeeReserve>(
        &mut self,
        call_frames: &Vec<CallFrame>,
        track: &mut Track<'s, R>,
        actor: &REActor,
        fn_identifier: &FnIdentifier,
        node_id: &RENodeId,
        node_pointer: RENodePointer,
        input: &ScryptoValue,
        next_owned_values: &HashMap<RENodeId, HeapRootRENode>,
    ) -> Result<(), RuntimeError> {
        if let RENodeId::Vault(vault_id) = node_id {
            /* TODO: Warning: depends on call frame's actor being the vault's parent component!
            This isn't always the case! For example, when vault is instantiated in a blueprint
            before the component is globalized (see: test_restricted_transfer in bucket.rs).
            For now, such vault calls are NOT traced.
            Possible solution:
            1. Separately record vault calls that have a blueprint parent
            2. Hook up to when the component is globalized and convert
               blueprint-parented vaults (if any) to regular
               trace entries with component parents. */
            if let Some(Receiver::Ref(RENodeId::Component(component_address))) = &actor.receiver {
                match fn_identifier {
                    FnIdentifier::Native(NativeFnIdentifier::Vault(VaultFnIdentifier::Put)) => {
                        let decoded_input = scrypto_decode(&input.raw).map_err(|e| {
                            RuntimeError::ApplicationError(ApplicationError::VaultError(
                                VaultError::InvalidRequestData(e),
                            ))
                        })?;

                        self.handle_vault_put(
                            component_address,
                            vault_id,
                            decoded_input,
                            next_owned_values,
                        )?;
                    }
                    FnIdentifier::Native(NativeFnIdentifier::Vault(VaultFnIdentifier::Take)) => {
                        let decoded_input = scrypto_decode(&input.raw).map_err(|e| {
                            RuntimeError::ApplicationError(ApplicationError::VaultError(
                                VaultError::InvalidRequestData(e),
                            ))
                        })?;

                        let mut vault_node_ref = node_pointer.to_ref(call_frames, track);

                        let resource_address = vault_node_ref.vault().resource_address();

                        self.handle_vault_take(
                            &resource_address,
                            component_address,
                            vault_id,
                            decoded_input,
                        )?;
                    }
                    _ => {} // no-op
                }
            }
        }

        Ok(())
    }

    fn handle_vault_put(
        &mut self,
        component_address: &ComponentAddress,
        vault_id: &VaultId,
        input: VaultPutInput,
        next_owned_values: &HashMap<RENodeId, HeapRootRENode>,
    ) -> Result<(), RuntimeError> {
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
                    component_address,
                    vault_id,
                    bucket.total_amount(),
                )
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
        resource_address: &ResourceAddress,
        component_address: &ComponentAddress,
        vault_id: &VaultId,
        input: VaultTakeInput,
    ) -> Result<(), RuntimeError> {
        self.record_resource_change(resource_address, component_address, vault_id, -input.amount)
    }

    fn record_resource_change(
        &mut self,
        resource_address: &ResourceAddress,
        component_address: &ComponentAddress,
        vault_id: &VaultId,
        amount: Decimal,
    ) -> Result<(), RuntimeError> {
        let component_changes = self
            .resource_changes
            .entry(component_address.clone())
            .or_insert(HashMap::new());

        let vault_change = component_changes
            .entry(vault_id.clone())
            .or_insert((resource_address.clone(), Decimal::zero()));

        vault_change.1 += amount;

        Ok(())
    }

    pub fn to_receipt(self) -> ExecutionTraceReceipt {
        let resource_changes: Vec<ResourceChange> = self
            .resource_changes
            .into_iter()
            .flat_map(|(component_address, v)| {
                v.into_iter().map(
                    move |(vault_id, (resource_address, amount))| ResourceChange {
                        resource_address,
                        component_address,
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
