use sbor::rust::collections::*;
use scrypto::buffer::scrypto_decode;
use scrypto::core::{FnIdentifier, NativeFnIdentifier, Receiver, VaultFnIdentifier};
use scrypto::engine::types::*;
use scrypto::prelude::{VaultPutInput, VaultTakeInput};
use scrypto::values::*;

use crate::engine::*;
use crate::model::*;

pub struct ExecutionTraceModule;

impl ExecutionTraceModule {
    pub fn trace_invoke_method(
        call_frames: &Vec<CallFrame>,
        track: &Track,
        actor: &REActor,
        fn_identifier: &FnIdentifier,
        node_id: &RENodeId,
        node_pointer: RENodePointer,
        input: &ScryptoValue,
        next_owned_values: &HashMap<RENodeId, HeapRootRENode>,
        execution_trace: &mut ExecutionTrace,
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
                            RuntimeError::VaultError(VaultError::InvalidRequestData(e))
                        })?;

                        ExecutionTraceModule::handle_vault_put(
                            component_address,
                            vault_id,
                            decoded_input,
                            next_owned_values,
                            execution_trace,
                        )?;
                    }
                    FnIdentifier::Native(NativeFnIdentifier::Vault(VaultFnIdentifier::Take)) => {
                        let decoded_input = scrypto_decode(&input.raw).map_err(|e| {
                            RuntimeError::VaultError(VaultError::InvalidRequestData(e))
                        })?;

                        let vault_node_ref = node_pointer.to_ref(call_frames, track);

                        let resource_address = vault_node_ref.vault().resource_address();

                        ExecutionTraceModule::handle_vault_take(
                            &resource_address,
                            component_address,
                            vault_id,
                            decoded_input,
                            execution_trace,
                        )?;
                    }
                    _ => {} // no-op
                }
            }
        }

        Ok(())
    }

    fn handle_vault_put(
        component_address: &ComponentAddress,
        vault_id: &VaultId,
        input: VaultPutInput,
        next_owned_values: &HashMap<RENodeId, HeapRootRENode>,
        execution_trace: &mut ExecutionTrace,
    ) -> Result<(), RuntimeError> {
        let bucket_id = input.bucket.0;
        let bucket_node_id = RENodeId::Bucket(bucket_id);

        let bucket_node = next_owned_values
            .get(&bucket_node_id)
            .ok_or(RuntimeError::RENodeNotFound(bucket_node_id))?;

        if let HeapRENode::Bucket(bucket) = &bucket_node.root {
            if let ResourceType::Fungible { divisibility: _ } = bucket.resource_type() {
                ExecutionTraceModule::record_resource_change(
                    &bucket.resource_address(),
                    component_address,
                    vault_id,
                    bucket.total_amount(),
                    execution_trace,
                )
            } else {
                /* TODO: Also handle non-fungible resource changes */
                Ok(())
            }
        } else {
            Err(RuntimeError::BucketNotFound(bucket_id))
        }
    }

    fn handle_vault_take(
        resource_address: &ResourceAddress,
        component_address: &ComponentAddress,
        vault_id: &VaultId,
        input: VaultTakeInput,
        execution_trace: &mut ExecutionTrace,
    ) -> Result<(), RuntimeError> {
        ExecutionTraceModule::record_resource_change(
            resource_address,
            component_address,
            vault_id,
            -input.amount,
            execution_trace,
        )
    }

    fn record_resource_change(
        resource_address: &ResourceAddress,
        component_address: &ComponentAddress,
        vault_id: &VaultId,
        amount: Decimal,
        execution_trace: &mut ExecutionTrace,
    ) -> Result<(), RuntimeError> {
        let component_changes = execution_trace
            .resource_changes
            .entry(component_address.clone())
            .or_insert(HashMap::new());

        let vault_change = component_changes
            .entry(vault_id.clone())
            .or_insert((resource_address.clone(), Decimal::zero()));

        vault_change.1 += amount;

        Ok(())
    }
}
