use crate::{
    blueprints::logger::LoggerSubstate, errors::RuntimeError,
    kernel::kernel_api::KernelSubstateApi, kernel::*, system::node::RENodeInit,
};
use radix_engine_interface::api::types::{RENodeId, RENodeType};
use sbor::rust::collections::BTreeMap;
use sbor::rust::vec::Vec;

pub struct LoggerModule;

impl KernelModule for LoggerModule {
    fn on_init<Y: KernelNodeApi + KernelSubstateApi>(api: &mut Y) -> Result<(), RuntimeError> {
        let logger = LoggerSubstate { logs: Vec::new() };
        let node_id = api.allocate_node_id(RENodeType::Logger)?;
        api.create_node(node_id, RENodeInit::Logger(logger), BTreeMap::new())?;
        Ok(())
    }

    fn on_teardown<Y: KernelNodeApi + KernelSubstateApi>(api: &mut Y) -> Result<(), RuntimeError> {
        api.drop_node(RENodeId::Logger)?;

        Ok(())
    }

    fn on_call_frame_enter<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        call_frame_update: &mut CallFrameUpdate,
        _actor: &ResolvedActor,
    ) -> Result<(), RuntimeError> {
        if api.get_visible_node_data(RENodeId::Logger).is_ok() {
            call_frame_update.node_refs_to_copy.insert(RENodeId::Logger);
        }

        Ok(())
    }
}
