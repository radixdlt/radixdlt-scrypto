use super::InstrumentedCode;
use crate::errors::InvokeError;
use crate::wasm::errors::*;
use radix_engine_interface::api::types::*;
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

    fn call_native(
        &mut self,
        native_fn_identifier: Vec<u8>,
        invocation: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn create_node(&mut self, node: Vec<u8>) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn drop_node(&mut self, node_id: Vec<u8>) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn lock_substate(
        &mut self,
        node_id: Vec<u8>,
        offset: Vec<u8>,
        mutable: bool,
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

    fn unlock_substate(&mut self, handle: LockHandle) -> Result<(), InvokeError<WasmRuntimeError>>;

    fn get_actor(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmRuntimeError>>;
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
}

/// A Scrypto WASM engine validates, instruments and runs Scrypto modules.
pub trait WasmEngine {
    type WasmInstance: WasmInstance;

    /// Instantiate a Scrypto module.
    fn instantiate(&self, instrumented_code: &InstrumentedCode) -> Self::WasmInstance;
}
