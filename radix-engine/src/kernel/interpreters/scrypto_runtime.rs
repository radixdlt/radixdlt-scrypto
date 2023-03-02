use crate::errors::InvokeError;
use crate::errors::RuntimeError;
use crate::system::kernel_modules::costing::*;
use crate::types::*;
use crate::wasm::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::ClientEventApi;
use radix_engine_interface::api::ClientLoggerApi;
use radix_engine_interface::api::{
    ClientActorApi, ClientNodeApi, ClientObjectApi, ClientPackageApi, ClientSubstateApi,
    ClientUnsafeApi,
};
use radix_engine_interface::blueprints::logger::Level;
use radix_engine_interface::blueprints::resource::AccessRules;
use sbor::rust::vec::Vec;
use utils::copy_u8_array;

/// A shim between ClientApi and WASM, with buffer capability.
pub struct ScryptoRuntime<'y, Y>
where
    Y: ClientUnsafeApi<RuntimeError>
        + ClientNodeApi<RuntimeError>
        + ClientSubstateApi<RuntimeError>
        + ClientPackageApi<RuntimeError>
        + ClientObjectApi<RuntimeError>
        + ClientActorApi<RuntimeError>
        + ClientEventApi<RuntimeError>,
{
    api: &'y mut Y,
    buffers: BTreeMap<BufferId, Vec<u8>>,
    next_buffer_id: BufferId,
}

impl<'y, Y> ScryptoRuntime<'y, Y>
where
    Y: ClientUnsafeApi<RuntimeError>
        + ClientNodeApi<RuntimeError>
        + ClientSubstateApi<RuntimeError>
        + ClientPackageApi<RuntimeError>
        + ClientObjectApi<RuntimeError>
        + ClientActorApi<RuntimeError>
        + ClientEventApi<RuntimeError>,
{
    pub fn new(api: &'y mut Y) -> Self {
        ScryptoRuntime {
            api,
            buffers: BTreeMap::new(),
            next_buffer_id: 0,
        }
    }
}

impl<'y, Y> WasmRuntime for ScryptoRuntime<'y, Y>
where
    Y: ClientUnsafeApi<RuntimeError>
        + ClientNodeApi<RuntimeError>
        + ClientSubstateApi<RuntimeError>
        + ClientPackageApi<RuntimeError>
        + ClientObjectApi<RuntimeError>
        + ClientActorApi<RuntimeError>
        + ClientEventApi<RuntimeError>
        + ClientLoggerApi<RuntimeError>,
{
    fn allocate_buffer(
        &mut self,
        buffer: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        assert!(buffer.len() <= 0xffffffff);

        let id = self.next_buffer_id;
        let len = buffer.len();

        self.buffers.insert(id, buffer);
        self.next_buffer_id += 1;

        Ok(Buffer::new(id, len as u32))
    }

    fn consume_buffer(
        &mut self,
        buffer_id: BufferId,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
        self.buffers
            .remove(&buffer_id)
            .ok_or(InvokeError::SelfError(WasmRuntimeError::BufferNotFound(
                buffer_id,
            )))
    }

    fn call_method(
        &mut self,
        receiver: Vec<u8>,
        module_id: u32,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let receiver =
            scrypto_decode::<RENodeId>(&receiver).map_err(WasmRuntimeError::InvalidReceiver)?;

        let ident = String::from_utf8(ident).map_err(|_| WasmRuntimeError::InvalidIdent)?;

        let node_module_id = NodeModuleId::from_u32(module_id)
            .ok_or(WasmRuntimeError::InvalidModuleId(module_id))?;

        let return_data =
            self.api
                .call_module_method(receiver, node_module_id, ident.as_str(), args)?;

        self.allocate_buffer(return_data)
    }

    fn call_function(
        &mut self,
        package_address: Vec<u8>,
        blueprint_ident: Vec<u8>,
        function_ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let package_address = scrypto_decode::<PackageAddress>(&package_address)
            .map_err(WasmRuntimeError::InvalidPackageAddress)?;
        let blueprint_ident =
            String::from_utf8(blueprint_ident).map_err(|_| WasmRuntimeError::InvalidIdent)?;
        let function_ident =
            String::from_utf8(function_ident).map_err(|_| WasmRuntimeError::InvalidIdent)?;

        let return_data =
            self.api
                .call_function(package_address, &blueprint_ident, &function_ident, args)?;

        self.allocate_buffer(return_data)
    }

    fn new_package(
        &mut self,
        code: Vec<u8>,
        abi: Vec<u8>,
        access_rules: Vec<u8>,
        royalty_config: Vec<u8>,
        metadata: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let abi = scrypto_decode::<BTreeMap<String, BlueprintAbi>>(&abi)
            .map_err(WasmRuntimeError::InvalidAbi)?;
        let access_rules = scrypto_decode::<AccessRules>(&access_rules)
            .map_err(WasmRuntimeError::InvalidAccessRulesChain)?;
        let royalty_config = scrypto_decode::<BTreeMap<String, RoyaltyConfig>>(&royalty_config)
            .map_err(WasmRuntimeError::InvalidRoyaltyConfig)?;
        let metadata = scrypto_decode::<BTreeMap<String, String>>(&metadata)
            .map_err(WasmRuntimeError::InvalidMetadata)?;

        let package_address =
            self.api
                .new_package(code, abi, access_rules, royalty_config, metadata)?;
        let package_address_encoded =
            scrypto_encode(&package_address).expect("Failed to encode package address");

        self.allocate_buffer(package_address_encoded)
    }

    fn new_component(
        &mut self,
        blueprint_ident: Vec<u8>,
        app_states: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let blueprint_ident =
            String::from_utf8(blueprint_ident).map_err(|_| WasmRuntimeError::InvalidIdent)?;
        let app_states = scrypto_decode::<BTreeMap<u8, Vec<u8>>>(&app_states)
            .map_err(WasmRuntimeError::InvalidAppStates)?;

        let component_id = self.api.new_object(blueprint_ident.as_ref(), app_states)?;
        let component_id_encoded =
            scrypto_encode(&component_id).expect("Failed to encode component id");

        self.allocate_buffer(component_id_encoded)
    }

    fn globalize_component(
        &mut self,
        component_id: Vec<u8>,
        modules: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let component_id = scrypto_decode::<RENodeId>(&component_id)
            .map_err(WasmRuntimeError::InvalidComponentId)?;
        let modules = scrypto_decode::<BTreeMap<NodeModuleId, Vec<u8>>>(&modules)
            .map_err(WasmRuntimeError::InvalidValue)?;

        let component_address = self.api.globalize(component_id, modules)?;
        let component_address_encoded =
            scrypto_encode(&component_address).expect("Failed to encode component id");

        self.allocate_buffer(component_address_encoded)
    }

    fn new_key_value_store(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let key_value_store_id = self.api.new_key_value_store()?;
        let key_value_store_id_encoded =
            scrypto_encode(&key_value_store_id).expect("Failed to encode package address");

        self.allocate_buffer(key_value_store_id_encoded)
    }

    fn drop_node(&mut self, node_id: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>> {
        let node_id =
            scrypto_decode::<RENodeId>(&node_id).map_err(WasmRuntimeError::InvalidNodeId)?;

        self.api.sys_drop_node(node_id)?;

        Ok(())
    }

    fn lock_substate(
        &mut self,
        node_id: Vec<u8>,
        offset: Vec<u8>,
        flags: u32,
    ) -> Result<LockHandle, InvokeError<WasmRuntimeError>> {
        let node_id =
            scrypto_decode::<RENodeId>(&node_id).map_err(WasmRuntimeError::InvalidNodeId)?;
        let offset =
            scrypto_decode::<SubstateOffset>(&offset).map_err(WasmRuntimeError::InvalidOffset)?;

        let flags = LockFlags::from_bits(flags).ok_or(WasmRuntimeError::InvalidLockFlags)?;
        let handle = self.api.sys_lock_substate(node_id, offset, flags)?;

        Ok(handle)
    }

    fn read_substate(
        &mut self,
        handle: LockHandle,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let substate = self.api.sys_read_substate(handle)?;

        self.allocate_buffer(substate)
    }

    fn write_substate(
        &mut self,
        handle: LockHandle,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.sys_write_substate(handle, data)?;

        Ok(())
    }

    fn drop_lock(&mut self, handle: LockHandle) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.sys_drop_lock(handle)?;

        Ok(())
    }

    fn get_actor(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let actor = self.api.get_fn_identifier()?;

        let buffer = scrypto_encode(&actor).expect("Failed to encode actor");
        self.allocate_buffer(buffer)
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api
            .consume_cost_units(n, ClientCostingReason::RunWasm)
            .map_err(InvokeError::downstream)
    }

    fn get_component_type_info(
        &mut self,
        node_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let node_id =
            scrypto_decode::<RENodeId>(&node_id).map_err(WasmRuntimeError::InvalidNodeId)?;
        let type_info = self.api.get_object_type_info(node_id)?;

        let buffer = scrypto_encode(&type_info).expect("Failed to encode type_info");
        self.allocate_buffer(buffer)
    }

    fn update_wasm_memory_usage(
        &mut self,
        size: usize,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api
            .update_wasm_memory_usage(size)
            .map_err(InvokeError::downstream)
    }

    fn emit_event(
        &mut self,
        schema_hash: Vec<u8>,
        event: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api
            .emit_raw_event(Hash(copy_u8_array(&schema_hash)), event)?;
        Ok(())
    }

    fn log_message(
        &mut self,
        level: Vec<u8>,
        message: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.log_message(
            scrypto_decode::<Level>(&level).expect("Failed to decode level"),
            scrypto_decode::<String>(&message).expect("Failed to decode message"),
        )?;
        Ok(())
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

#[allow(unused_variables)]
impl WasmRuntime for NopWasmRuntime {
    fn allocate_buffer(
        &mut self,
        buffer: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn consume_buffer(
        &mut self,
        buffer_id: BufferId,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn call_method(
        &mut self,
        receiver: Vec<u8>,
        node_module_id: u32,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn call_function(
        &mut self,
        package_address: Vec<u8>,
        blueprint_ident: Vec<u8>,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn new_package(
        &mut self,
        code: Vec<u8>,
        abi: Vec<u8>,
        access_rules_chain: Vec<u8>,
        royalty_config: Vec<u8>,
        metadata: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn new_component(
        &mut self,
        blueprint_ident: Vec<u8>,
        app_states: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn globalize_component(
        &mut self,
        component_id: Vec<u8>,
        access_rules: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn new_key_value_store(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn drop_node(&mut self, node_id: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn lock_substate(
        &mut self,
        node_id: Vec<u8>,
        offset: Vec<u8>,
        flags: u32,
    ) -> Result<u32, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn read_substate(&mut self, handle: u32) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn write_substate(
        &mut self,
        handle: u32,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn drop_lock(&mut self, handle: u32) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn get_actor(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.fee_reserve
            .consume_execution(n, CostingReason::RunWasm)
            .map_err(|e| InvokeError::SelfError(WasmRuntimeError::FeeReserveError(e)))
    }

    fn get_component_type_info(
        &mut self,
        component_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn update_wasm_memory_usage(
        &mut self,
        size: usize,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn emit_event(
        &mut self,
        schema_hash: Vec<u8>,
        event: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn log_message(
        &mut self,
        level: Vec<u8>,
        message: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }
}
