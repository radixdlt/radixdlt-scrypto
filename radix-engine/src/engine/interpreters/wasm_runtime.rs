use crate::engine::*;
use crate::fee::*;
use crate::model::InvokeError;
use crate::types::{scrypto_decode, scrypto_encode, ScryptoInvocation};
use crate::wasm::*;
use radix_engine_interface::api::api::{EngineApi, Invokable, InvokableNative};
use radix_engine_interface::data::{IndexedScryptoValue, ScryptoEncode};
use radix_engine_interface::wasm::*;
use sbor::rust::vec::Vec;

/// A glue between system api (call frame and track abstraction) and WASM.
///
/// Execution is free from a costing perspective, as we assume
/// the system api will bill properly.
pub struct RadixEngineWasmRuntime<'y, Y>
where
    Y: SystemApi + EngineApi<RuntimeError> + Invokable<ScryptoInvocation, RuntimeError>,
{
    system_api: &'y mut Y,
}

impl<'y, Y> RadixEngineWasmRuntime<'y, Y>
where
    Y: SystemApi + EngineApi<RuntimeError> + Invokable<ScryptoInvocation, RuntimeError>,
{
    pub fn new(system_api: &'y mut Y) -> Self {
        RadixEngineWasmRuntime { system_api }
    }
}

fn encode<T: ScryptoEncode>(output: T) -> Result<Vec<u8>, InvokeError<WasmError>> {
    scrypto_encode(&output).map_err(|err| {
        InvokeError::Downstream(RuntimeError::KernelError(
            KernelError::InvalidSborValueOnEncode(err),
        ))
    })
}

impl<'y, Y> WasmRuntime for RadixEngineWasmRuntime<'y, Y>
where
    Y: SystemApi
        + EngineApi<RuntimeError>
        + Invokable<ScryptoInvocation, RuntimeError>
        + InvokableNative<RuntimeError>,
{
    // TODO: expose API for reading blobs
    // TODO: do we want to allow dynamic creation of blobs?
    // TODO: do we check existence of blobs when being passed as arguments/return?

    fn main(&mut self, input: IndexedScryptoValue) -> Result<Vec<u8>, InvokeError<WasmError>> {
        let input: RadixEngineInput = scrypto_decode(&input.raw)
            .map_err(|_| InvokeError::Error(WasmError::InvalidRadixEngineInput))?;
        let rtn = match input {
            RadixEngineInput::InvokeScrypto(invocation) => {
                self.system_api.invoke_scrypto(invocation)?
            }
            RadixEngineInput::InvokeNativeFn(native_fn) => {
                native_fn.invoke(self.system_api).map(|v| v.raw)?
            }
            RadixEngineInput::CreateNode(node) => encode(self.system_api.sys_create_node(node)?)?,
            RadixEngineInput::GetVisibleNodeIds() => {
                encode(self.system_api.sys_get_visible_nodes()?)?
            }
            RadixEngineInput::DropNode(node_id) => encode(self.system_api.sys_drop_node(node_id)?)?,
            RadixEngineInput::LockSubstate(node_id, offset, mutable) => encode(
                self.system_api
                    .sys_lock_substate(node_id, offset, mutable)?,
            )?,
            RadixEngineInput::Read(lock_handle) => self.system_api.sys_read(lock_handle)?,
            RadixEngineInput::Write(lock_handle, value) => {
                encode(self.system_api.sys_write(lock_handle, value)?)?
            }
            RadixEngineInput::DropLock(lock_handle) => {
                encode(self.system_api.sys_drop_lock(lock_handle)?)?
            }
            RadixEngineInput::GetActor() => encode(self.system_api.sys_get_actor()?)?,
            RadixEngineInput::GetTransactionHash() => {
                encode(self.system_api.sys_get_transaction_hash()?)?
            }
            RadixEngineInput::GenerateUuid() => encode(self.system_api.sys_generate_uuid()?)?,
            RadixEngineInput::EmitLog(level, message) => {
                encode(self.system_api.sys_emit_log(level, message)?)?
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
            .consume_execution(n, 1, "run_wasm", false)
            .map_err(|e| InvokeError::Error(WasmError::CostingError(e)))
    }
}
