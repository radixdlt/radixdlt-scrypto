use crate::kernel::kernel_api::KernelInvocation;
use crate::system::module::SystemModule;
use crate::system::system_callback::SystemConfig;
use crate::system::system_callback_api::SystemCallbackObject;
use crate::track::interface::StoreAccessInfo;
use crate::transaction::ExecutionMetrics;
use crate::types::*;
use crate::{errors::RuntimeError, errors::SystemModuleError, kernel::kernel_api::KernelApi};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TransactionLimitsError {
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

    MaxCallDepthLimitReached,

    LogSizeTooLarge {
        actual: usize,
        max: usize,
    },
    EventSizeTooLarge {
        actual: usize,
        max: usize,
    },
    PanicMessageSizeTooLarge {
        actual: usize,
        max: usize,
    },
    TooManyLogs,
    TooManyEvents,
}

pub struct TransactionLimitsConfig {
    /// Maximum Substates reads for a transaction.
    pub max_substate_read_count: usize,
    /// Maximum Substates writes for a transaction.
    pub max_substate_write_count: usize,
    /// Maximum Substate read and write size.
    pub max_substate_size: usize,
    /// Maximum Invoke payload size.
    pub max_invoke_payload_size: usize,
    pub max_event_size: usize,
    pub max_log_size: usize,
    pub max_panic_message_size: usize,
    pub max_number_of_logs: usize,
    pub max_number_of_events: usize,
}

/// Tracks and verifies transaction limits during transactino execution,
/// if exceeded breaks execution with appropriate error.
/// Default limits values are defined in radix-engine-constants lib.
/// Stores boundary values of the limits and returns them in transaction receipt.
pub struct LimitsModule {
    /// Definitions of the limits levels.
    limits_config: TransactionLimitsConfig,
    /// Substate store read count.
    substate_db_read_count: usize,
    /// Substate store write count.
    substate_db_write_count: usize,
    /// Substate store read size total.
    substate_db_read_size_total: usize,
    /// Substate store write size total.
    substate_db_write_size_total: usize,
    /// Maximum WASM.
    wasm_max_memory: usize,
    /// Maximum Invoke payload size.
    invoke_payload_max_size: usize,
}

impl LimitsModule {
    pub fn new(limits_config: TransactionLimitsConfig) -> Self {
        LimitsModule {
            limits_config,
            substate_db_read_count: 0,
            substate_db_write_count: 0,
            substate_db_read_size_total: 0,
            substate_db_write_size_total: 0,
            wasm_max_memory: 0,
            invoke_payload_max_size: 0,
        }
    }

    pub fn config(&self) -> &TransactionLimitsConfig {
        &self.limits_config
    }

    /// Exports metrics to transaction receipt.
    pub fn finalize(self, execution_cost_units_consumed: u32) -> ExecutionMetrics {
        ExecutionMetrics {
            substate_read_count: self.substate_db_read_count,
            substate_write_count: self.substate_db_write_count,
            substate_read_size: self.substate_db_read_size_total,
            substate_write_size: self.substate_db_write_size_total,
            max_wasm_memory_used: self.wasm_max_memory,
            max_invoke_payload_size: self.invoke_payload_max_size,
            execution_cost_units_consumed,
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
                return Err(RuntimeError::SystemModuleError(
                    SystemModuleError::TransactionLimitsError(
                        TransactionLimitsError::MaxSubstateReadSizeExceeded(size),
                    ),
                ));
            }
        }
        if let Some(size) = write_size {
            if size > self.limits_config.max_substate_size {
                return Err(RuntimeError::SystemModuleError(
                    SystemModuleError::TransactionLimitsError(
                        TransactionLimitsError::MaxSubstateWriteSizeExceeded(size),
                    ),
                ));
            }
        }

        if self.substate_db_read_count > self.limits_config.max_substate_read_count {
            Err(RuntimeError::SystemModuleError(
                SystemModuleError::TransactionLimitsError(
                    TransactionLimitsError::MaxSubstateReadCountExceeded,
                ),
            ))
        } else if self.substate_db_write_count > self.limits_config.max_substate_write_count {
            Err(RuntimeError::SystemModuleError(
                SystemModuleError::TransactionLimitsError(
                    TransactionLimitsError::MaxSubstateWriteCountExceeded,
                ),
            ))
        } else {
            Ok(())
        }
    }
}

impl<V: SystemCallbackObject> SystemModule<SystemConfig<V>> for LimitsModule {
    fn before_invoke<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        invocation: &KernelInvocation,
    ) -> Result<(), RuntimeError> {
        let current_depth = api.kernel_get_current_depth();
        if current_depth == api.kernel_get_system().modules.costing.max_call_depth {
            return Err(RuntimeError::SystemModuleError(
                SystemModuleError::TransactionLimitsError(
                    TransactionLimitsError::MaxCallDepthLimitReached,
                ),
            ));
        }

        let module = &mut api.kernel_get_system().modules.limits;
        let input_size = invocation.len();
        if input_size > module.invoke_payload_max_size {
            module.invoke_payload_max_size = input_size;
        }

        if input_size > module.limits_config.max_invoke_payload_size {
            Err(RuntimeError::SystemModuleError(
                SystemModuleError::TransactionLimitsError(
                    TransactionLimitsError::MaxInvokePayloadSizeExceeded(input_size),
                ),
            ))
        } else {
            Ok(())
        }
    }

    fn on_read_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _lock_handle: LockHandle,
        value_size: usize,
        _store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        let limits = &mut api.kernel_get_system().modules.limits;

        // Increase read coutner.
        limits.substate_db_read_count += 1;

        // Increase total size.
        limits.substate_db_read_size_total += value_size;

        // Validate
        limits.validate_substates(Some(value_size), None)
    }

    fn on_write_substate<Y: KernelApi<SystemConfig<V>>>(
        api: &mut Y,
        _lock_handle: LockHandle,
        value_size: usize,
        _store_access: &StoreAccessInfo,
    ) -> Result<(), RuntimeError> {
        let limits = &mut api.kernel_get_system().modules.limits;

        // Increase write coutner.
        limits.substate_db_write_count += 1;

        // Increase total size.
        limits.substate_db_write_size_total += value_size;

        // Validate
        limits.validate_substates(None, Some(value_size))
    }
}
