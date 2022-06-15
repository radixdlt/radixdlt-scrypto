use sbor::rust::marker::PhantomData;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::abi::BlueprintAbi;
use scrypto::buffer::scrypto_decode;
use scrypto::core::SNodeRef;
use scrypto::core::ScryptoActorInfo;
use scrypto::engine::api::RadixEngineInput;
use scrypto::engine::types::*;
use scrypto::resource::AccessRule;
use scrypto::resource::AccessRules;
use scrypto::values::ScryptoValue;

use crate::engine::RuntimeError;
use crate::engine::RuntimeError::BlueprintFunctionDoesNotExist;
use crate::engine::SystemApi;
use crate::fee::*;
use crate::model::Component;
use crate::wasm::*;

pub struct RadixEngineWasmRuntime<'s, S, W, I>
where
    S: SystemApi<W, I>,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    this: ScryptoActorInfo,
    blueprint_abi: BlueprintAbi,
    system_api: &'s mut S,
    phantom1: PhantomData<W>,
    phantom2: PhantomData<I>,
}

impl<'s, S, W, I> RadixEngineWasmRuntime<'s, S, W, I>
where
    S: SystemApi<W, I>,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    pub fn new(this: ScryptoActorInfo, blueprint_abi: BlueprintAbi, system_api: &'s mut S) -> Self {
        RadixEngineWasmRuntime {
            this,
            blueprint_abi,
            system_api,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    fn cost_unit_counter(&mut self) -> &mut CostUnitCounter {
        self.system_api.cost_unit_counter()
    }

    fn fee_table(&self) -> &FeeTable {
        self.system_api.fee_table()
    }

    // FIXME: limit access to the API

    fn handle_invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        fn_ident: String,
        input: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let call_data = ScryptoValue::from_slice(&input).map_err(RuntimeError::DecodeError)?;
        let result = self
            .system_api
            .invoke_snode(snode_ref, fn_ident, call_data)?;
        Ok(result.raw)
    }

    fn handle_create_component(
        &mut self,
        blueprint_name: String,
        state: Vec<u8>,
        access_rules_list: Vec<AccessRules>,
    ) -> Result<ComponentAddress, RuntimeError> {
        // Abi checks
        // TODO: Move this to a more appropriate place
        for access_rules in &access_rules_list {
            for (func_name, _) in access_rules.iter() {
                if !self.blueprint_abi.contains_fn(func_name.as_str()) {
                    return Err(BlueprintFunctionDoesNotExist(func_name.to_string()));
                }
            }
        }

        // Create component
        let component = Component::new(
            self.this.package_address().clone(),
            blueprint_name,
            access_rules_list,
            state,
        );
        let component_address = self.system_api.create_component(component)?;
        Ok(component_address)
    }

    fn handle_get_component_state(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<Vec<u8>, RuntimeError> {
        let state = self.system_api.read_component_state(component_address)?;
        Ok(state)
    }

    fn handle_put_component_state(
        &mut self,
        component_address: ComponentAddress,
        state: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let value = ScryptoValue::from_slice(&state).map_err(RuntimeError::DecodeError)?;
        self.system_api
            .write_component_state(component_address, value)?;
        Ok(())
    }

    fn handle_get_component_info(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<(PackageAddress, String), RuntimeError> {
        let (package_address, blueprint_name) =
            self.system_api.get_component_info(component_address)?;
        Ok((package_address, blueprint_name))
    }

    fn handle_create_kv_store(&mut self) -> Result<KeyValueStoreId, RuntimeError> {
        let kv_store_id = self.system_api.create_kv_store();
        Ok(kv_store_id)
    }

    fn handle_get_kv_store_entry(
        &mut self,
        kv_store_id: KeyValueStoreId,
        key: Vec<u8>,
    ) -> Result<ScryptoValue, RuntimeError> {
        let scrypto_key = ScryptoValue::from_slice(&key).map_err(RuntimeError::DecodeError)?;
        let value = self
            .system_api
            .read_kv_store_entry(kv_store_id, scrypto_key)?;
        Ok(value)
    }

    fn handle_put_kv_store_entry(
        &mut self,
        kv_store_id: KeyValueStoreId,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), RuntimeError> {
        let scrypto_key = ScryptoValue::from_slice(&key).map_err(RuntimeError::DecodeError)?;
        let scrypto_value = ScryptoValue::from_slice(&value).map_err(RuntimeError::DecodeError)?;
        self.system_api
            .write_kv_store_entry(kv_store_id, scrypto_key, scrypto_value)?;
        Ok(())
    }

    fn handle_get_actor(&mut self) -> Result<ScryptoActorInfo, RuntimeError> {
        return Ok(self.this.clone());
    }

    fn handle_generate_uuid(&mut self) -> Result<u128, RuntimeError> {
        let uuid = self.system_api.generate_uuid();
        Ok(uuid)
    }

    fn handle_user_log(&mut self, level: Level, message: String) -> Result<(), RuntimeError> {
        self.system_api.user_log(level, message);
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

impl<'s, S: SystemApi<W, I>, W: WasmEngine<I>, I: WasmInstance> WasmRuntime
    for RadixEngineWasmRuntime<'s, S, W, I>
{
    fn main(&mut self, input: ScryptoValue) -> Result<ScryptoValue, InvokeError> {
        let cost = self.fee_table().wasm_engine_call_cost();
        self.consume_cost_units(cost)?;

        let input: RadixEngineInput =
            scrypto_decode(&input.raw).map_err(|_| InvokeError::InvalidCallData)?;
        match input {
            RadixEngineInput::InvokeSNode(snode_ref, fn_ident, input_bytes) => self
                .handle_invoke_snode(snode_ref, fn_ident, input_bytes)
                .map(encode),
            RadixEngineInput::CreateComponent(blueprint_name, state, access_rules_list) => self
                .handle_create_component(blueprint_name, state, access_rules_list)
                .map(encode),
            RadixEngineInput::GetComponentInfo(component_address) => self
                .handle_get_component_info(component_address)
                .map(encode),
            RadixEngineInput::GetComponentState(component_address) => self
                .handle_get_component_state(component_address)
                .map(encode),
            RadixEngineInput::PutComponentState(component_address, state) => self
                .handle_put_component_state(component_address, state)
                .map(encode),
            RadixEngineInput::CreateKeyValueStore() => self.handle_create_kv_store().map(encode),
            RadixEngineInput::GetKeyValueStoreEntry(kv_store_id, key) => {
                self.handle_get_kv_store_entry(kv_store_id, key)
            }
            RadixEngineInput::PutKeyValueStoreEntry(kv_store_id, key, value) => self
                .handle_put_kv_store_entry(kv_store_id, key, value)
                .map(encode),
            RadixEngineInput::GetActor() => self.handle_get_actor().map(encode),
            RadixEngineInput::GenerateUuid() => self.handle_generate_uuid().map(encode),
            RadixEngineInput::EmitLog(level, message) => {
                self.handle_user_log(level, message).map(encode)
            }
            RadixEngineInput::CheckAccessRule(rule, proof_ids) => {
                self.handle_check_access_rule(rule, proof_ids).map(encode)
            }
        }
        .map_err(InvokeError::RuntimeError)
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError> {
        self.cost_unit_counter()
            .consume(n)
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
            .consume(n)
            .map_err(InvokeError::CostingError)
    }
}
