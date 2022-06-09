use sbor::rust::boxed::Box;
use scrypto::values::ScryptoValue;

use crate::wasm::errors::*;

/// Represents the runtime that can be invoked by Scrypto modules.
pub trait WasmRuntime {
    fn main(&mut self, input: ScryptoValue) -> Result<ScryptoValue, InvokeError>;

    fn consume_cost_unit(&mut self, cost_unit: u32) -> Result<(), InvokeError>;
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
        name: &str,
        method_name: &str,
        arg: &ScryptoValue,
        runtime: &mut Box<dyn WasmRuntime + 'r>,
    ) -> Result<ScryptoValue, InvokeError>;
}

/// A Scrypto WASM engine validates, instruments and runs Scrypto modules.
pub trait WasmEngine<I: WasmInstance> {
    /// Instantiate a Scrypto module.
    fn instantiate(&mut self, code: &[u8]) -> I;
}

/// A `Nop` runtime accepts any external function calls by doing nothing and returning void.
pub struct NopWasmRuntime {
    cost_unit_limit: u32,
    cost_unit_balance: u32,
}

impl NopWasmRuntime {
    pub fn new(cost_unit_limit: u32) -> Self {
        Self {
            cost_unit_limit,
            cost_unit_balance: cost_unit_limit,
        }
    }
}

impl WasmRuntime for NopWasmRuntime {
    fn main(&mut self, _input: ScryptoValue) -> Result<ScryptoValue, InvokeError> {
        Ok(ScryptoValue::unit())
    }

    fn consume_cost_unit(&mut self, cost_unit: u32) -> Result<(), InvokeError> {
        if self.cost_unit_balance >= cost_unit {
            self.cost_unit_balance -= cost_unit;
            Ok(())
        } else {
            self.cost_unit_balance = 0;
            Err(InvokeError::OutOfCostUnit {
                limit: self.cost_unit_limit,
                balance: self.cost_unit_balance,
                required: cost_unit,
            })
        }
    }
}
