use crate::{
    blueprints::logger::LoggerSubstate, errors::RuntimeError, kernel::kernel_api::SystemApi,
    kernel::*,
};
use radix_engine_interface::api::types::{RENodeId, RENodeType};
use sbor::rust::vec::Vec;

pub struct LoggerModule;

impl LoggerModule {
    pub fn initialize<Y: SystemApi>(api: &mut Y) -> Result<(), RuntimeError> {
        let logger = LoggerSubstate { logs: Vec::new() };
        let node_id = api.allocate_node_id(RENodeType::Logger)?;
        api.create_node(node_id, RENodeInit::Logger(logger))?;
        Ok(())
    }

    pub fn on_call_frame_enter<Y: SystemApi>(
        call_frame_update: &mut CallFrameUpdate,
        _actor: &ResolvedActor,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let refed = api.get_visible_nodes()?;
        let maybe_id = refed.into_iter().find(|e| matches!(e, RENodeId::Logger));
        if let Some(logger_id) = maybe_id {
            call_frame_update.node_refs_to_copy.insert(logger_id);
        }

        Ok(())
    }
}
