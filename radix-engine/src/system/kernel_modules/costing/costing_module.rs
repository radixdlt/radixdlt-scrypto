use super::*;
use super::{CostingReason, FeeReserveError, FeeTable, SystemLoanFeeReserve};
use crate::kernel::LockFlags;
use crate::system::node::RENodeModuleInit;
use crate::{
    errors::{CanBeAbortion, ModuleError, RuntimeError},
    kernel::{kernel_api::KernelSubstateApi, KernelModule, KernelNodeApi},
    kernel::{CallFrameUpdate, KernelModuleId, KernelModuleState, ResolvedActor},
    system::node::RENodeInit,
    transaction::AbortReason,
};
use radix_engine_interface::api::types::{FnIdentifier, LockHandle, NodeModuleId, SubstateOffset};
use radix_engine_interface::{api::types::RENodeId, *};
use sbor::rust::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct CostingModule {
    fee_reserve: SystemLoanFeeReserve,
    fee_table: FeeTable,
    max_depth: usize,
}

impl KernelModuleState for CostingModule {
    const ID: u8 = KernelModuleId::Costing as u8;
}

fn apply_execution_cost<Y: KernelNodeApi + KernelSubstateApi, F>(
    api: &mut Y,
    reason: CostingReason,
    base_price: F,
    multiplier: usize,
) -> Result<(), RuntimeError>
where
    F: Fn(&FeeTable) -> u32,
{
    if let Some(state) = api.get_module_state::<CostingModule>() {
        let cost_units = base_price(&state.fee_table);
        state
            .fee_reserve
            .consume_multiplied_execution(cost_units, multiplier, reason)
            .map_err(|e| {
                RuntimeError::ModuleError(ModuleError::CostingError(CostingError::FeeReserveError(
                    e,
                )))
            })
    } else {
        Ok(())
    }
}

fn apply_royalty_cost<Y: KernelNodeApi + KernelSubstateApi>(
    api: &mut Y,
    receiver: RoyaltyReceiver,
    amount: u32,
) -> Result<(), RuntimeError> {
    if let Some(state) = api.get_module_state::<CostingModule>() {
        state
            .fee_reserve
            .consume_royalty(receiver, amount)
            .map_err(|e| {
                RuntimeError::ModuleError(ModuleError::CostingError(CostingError::FeeReserveError(
                    e,
                )))
            })
    } else {
        Ok(())
    }
}

impl KernelModule for CostingModule {
    fn before_invoke<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        _fn_identifier: &FnIdentifier,
        input_size: usize,
    ) -> Result<(), RuntimeError> {
        let current_depth = api.get_current_depth();

        if let Some(state) = api.get_module_state::<CostingModule>() {
            if current_depth == state.max_depth {
                return Err(RuntimeError::ModuleError(ModuleError::CostingError(
                    CostingError::MaxCallDepthLimitReached,
                )));
            }

            if current_depth > 0 {
                apply_execution_cost(
                    api,
                    CostingReason::Invoke,
                    |fee_table| {
                        fee_table.kernel_api_cost(CostingEntry::Invoke {
                            input_size: input_size as u32,
                        })
                    },
                    1,
                )?;
            }
        }

        Ok(())
    }

    fn before_new_frame<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        callee: &ResolvedActor,
        _nodes_and_refs: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        match &callee.identifier {
            FnIdentifier::Native(native_fn) => apply_execution_cost(
                api,
                CostingReason::RunNative,
                |fee_table| fee_table.run_native_fn_cost(&native_fn),
                1,
            ),
            _ => Ok(()),
        }
    }

    fn before_create_node<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        _node_id: &RENodeId,
        _node_init: &RENodeInit,
        _node_module_init: &BTreeMap<NodeModuleId, RENodeModuleInit>,
    ) -> Result<(), RuntimeError> {
        // TODO: calculate size
        apply_execution_cost(
            api,
            CostingReason::CreateNode,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::CreateNode { size: 0 }),
            1,
        )?;
        Ok(())
    }

    fn after_drop_node<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        // TODO: calculate size
        apply_execution_cost(
            api,
            CostingReason::DropNode,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::DropNode { size: 0 }),
            1,
        )?;

        Ok(())
    }

    fn on_lock_substate<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        _node_id: &RENodeId,
        _module_id: &NodeModuleId,
        _offset: &SubstateOffset,
        _flags: &LockFlags,
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            api,
            CostingReason::LockSubstate,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::LockSubstate),
            1,
        )?;
        Ok(())
    }

    fn on_read_substate<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        _lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            api,
            CostingReason::ReadSubstate,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::ReadSubstate { size: size as u32 }),
            1,
        )?;
        Ok(())
    }

    fn on_write_substate<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        _lock_handle: LockHandle,
        size: usize,
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            api,
            CostingReason::WriteSubstate,
            |fee_table| {
                fee_table.kernel_api_cost(CostingEntry::WriteSubstate { size: size as u32 })
            },
            1,
        )?;
        Ok(())
    }

    fn on_drop_lock<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        _lock_handle: LockHandle,
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            api,
            CostingReason::DropLock,
            |fee_table| fee_table.kernel_api_cost(CostingEntry::DropLock),
            1,
        )?;
        Ok(())
    }

    fn on_wasm_instantiation<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        code: &[u8],
    ) -> Result<(), RuntimeError> {
        apply_execution_cost(
            api,
            CostingReason::InstantiateWasm,
            |fee_table| fee_table.wasm_instantiation_per_byte(),
            code.len(),
        )
    }

    fn on_consume_cost_units<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        units: u32,
    ) -> Result<(), RuntimeError> {
        // We multiply by a large enough factor to ensure spin loops end within a fraction of a second.
        // These values will be tweaked, alongside the whole fee table.
        apply_execution_cost(api, CostingReason::RunWasm, |_| units, 5)
    }
}
