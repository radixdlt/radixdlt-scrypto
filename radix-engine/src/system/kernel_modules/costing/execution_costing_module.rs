use super::CostingReason;
use super::ExecutionFeeReserve;
use super::FeeTable;
use crate::errors::*;
use crate::kernel::*;
use crate::kernel::{CallFrameUpdate, ResolvedActor};
use crate::system::kernel_modules::costing::CostingEntry;
use crate::system::kernel_modules::costing::CostingError;
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::transaction::AbortReason;
use crate::types::*;
use radix_engine_interface::api::types::RENodeId;
use sbor::rust::ops::Fn;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoEncode, ScryptoDecode, ScryptoCategorize)]
pub enum ExecutionCostingError {
    CostingError(CostingError),
    MaxCallDepthLimitReached,
    CallFrameError(CallFrameError),
}

impl CanBeAbortion for ExecutionCostingError {
    fn abortion(&self) -> Option<&AbortReason> {
        match self {
            Self::CostingError(err) => err.abortion(),
            _ => None,
        }
    }
}

pub struct ExecutionCostingModule {
    max_depth: usize,
}

impl ExecutionCostingModule {
    pub fn new(max_depth: usize) -> Self {
        Self { max_depth }
    }
}

fn apply_execution_cost<F>(
    heap: &mut Heap,
    reason: CostingReason,
    base_price: F,
    multiplier: usize,
) -> Result<(), ModuleError>
where
    F: Fn(&FeeTable) -> u32,
{
    if let Ok(mut substate) = heap.get_substate_mut(
        RENodeId::FeeReserve,
        NodeModuleId::SELF,
        &SubstateOffset::FeeReserve(FeeReserveOffset::FeeReserve),
    ) {
        let fee_reserve_substate = substate.fee_reserve();

        let cost_units = base_price(&fee_reserve_substate.fee_table);
        fee_reserve_substate
            .fee_reserve
            .consume_multiplied_execution(cost_units, multiplier, reason)
            .map_err(|e| ModuleError::ExecutionCostingError(ExecutionCostingError::CostingError(e)))
    } else {
        Ok(())
    }
}

impl KernelModule for ExecutionCostingModule {
    fn pre_kernel_invoke(
        &mut self,
        current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        _fn_identifier: &FnIdentifier,
        input_size: usize,
    ) -> Result<(), ModuleError> {
        if current_frame.depth == self.max_depth {
            return Err(ModuleError::ExecutionCostingError(
                ExecutionCostingError::MaxCallDepthLimitReached,
            ));
        }

        if current_frame.depth > 0 {
            apply_execution_cost(
                heap,
                CostingReason::Invoke,
                |fee_table| {
                    fee_table.kernel_api_cost(CostingEntry::Invoke {
                        input_size: input_size as u32,
                    })
                },
                1,
            )?;
        }

        Ok(())
    }

    fn pre_kernel_execute(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        callee: &ResolvedActor,
        _nodes_and_refs: &CallFrameUpdate,
    ) -> Result<(), ModuleError> {
        match &callee.identifier {
            FnIdentifier::Native(native_fn) => apply_execution_cost(
                heap,
                CostingReason::RunNative,
                |fee_table| fee_table.run_native_fn_cost(&native_fn),
                1,
            ),
            _ => Ok(()),
        }
    }

    fn pre_create_node(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        _node_id: &RENodeId,
        _node_init: &RENodeInit,
        _node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), ModuleError> {
        // TODO: calculate size
        apply_execution_cost(
            heap,
            CostingReason::CreateNode,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::CreateNode { size: 0 }),
            1,
        )?;
        Ok(())
    }

    fn post_drop_node(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
    ) -> Result<(), ModuleError> {
        // TODO: calculate size
        apply_execution_cost(
            heap,
            CostingReason::DropNode,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::DropNode { size: 0 }),
            1,
        )?;
        Ok(())
    }

    fn on_lock_substate(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        _node_id: &RENodeId,
        _module_id: &NodeModuleId,
        _offset: &SubstateOffset,
        _flags: &LockFlags,
    ) -> Result<(), ModuleError> {
        apply_execution_cost(
            heap,
            CostingReason::LockSubstate,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::LockSubstate),
            1,
        )?;
        Ok(())
    }

    fn on_read_substate(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        _lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), ModuleError> {
        apply_execution_cost(
            heap,
            CostingReason::ReadSubstate,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::ReadSubstate { size: size as u32 }),
            1,
        )?;
        Ok(())
    }

    fn on_write_substate(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        _lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), ModuleError> {
        apply_execution_cost(
            heap,
            CostingReason::WriteSubstate,
            |fee_table| {
                fee_table.kernel_api_cost(CostingEntry::WriteSubstate { size: size as u32 })
            },
            1,
        )?;
        Ok(())
    }

    fn on_drop_lock(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        _lock_handle: LockHandle,
    ) -> Result<(), ModuleError> {
        apply_execution_cost(
            heap,
            CostingReason::DropLock,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::DropLock),
            1,
        )?;
        Ok(())
    }

    fn on_wasm_instantiation(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        code: &[u8],
    ) -> Result<(), ModuleError> {
        apply_execution_cost(
            heap,
            CostingReason::InstantiateWasm,
            |fee_table| fee_table.wasm_instantiation_per_byte(),
            code.len(),
        )
    }

    fn on_wasm_costing(
        &mut self,
        _current_frame: &CallFrame,
        heap: &mut Heap,
        _track: &mut Track,
        units: u32,
    ) -> Result<(), ModuleError> {
        // We multiply by a large enough factor to ensure spin loops end within a fraction of a second.
        // These values will be tweaked, alongside the whole fee table.
        apply_execution_cost(heap, CostingReason::RunWasm, |_| units, 5)
    }
}
