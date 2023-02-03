use crate::{
    blueprints::logger::LoggerSubstate, errors::RuntimeError,
    kernel::kernel_api::KernelSubstateApi, kernel::*,
};
use radix_engine_interface::api::types::{RENodeId, RENodeType};
use sbor::rust::collections::BTreeMap;
use sbor::rust::vec::Vec;

pub struct LoggerModule;

impl LoggerModule {
    pub fn initialize<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let logger = LoggerSubstate { logs: Vec::new() };
        let node_id = api.allocate_node_id(RENodeType::Logger)?;
        api.create_node(node_id, RENodeInit::Logger(logger), BTreeMap::new())?;
        Ok(())
    }

    pub fn on_call_frame_enter<Y: KernelNodeApi + KernelSubstateApi>(
        call_frame_update: &mut CallFrameUpdate,
        _actor: &ResolvedActor,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        if api.get_visible_node_data(RENodeId::Logger).is_ok() {
            call_frame_update.node_refs_to_copy.insert(RENodeId::Logger);
        }

        Ok(())
    }
}
