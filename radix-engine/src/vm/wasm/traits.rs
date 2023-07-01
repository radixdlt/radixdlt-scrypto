use crate::errors::InvokeError;
use crate::types::*;
use crate::vm::wasm::errors::*;
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

    fn actor_call_module_method(
        &mut self,
        object_handle: u32,
        module_id: u32,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn call_method(
        &mut self,
        receiver: Vec<u8>,
        direct_access: u32,
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

    fn allocate_global_address(
        &mut self,
        blueprint_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn globalize_object(
        &mut self,
        modules: Vec<u8>,
        address: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn key_value_store_new(
        &mut self,
        schema: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn key_value_store_open_entry(
        &mut self,
        node_id: Vec<u8>,
        key: Vec<u8>,
        flags: u32,
    ) -> Result<LockHandle, InvokeError<WasmRuntimeError>>;

    fn key_value_entry_get(&mut self, handle: u32)
        -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn key_value_entry_set(
        &mut self,
        handle: u32,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn key_value_entry_release(&mut self, handle: u32)
        -> Result<(), InvokeError<WasmRuntimeError>>;

    fn key_value_store_remove_entry(
        &mut self,
        node_id: Vec<u8>,
        key: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn get_object_info(
        &mut self,
        component_id: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn drop_object(&mut self, node_id: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn actor_open_field(
        &mut self,
        object_handle: u32,
        field: u8,
        flags: u32,
    ) -> Result<LockHandle, InvokeError<WasmRuntimeError>>;

    fn field_lock_read(
        &mut self,
        handle: LockHandle,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn field_lock_write(
        &mut self,
        handle: LockHandle,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn field_lock_release(
        &mut self,
        handle: LockHandle,
    ) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn get_node_id(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn get_global_address(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn get_blueprint(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn get_auth_zone(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn assert_access_rule(&mut self, rule: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn consume_wasm_execution_units(&mut self, n: u32)
        -> Result<(), InvokeError<WasmRuntimeError>>;

    fn cost_unit_limit(&mut self) -> Result<u32, InvokeError<WasmRuntimeError>>;

    fn cost_unit_price(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn tip_percentage(&mut self) -> Result<u32, InvokeError<WasmRuntimeError>>;

    fn fee_balance(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn emit_event(
        &mut self,
        event_name: Vec<u8>,
        event: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn emit_log(
        &mut self,
        level: Vec<u8>,
        message: Vec<u8>,
    ) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn panic(&mut self, message: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn get_transaction_hash(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn generate_ruid(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;
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

    /// Returns memory consumed by this instance during invoke_export() call
    fn consumed_memory(&self) -> Result<usize, InvokeError<WasmRuntimeError>>;
}

/// A Scrypto WASM engine validates, instruments and runs Scrypto modules.
pub trait WasmEngine {
    type WasmInstance: WasmInstance;

    /// Instantiate a Scrypto module.
    ///
    /// The code must have been validated and instrumented!!!
    fn instantiate(&self, code_hash: Hash, instrumented_code: &[u8]) -> Self::WasmInstance;
}
