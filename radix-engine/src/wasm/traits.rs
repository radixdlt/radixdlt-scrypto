use scrypto::values::ScryptoValue;

use super::WasmValidationError;

/// Represents an error when invoking an export of a wasm module.
pub enum InvokeError {}

/// A common trait for Scrypto modules, a.k.a., packages.
pub trait ScryptoModule {
    /// Invokes an export defined in this module.
    fn invoke_export<R: ScryptoRuntime>(
        &self,
        name: &str,
        args: &[ScryptoValue],
        host: &mut R,
    ) -> Result<Option<ScryptoValue>, InvokeError>;

    /// Lists all functions exported by this module.
    ///
    /// TODO: Currently a hack so that we don't require a package_init function.
    /// TODO: Remove this by implementing package metadata along with the code during compilation.
    fn function_exports(&self) -> Vec<String>;
}

/// Denotes a runtime object that may be invoked by wasm code.
pub trait ScryptoRuntime {
    type Error;

    fn invoke_function(
        &mut self,
        name: &str,
        args: &[ScryptoValue],
    ) -> Result<Option<ScryptoValue>, Self::Error>;
}

pub trait ScryptoWasmValidator {
    fn validate(&mut self, code: &[u8]) -> Result<(), WasmValidationError>;
}

pub trait ScryptoWasmInstrumenter {
    fn instrument(&mut self, code: &[u8]) -> Result<(), WasmValidationError>;
}

pub trait ScryptoWasmExecutor<T: ScryptoModule> {
    fn instantiate(&mut self, code: &[u8]) -> T;
}

pub struct NopScryptoRuntime;

impl ScryptoRuntime for NopScryptoRuntime {
    type Error = ();

    fn invoke_function(
        &mut self,
        _name: &str,
        _args: &[ScryptoValue],
    ) -> Result<Option<ScryptoValue>, Self::Error> {
        Ok(None)
    }
}
