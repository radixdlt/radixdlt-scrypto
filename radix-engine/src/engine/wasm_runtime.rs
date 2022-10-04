use crate::engine::RuntimeError;
use crate::engine::{HeapRENode, SystemApi};
use crate::fee::*;
use crate::model::{
    Component, ComponentInfoSubstate, ComponentStateSubstate, HeapKeyValueStore, InvokeError,
};
use crate::types::*;
use crate::wasm::*;
use scrypto::core::FnIdent;

use super::KernelError;

/// A glue between system api (call frame and track abstraction) and WASM.
///
/// Execution is free from a costing perspective, as we assume
/// the system api will bill properly.
pub struct RadixEngineWasmRuntime<'y, 's, Y, W, I, R>
where
    Y: SystemApi<'s, W, I, R>,
    W: WasmEngine<I>,
    I: WasmInstance,
    R: FeeReserve,
{
    actor: ScryptoActor,
    system_api: &'y mut Y,
    phantom1: PhantomData<W>,
    phantom2: PhantomData<I>,
    phantom3: PhantomData<R>,
    phantom4: PhantomData<&'s ()>,
}

impl<'y, 's, Y, W, I, R> RadixEngineWasmRuntime<'y, 's, Y, W, I, R>
where
    Y: SystemApi<'s, W, I, R>,
    W: WasmEngine<I>,
    I: WasmInstance,
    R: FeeReserve,
{
    // TODO: expose API for reading blobs

    // TODO: do we want to allow dynamic creation of blobs?

    // TODO: do we check existence of blobs when being passed as arguments/return?

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

    // FIXME: limit access to the API
    fn handle_invoke(
        &mut self,
        fn_ident: FnIdent,
        input: Vec<u8>,
    ) -> Result<ScryptoValue, RuntimeError> {
        let call_data = ScryptoValue::from_slice(&input)
            .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
        self.system_api.invoke(fn_ident, call_data)
    }

    fn handle_node_create(
        &mut self,
        scrypto_node: ScryptoRENode,
    ) -> Result<ScryptoValue, RuntimeError> {
        let node = match scrypto_node {
            ScryptoRENode::Component(package_address, blueprint_name, state) => {
                // TODO: Move these two checks into kernel
                if !blueprint_name.eq(self.actor.blueprint_name()) {
                    return Err(RuntimeError::KernelError(
                        KernelError::RENodeCreateInvalidPermission,
                    ));
                }
                if !package_address.eq(self.actor.package_address()) {
                    return Err(RuntimeError::KernelError(
                        KernelError::RENodeCreateInvalidPermission,
                    ));
                }

                // TODO: Check state against blueprint schema

                // Create component
                HeapRENode::Component(Component {
                    info: ComponentInfoSubstate::new(package_address, blueprint_name, Vec::new()),
                    state: ComponentStateSubstate::new(state),
                })
            }
            ScryptoRENode::KeyValueStore => HeapRENode::KeyValueStore(HeapKeyValueStore::new()),
        };

        let id = self.system_api.node_create(node)?;
        Ok(ScryptoValue::from_typed(&id))
    }

    fn handle_get_owned_node_ids(&mut self) -> Result<ScryptoValue, RuntimeError> {
        let node_ids = self.system_api.get_owned_node_ids()?;
        Ok(ScryptoValue::from_typed(&node_ids))
    }

    // TODO: This logic should move into KeyValueEntry decoding
    fn verify_stored_key(value: &ScryptoValue) -> Result<(), RuntimeError> {
        if !value.bucket_ids.is_empty() {
            return Err(RuntimeError::KernelError(KernelError::BucketNotAllowed));
        }
        if !value.proof_ids.is_empty() {
            return Err(RuntimeError::KernelError(KernelError::ProofNotAllowed));
        }
        if !value.vault_ids.is_empty() {
            return Err(RuntimeError::KernelError(KernelError::VaultNotAllowed));
        }
        if !value.kv_store_ids.is_empty() {
            return Err(RuntimeError::KernelError(
                KernelError::KeyValueStoreNotAllowed,
            ));
        }
        Ok(())
    }

    fn handle_node_globalize(&mut self, node_id: RENodeId) -> Result<ScryptoValue, RuntimeError> {
        let global_address = self.system_api.node_globalize(node_id)?;
        Ok(ScryptoValue::from_typed(&global_address))
    }

    fn handle_substate_read(
        &mut self,
        substate_id: SubstateId,
    ) -> Result<ScryptoValue, RuntimeError> {
        match &substate_id {
            SubstateId(
                RENodeId::KeyValueStore(..),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key_bytes)),
            ) => {
                let key_data = ScryptoValue::from_slice(&key_bytes)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
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
            SubstateId(
                RENodeId::KeyValueStore(..),
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key_bytes)),
            ) => {
                let key_data = ScryptoValue::from_slice(&key_bytes)
                    .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
                Self::verify_stored_key(&key_data)?;
            }
            _ => {}
        }
        let scrypto_value = ScryptoValue::from_slice(&value)
            .map_err(|e| RuntimeError::KernelError(KernelError::DecodeError(e)))?;
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
}

fn encode<T: Encode>(output: T) -> ScryptoValue {
    ScryptoValue::from_typed(&output)
}

impl<'y, 's, Y, W, I, R> WasmRuntime for RadixEngineWasmRuntime<'y, 's, Y, W, I, R>
where
    Y: SystemApi<'s, W, I, R>,
    W: WasmEngine<I>,
    I: WasmInstance,
    R: FeeReserve,
{
    fn main(&mut self, input: ScryptoValue) -> Result<ScryptoValue, InvokeError<WasmError>> {
        let input: RadixEngineInput = scrypto_decode(&input.raw)
            .map_err(|_| InvokeError::Error(WasmError::InvalidRadixEngineInput))?;
        match input {
            RadixEngineInput::Invoke(fn_ident, input_bytes) => {
                self.handle_invoke(fn_ident, input_bytes)
            }
            RadixEngineInput::RENodeGlobalize(node_id) => self.handle_node_globalize(node_id),
            RadixEngineInput::RENodeCreate(node) => self.handle_node_create(node),
            RadixEngineInput::GetOwnedRENodeIds() => self.handle_get_owned_node_ids(),

            RadixEngineInput::SubstateRead(substate_id) => self.handle_substate_read(substate_id),
            RadixEngineInput::SubstateWrite(substate_id, value) => {
                self.handle_substate_write(substate_id, value)
            }

            RadixEngineInput::GetActor() => self.handle_get_actor().map(encode),
            RadixEngineInput::GenerateUuid() => self.handle_generate_uuid().map(encode),
            RadixEngineInput::EmitLog(level, message) => {
                self.handle_emit_log(level, message).map(encode)
            }
        }
        .map_err(InvokeError::downstream)
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmError>> {
        self.system_api
            .consume_cost_units(n)
            .map_err(InvokeError::downstream)
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
    fn main(&mut self, _input: ScryptoValue) -> Result<ScryptoValue, InvokeError<WasmError>> {
        Ok(ScryptoValue::unit())
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmError>> {
        self.fee_reserve
            .consume(n, "run_wasm", false)
            .map_err(|e| InvokeError::Error(WasmError::CostingError(e)))
    }
}
