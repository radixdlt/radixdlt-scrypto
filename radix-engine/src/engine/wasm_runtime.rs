use crate::engine::RuntimeError;
use crate::engine::{HeapRENode, SystemApi};
use crate::fee::*;
use crate::model::{ComponentInfo, ComponentState, HeapKeyValueStore};
use crate::types::*;
use crate::wasm::*;

/// A glue between system api (call frame and track abstraction) and WASM.
///
/// Execution is free from a costing perspective, as we assume
/// the system api will bill properly.
pub struct RadixEngineWasmRuntime<'y, 's, Y, W, I, C>
where
    Y: SystemApi<'s, W, I, C>,
    W: WasmEngine<I>,
    I: WasmInstance,
    C: FeeReserve,
{
    actor: ScryptoActor,
    system_api: &'y mut Y,
    phantom1: PhantomData<W>,
    phantom2: PhantomData<I>,
    phantom3: PhantomData<C>,
    phantom4: PhantomData<&'s ()>,
}

impl<'y, 's, Y, W, I, C> RadixEngineWasmRuntime<'y, 's, Y, W, I, C>
where
    Y: SystemApi<'s, W, I, C>,
    W: WasmEngine<I>,
    I: WasmInstance,
    C: FeeReserve,
{
    pub fn new(actor: ScryptoActor, system_api: &'y mut Y) -> Self {
        RadixEngineWasmRuntime {
            actor,
            system_api,
            phantom1: PhantomData,
            phantom2: PhantomData,
            phantom3: PhantomData,
            phantom4: PhantomData,
        }
    }

    fn fee_reserve(&mut self) -> &mut C {
        self.system_api.fee_reserve()
    }

    // FIXME: limit access to the API

    fn handle_invoke_function(
        &mut self,
        fn_identifier: FnIdentifier,
        input: Vec<u8>,
    ) -> Result<ScryptoValue, RuntimeError> {
        let call_data = ScryptoValue::from_slice(&input).map_err(RuntimeError::DecodeError)?;
        self.system_api.invoke_function(fn_identifier, call_data)
    }

    fn handle_invoke_method(
        &mut self,
        receiver: Receiver,
        fn_identifier: FnIdentifier,
        input: Vec<u8>,
    ) -> Result<ScryptoValue, RuntimeError> {
        let call_data = ScryptoValue::from_slice(&input).map_err(RuntimeError::DecodeError)?;
        self.system_api
            .invoke_method(receiver, fn_identifier, call_data)
    }

    fn handle_node_create(
        &mut self,
        scrypto_node: ScryptoRENode,
    ) -> Result<ScryptoValue, RuntimeError> {
        let node = match scrypto_node {
            ScryptoRENode::Component(package_address, blueprint_name, state) => {
                // TODO: Move these two checks into CallFrame/System
                if !blueprint_name.eq(self.actor.blueprint_name()) {
                    return Err(RuntimeError::RENodeCreateInvalidPermission);
                }
                if !package_address.eq(self.actor.package_address()) {
                    return Err(RuntimeError::RENodeCreateInvalidPermission);
                }

                // TODO: Check state against blueprint schema

                // Create component
                let component_info =
                    ComponentInfo::new(package_address, blueprint_name, Vec::new());
                let component_state = ComponentState::new(state);
                HeapRENode::Component(component_info, component_state)
            }
            ScryptoRENode::KeyValueStore => HeapRENode::KeyValueStore(HeapKeyValueStore::new()),
        };

        let id = self.system_api.node_create(node)?;
        Ok(ScryptoValue::from_typed(&id))
    }

    // TODO: This logic should move into KeyValueEntry decoding
    fn verify_stored_key(value: &ScryptoValue) -> Result<(), RuntimeError> {
        if !value.bucket_ids.is_empty() {
            return Err(RuntimeError::BucketNotAllowed);
        }
        if !value.proof_ids.is_empty() {
            return Err(RuntimeError::ProofNotAllowed);
        }
        if !value.vault_ids.is_empty() {
            return Err(RuntimeError::VaultNotAllowed);
        }
        if !value.kv_store_ids.is_empty() {
            return Err(RuntimeError::KeyValueStoreNotAllowed);
        }
        Ok(())
    }

    fn handle_node_globalize(&mut self, node_id: RENodeId) -> Result<ScryptoValue, RuntimeError> {
        self.system_api.node_globalize(node_id)?;
        Ok(ScryptoValue::unit())
    }

    fn handle_substate_read(
        &mut self,
        substate_id: SubstateId,
    ) -> Result<ScryptoValue, RuntimeError> {
        match &substate_id {
            SubstateId::KeyValueStoreEntry(_kv_store_id, key_bytes) => {
                let key_data =
                    ScryptoValue::from_slice(&key_bytes).map_err(RuntimeError::DecodeError)?;
                Self::verify_stored_key(&key_data)?;
            }
            _ => {}
        }

        self.system_api.substate_read(substate_id)
    }

    fn handle_substate_write(
        &mut self,
        substate_id: SubstateId,
        value: Vec<u8>,
    ) -> Result<ScryptoValue, RuntimeError> {
        match &substate_id {
            SubstateId::KeyValueStoreEntry(_kv_store_id, key_bytes) => {
                let key_data =
                    ScryptoValue::from_slice(&key_bytes).map_err(RuntimeError::DecodeError)?;
                Self::verify_stored_key(&key_data)?;
            }
            _ => {}
        }
        let scrypto_value = ScryptoValue::from_slice(&value).map_err(RuntimeError::DecodeError)?;
        self.system_api.substate_write(substate_id, scrypto_value)?;
        Ok(ScryptoValue::unit())
    }

    fn handle_get_actor(&mut self) -> Result<ScryptoActor, RuntimeError> {
        return Ok(self.actor.clone());
    }

    fn handle_generate_uuid(&mut self) -> Result<u128, RuntimeError> {
        self.system_api.generate_uuid()
    }

    fn handle_emit_log(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        self.system_api.emit_log(level, message)
    }

    fn handle_check_access_rule(
        &mut self,
        access_rule: AccessRule,
        proof_ids: Vec<ProofId>,
    ) -> Result<bool, RuntimeError> {
        self.system_api.check_access_rule(access_rule, proof_ids)
    }
}

fn encode<T: Encode>(output: T) -> ScryptoValue {
    ScryptoValue::from_typed(&output)
}

impl<'y, 's, Y: SystemApi<'s, W, I, C>, W: WasmEngine<I>, I: WasmInstance, C: FeeReserve>
    WasmRuntime for RadixEngineWasmRuntime<'y, 's, Y, W, I, C>
{
    fn main(&mut self, input: ScryptoValue) -> Result<ScryptoValue, InvokeError> {
        let input: RadixEngineInput =
            scrypto_decode(&input.raw).map_err(|_| InvokeError::InvalidRadixEngineInput)?;
        match input {
            RadixEngineInput::InvokeFunction(fn_identifier, input_bytes) => {
                self.handle_invoke_function(fn_identifier, input_bytes)
            }
            RadixEngineInput::InvokeMethod(receiver, fn_identifier, input_bytes) => {
                self.handle_invoke_method(receiver, fn_identifier, input_bytes)
            }
            RadixEngineInput::RENodeGlobalize(node_id) => self.handle_node_globalize(node_id),
            RadixEngineInput::RENodeCreate(node) => self.handle_node_create(node),
            RadixEngineInput::SubstateRead(substate_id) => self.handle_substate_read(substate_id),
            RadixEngineInput::SubstateWrite(substate_id, value) => {
                self.handle_substate_write(substate_id, value)
            }
            RadixEngineInput::GetActor() => self.handle_get_actor().map(encode),
            RadixEngineInput::GenerateUuid() => self.handle_generate_uuid().map(encode),
            RadixEngineInput::EmitLog(level, message) => {
                self.handle_emit_log(level, message).map(encode)
            }
            RadixEngineInput::CheckAccessRule(rule, proof_ids) => {
                self.handle_check_access_rule(rule, proof_ids).map(encode)
            }
        }
        .map_err(InvokeError::RuntimeError)
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError> {
        self.fee_reserve()
            .consume(n, "run_wasm")
            .map_err(InvokeError::CostingError)
    }
}

/// A `Nop` runtime accepts any external function calls by doing nothing and returning void.
pub struct NopWasmRuntime {
    fee_reserve: SystemLoanFeeReserve,
}

impl NopWasmRuntime {
    pub fn new(fee_reserve: SystemLoanFeeReserve) -> Self {
        Self { fee_reserve }
    }
}

impl WasmRuntime for NopWasmRuntime {
    fn main(&mut self, _input: ScryptoValue) -> Result<ScryptoValue, InvokeError> {
        Ok(ScryptoValue::unit())
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError> {
        self.fee_reserve
            .consume(n, "run_wasm")
            .map_err(InvokeError::CostingError)
    }
}
