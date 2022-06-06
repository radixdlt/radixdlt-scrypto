use sbor::rust::marker::PhantomData;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::core::SNodeRef;
use scrypto::core::ScryptoActorInfo;
use scrypto::engine::api::RadixEngineInput;
use scrypto::engine::types::*;
use scrypto::resource::AccessRule;
use scrypto::resource::AccessRules;
use scrypto::values::ScryptoValue;

use crate::engine::RuntimeError;
use crate::engine::SystemApi;
use crate::model::Component;
use crate::wasm::*;

pub struct RadixEngineWasmRuntime<'s, S, W, I>
where
    S: SystemApi<W, I>,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    this: ScryptoActorInfo,
    system_api: &'s mut S,
    tbd_limit: u32,
    tbd_balance: u32,
    phantom1: PhantomData<W>,
    phantom2: PhantomData<I>,
}

impl<'s, S, W, I> RadixEngineWasmRuntime<'s, S, W, I>
where
    S: SystemApi<W, I>,
    W: WasmEngine<I>,
    I: WasmInstance,
{
    pub fn new(this: ScryptoActorInfo, system_api: &'s mut S, tbd_limit: u32) -> Self {
        RadixEngineWasmRuntime {
            this,
            system_api,
            tbd_limit,
            tbd_balance: tbd_limit,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    pub fn tbd_used(&self) -> u32 {
        self.tbd_limit - self.tbd_balance
    }

    // FIXME: limit access to the API

    fn handle_invoke_snode(
        &mut self,
        snode_ref: SNodeRef,
        call_data: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let call_data =
            ScryptoValue::from_slice(&call_data).map_err(RuntimeError::ParseScryptoValueError)?;
        let result = self.system_api.invoke_snode(snode_ref, call_data)?;
        Ok(result.raw)
    }

    fn handle_invoke_snode2(
        &mut self,
        snode_ref: SNodeRef,
        method_name: String,
        call_data: Vec<u8>,
    ) -> Result<Vec<u8>, RuntimeError> {
        let call_data =
            ScryptoValue::from_slice(&call_data).map_err(RuntimeError::ParseScryptoValueError)?;
        let result = self
            .system_api
            .invoke_snode2(snode_ref, method_name, call_data)?;
        Ok(result.raw)
    }

    fn handle_create_component(
        &mut self,
        blueprint_name: String,
        state: Vec<u8>,
        access_rules_list: Vec<AccessRules>,
    ) -> Result<ComponentAddress, RuntimeError> {
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
        let value =
            ScryptoValue::from_slice(&state).map_err(RuntimeError::ParseScryptoValueError)?;
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
        let scrypto_key =
            ScryptoValue::from_slice(&key).map_err(RuntimeError::ParseScryptoValueError)?;
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
        let scrypto_key =
            ScryptoValue::from_slice(&key).map_err(RuntimeError::ParseScryptoValueError)?;
        let scrypto_value =
            ScryptoValue::from_slice(&value).map_err(RuntimeError::ParseScryptoValueError)?;
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
    ScryptoValue::from_value(&output)
}

impl<'s, S: SystemApi<W, I>, W: WasmEngine<I>, I: WasmInstance> WasmRuntime
    for RadixEngineWasmRuntime<'s, S, W, I>
{
    fn main(&mut self, input: ScryptoValue) -> Result<ScryptoValue, InvokeError> {
        let input: RadixEngineInput =
            scrypto_decode(&input.raw).map_err(|_| InvokeError::InvalidCallData)?;
        match input {
            RadixEngineInput::InvokeSNode(snode_ref, call_data) => {
                self.handle_invoke_snode(snode_ref, call_data).map(encode)
            }
            RadixEngineInput::InvokeSNode2(snode_ref, method_name, call_data) => self
                .handle_invoke_snode2(snode_ref, method_name, call_data)
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

    fn use_tbd(&mut self, tbd: u32) -> Result<(), InvokeError> {
        if self.tbd_balance >= tbd {
            self.tbd_balance -= tbd;
            Ok(())
        } else {
            self.tbd_balance = 0;
            Err(InvokeError::OutOfTbd {
                limit: self.tbd_limit,
                balance: self.tbd_balance,
                required: tbd,
            })
        }
    }
}
