use crate::errors::*;
use crate::kernel::*;
use crate::system::kernel_modules::fee::CostingEntry;
use crate::system::kernel_modules::fee::FeeReserve;
use crate::system::kernel_modules::fee::FeeReserveError;
use crate::transaction::AbortReason;
use crate::types::*;
use radix_engine_interface::api::types::VaultId;
use radix_engine_interface::blueprints::resource::Resource;

use super::CostingReason;
use super::ExecutionFeeReserve;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Categorize)]
pub enum CostingError {
    FeeReserveError(FeeReserveError),
    MaxCallDepthLimitReached,
}

impl CanBeAbortion for CostingError {
    fn abortion(&self) -> Option<&AbortReason> {
        match self {
            Self::FeeReserveError(err) => err.abortion(),
            _ => None,
        }
    }
}

pub struct CostingModule {
    max_depth: usize,
}

impl CostingModule {
    pub fn new(max_depth: usize) -> Self {
        Self { max_depth }
    }
}

pub fn consume_api_cost<R: FeeReserve>(
    track: &mut Track<R>,
    reason: CostingReason,
    costing_entry: CostingEntry,
) -> Result<(), ModuleError> {
    let cost_units = track.fee_table.system_api_cost(costing_entry);
    track
        .fee_reserve()
        .consume_execution(cost_units, reason)
        .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))?;
    Ok(())
}

impl<R: FeeReserve> BaseModule<R> for CostingModule {
    fn pre_kernel_api_call(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        track: &mut Track<R>,
        input: KernelApiCallInput,
    ) -> Result<(), ModuleError> {
        match input {
            KernelApiCallInput::Invoke {
                depth, input_size, ..
            } => {
                if depth == self.max_depth {
                    return Err(ModuleError::CostingError(
                        CostingError::MaxCallDepthLimitReached,
                    ));
                }

                if depth > 0 {
                    consume_api_cost(
                        track,
                        CostingReason::Invoke,
                        CostingEntry::Invoke { input_size },
                    )?;
                }
            }
            KernelApiCallInput::GetVisibleNodes => {
                consume_api_cost(
                    track,
                    CostingReason::GetVisibleNodes,
                    CostingEntry::ReadOwnedNodes,
                )?;
            }
            KernelApiCallInput::DropNode { .. } => {
                // TODO: get size of the value
                consume_api_cost(
                    track,
                    CostingReason::DropNode,
                    CostingEntry::DropNode { size: 0 },
                )?;
            }
            KernelApiCallInput::CreateNode { .. } => {
                // TODO: get size of the value
                consume_api_cost(
                    track,
                    CostingReason::CreateNode,
                    CostingEntry::CreateNode { size: 0 },
                )?;
            }
            KernelApiCallInput::LockSubstate { .. } => {
                // TODO: get size of the value
                consume_api_cost(
                    track,
                    CostingReason::LockSubstate,
                    CostingEntry::LockSubstate { size: 0 },
                )?;
            }
            KernelApiCallInput::GetRef { .. } => {
                // TODO: get size of the value
                consume_api_cost(
                    track,
                    CostingReason::ReadSubstate,
                    CostingEntry::ReadSubstate { size: 0 },
                )?;
            }
            KernelApiCallInput::GetRefMut { .. } => {
                // TODO: get size of the value
                consume_api_cost(
                    track,
                    CostingReason::WriteSubstate,
                    CostingEntry::WriteSubstate { size: 0 },
                )?;
            }
            KernelApiCallInput::DropLock { .. } => {
                consume_api_cost(track, CostingReason::DropLock, CostingEntry::DropLock)?;
            }
        }

        Ok(())
    }

    fn post_kernel_api_call(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        _track: &mut Track<R>,
        _output: KernelApiCallOutput,
    ) -> Result<(), ModuleError> {
        Ok(())
    }

    fn on_wasm_instantiation(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        track: &mut Track<R>,
        code: &[u8],
    ) -> Result<(), ModuleError> {
        let cost_units_per_byte = track.fee_table.wasm_instantiation_per_byte();
        let byte_length = code.len();
        track
            .fee_reserve()
            .consume_multiplied_execution(
                cost_units_per_byte,
                byte_length,
                CostingReason::InstantiateWasm,
            )
            .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))
    }

    fn on_wasm_costing(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        track: &mut Track<R>,
        units: u32,
    ) -> Result<(), ModuleError> {
        track
            .fee_reserve()
            .consume_execution(units, CostingReason::RunWasm)
            .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))
    }

    fn on_lock_fee(
        &mut self,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        track: &mut Track<R>,
        vault_id: VaultId,
        fee: Resource,
        contingent: bool,
    ) -> Result<Resource, ModuleError> {
        track
            .fee_reserve()
            .lock_fee(vault_id, fee, contingent)
            .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))
    }

    fn pre_execute_invocation(
        &mut self,
        actor: &ResolvedActor,
        _call_frame_update: &CallFrameUpdate,
        _call_frame: &CallFrame,
        _heap: &mut Heap,
        track: &mut Track<R>,
    ) -> Result<(), ModuleError> {
        match &actor.identifier {
            FnIdentifier::Native(native_fn) => {
                let cost_units = track.fee_table.run_native_fn_cost(&native_fn);
                track
                    .fee_reserve()
                    .consume_execution(cost_units, CostingReason::RunNative)
                    .map_err(|e| ModuleError::CostingError(CostingError::FeeReserveError(e)))
            }
            _ => Ok(()),
        }
    }
}
