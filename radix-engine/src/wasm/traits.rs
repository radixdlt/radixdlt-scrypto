use crate::wasm::errors::*;
use crate::wasm::WasmValidationError;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;
use scrypto::values::ScryptoValue;

/// Represents an instantiated, invoke-able scrypto module.
pub trait ScryptoModule {
    /// Invokes an export defined in this module.
    ///
    /// For simplicity, we require the export to have a signature of `f(u32) -> u32` where
    /// both argument and return are a pointer to a `ScryptoValue`.
    fn invoke_export<R: ScryptoRuntime>(
        &self,
        name: &str,
        input: &ScryptoValue,
        runtime: &mut R,
    ) -> Result<ScryptoValue, InvokeError>;

    /// Lists all functions exported by this module.
    ///
    /// TODO: Currently a hack so that we don't require a package_init function.
    /// TODO: Remove this by implementing package metadata along with the code during compilation.
    fn function_exports(&self) -> Vec<String>;
}

/// Represents the runtime object that can be invoked by scrypto modules.
pub trait ScryptoRuntime {
    fn main(&mut self, name: &str, args: &[ScryptoValue]) -> Result<ScryptoValue, InvokeError>;
}

/// Trait for validating scrypto modules.
pub trait ScryptoWasmValidator {
    fn validate(&mut self, code: &[u8]) -> Result<(), WasmValidationError>;
}

/// Trait for instrumenting, a.k.a. metering, scrypto modules.
pub trait ScryptoWasmInstrumenter {
    fn instrument(&mut self, code: &[u8]) -> Result<(), WasmValidationError>;
}

/// Trait for instantiating scrypto modules.
pub trait ScryptoWasmExecutor<T: ScryptoModule> {
    fn instantiate(&mut self, code: &[u8]) -> T;
}

/// A `Nop` runtime accepts any external function calls by doing nothing and returning void.
pub struct NopScryptoRuntime;

impl ScryptoRuntime for NopScryptoRuntime {
    fn main(&mut self, _name: &str, _args: &[ScryptoValue]) -> Result<ScryptoValue, InvokeError> {
        Ok(ScryptoValue::unit())
    }
}
