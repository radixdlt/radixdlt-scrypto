use crate::kernel::actor::Actor;
use crate::types::*;
use crate::{
    errors::ModuleError,
    errors::RuntimeError,
    kernel::{call_frame::CallFrameUpdate, kernel_api::KernelModuleApi, module::KernelModule},
    types::Vec,
};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TransactionLimitsError {
    /// Retruned when WASM memory consumed during transaction execution exceeds defined limit,
    /// as parameter current memory value is returned.
    MaxWasmMemoryExceeded(usize),
    /// Retruned when one instance WASM memory consumed during transaction execution exceeds defined limit,
    /// as parameter memory consumed by that instave is returned.
    MaxWasmInstanceMemoryExceeded(usize),
    /// Returned when substate read count during transaction execution
    /// exceeds defined limit just after read occurs.
    MaxSubstateReadCountExceeded,
    /// Returned when substate write count during transaction execution
    /// exceeds defined limit just after write occurs.
    MaxSubstateWriteCountExceeded,
    /// Returned when substate read size exceeds defined limit just after read occurs.
    MaxSubstateReadSizeExceeded(usize),
    /// Returned when substate write size exceeds defined limit just after write occurs.
    MaxSubstateWriteSizeExceeded(usize),
    /// Returned when function or method invocation payload size exceeds defined limit,
    /// as parameter actual payload size is returned.
    MaxInvokePayloadSizeExceeded(usize),
}

/// Representation of data which needs to be limited for each call frame.
#[derive(Default)]
struct CallFrameLimitInfo {
    /// Consumed WASM memory.
    wasm_memory_usage: usize,
}

pub struct TransactionLimitsConfig {
    /// Maximum WASM memory which can be consumed during transaction execution.
    pub max_wasm_memory: usize,
    /// Maximum WASM memory which can be consumed in one call frame.
    pub max_wasm_memory_per_call_frame: usize,
    /// Maximum Substates reads for a transaction.
    pub max_substate_read_count: usize,
    /// Maximum Substates writes for a transaction.
    pub max_substate_write_count: usize,
    /// Maximum Substate read and write size.
    pub max_substate_size: usize,
    /// Maximum Invoke payload size.
    pub max_invoke_payload_size: usize,
}

/// Tracks and verifies transaction limits during transactino execution,
/// if exceeded breaks execution with appropriate error.
/// Default limits values are defined in radix-engine-constants lib.
pub struct TransactionLimitsModule {
    /// Definitions of the limits levels.
    limits_config: TransactionLimitsConfig,
    /// Internal stack of data for each call frame.
    call_frames_stack: Vec<CallFrameLimitInfo>,
    /// Substate store read count.
    substate_db_read_count: usize,
    /// Substate store write count.
    substate_db_write_count: usize,
}

impl TransactionLimitsModule {
    pub fn new(limits_config: TransactionLimitsConfig) -> Self {
        TransactionLimitsModule {
            limits_config,
            call_frames_stack: Vec::with_capacity(8),
            substate_db_read_count: 0,
            substate_db_write_count: 0,
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
        if current_call_frame.wasm_memory_usage > self.limits_config.max_wasm_memory_per_call_frame
        {
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
        if max_value > self.limits_config.max_wasm_memory {
            Err(RuntimeError::ModuleError(
                ModuleError::TransactionLimitsError(TransactionLimitsError::MaxWasmMemoryExceeded(
                    max_value,
                )),
            ))
        } else {
            Ok(())
        }
    }

    /// Checks if substate reads/writes count and size is in the limit.
    fn validate_substates(
        &self,
        read_size: Option<usize>,
        write_size: Option<usize>,
    ) -> Result<(), RuntimeError> {
        if let Some(size) = read_size {
            if size > self.limits_config.max_substate_size {
                return Err(RuntimeError::ModuleError(
                    ModuleError::TransactionLimitsError(
                        TransactionLimitsError::MaxSubstateReadSizeExceeded(size),
                    ),
                ));
            }
        }
        if let Some(size) = write_size {
            if size > self.limits_config.max_substate_size {
                return Err(RuntimeError::ModuleError(
                    ModuleError::TransactionLimitsError(
                        TransactionLimitsError::MaxSubstateWriteSizeExceeded(size),
                    ),
                ));
            }
        }

        if self.substate_db_read_count > self.limits_config.max_substate_read_count {
            Err(RuntimeError::ModuleError(
                ModuleError::TransactionLimitsError(
                    TransactionLimitsError::MaxSubstateReadCountExceeded,
                ),
            ))
        } else if self.substate_db_write_count > self.limits_config.max_substate_write_count {
            Err(RuntimeError::ModuleError(
                ModuleError::TransactionLimitsError(
                    TransactionLimitsError::MaxSubstateWriteCountExceeded,
                ),
            ))
        } else {
            Ok(())
        }
    }

    // This event handler is called from two places:
    //  1. Before wasm nested function call
    //  2. After wasm invocation
    pub fn update_wasm_memory_usage(
        &mut self,
        depth: usize,
        consumed_memory: usize,
    ) -> Result<(), RuntimeError> {
        // update current frame consumed memory
        if let Some(val) = self.call_frames_stack.get_mut(depth) {
            val.wasm_memory_usage = consumed_memory;
        } else {
            // When kernel pops the call frame there are some nested calls which
            // are not aligned with before_push_frame() which requires pushing
            // new value on a stack instead of updating it.
            self.call_frames_stack.push(CallFrameLimitInfo {
                wasm_memory_usage: consumed_memory,
            })
        }

        self.validate_wasm_memory()
    }
}

impl KernelModule for TransactionLimitsModule {
    fn before_invoke<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _identifier: &InvocationDebugIdentifier,
        input_size: usize,
    ) -> Result<(), RuntimeError> {
        if input_size
            > api
                .kernel_get_module_state()
                .transaction_limits
                .limits_config
                .max_invoke_payload_size
        {
            Err(RuntimeError::ModuleError(
                ModuleError::TransactionLimitsError(
                    TransactionLimitsError::MaxInvokePayloadSizeExceeded(input_size),
                ),
            ))
        } else {
            Ok(())
        }
    }

    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _callee: &Actor,
        _down_movement: &mut CallFrameUpdate,
        _args: &IndexedScryptoValue,
    ) -> Result<(), RuntimeError> {
        // push new empty wasm memory value refencing current call frame to internal stack
        api.kernel_get_module_state()
            .transaction_limits
            .call_frames_stack
            .push(CallFrameLimitInfo::default());
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
        size: usize,
    ) -> Result<(), RuntimeError> {
        let tlimit = &mut api.kernel_get_module_state().transaction_limits;

        // Increase read coutner.
        tlimit.substate_db_read_count += 1;

        // Validate
        tlimit.validate_substates(Some(size), None)
    }

    fn on_write_substate<Y: KernelModuleApi<RuntimeError>>(
        api: &mut Y,
        _lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        let tlimit = &mut api.kernel_get_module_state().transaction_limits;

        // Increase write coutner.
        tlimit.substate_db_write_count += 1;

        // Validate
        tlimit.validate_substates(None, Some(size))
    }
}
