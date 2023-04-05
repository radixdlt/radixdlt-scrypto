use crate::errors::InvokeError;
use crate::errors::RuntimeError;
use crate::system::kernel_modules::costing::*;
use crate::types::*;
use crate::wasm::*;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::AccessRule;
use radix_engine_interface::schema::KeyValueStoreSchema;
use radix_engine_interface::types::ClientCostingReason;
use radix_engine_interface::types::Level;
use sbor::rust::vec::Vec;

/// A shim between ClientApi and WASM, with buffer capability.
pub struct ScryptoRuntime<'y, Y>
where
    Y: ClientApi<RuntimeError>,
{
    api: &'y mut Y,
    buffers: BTreeMap<BufferId, Vec<u8>>,
    next_buffer_id: BufferId,
}

impl<'y, Y> ScryptoRuntime<'y, Y>
where
    Y: ClientApi<RuntimeError>,
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
    Y: ClientApi<RuntimeError>,
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
        let receiver = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(receiver.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let ident = String::from_utf8(ident).map_err(|_| WasmRuntimeError::InvalidString)?;

        let module_id = u8::try_from(module_id)
            .ok()
            .and_then(|x| TypedModuleId::from_repr(x))
            .ok_or(WasmRuntimeError::InvalidModuleId(module_id))?;

        let return_data =
            self.api
                .call_module_method(&receiver, module_id, ident.as_str(), args)?;

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
            String::from_utf8(blueprint_ident).map_err(|_| WasmRuntimeError::InvalidString)?;
        let function_ident =
            String::from_utf8(function_ident).map_err(|_| WasmRuntimeError::InvalidString)?;

        let return_data =
            self.api
                .call_function(package_address, &blueprint_ident, &function_ident, args)?;

        self.allocate_buffer(return_data)
    }

    fn new_object(
        &mut self,
        blueprint_ident: Vec<u8>,
        object_states: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let blueprint_ident =
            String::from_utf8(blueprint_ident).map_err(|_| WasmRuntimeError::InvalidString)?;
        let object_states = scrypto_decode::<Vec<Vec<u8>>>(&object_states)
            .map_err(WasmRuntimeError::InvalidAppStates)?;

        let component_id = self
            .api
            .new_object(blueprint_ident.as_ref(), object_states)?;
        let component_id_encoded =
            scrypto_encode(&component_id).expect("Failed to encode component id");

        self.allocate_buffer(component_id_encoded)
    }

    fn globalize_object(
        &mut self,
        component_id: Vec<u8>,
        modules: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let component_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(component_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let modules = scrypto_decode::<BTreeMap<TypedModuleId, NodeId>>(&modules)
            .map_err(WasmRuntimeError::InvalidModules)?;

        let component_address = self.api.globalize(component_id, modules)?;
        let component_address_encoded =
            scrypto_encode(&component_address).expect("Failed to encode component id");

        self.allocate_buffer(component_address_encoded)
    }

    fn new_key_value_store(
        &mut self,
        schema: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let schema = scrypto_decode::<KeyValueStoreSchema>(&schema)
            .map_err(WasmRuntimeError::InvalidKeyValueStoreSchema)?;

        let key_value_store_id = self.api.new_key_value_store(schema)?;
        let key_value_store_id_encoded =
            scrypto_encode(&key_value_store_id).expect("Failed to encode package address");

        self.allocate_buffer(key_value_store_id_encoded)
    }

    fn drop_object(&mut self, node_id: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>> {
        let node_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(node_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );

        self.api.drop_object(node_id)?;

        Ok(())
    }

    fn lock_substate(
        &mut self,
        node_id: Vec<u8>,
        substate_key: Vec<u8>,
        flags: u32,
    ) -> Result<LockHandle, InvokeError<WasmRuntimeError>> {
        let node_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(node_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let substate_key =
            SubstateKey::from_vec(substate_key).ok_or(WasmRuntimeError::InvalidSubstateKey)?;

        let flags = LockFlags::from_bits(flags).ok_or(WasmRuntimeError::InvalidLockFlags)?;
        let handle = self.api.sys_lock_substate(&node_id, &substate_key, flags)?;

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

    fn get_global_address(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let address = self.api.get_global_address()?;

        let buffer = scrypto_encode(&address).expect("Failed to encode address");
        self.allocate_buffer(buffer)
    }

    fn get_blueprint(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let actor = self.api.get_blueprint()?;

        let buffer = scrypto_encode(&actor).expect("Failed to encode actor");
        self.allocate_buffer(buffer)
    }

    fn get_auth_zone(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let auth_zone = self.api.get_auth_zone()?;

        let buffer = scrypto_encode(&auth_zone).expect("Failed to encode auth_zone");
        self.allocate_buffer(buffer)
    }

    fn assert_access_rule(&mut self, rule: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>> {
        let rule =
            scrypto_decode::<AccessRule>(&rule).map_err(WasmRuntimeError::InvalidAccessRules)?;

        self.api
            .assert_access_rule(rule)
            .map_err(InvokeError::downstream)
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api
            .consume_cost_units(n, ClientCostingReason::RunWasm)
            .map_err(InvokeError::downstream)
    }

    fn get_object_info(
        &mut self,
        node_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let node_id = NodeId(
            TryInto::<[u8; NodeId::LENGTH]>::try_into(node_id.as_ref())
                .map_err(|_| WasmRuntimeError::InvalidNodeId)?,
        );
        let type_info = self.api.get_object_info(&node_id)?;

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
        event_name: Vec<u8>,
        event: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.emit_event(
            String::from_utf8(event_name).map_err(|_| WasmRuntimeError::InvalidString)?,
            event,
        )?;
        Ok(())
    }

    fn log_message(
        &mut self,
        level: Vec<u8>,
        message: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.log_message(
            scrypto_decode::<Level>(&level).map_err(WasmRuntimeError::InvalidLogLevel)?,
            String::from_utf8(message).map_err(|_| WasmRuntimeError::InvalidString)?,
        )?;
        Ok(())
    }

    fn get_transaction_hash(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let hash = self.api.get_transaction_hash()?;

        self.allocate_buffer(scrypto_encode(&hash).expect("Failed to encode transaction hash"))
    }

    fn generate_uuid(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let uuid = self.api.generate_uuid()?;

        self.allocate_buffer(scrypto_encode(&uuid).expect("Failed to encode UUID"))
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
        module_id: u32,
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

    fn new_object(
        &mut self,
        blueprint_ident: Vec<u8>,
        object_states: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn globalize_object(
        &mut self,
        component_id: Vec<u8>,
        access_rules: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn new_key_value_store(
        &mut self,
        schema: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn drop_object(&mut self, node_id: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>> {
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

    fn get_global_address(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn get_blueprint(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn get_auth_zone(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.fee_reserve
            .consume_execution(n, CostingReason::RunWasm)
            .map_err(|e| InvokeError::SelfError(WasmRuntimeError::FeeReserveError(e)))
    }

    fn get_object_info(
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
        event_name: Vec<u8>,
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

    fn get_transaction_hash(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn generate_uuid(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn assert_access_rule(&mut self, rule: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }
}
