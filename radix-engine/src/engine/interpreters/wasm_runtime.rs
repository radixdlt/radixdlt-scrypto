use crate::engine::*;
use crate::fee::*;
use crate::model::InvokeError;
use crate::types::{scrypto_decode, scrypto_encode, Encode, ScryptoInvocation};
use crate::wasm::*;
use radix_engine_interface::api::api::{EngineApi, SysInvokableNative2};
use radix_engine_interface::data::{IndexedScryptoValue, ScryptoCustomTypeId};
use radix_engine_interface::wasm::*;
use sbor::rust::vec::Vec;

/// A glue between system api (call frame and track abstraction) and WASM.
///
/// Execution is free from a costing perspective, as we assume
/// the system api will bill properly.
pub struct RadixEngineWasmRuntime<'y, Y>
where
    Y: SystemApi + EngineApi<RuntimeError> + Invokable<ScryptoInvocation>,
{
    system_api: &'y mut Y,
}

impl<'y, Y> RadixEngineWasmRuntime<'y, Y>
where
    Y: SystemApi + EngineApi<RuntimeError> + Invokable<ScryptoInvocation>,
{
    pub fn new(system_api: &'y mut Y) -> Self {
        RadixEngineWasmRuntime { system_api }
    }
}

fn encode<T: Encode<ScryptoCustomTypeId>>(output: T) -> Vec<u8> {
    scrypto_encode(&output)
}

impl<'y, Y> WasmRuntime for RadixEngineWasmRuntime<'y, Y>
where
    Y: SystemApi
        + EngineApi<RuntimeError>
        + Invokable<ScryptoInvocation>
        + SysInvokableNative2<RuntimeError>,
{
    // TODO: expose API for reading blobs
    // TODO: do we want to allow dynamic creation of blobs?
    // TODO: do we check existence of blobs when being passed as arguments/return?

    fn main(&mut self, input: IndexedScryptoValue) -> Result<Vec<u8>, InvokeError<WasmError>> {
        let input: RadixEngineInput = scrypto_decode(&input.raw)
            .map_err(|_| InvokeError::Error(WasmError::InvalidRadixEngineInput))?;
        let rtn = match input {
            RadixEngineInput::InvokeScryptoFunction(function_ident, args) => self
                .system_api
                .sys_invoke_scrypto_function(function_ident, args)?,
            RadixEngineInput::InvokeScryptoMethod(method_ident, args) => self
                .system_api
                .sys_invoke_scrypto_method(method_ident, args)?,
            RadixEngineInput::InvokeNativeFn(native_fn) => {
                native_fn.invoke(self.system_api).map(|v| v.raw)?
            }
            RadixEngineInput::CreateNode(node) => {
                self.system_api.sys_create_node(node).map(encode)?
            }
            RadixEngineInput::GetVisibleNodeIds() => {
                self.system_api.sys_get_visible_nodes().map(encode)?
            }
            RadixEngineInput::DropNode(node_id) => {
                self.system_api.sys_drop_node(node_id).map(encode)?
            }
            RadixEngineInput::LockSubstate(node_id, offset, mutable) => self
                .system_api
                .sys_lock_substate(node_id, offset, mutable)
                .map(encode)?,
            RadixEngineInput::Read(lock_handle) => self.system_api.sys_read(lock_handle)?,
            RadixEngineInput::Write(lock_handle, value) => {
                self.system_api.sys_write(lock_handle, value).map(encode)?
            }
            RadixEngineInput::DropLock(lock_handle) => {
                self.system_api.sys_drop_lock(lock_handle).map(encode)?
            }
            RadixEngineInput::GetActor() => self.system_api.sys_get_actor().map(encode)?,
            RadixEngineInput::GetTransactionHash() => {
                self.system_api.sys_get_transaction_hash().map(encode)?
            }
            RadixEngineInput::GenerateUuid() => self.system_api.sys_generate_uuid().map(encode)?,
            RadixEngineInput::EmitLog(level, message) => {
                self.system_api.sys_emit_log(level, message).map(encode)?
            }
        };

        Ok(rtn)
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmError>> {
        self.system_api
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
    fn main(&mut self, _input: IndexedScryptoValue) -> Result<Vec<u8>, InvokeError<WasmError>> {
        Ok(IndexedScryptoValue::unit().raw)
    }

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmError>> {
        self.fee_reserve
            .consume_flat(n, "run_wasm", false)
            .map_err(|e| InvokeError::Error(WasmError::CostingError(e)))
    }
}
