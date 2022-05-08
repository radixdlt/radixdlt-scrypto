use crate::errors::RuntimeError;
use crate::wasm::WasmValidationError;
use scrypto::values::ParseScryptoValueError;
use scrypto::values::ScryptoValue;

/// Represents an error when invoking an export of a scrypto module.
pub enum InvokeError {
    MemoryAllocError,

    MemoryAccessError,

    InvalidScryptoValue(ParseScryptoValueError),

    WasmError,

    RuntimeError(RuntimeError),

    MissingReturnData,

    InvalidReturn,
}

/// Represents an instantiated, invoke-able scrypto module.
pub trait ScryptoModule {
    /// Invokes an export defined in this module.
    fn invoke_export(&self, name: &str, args: &[ScryptoValue])
        -> Result<ScryptoValue, InvokeError>;

    /// Lists all functions exported by this module.
    ///
    /// TODO: Currently a hack so that we don't require a package_init function.
    /// TODO: Remove this by implementing package metadata along with the code during compilation.
    fn function_exports(&self) -> Vec<String>;
}

/// Represents the runtime object that can be invoked by scrypto modules.
pub trait ScryptoRuntime {
    type Error;

    fn invoke_function(
        &mut self,
        name: u32, // TODO: this will likely be changed
        args: &[ScryptoValue],
    ) -> Result<ScryptoValue, Self::Error>;
}

/// Trait for validating scrypto module.
pub trait ScryptoWasmValidator {
    fn validate(&mut self, code: &[u8]) -> Result<(), WasmValidationError>;
}

/// Trait for instrumenting, a.k.a. metering, scrypto module.
pub trait ScryptoWasmInstrumenter {
    fn instrument(&mut self, code: &[u8]) -> Result<(), WasmValidationError>;
}

/// Trait for instantiating and executing scrypto module.
pub trait ScryptoWasmExecutor<T: ScryptoModule> {
    fn instantiate(&mut self, code: &[u8]) -> T;
}

/// A `Nop` runtime accepts any external function calls by doing nothing and returning void.
pub struct NopScryptoRuntime;

impl ScryptoRuntime for NopScryptoRuntime {
    type Error = ();

    fn invoke_function(
        &mut self,
        _name: u32,
        _args: &[ScryptoValue],
    ) -> Result<ScryptoValue, Self::Error> {
        Ok(ScryptoValue::from_value(&()))
    }
}
