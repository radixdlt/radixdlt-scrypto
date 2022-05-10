use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::values::ScryptoValue;

use crate::wasm::errors::*;
use crate::wasm::WasmValidationError;

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
    fn main(&mut self, input: ScryptoValue) -> Result<ScryptoValue, InvokeError>;

    fn use_tbd(&mut self, amount: u32) -> Result<(), InvokeError>;
}

/// Trait for validating scrypto modules.
pub trait ScryptoWasmValidator {
    fn validate(&mut self, code: &[u8]) -> Result<(), WasmValidationError>;
}

/// Trait for instrumenting, a.k.a. metering, scrypto modules.
pub trait ScryptoWasmInstrumenter {
    fn instrument(&mut self, code: &[u8]) -> Result<Vec<u8>, InstrumentError>;
}

/// Trait for instantiating scrypto modules.
pub trait ScryptoWasmExecutor<T: ScryptoModule> {
    fn instantiate(&mut self, code: &[u8]) -> T;
}

/// A `Nop` runtime accepts any external function calls by doing nothing and returning void.
pub struct NopScryptoRuntime {
    tbd_limit: u32,
    tbd_balance: u32,
}

impl NopScryptoRuntime {
    pub fn new(tbd_limit: u32) -> Self {
        Self {
            tbd_limit,
            tbd_balance: tbd_limit,
        }
    }
}

impl ScryptoRuntime for NopScryptoRuntime {
    fn main(&mut self, _input: ScryptoValue) -> Result<ScryptoValue, InvokeError> {
        Ok(ScryptoValue::unit())
    }

    fn use_tbd(&mut self, amount: u32) -> Result<(), InvokeError> {
        if self.tbd_balance >= amount {
            self.tbd_balance -= amount;
            Ok(())
        } else {
            self.tbd_balance = 0;
            Err(InvokeError::OutOfTbd {
                limit: self.tbd_limit,
                balance: self.tbd_balance,
                required: amount,
            })
        }
    }
}
