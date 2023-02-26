use crate::kernel::actor::ResolvedActor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::{blueprints::logger::LoggerSubstate, errors::RuntimeError, system::node::RENodeInit};
use radix_engine_interface::api::types::{
    LoggerOffset, NodeModuleId, RENodeId, RENodeType, SubstateOffset,
};
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::blueprints::logger::Level;
use radix_engine_interface::data::ScryptoValue;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone)]
pub struct LoggerModule(pub Vec<(Level, String)>);

impl Default for LoggerModule {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl KernelModule for LoggerModule {
    fn on_init<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let logger = LoggerSubstate { logs: Vec::new() };
        let node_id = api.kernel_allocate_node_id(RENodeType::Logger)?;
        api.kernel_create_node(node_id, RENodeInit::Logger(logger), BTreeMap::new())?;
        Ok(())
    }

    fn on_teardown<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        // Read all of the events stored in the RENode and store them in the module state. This is
        // done to allow for the inclusion of events into receipts.
        let logs = {
            let handle = api.kernel_lock_substate(
                RENodeId::Logger,
                NodeModuleId::SELF,
                SubstateOffset::Logger(LoggerOffset::Logger),
                LockFlags::read_only(),
            )?;
            let logger = api.kernel_get_substate_ref::<LoggerSubstate>(handle)?;
            let logs = logger.logs.clone();
            api.kernel_drop_lock(handle)?;
            logs
        };
        api.kernel_get_module_state().logger.0 = logs;

        // Drop the RENode that stored the logs; they're now all stored in the kernel module state.
        api.kernel_drop_node(RENodeId::Logger)?;

        Ok(())
    }

    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _actor: &Option<ResolvedActor>,
        down_movement: &mut CallFrameUpdate,
        _args: &ScryptoValue,
    ) -> Result<(), RuntimeError> {
        down_movement.node_refs_to_copy.insert(RENodeId::Logger);

        Ok(())
    }
}
