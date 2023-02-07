use crate::{
    blueprints::fee_reserve::FeeReserveSubstate,
    errors::RuntimeError,
    kernel::{kernel_api::KernelSubstateApi, KernelNodeApi},
    kernel::{CallFrameUpdate, ResolvedActor},
    system::node::RENodeInit,
};
use radix_engine_interface::api::types::{RENodeId, RENodeType};
use sbor::rust::collections::BTreeMap;

use super::{FeeTable, SystemLoanFeeReserve};

pub struct CostingModule;

impl CostingModule {
    pub fn initialize<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        fee_reserve: SystemLoanFeeReserve,
        fee_table: FeeTable,
    ) -> Result<(), RuntimeError> {
        let node_id = api.allocate_node_id(RENodeType::FeeReserve)?;
        api.create_node(
            node_id,
            RENodeInit::FeeReserve(FeeReserveSubstate {
                fee_reserve,
                fee_table,
            }),
            BTreeMap::new(),
        )?;
        Ok(())
    }

    pub fn teardown<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
    ) -> Result<FeeReserveSubstate, RuntimeError> {
        let substate: FeeReserveSubstate = api.drop_node(RENodeId::FeeReserve)?.into();

        Ok(substate)
    }

    pub fn on_call_frame_enter<Y: KernelNodeApi + KernelSubstateApi>(
        call_frame_update: &mut CallFrameUpdate,
        _actor: &ResolvedActor,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        if api.get_visible_node_data(RENodeId::FeeReserve).is_ok() {
            call_frame_update
                .node_refs_to_copy
                .insert(RENodeId::FeeReserve);
        }

        Ok(())
    }
}
