use crate::engine::call_frame::{DataInstruction, SubstateAddress};
use sbor::rust::marker::PhantomData;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::core::ScryptoActorInfo;
use scrypto::core::{DataAddress, SNodeRef};
use scrypto::engine::api::RadixEngineInput;
use scrypto::engine::types::*;
use scrypto::resource::AccessRule;
use scrypto::values::ScryptoValue;

use crate::engine::SystemApi;
use crate::engine::{PreCommittedKeyValueStore, RuntimeError};
use crate::fee::*;
use crate::model::Component;
use crate::wasm::*;

/// A glue between system api (call frame and track abstraction) and WASM.
///
/// Execution is free from a cost perspective, as we assume
/// the system api will charge properly.
pub struct RadixEngineWasmRuntime<'borrowed, 's, S, W, I>
where
    S: SystemApi<'borrowed, W, I>,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    this: ScryptoActorInfo,
    system_api: &'s mut S,
    phantom1: PhantomData<W>,
    phantom2: PhantomData<I>,
    phantom3: PhantomData<&'borrowed ()>,
}

impl<'borrowed, 's, S, W, I> RadixEngineWasmRuntime<'borrowed, 's, S, W, I>
where
    S: SystemApi<'borrowed, W, I>,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    pub fn new(this: ScryptoActorInfo, system_api: &'s mut S) -> Self {
        RadixEngineWasmRuntime {
            this,
            system_api,
            phantom1: PhantomData,
            phantom2: PhantomData,
            phantom3: PhantomData,
        }
    }

    fn cost_unit_counter(&mut self) -> &mut CostUnitCounter {
        self.system_api.cost_unit_counter()
    }

    // FIXME: limit access to the API

    fn handle_invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        fn_ident: String,
        input: Vec<u8>,
    ) -> Result<ScryptoValue, RuntimeError> {
        let call_data = ScryptoValue::from_slice(&input).map_err(RuntimeError::DecodeError)?;
        self.system_api.invoke_snode(snode_ref, fn_ident, call_data)
    }

    fn handle_create_local_component(
        &mut self,
        blueprint_name: String,
        state: Vec<u8>,
    ) -> Result<ComponentAddress, RuntimeError> {
        // Create component
        let component = Component::new(
            self.this.package_address().clone(),
            blueprint_name,
            Vec::new(),
            state,
        );

        let id = self.system_api.native_create(component)?;
        Ok(id.into())
    }

    fn handle_create_kv_store(&mut self) -> Result<KeyValueStoreId, RuntimeError> {
        let value_id = self
            .system_api
            .native_create(PreCommittedKeyValueStore::new())?;
        match value_id {
            ValueId::Stored(StoredValueId::KeyValueStoreId(kv_store_id)) => Ok(kv_store_id),
            _ => panic!("Expected to be a kv store"),
        }
    }

    fn handle_read_data(&mut self, address: DataAddress) -> Result<ScryptoValue, RuntimeError> {
        let address = match address {
            DataAddress::KeyValueEntry(kv_store_id, key_bytes) => {
                let scrypto_key =
                    ScryptoValue::from_slice(&key_bytes).map_err(RuntimeError::DecodeError)?;
                SubstateAddress::KeyValueEntry(kv_store_id, scrypto_key)
            }
            DataAddress::Component(component_address, offset) => {
                SubstateAddress::Component(component_address, offset)
            }
        };

        self.system_api.data(address, DataInstruction::Read)
    }

    fn handle_write_data(
        &mut self,
        address: DataAddress,
        value: Vec<u8>,
    ) -> Result<ScryptoValue, RuntimeError> {
        let address = match address {
            DataAddress::KeyValueEntry(kv_store_id, key_bytes) => {
                let scrypto_key =
                    ScryptoValue::from_slice(&key_bytes).map_err(RuntimeError::DecodeError)?;
                SubstateAddress::KeyValueEntry(kv_store_id, scrypto_key)
            }
            DataAddress::Component(component_address, offset) => {
                SubstateAddress::Component(component_address, offset)
            }
        };
        let scrypto_value = ScryptoValue::from_slice(&value).map_err(RuntimeError::DecodeError)?;
        self.system_api
            .data(address, DataInstruction::Write(scrypto_value))
    }

    fn handle_get_actor(&mut self) -> Result<ScryptoActorInfo, RuntimeError> {
        return Ok(self.this.clone());
    }

    fn handle_generate_uuid(&mut self) -> Result<u128, RuntimeError> {
        let uuid = self
            .system_api
            .generate_uuid()
            .map_err(RuntimeError::CostingError)?;
        Ok(uuid)
    }

    fn handle_emit_log(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        self.system_api
            .emit_log(level, message)
            .map_err(RuntimeError::CostingError)?;
        Ok(())
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

impl<'borrowed, 's, S: SystemApi<'borrowed, W, I>, W: WasmEngine<I>, I: WasmInstance> WasmRuntime
    for RadixEngineWasmRuntime<'borrowed, 's, S, W, I>
{
    fn main(&mut self, input: ScryptoValue) -> Result<ScryptoValue, InvokeError> {
        let input: RadixEngineInput =
            scrypto_decode(&input.raw).map_err(|_| InvokeError::InvalidRadixEngineInput)?;
        match input {
            RadixEngineInput::InvokeSNode(snode_ref, fn_ident, input_bytes) => {
                self.handle_invoke_snode(snode_ref, fn_ident, input_bytes)
            }
            RadixEngineInput::CreateComponent(blueprint_name, state) => self
                .handle_create_local_component(blueprint_name, state)
                .map(encode),
            RadixEngineInput::CreateKeyValueStore() => self.handle_create_kv_store().map(encode),
            RadixEngineInput::GetActor() => self.handle_get_actor().map(encode),
            RadixEngineInput::ReadData(address) => self.handle_read_data(address),
            RadixEngineInput::WriteData(address, value) => self.handle_write_data(address, value),
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
        self.cost_unit_counter()
            .consume(n, "wasm")
            .map_err(InvokeError::CostingError)
    }
}

/// A `Nop` runtime accepts any external function calls by doing nothing and returning void.
pub struct NopWasmRuntime {
    cost_unit_counter: CostUnitCounter,
}

impl NopWasmRuntime {
    pub fn new(cost_unit_limit: u32) -> Self {
        Self {
            cost_unit_counter: CostUnitCounter::new(cost_unit_limit, cost_unit_limit),
        }
    }
}

impl WasmRuntime for NopWasmRuntime {
    fn main(&mut self, _input: ScryptoValue) -> Result<ScryptoValue, InvokeError> {
        Ok(ScryptoValue::unit())
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError> {
        self.cost_unit_counter
            .consume(n, "wasm")
            .map_err(InvokeError::CostingError)
    }
}
