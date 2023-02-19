use crate::{
    errors::ModuleError,
    errors::RuntimeError,
    kernel::{
        actor::ResolvedActor, call_frame::CallFrameUpdate, kernel_api::KernelModuleApi,
        module::KernelModule,
    },
    types::Vec,
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
    max_wasm_memory_per_call_frame: usize,
    /// Consumed WASM memory for each invocation call.
    wasm_memory_usage_stack: Vec<usize>,
}

impl TransactionLimitsModule {
    pub fn new(max_wasm_memory: usize, max_wasm_instance_memory: usize) -> Self {
        TransactionLimitsModule {
            max_wasm_memory,
            max_wasm_memory_per_call_frame: max_wasm_instance_memory,
            wasm_memory_usage_stack: Vec::with_capacity(8),
        }
    }

    /// Checks if maximum WASM memory limit for one instance was exceeded and then
    /// checks if memory limit for all instances was exceeded.
    fn validate(&self) -> Result<(), RuntimeError> {
        // check last (current) call frame
        let current_instance_memory = *self
            .wasm_memory_usage_stack
            .last()
            .expect("Wasm memory usage stack should not be empty.");
        if current_instance_memory > self.max_wasm_memory_per_call_frame {
            return Err(RuntimeError::ModuleError(
                ModuleError::TransactionLimitsError(
                    TransactionLimitsError::MaxWasmInstanceMemoryExceeded(current_instance_memory),
                ),
            ));
        };

        // calculate current maximum consumed memory
        // sum all call stack values
        let max_value = self.wasm_memory_usage_stack.iter().sum();

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
    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _actor: &Option<ResolvedActor>,
        _down_movement: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        // push new empty wasm memory value refencing current call frame to internal stack
        api.kernel_get_module_state()
            .transaction_limits
            .wasm_memory_usage_stack
            .push(0);
        Ok(())
    }

    fn after_pop_frame<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        // pop from internal stack
        api.kernel_get_module_state()
            .transaction_limits
            .wasm_memory_usage_stack
            .pop();
        Ok(())
    }

    // This event handler is called from two places:
    //  1. Before wasm nested function call
    //  2. After wasm invocation
    fn on_update_wasm_memory_usage<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        consumed_memory: usize,
    ) -> Result<(), RuntimeError> {
        let depth = api.kernel_get_current_depth();
        let tlimit = &mut api.kernel_get_module_state().transaction_limits;

        // update current frame consumed memory
        if let Some(val) = tlimit.wasm_memory_usage_stack.get_mut(depth) {
            *val = consumed_memory;
        } else {
            // When kernel pops the call frame there are some nested calls which
            // are not aligned with before_push_frame() which requires pushing
            // new value on a stack instead of updating it.
            tlimit.wasm_memory_usage_stack.push(consumed_memory)
        }

        tlimit.validate()
    }
}
