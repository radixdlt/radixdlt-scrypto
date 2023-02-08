use crate::errors::InvokeError;
use crate::errors::RuntimeError;
use crate::system::kernel_modules::costing::*;
use crate::types::*;
use crate::wasm::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::{
    ClientActorApi, ClientComponentApi, ClientEventApi, ClientNativeInvokeApi, ClientNodeApi,
    ClientPackageApi, ClientSubstateApi,
};
use radix_engine_interface::blueprints::resource::AccessRules;
use sbor::rust::vec::Vec;

/// A shim between ClientApi and WASM, with buffer capability.
pub struct ScryptoRuntime<'y, Y>
where
    Y: ClientEventApi<RuntimeError>
        + ClientNodeApi<RuntimeError>
        + ClientSubstateApi<RuntimeError>
        + ClientPackageApi<RuntimeError>
        + ClientComponentApi<RuntimeError>
        + ClientActorApi<RuntimeError>
        + ClientNativeInvokeApi<RuntimeError>,
{
    api: &'y mut Y,
    buffers: BTreeMap<BufferId, Vec<u8>>,
    next_buffer_id: BufferId,
}

impl<'y, Y> ScryptoRuntime<'y, Y>
where
    Y: ClientEventApi<RuntimeError>
        + ClientNodeApi<RuntimeError>
        + ClientSubstateApi<RuntimeError>
        + ClientPackageApi<RuntimeError>
        + ClientComponentApi<RuntimeError>
        + ClientActorApi<RuntimeError>
        + ClientNativeInvokeApi<RuntimeError>,
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
    Y: ClientEventApi<RuntimeError>
        + ClientNodeApi<RuntimeError>
        + ClientSubstateApi<RuntimeError>
        + ClientPackageApi<RuntimeError>
        + ClientComponentApi<RuntimeError>
        + ClientActorApi<RuntimeError>
        + ClientNativeInvokeApi<RuntimeError>,
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
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let receiver = scrypto_decode::<ScryptoReceiver>(&receiver)
            .map_err(WasmRuntimeError::InvalidReceiver)?;

        let ident = String::from_utf8(ident).map_err(|_| WasmRuntimeError::InvalidIdent)?;

        let return_data = self.api.call_method(receiver, ident.as_str(), args)?;

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

    fn call_native(
        &mut self,
        native_fn: Vec<u8>,
        invocation: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let native_fn = scrypto_decode::<NativeFn>(&native_fn)
            .map_err(WasmRuntimeError::InvalidNativeFnIdentifier)?;

        let return_data = self.api.call_native_raw(native_fn, invocation)?;

        self.allocate_buffer(return_data)
    }

    fn new_package(
        &mut self,
        code: Vec<u8>,
        abi: Vec<u8>,
        access_rules_chain: Vec<u8>,
        royalty_config: Vec<u8>,
        metadata: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let access_rules_chain = scrypto_decode::<Vec<AccessRules>>(&access_rules_chain)
            .map_err(WasmRuntimeError::InvalidAccessRulesChain)?;
        let royalty_config = scrypto_decode::<BTreeMap<String, RoyaltyConfig>>(&royalty_config)
            .map_err(WasmRuntimeError::InvalidRoyaltyConfig)?;
        let metadata = scrypto_decode::<BTreeMap<String, String>>(&metadata)
            .map_err(WasmRuntimeError::InvalidMetadata)?;

        let package_address =
            self.api
                .new_package(code, abi, access_rules_chain, royalty_config, metadata)?;
        let package_address_encoded =
            scrypto_encode(&package_address).expect("Failed to encode package address");

        self.allocate_buffer(package_address_encoded)
    }

    fn new_component(
        &mut self,
        blueprint_ident: Vec<u8>,
        app_states: Vec<u8>,
        access_rules_chain: Vec<u8>,
        royalty_config: Vec<u8>,
        metadata: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let blueprint_ident =
            String::from_utf8(blueprint_ident).map_err(|_| WasmRuntimeError::InvalidIdent)?;
        let app_states = scrypto_decode::<BTreeMap<u8, Vec<u8>>>(&app_states)
            .map_err(WasmRuntimeError::InvalidAppStates)?;
        let access_rules_chain = scrypto_decode::<Vec<AccessRules>>(&access_rules_chain)
            .map_err(WasmRuntimeError::InvalidAccessRulesChain)?;
        let royalty_config = scrypto_decode::<RoyaltyConfig>(&royalty_config)
            .map_err(WasmRuntimeError::InvalidRoyaltyConfig)?;
        let metadata = scrypto_decode::<BTreeMap<String, String>>(&metadata)
            .map_err(WasmRuntimeError::InvalidMetadata)?;

        let component_id = self.api.new_component(
            blueprint_ident.as_ref(),
            app_states,
            access_rules_chain,
            royalty_config,
            metadata,
        )?;
        let component_id_encoded =
            scrypto_encode(&component_id).expect("Failed to encode component id");

        self.allocate_buffer(component_id_encoded)
    }

    fn globalize_component(
        &mut self,
        component_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let component_id = scrypto_decode::<ComponentId>(&component_id)
            .map_err(WasmRuntimeError::InvalidComponentId)?;

        let component_address = self.api.globalize_component(component_id)?;
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
        mutable: bool,
    ) -> Result<LockHandle, InvokeError<WasmRuntimeError>> {
        let node_id =
            scrypto_decode::<RENodeId>(&node_id).map_err(WasmRuntimeError::InvalidNodeId)?;
        let offset =
            scrypto_decode::<SubstateOffset>(&offset).map_err(WasmRuntimeError::InvalidOffset)?;

        let handle = self.api.sys_lock_substate(node_id, offset, mutable)?;
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
        let actor = self.api.fn_identifier()?;

        let buffer = scrypto_encode(&actor).expect("Failed to encode actor");
        self.allocate_buffer(buffer)
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api
            .consume_cost_units(n)
            .map_err(InvokeError::downstream)
    }

    fn lookup_global_component(
        &mut self,
        component_address: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let component_address = scrypto_decode::<ComponentAddress>(&component_address)
            .map_err(WasmRuntimeError::InvalidComponentAddress)?;
        let component_id = self.api.lookup_global_component(component_address)?;

        let buffer = scrypto_encode(&component_id).expect("Failed to encode component_id");
        self.allocate_buffer(buffer)
    }

    fn get_component_type_info(
        &mut self,
        component_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let component_id = scrypto_decode::<ComponentId>(&component_id)
            .map_err(WasmRuntimeError::InvalidComponentAddress)?;
        let type_info = self.api.get_component_type_info(component_id)?;

        let buffer = scrypto_encode(&type_info).expect("Failed to encode type_info");
        self.allocate_buffer(buffer)
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

    fn call_native(
        &mut self,
        native_fn_identifier: Vec<u8>,
        invocation: Vec<u8>,
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
        access_rules_chain: Vec<u8>,
        royalty_config: Vec<u8>,
        metadata: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn globalize_component(
        &mut self,
        component_id: Vec<u8>,
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
        mutable: bool,
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

    fn lookup_global_component(
        &mut self,
        component_address: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn get_component_type_info(
        &mut self,
        component_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }
}
