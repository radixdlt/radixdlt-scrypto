use super::InstrumentedCode;
use crate::model::InvokeError;
use crate::wasm::errors::*;
use radix_engine_interface::api::types::LockHandle;
use radix_engine_interface::api::wasm::*;
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

    fn invoke_method(
        &mut self,
        receiver: Vec<u8>,
        ident: Vec<u8>,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn invoke(&mut self, invocation: Vec<u8>) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn create_node(&mut self, node: Vec<u8>) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

    fn get_visible_nodes(&mut self) -> Result<Buffer, InvokeError<WasmRuntimeError>>;

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
    fn invoke_export(
        &mut self,
        func_name: &str,
        args: Vec<Buffer>,
    ) -> Result<Vec<u8>, InvokeError<WasmRuntimeError>>;
}

pub trait TemplateWasmInstance {
    /// The lifetime parameter `'r` represents the lifetime of the &mut reference to the runtime.
    /// The lifetime parameter `'a` represents the lifetime of the runtime.
    type WasmInstance<'r, 'a: 'r>: WasmInstance;

    /// Install the runtime in a Scrypto module.
    fn install_runtime<'r, 'a: 'r>(
        self,
        runtime: &'r mut Box<dyn WasmRuntime + 'a>,
    ) -> Self::WasmInstance<'r, 'a>;
}

/// A Scrypto WASM engine validates, instruments and runs Scrypto modules.
pub trait WasmEngine {
    type TemplateWasmInstance: TemplateWasmInstance;

    /// Instantiate a Scrypto module.
    fn instantiate_template_instance(
        &'_ self,
        instrumented_code: &'_ InstrumentedCode,
    ) -> Self::TemplateWasmInstance;
}
