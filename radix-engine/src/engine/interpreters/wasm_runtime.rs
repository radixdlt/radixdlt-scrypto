use crate::engine::*;
use crate::fee::*;
use crate::model::{invoke_call_table, InvokeError};
use crate::types::{scrypto_encode, *};
use crate::wasm::*;
use radix_engine_interface::api::wasm::*;
use radix_engine_interface::api::{ActorApi, ComponentApi, EngineApi, Invokable, InvokableModel};
use radix_engine_interface::data::ScryptoEncode;
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
        }
    }

    pub fn insert_buffer(&mut self, buffer: Vec<u8>) -> Buffer {
        let id = self.next_buffer_id;
        let len = buffer.len();
        self.buffers.insert(id, buffer);
        buffer!(id, len)
    }
}

fn encode<T: ScryptoEncode>(output: T) -> Result<Vec<u8>, InvokeError<WasmShimError>> {
    scrypto_encode(&output).map_err(|err| {
        InvokeError::Downstream(RuntimeError::KernelError(KernelError::SborEncodeError(err)))
    })
}

impl<'y, Y> WasmRuntime for RadixEngineWasmRuntime<'y, Y>
where
    Y: SystemApi
        + ComponentApi<RuntimeError>
        + EngineApi<RuntimeError>
        + InvokableModel<RuntimeError>
        + ActorApi<RuntimeError>,
{
    fn get_buffer(&mut self, buffer_id: BufferId) -> Result<&[u8], InvokeError<WasmShimError>> {
        self.buffers
            .get(&buffer_id)
            .map(|b| b.as_slice())
            .ok_or(InvokeError::Error(WasmShimError::BufferNotFound(buffer_id)))
    }

    fn invoke_method(
        &mut self,
        receiver: Vec<u8>,
        ident: String,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmShimError>> {
        let receiver = scrypto_decode::<ScryptoReceiver>(&receiver)
            .map_err(WasmShimError::InvalidReceiver)
            .map_err(InvokeError::Error)?;

        let return_data = self.api.invoke_method(receiver, ident.as_str(), args)?;

        Ok(self.insert_buffer(return_data))
    }

    fn invoke(&mut self, invocation: Vec<u8>) -> Result<Buffer, InvokeError<WasmShimError>> {
        let invocation = scrypto_decode::<CallTableInvocation>(&invocation)
            .map_err(WasmShimError::InvalidInvocation)
            .map_err(InvokeError::Error)?;

        let return_data = invoke_call_table(invocation, self.api)?.into_vec();

        Ok(self.insert_buffer(return_data))
    }

    fn create_node(&mut self, node: Vec<u8>) -> Result<Buffer, InvokeError<WasmShimError>> {
        let node = scrypto_decode::<ScryptoRENode>(&node)
            .map_err(WasmShimError::InvalidNode)
            .map_err(InvokeError::Error)?;

        let node_id = self.api.sys_create_node(node)?;

        let buffer = scrypto_encode(&node_id).expect("Failed to encode node id");
        Ok(self.insert_buffer(buffer))
    }

    fn get_visible_nodes(&mut self) -> Result<Buffer, InvokeError<WasmShimError>> {
        let node_ids = self.api.sys_get_visible_nodes()?;

        let buffer = scrypto_encode(&node_ids).expect("Failed to encode node id list");
        Ok(self.insert_buffer(buffer))
    }

    fn drop_node(&mut self, node_id: Vec<u8>) -> Result<(), InvokeError<WasmShimError>> {
        let node_id = scrypto_decode::<RENodeId>(&node_id)
            .map_err(WasmShimError::InvalidNodeId)
            .map_err(InvokeError::Error)?;

        Ok(())
    }

    fn lock_substate(
        &mut self,
        node_id: Vec<u8>,
        offset: Vec<u8>,
        mutable: bool,
    ) -> Result<u32, InvokeError<WasmShimError>> {
        let node_id = scrypto_decode::<RENodeId>(&node_id)
            .map_err(WasmShimError::InvalidNodeId)
            .map_err(InvokeError::Error)?;

        let offset = scrypto_decode::<SubstateOffset>(&offset)
            .map_err(WasmShimError::InvalidOffset)
            .map_err(InvokeError::Error)?;

        let handle = self.api.sys_lock_substate(node_id, offset, mutable)?;
        Ok(handle)
    }

    fn read_substate(&mut self, handle: u32) -> Result<Buffer, InvokeError<WasmShimError>> {
        let substate = self.api.sys_read(handle)?;

        Ok(self.insert_buffer(substate))
    }

    fn write_substate(
        &mut self,
        handle: u32,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmShimError>> {
        self.api.sys_write(handle, data)?;

        Ok(())
    }

    fn unlock_substate(&mut self, handle: u32) -> Result<(), InvokeError<WasmShimError>> {
        self.api.sys_drop_lock(handle)?;

        Ok(())
    }

    fn get_actor(&mut self) -> Result<Buffer, InvokeError<WasmShimError>> {
        let actor = self.api.fn_identifier()?;

        let buffer = scrypto_encode(&actor).expect("Failed to encode actor");
        Ok(self.insert_buffer(buffer))
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmShimError>> {
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

impl WasmRuntime for NopWasmRuntime {
    fn get_buffer(&mut self, buffer_id: BufferId) -> Result<&[u8], InvokeError<WasmShimError>> {
        Err(InvokeError::Error(WasmShimError::NotImplemented))
    }

    fn invoke_method(
        &mut self,
        receiver: Vec<u8>,
        ident: String,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmShimError>> {
        Err(InvokeError::Error(WasmShimError::NotImplemented))
    }

    fn invoke(&mut self, invocation: Vec<u8>) -> Result<Buffer, InvokeError<WasmShimError>> {
        Err(InvokeError::Error(WasmShimError::NotImplemented))
    }

    fn create_node(&mut self, node: Vec<u8>) -> Result<Buffer, InvokeError<WasmShimError>> {
        Err(InvokeError::Error(WasmShimError::NotImplemented))
    }

    fn get_visible_nodes(&mut self) -> Result<Buffer, InvokeError<WasmShimError>> {
        Err(InvokeError::Error(WasmShimError::NotImplemented))
    }

    fn drop_node(&mut self, node_id: Vec<u8>) -> Result<(), InvokeError<WasmShimError>> {
        Err(InvokeError::Error(WasmShimError::NotImplemented))
    }

    fn lock_substate(
        &mut self,
        node_id: Vec<u8>,
        offset: Vec<u8>,
        mutable: bool,
    ) -> Result<u32, InvokeError<WasmShimError>> {
        Err(InvokeError::Error(WasmShimError::NotImplemented))
    }

    fn read_substate(&mut self, handle: u32) -> Result<Buffer, InvokeError<WasmShimError>> {
        Err(InvokeError::Error(WasmShimError::NotImplemented))
    }

    fn write_substate(
        &mut self,
        handle: u32,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmShimError>> {
        Err(InvokeError::Error(WasmShimError::NotImplemented))
    }

    fn unlock_substate(&mut self, handle: u32) -> Result<(), InvokeError<WasmShimError>> {
        Err(InvokeError::Error(WasmShimError::NotImplemented))
    }

    fn get_actor(&mut self) -> Result<Buffer, InvokeError<WasmShimError>> {
        Err(InvokeError::Error(WasmShimError::NotImplemented))
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmShimError>> {
        self.fee_reserve
            .consume_execution(n, 1, "run_wasm", false)
            .map_err(|e| InvokeError::Error(WasmShimError::CostingError(e)))
    }
}
