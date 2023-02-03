use crate::{errors::ModuleError, kernel::*, system::kernel_modules::fee::FeeReserve};
use radix_engine_interface::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TransactionLimitsError {
    /// Used when WASM memory consumed during transaction execution exceeds defined limit,
    /// as parameter current memory value is returned.
    MaxWasmMemoryExceeded(usize),
}

/// Tracks and verifies transaction limits during transactino execution,
/// if exceeded breaks execution with appropriate error.
pub struct TransactionLimitsModule {
    /// Maximum WASM memory which can be consumed during transaction execution.
    max_wasm_memory: usize,
    /// Currently consumed WASM memory.
    wasm_memory_consumed: usize,
}

impl TransactionLimitsModule {
    pub fn new(max_wasm_memory: usize) -> Self {
        TransactionLimitsModule {
            max_wasm_memory,
            wasm_memory_consumed: 0,
        }
    }
}

impl<R: FeeReserve> BaseModule<R> for TransactionLimitsModule {
    fn post_wasm_instantiation(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        consumed_memory: usize,
    ) -> Result<(), ModuleError> {
        self.wasm_memory_consumed += consumed_memory;

        if self.wasm_memory_consumed > self.max_wasm_memory {
            Err(ModuleError::TransactionLimitsError(
                TransactionLimitsError::MaxWasmMemoryExceeded(self.wasm_memory_consumed),
            ))
        } else {
            Ok(())
        }
    }
}
