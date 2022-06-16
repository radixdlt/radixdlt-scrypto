use sbor::rust::boxed::Box;
use scrypto::values::ScryptoValue;

use crate::wasm::errors::*;

/// Represents the runtime that can be invoked by Scrypto modules.
pub trait WasmRuntime {
    fn main(&mut self, input: ScryptoValue) -> Result<ScryptoValue, InvokeError>;

    fn consume_cost_units(&mut self, n: u32) -> Result<(), InvokeError>;
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
        arg: &ScryptoValue,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<ScryptoValue, InvokeError>;
}

/// A Scrypto WASM engine validates, instruments and runs Scrypto modules.
pub trait WasmEngine<I: WasmInstance> {
    /// Instantiate a Scrypto module.
    fn instantiate(&mut self, code: &[u8]) -> I;
}
