use crate::errors::InvokeError;
use crate::errors::RuntimeError;
use crate::system::invocation::invoke::invoke_call_table;
use crate::system::kernel_modules::fee::*;
use crate::types::*;
use crate::wasm::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientMeteringApi;
use radix_engine_interface::api::ClientPackageApi;
use radix_engine_interface::api::{
    ClientActorApi, ClientComponentApi, ClientNodeApi, ClientStaticInvokeApi, ClientSubstateApi,
};
use sbor::rust::vec::Vec;

/// A shim between ClientApi and WASM, with buffer capability.
pub struct ScryptoRuntime<'y, Y>
where
    Y: ClientMeteringApi<RuntimeError>
        + ClientNodeApi<RuntimeError>
        + ClientSubstateApi<RuntimeError>
        + ClientPackageApi<RuntimeError>
        + ClientComponentApi<RuntimeError>
        + ClientActorApi<RuntimeError>
        + ClientStaticInvokeApi<RuntimeError>,
{
    api: &'y mut Y,
    buffers: BTreeMap<BufferId, Vec<u8>>,
    next_buffer_id: BufferId,
}

impl<'y, Y> ScryptoRuntime<'y, Y>
where
    Y: ClientMeteringApi<RuntimeError>
        + ClientNodeApi<RuntimeError>
        + ClientSubstateApi<RuntimeError>
        + ClientPackageApi<RuntimeError>
        + ClientComponentApi<RuntimeError>
        + ClientActorApi<RuntimeError>
        + ClientStaticInvokeApi<RuntimeError>,
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
    Y: ClientMeteringApi<RuntimeError>
        + ClientNodeApi<RuntimeError>
        + ClientSubstateApi<RuntimeError>
        + ClientPackageApi<RuntimeError>
        + ClientComponentApi<RuntimeError>
        + ClientActorApi<RuntimeError>
        + ClientStaticInvokeApi<RuntimeError>,
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

        self.allocate_buffer(return_data.into_vec())
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
                .call_function(package_address, blueprint_ident, function_ident, args)?;

        self.allocate_buffer(return_data.into_vec())
    }

    fn invoke(&mut self, invocation: Vec<u8>) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let invocation = scrypto_decode::<CallTableInvocation>(&invocation)
            .map_err(WasmRuntimeError::InvalidInvocation)?;

        let return_data = invoke_call_table(invocation, self.api)?.into_vec();

        self.allocate_buffer(return_data)
    }

    fn create_node(&mut self, node: Vec<u8>) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let node = scrypto_decode::<ScryptoRENode>(&node).map_err(WasmRuntimeError::InvalidNode)?;

        let node_id = self.api.sys_create_node(node)?;
        let node_id_encoded = scrypto_encode(&node_id).expect("Failed to encode node id");

        self.allocate_buffer(node_id_encoded)
    }

    fn get_visible_nodes(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let node_ids = self.api.sys_get_visible_nodes()?;
        let node_ids_encoded = scrypto_encode(&node_ids).expect("Failed to encode node id list");

        self.allocate_buffer(node_ids_encoded)
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
        let substate = self.api.sys_read(handle)?;

        self.allocate_buffer(substate)
    }

    fn write_substate(
        &mut self,
        handle: LockHandle,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.api.sys_write(handle, data)?;

        Ok(())
    }

    fn unlock_substate(&mut self, handle: LockHandle) -> Result<(), InvokeError<WasmRuntimeError>> {
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

    fn invoke(&mut self, invocation: Vec<u8>) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn create_node(&mut self, node: Vec<u8>) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn get_visible_nodes(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
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

    fn unlock_substate(&mut self, handle: u32) -> Result<(), InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn get_actor(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        Err(InvokeError::SelfError(WasmRuntimeError::NotImplemented))
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmRuntimeError>> {
        self.fee_reserve
            .consume_execution(n, CostingReason::RunWasm)
            .map_err(|e| InvokeError::SelfError(WasmRuntimeError::CostingError(e)))
    }
}
