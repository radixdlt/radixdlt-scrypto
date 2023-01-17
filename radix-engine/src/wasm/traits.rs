use super::InstrumentedCode;
use crate::model::InvokeError;
use crate::wasm::errors::*;
use radix_engine_interface::api::wasm::*;
use radix_engine_interface::data::IndexedScryptoValue;
use sbor::rust::boxed::Box;
use sbor::rust::vec::Vec;

/// Represents the runtime that can be invoked by Scrypto modules.
pub trait WasmRuntime {
    fn get_buffer(&mut self, buffer_id: BufferId) -> Result<&[u8], InvokeError<WasmShimError>>;

    fn invoke_method(
        &mut self,
        receiver: Vec<u8>,
        ident: String,
        args: Vec<u8>,
    ) -> Result<Buffer, InvokeError<WasmShimError>>;

    fn invoke(&mut self, invocation: Vec<u8>) -> Result<Buffer, InvokeError<WasmShimError>>;

    fn create_node(&mut self, node: Vec<u8>) -> Result<Buffer, InvokeError<WasmShimError>>;

    fn get_visible_node_ids(&mut self) -> Result<Buffer, InvokeError<WasmShimError>>;

    fn drop_node(&mut self, node_id: Vec<u8>) -> Result<(), InvokeError<WasmShimError>>;

    fn lock_substate(
        &mut self,
        node_id: Vec<u8>,
        offset: Vec<u8>,
        mutable: bool,
    ) -> Result<u32, InvokeError<WasmShimError>>;

    fn read_substate(&mut self, handle: u32) -> Result<Buffer, InvokeError<WasmShimError>>;

    fn write_substate(
        &mut self,
        handle: u32,
        data: Vec<u8>,
    ) -> Result<(), InvokeError<WasmShimError>>;

    fn unlock_substate(&mut self, handle: u32) -> Result<(), InvokeError<WasmShimError>>;

    fn get_actor(&mut self) -> Result<Buffer, InvokeError<WasmShimError>>;

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError<WasmShimError>>;
}

/// Represents an instantiated, invokable Scrypto module.
pub trait WasmInstance {
    /// Invokes an export defined in this module.
    ///
    /// The export must have a signature of `f(u32) -> u32` where both arguments and return
    /// are pointers to a Scrypto buffer.
    ///
    /// Note that trait objects are "fat pointer" (16 bytes). We wrap it with a `Box` so
    /// to be able to store them in `usize`.
    fn invoke_export<'r>(
        &mut self,
        func_name: &str,
        args: Vec<Vec<u8>>,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<IndexedScryptoValue, InvokeError<WasmShimError>>;
}

/// A Scrypto WASM engine validates, instruments and runs Scrypto modules.
pub trait WasmEngine {
    type WasmInstance: WasmInstance;

    /// Instantiate a Scrypto module.
    fn instantiate(&self, instrumented_code: &InstrumentedCode) -> Self::WasmInstance;
}
