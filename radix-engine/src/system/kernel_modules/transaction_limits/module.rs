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
    /// Consumed WASM memory for each invocation call.
    wasm_memory: Vec<usize>,
}

impl TransactionLimitsModule {
    pub fn new(max_wasm_memory: usize) -> Self {
        TransactionLimitsModule {
            max_wasm_memory,
            wasm_memory: Vec::new(),
        }
    }
}

impl<R: FeeReserve> BaseModule<R> for TransactionLimitsModule {
    fn pre_execute_invocation(
        &mut self,
        _actor: &ResolvedActor,
        _call_frame_update: &CallFrameUpdate,
        call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        self.wasm_memory.push(0);

        if call_frame.consumed_wasm_memory > 0 {
            self.wasm_memory[call_frame.depth] = call_frame.consumed_wasm_memory;
            // calculate current maximum consumed memory
            let max_value = self.wasm_memory.iter().sum();

            // validate if limit was exceeded
            if max_value > self.max_wasm_memory {
                Err(ModuleError::TransactionLimitsError(
                    TransactionLimitsError::MaxWasmMemoryExceeded(max_value),
                ))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn post_execute_invocation(
        &mut self,
        _caller: &ResolvedActor,
        _update: &CallFrameUpdate,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        self.wasm_memory.pop();
        Ok(())
    }

    fn post_wasm_instantiation(
        &mut self,
        call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        consumed_memory: usize,
    ) -> Result<(), ModuleError> {
        // update current frame consumed memory value
        if self.wasm_memory.len() == call_frame.depth {
            self.wasm_memory.push(consumed_memory)
        } else {
            self.wasm_memory[call_frame.depth] = consumed_memory;
        }

        // calculate current maximum consumed memory
        let max_value = self.wasm_memory.iter().sum();

        // validate if limit was exceeded
        if max_value > self.max_wasm_memory {
            Err(ModuleError::TransactionLimitsError(
                TransactionLimitsError::MaxWasmMemoryExceeded(max_value),
            ))
        } else {
            Ok(())
        }
    }
}
