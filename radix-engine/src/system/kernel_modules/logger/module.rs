use crate::kernel::actor::ResolvedActor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::{blueprints::logger::LoggerSubstate, errors::RuntimeError, system::node::RENodeInit};
use radix_engine_interface::api::types::{RENodeId, RENodeType};
use sbor::rust::collections::BTreeMap;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone)]
pub struct LoggerModule {}

impl KernelModule for LoggerModule {
    fn on_init<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let logger = LoggerSubstate { logs: Vec::new() };
        let node_id = api.kernel_allocate_node_id(RENodeType::Logger)?;
        api.kernel_create_node(node_id, RENodeInit::Logger(logger), BTreeMap::new())?;
        Ok(())
    }

    fn on_teardown<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        api.kernel_drop_node(RENodeId::Logger)?;

        Ok(())
    }

    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _actor: &ResolvedActor,
        call_frame_update: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        call_frame_update.node_refs_to_copy.insert(RENodeId::Logger);

        Ok(())
    }
}
