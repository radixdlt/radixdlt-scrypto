use crate::{
    errors::ModuleError,
    errors::RuntimeError,
    kernel::{
        actor::ResolvedActor, call_frame::CallFrameUpdate, kernel_api::KernelModuleApi,
        module::KernelModule,
    },
    types::Vec,
};
use radix_engine_interface::{
    api::types::LockHandle, ScryptoCategorize, ScryptoDecode, ScryptoEncode,
};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum TransactionLimitsError {
    /// Used when WASM memory consumed during transaction execution exceeds defined limit,
    /// as parameter current memory value is returned.
    MaxWasmMemoryExceeded(usize),
    /// Used when one instance WASM memory consumed during transaction execution exceeds defined limit,
    /// as parameter memory consumed by that instave is returned.
    MaxWasmInstanceMemoryExceeded(usize),
}

/// Representatino of data which needs to be limited for each call frame.
#[derive(Default)]
struct CallFrameInfo {
    /// Consumed WASM memory.
    wasm_memory_usage: usize,
    /// Substate store read count.
    substate_store_read: usize,
}

/// Tracks and verifies transaction limits during transactino execution,
/// if exceeded breaks execution with appropriate error.
pub struct TransactionLimitsModule {
    /// Maximum WASM memory which can be consumed during transaction execution.
    max_wasm_memory: usize,
    /// Maximum WASM memory which can be consumed during transaction execution.
    max_wasm_memory_per_call_frame: usize,
    /// Internal stack of data for each call frame.
    call_frames_stack: Vec<CallFrameInfo>,
}

impl TransactionLimitsModule {
    pub fn new(max_wasm_memory: usize, max_wasm_instance_memory: usize) -> Self {
        TransactionLimitsModule {
            max_wasm_memory,
            max_wasm_memory_per_call_frame: max_wasm_instance_memory,
            call_frames_stack: Vec::with_capacity(8),
        }
    }

    /// Checks if maximum WASM memory limit for one instance was exceeded and then
    /// checks if memory limit for all instances was exceeded.
    fn validate_wasm_memory(&self) -> Result<(), RuntimeError> {
        // check last (current) call frame
        let current_call_frame = self
            .call_frames_stack
            .last()
            .expect("Call frames stack (Wasm memory) should not be empty.");
        if current_call_frame.wasm_memory_usage > self.max_wasm_memory_per_call_frame {
            return Err(RuntimeError::ModuleError(
                ModuleError::TransactionLimitsError(
                    TransactionLimitsError::MaxWasmInstanceMemoryExceeded(
                        current_call_frame.wasm_memory_usage,
                    ),
                ),
            ));
        };

        // calculate current maximum consumed memory
        // sum all call stack values
        let max_value = self
            .call_frames_stack
            .iter()
            .map(|item| item.wasm_memory_usage)
            .sum();

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

    fn validate_substates(&self) -> Result<(), RuntimeError> {
        Ok(())
    }
}

impl KernelModule for TransactionLimitsModule {
    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _actor: &ResolvedActor,
        _down_movement: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        // push new empty wasm memory value refencing current call frame to internal stack
        api.kernel_get_module_state()
            .transaction_limits
            .call_frames_stack
            .push(CallFrameInfo::default());
        Ok(())
    }

    fn after_pop_frame<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        // pop from internal stack
        api.kernel_get_module_state()
            .transaction_limits
            .call_frames_stack
            .pop();
        Ok(())
    }

    fn on_read_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _lock_handle: LockHandle,
        _size: usize,
    ) -> Result<(), RuntimeError> {
        let depth = api.kernel_get_current_depth();
        let tlimit = &mut api.kernel_get_module_state().transaction_limits;

        // update current frame substate read count
        if let Some(val) = tlimit.call_frames_stack.get_mut(depth) {
            val.substate_store_read += 1;
        } else {
            // Kernel can call this event before calling before_push_frame()
            // in that case we are pushing new itme on internal stack.
            tlimit.call_frames_stack.push(CallFrameInfo {
                wasm_memory_usage: 0,
                substate_store_read: 1,
            })
        }

        tlimit.validate_substates()
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
        if let Some(val) = tlimit.call_frames_stack.get_mut(depth) {
            val.wasm_memory_usage = consumed_memory;
        } else {
            // When kernel pops the call frame there are some nested calls which
            // are not aligned with before_push_frame() which requires pushing
            // new value on a stack instead of updating it.
            tlimit.call_frames_stack.push(CallFrameInfo {
                wasm_memory_usage: consumed_memory,
                substate_store_read: 0,
            })
        }

        tlimit.validate_wasm_memory()
    }
}
