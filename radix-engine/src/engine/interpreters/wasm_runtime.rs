use crate::engine::*;
use crate::fee::*;
use crate::model::{invoke_call_table, InvokeError};
use crate::types::*;
use crate::wasm::*;
use radix_engine_interface::api::wasm::*;
use radix_engine_interface::api::{ActorApi, ComponentApi, EngineApi, Invokable, InvokableModel};
use radix_engine_interface::model::ScryptoInvocation;
use sbor::rust::vec::Vec;

/// A glue between system api (call frame and track abstraction) and WASM.
///
/// Execution is free from a costing perspective, as we assume
/// the system api will bill properly.
pub struct RadixEngineWasmRuntime<'y, Y>
where
    Y: SystemApi + EngineApi<RuntimeError> + Invokable<ScryptoInvocation, RuntimeError>,
{
    api: &'y mut Y,
    cost_units_buffer: u32,
    buffers: BTreeMap<BufferId, Vec<u8>>,
    next_buffer_id: BufferId,
}

impl<'y, Y> RadixEngineWasmRuntime<'y, Y>
where
    Y: SystemApi + EngineApi<RuntimeError> + Invokable<ScryptoInvocation, RuntimeError>,
{
    pub fn new(api: &'y mut Y) -> Self {
        RadixEngineWasmRuntime {
            api,
            buffers: BTreeMap::new(),
            next_buffer_id: 0,
            cost_units_buffer: 0,
        }
    }
}

impl<'y, Y> WasmRuntime for RadixEngineWasmRuntime<'y, Y>
where
    Y: SystemApi
        + ComponentApi<RuntimeError>
        + EngineApi<RuntimeError>
        + InvokableModel<RuntimeError>
        + ActorApi<RuntimeError>,
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

    fn invoke_method(
        &mut self,
        receiver: Vec<u8>,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>> {
        let receiver = scrypto_decode::<ScryptoReceiver>(&receiver)
            .map_err(WasmRuntimeError::InvalidReceiver)?;

        let ident = String::from_utf8(ident).map_err(|_| WasmRuntimeError::InvalidIdent)?;

        let return_data = self.api.invoke_method(receiver, ident.as_str(), args)?;

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

        self.api.drop_node(node_id)?;

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
        self.cost_units_buffer += n;
        // We buffer cost units to avoid the overhead of calling the fee module too often.
        // This also means you get up to 1000 free cost units at the end of a call frame
        if self.cost_units_buffer > 1000 {
            self.api
                .consume_cost_units(self.cost_units_buffer)
                .map_err(InvokeError::downstream)?;
            self.cost_units_buffer = 0;
        }
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

    fn invoke_method(
        &mut self,
        receiver: Vec<u8>,
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
            .consume_execution(n, "run_wasm")
            .map_err(|e| InvokeError::SelfError(WasmRuntimeError::CostingError(e)))
    }
}
