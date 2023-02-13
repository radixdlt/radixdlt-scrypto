use crate::{errors::ModuleError, kernel::*, system::kernel_modules::fee::FeeReserve};
use radix_engine_interface::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TransactionLimitsError {
    /// Used when WASM memory consumed during transaction execution exceeds defined limit,
    /// as parameter current memory value is returned.
    MaxWasmMemoryExceeded(usize),
    /// Used when one instance WASM memory consumed during transaction execution exceeds defined limit,
    /// as parameter memory consumed by that instave is returned.
    MaxWasmInstanceMemoryExceeded(usize),
}

/// Tracks and verifies transaction limits during transactino execution,
/// if exceeded breaks execution with appropriate error.
pub struct TransactionLimitsModule {
    /// Maximum WASM memory which can be consumed during transaction execution.
    max_wasm_memory: usize,
    /// Maximum WASM memory which can be consumed during transaction execution.
    max_wasm_instance_memory: usize,
    /// Consumed WASM memory for each invocation call.
    wasm_memory: Vec<usize>,
}

impl TransactionLimitsModule {
    pub fn new(max_wasm_memory: usize, max_wasm_instance_memory: usize) -> Self {
        TransactionLimitsModule {
            max_wasm_memory,
            max_wasm_instance_memory,
            wasm_memory: Vec::with_capacity(8),
        }
    }

    /// Checks if maximum WASM memory limit for one instance was exceeded and then
    /// checks if memory limit for all instances was exceeded.
    fn validate(&self) -> Result<(), ModuleError> {
        // check last (current) call frame
        let current_instance_memory = *self.wasm_memory.last().unwrap();
        if current_instance_memory > self.max_wasm_instance_memory {
            return Err(ModuleError::TransactionLimitsError(
                TransactionLimitsError::MaxWasmInstanceMemoryExceeded(current_instance_memory),
            ));
        };

        // calculate current maximum consumed memory
        // sum all call stack values
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

impl<R: FeeReserve> BaseModule<R> for TransactionLimitsModule {
    /// If there is a nested call of WASM instance, call_frame argument
    /// contains currently alocated memory by WASM instance which invokes
    /// nested call (call frame references that instance).
    fn pre_execute_invocation(
        &mut self,
        _actor: &ResolvedActor,
        _call_frame_update: &CallFrameUpdate,
        call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        // push current call frame WASM memory on stack
        self.wasm_memory.push(call_frame.consumed_wasm_memory);

        self.validate()
    }

    fn post_execute_invocation(
        &mut self,
        _caller: &ResolvedActor,
        _update: &CallFrameUpdate,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        // pop from stack
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
        // update current frame consumed memory value after WASM invokation is done
        if let Some(val) = self.wasm_memory.get_mut(call_frame.depth) {
            *val = consumed_memory;
        } else {
            // When kernel pops the call frame there are some nested calls which
            // are not aligned with pre_execute_invocation() which requires pushing
            // new value on a stack instead of updating it.
            self.wasm_memory.push(consumed_memory)
        }

        self.validate()
    }
}
