use super::InstrumentedCode;
use crate::errors::InvokeError;
use crate::types::*;
use crate::wasm::errors::*;
use sbor::rust::boxed::Box;
use sbor::rust::vec::Vec;

/// Represents the runtime that can be invoked by Scrypto modules.
pub trait WasmRuntime {
    fn allocate_buffer(&mut self, buffer: Vec<u8>)
        -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn consume_buffer(
        &mut self,
        buffer_id: BufferId,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>>;

    fn call_method(
        &mut self,
        receiver: Vec<u8>,
        module_id: u32,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn call_function(
        &mut self,
        package_address: Vec<u8>,
        blueprint_ident: Vec<u8>,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn new_object(
        &mut self,
        blueprint_ident: Vec<u8>,
        object_states: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn globalize_object(
        &mut self,
        component_id: Vec<u8>,
        access_rules: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn new_key_value_store(
        &mut self,
        schema: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn get_object_info(
        &mut self,
        component_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn drop_object(&mut self, node_id: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn lock_substate(
        &mut self,
        node_id: Vec<u8>,
        offset: Vec<u8>,
        flags: u32,
    ) -> Result<LockHandle, InvokeError<WasmRuntimeError>>;

    fn read_substate(
        &mut self,
        handle: LockHandle,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn write_substate(
        &mut self,
        handle: LockHandle,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn drop_lock(&mut self, handle: LockHandle) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn get_global_address(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn get_blueprint(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn get_auth_zone(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn assert_access_rule(&mut self, rule: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn update_wasm_memory_usage(
        &mut self,
        size: usize,
    ) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn emit_event(
        &mut self,
        event_name: Vec<u8>,
        event: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn log_message(
        &mut self,
        level: Vec<u8>,
        message: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn get_transaction_hash(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn generate_uuid(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;
}

/// Represents an instantiated, invokable Scrypto module.
pub trait WasmInstance {
    /// Invokes an export defined in this module.
    ///
    /// The expected signature is as follows:
    /// - The input is a list of U64, each of which represents a `(BufferId, BufferLen)`.
    /// - The return data is U64, which represents a `(SlicePtr, SliceLen)`.
    ///
    /// The return data is copied into a `Vec<u8>`.
    fn invoke_export<'r>(
        &mut self,
        func_name: &str,
        args: Vec<Buffer>,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>>;

    /// Retruns memory consumed by this instance during invoke_export() call
    fn consumed_memory(&self) -> Result<usize, InvokeError<WasmRuntimeError>>;
}

/// A Scrypto WASM engine validates, instruments and runs Scrypto modules.
pub trait WasmEngine {
    type WasmInstance: WasmInstance;

    /// Instantiate a Scrypto module.
    fn instantiate(&self, instrumented_code: &InstrumentedCode) -> Self::WasmInstance;
}
