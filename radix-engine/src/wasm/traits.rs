use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::values::ScryptoValue;

use crate::wasm::errors::*;
use crate::wasm::WasmValidationError;

/// Represents a parsed Scrypto module (may be shared).
pub trait ScryptoModule<'r, I, R>
where
    I: ScryptoInstance,
    R: ScryptoRuntime,
{
    /// Instantiate this module with the given runtime
    fn instantiate(&self, runtime: &'r mut R) -> I;
}

/// Represents an instantiated, invoke-able scrypto module.
pub trait ScryptoInstance {
    /// Invokes an export defined in this module.
    ///
    /// For simplicity, we require the export to have a signature of `f(u32) -> u32` where
    /// both argument and return are a pointer to a `ScryptoValue`.
    fn invoke_export(
        &mut self,
        name: &str,
        input: &ScryptoValue,
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

    fn use_tbd(&mut self, tbd: u32) -> Result<(), InvokeError>;
}

/// Trait for validating scrypto modules.
pub trait ScryptoValidator {
    fn validate(&mut self, code: &[u8]) -> Result<(), WasmValidationError>;
}

/// Trait for instrumenting, a.k.a. metering, scrypto modules.
pub trait ScryptoInstrumenter {
    fn instrument(&mut self, code: &[u8]) -> Result<Vec<u8>, InstrumentError>;
}

/// Trait for loading scrypto modules.
pub trait ScryptoLoader<
    'l, /* Loader lifetime */
    'r, /* Runtime  lifetime */
    M,  /* Module generic type */
    I,  /* Instance generic type */
    R,  /* Runtime generic type */
> where
    M: ScryptoModule<'r, I, R>,
    I: ScryptoInstance,
    R: ScryptoRuntime,
{
    fn load(&'l mut self, code: &[u8]) -> M;
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

    fn use_tbd(&mut self, tbd: u32) -> Result<(), InvokeError> {
        if self.tbd_balance >= tbd {
            self.tbd_balance -= tbd;
            Ok(())
        } else {
            self.tbd_balance = 0;
            Err(InvokeError::OutOfTbd {
                limit: self.tbd_limit,
                balance: self.tbd_balance,
                required: tbd,
            })
        }
    }
}
