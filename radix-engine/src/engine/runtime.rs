use sbor::rust::str::FromStr;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::core::ScryptoActorInfo;
use scrypto::engine::api::*;
use scrypto::values::ScryptoValue;

use crate::engine::RuntimeError;
use crate::engine::SystemApi;
use crate::model::Component;
use crate::wasm::{InvokeError, ScryptoRuntime};

pub struct RadixEngineScryptoRuntime<'a, S: SystemApi> {
    this: ScryptoActorInfo,
    call_data: ScryptoValue, // TODO: remove this
    system_api: &'a mut S,
}

impl<'a, S: SystemApi> RadixEngineScryptoRuntime<'a, S> {
    pub fn new(this: ScryptoActorInfo, call_data: ScryptoValue, system_api: &'a mut S) -> Self {
        RadixEngineScryptoRuntime {
            this,
            call_data,
            system_api,
        }
    }

    // FIXME: limit access to the API

    fn handle_get_call_data(
        &mut self,
        _input: GetCallDataInput,
    ) -> Result<GetCallDataOutput, RuntimeError> {
        Ok(GetCallDataOutput {
            component: self.this.component_address(),
            call_data: self.call_data.raw.clone(),
        })
    }

    fn handle_create_component(
        &mut self,
        input: CreateComponentInput,
    ) -> Result<CreateComponentOutput, RuntimeError> {
        let component = Component::new(
            self.this.package_address().clone(),
            input.blueprint_name,
            input.access_rules_list,
            input.state,
        );
        let component_address = self.system_api.create_component(component)?;
        Ok(CreateComponentOutput { component_address })
    }

    fn handle_get_component_state(
        &mut self,
        input: GetComponentStateInput,
    ) -> Result<GetComponentStateOutput, RuntimeError> {
        let state = self
            .system_api
            .read_component_state(input.component_address)?;
        Ok(GetComponentStateOutput { state })
    }

    fn handle_put_component_state(
        &mut self,
        input: PutComponentStateInput,
    ) -> Result<PutComponentStateOutput, RuntimeError> {
        self.system_api
            .write_component_state(input.component_address, input.state)?;
        Ok(PutComponentStateOutput {})
    }

    fn handle_get_component_info(
        &mut self,
        input: GetComponentInfoInput,
    ) -> Result<GetComponentInfoOutput, RuntimeError> {
        let (package_address, blueprint_name) = self
            .system_api
            .get_component_info(input.component_address)?;
        Ok(GetComponentInfoOutput {
            package_address,
            blueprint_name,
        })
    }

    fn handle_create_lazy_map(
        &mut self,
        _input: CreateLazyMapInput,
    ) -> Result<CreateLazyMapOutput, RuntimeError> {
        let lazy_map_id = self.system_api.create_lazy_map();
        Ok(CreateLazyMapOutput { lazy_map_id })
    }

    fn handle_get_lazy_map_entry(
        &mut self,
        input: GetLazyMapEntryInput,
    ) -> Result<GetLazyMapEntryOutput, RuntimeError> {
        let value = self
            .system_api
            .read_lazy_map_entry(input.lazy_map_id, input.key)?;
        Ok(GetLazyMapEntryOutput { value })
    }

    fn handle_put_lazy_map_entry(
        &mut self,
        input: PutLazyMapEntryInput,
    ) -> Result<PutLazyMapEntryOutput, RuntimeError> {
        self.system_api
            .write_lazy_map_entry(input.lazy_map_id, input.key, input.value)?;
        Ok(PutLazyMapEntryOutput {})
    }

    fn handle_get_actor(&mut self, _input: GetActorInput) -> Result<GetActorOutput, RuntimeError> {
        return Ok(GetActorOutput {
            actor: self.this.clone(),
        });
    }

    fn handle_invoke_snode(
        &mut self,
        input: InvokeSNodeInput,
    ) -> Result<InvokeSNodeOutput, RuntimeError> {
        let call_data = ScryptoValue::from_slice(&input.call_data)
            .map_err(RuntimeError::ParseScryptoValueError)?;
        let result = self.system_api.invoke_snode(input.snode_ref, call_data)?;
        Ok(InvokeSNodeOutput { rtn: result.raw })
    }

    fn handle_generate_uuid(
        &mut self,
        _input: GenerateUuidInput,
    ) -> Result<GenerateUuidOutput, RuntimeError> {
        let uuid = self.system_api.generate_uuid();
        Ok(GenerateUuidOutput { uuid })
    }

    fn handle_emit_log(&mut self, input: EmitLogInput) -> Result<EmitLogOutput, RuntimeError> {
        self.system_api.emit_log(input.level, input.message);
        Ok(EmitLogOutput {})
    }
}

// TODO: Remove this temporary solutions once wasm ABI is stable.
fn decode<T: Decode>(args: &[ScryptoValue]) -> T {
    scrypto_decode(&args[0].raw).unwrap()
}

// TODO: Remove this temporary solutions once wasm ABI is stable.
fn encode<T: Encode>(output: T) -> ScryptoValue {
    ScryptoValue::from_value(&output)
}

impl<'a, S: SystemApi> ScryptoRuntime for RadixEngineScryptoRuntime<'a, S> {
    fn main(&mut self, name: &str, args: &[ScryptoValue]) -> Result<ScryptoValue, InvokeError> {
        let code = u32::from_str(name).unwrap(); // FIXME: update method name
        match code {
            INVOKE_SNODE => self
                .handle_invoke_snode(decode::<InvokeSNodeInput>(args))
                .map(encode),
            GET_CALL_DATA => self
                .handle_get_call_data(decode::<GetCallDataInput>(args))
                .map(encode),
            CREATE_COMPONENT => self
                .handle_create_component(decode::<CreateComponentInput>(args))
                .map(encode),
            GET_COMPONENT_INFO => self
                .handle_get_component_info(decode::<GetComponentInfoInput>(args))
                .map(encode),
            GET_COMPONENT_STATE => self
                .handle_get_component_state(decode::<GetComponentStateInput>(args))
                .map(encode),
            PUT_COMPONENT_STATE => self
                .handle_put_component_state(decode::<PutComponentStateInput>(args))
                .map(encode),
            CREATE_LAZY_MAP => self
                .handle_create_lazy_map(decode::<CreateLazyMapInput>(args))
                .map(encode),
            GET_LAZY_MAP_ENTRY => self
                .handle_get_lazy_map_entry(decode::<GetLazyMapEntryInput>(args))
                .map(encode),
            PUT_LAZY_MAP_ENTRY => self
                .handle_put_lazy_map_entry(decode::<PutLazyMapEntryInput>(args))
                .map(encode),
            GET_ACTOR => self
                .handle_get_actor(decode::<GetActorInput>(args))
                .map(encode),
            GENERATE_UUID => self
                .handle_generate_uuid(decode::<GenerateUuidInput>(args))
                .map(encode),
            EMIT_LOG => self
                .handle_emit_log(decode::<EmitLogInput>(args))
                .map(encode),
            _ => Err(RuntimeError::UnknownMethod(name.to_string())),
        }
        .map_err(InvokeError::RuntimeError)
    }
}
