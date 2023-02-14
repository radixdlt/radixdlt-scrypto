use crate::{
    errors::ModuleError,
    errors::RuntimeError,
    kernel::*,
    kernel::{KernelModule, KernelModuleApi},
};
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
    fn validate(&self) -> Result<(), RuntimeError> {
        // check last (current) call frame
        let current_instance_memory = *self.wasm_memory.last().unwrap();
        if current_instance_memory > self.max_wasm_instance_memory {
            return Err(RuntimeError::ModuleError(
                ModuleError::TransactionLimitsError(
                    TransactionLimitsError::MaxWasmInstanceMemoryExceeded(current_instance_memory),
                ),
            ));
        };

        // calculate current maximum consumed memory
        // sum all call stack values
        let max_value = self.wasm_memory.iter().sum();

        // validate if limit was exceeded
        if max_value > self.max_wasm_memory {
            Err(RuntimeError::ModuleError(
                ModuleError::TransactionLimitsError(TransactionLimitsError::MaxWasmMemoryExceeded(
                    max_value,
                )),
            ))
        } else {
            Ok(())
        }
    }
}

impl KernelModule for TransactionLimitsModule {
    /// If there is a nested call of WASM instance, api.get_current_wasm_memory_consumption()
    /// returns currently alocated memory by WASM instance which invokes
    /// nested call (call frame references that instance).
    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _actor: &ResolvedActor,
        _down_movement: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        let memory = api.get_current_wasm_memory_consumption();
        api.get_module_state()
            .transaction_limits
            .wasm_memory
            .push(memory);

        api.get_module_state().transaction_limits.validate()
    }

    fn after_pop_frame<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        // pop from stack
        api.get_module_state().transaction_limits.wasm_memory.pop();

        Ok(())
    }

    fn after_wasm_instantiation<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        consumed_memory: usize,
    ) -> Result<(), RuntimeError> {
        let depth = api.get_current_depth();
        let tlimit = &mut api.get_module_state().transaction_limits;

        // update current frame consumed memory value after WASM invokation is done
        if let Some(val) = tlimit.wasm_memory.get_mut(depth) {
            *val = consumed_memory;
        } else {
            // When kernel pops the call frame there are some nested calls which
            // are not aligned with before_push_frame() which requires pushing
            // new value on a stack instead of updating it.
            tlimit.wasm_memory.push(consumed_memory)
        }

        tlimit.validate()
    }
}
