use crate::errors::RuntimeError;
use crate::kernel::actor::ResolvedActor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::system::node::RENodeInit;

use radix_engine_interface::api::types::{RENodeId, RENodeType};
use radix_engine_interface::blueprints::logger::Level;
use radix_engine_interface::data::ScryptoValue;
use sbor::rust::collections::BTreeMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone)]
pub struct LoggerModule(Vec<(Level, String)>);

impl Default for LoggerModule {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl LoggerModule {
    pub fn add_log(&mut self, level: Level, message: String) {
        self.0.push((level, message))
    }

    pub fn logs(self) -> Vec<(Level, String)> {
        self.0
    }
}

impl KernelModule for LoggerModule {
    fn on_init<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let node_id = api.kernel_allocate_node_id(RENodeType::Logger)?;
        api.kernel_create_node(node_id, RENodeInit::Logger, BTreeMap::new())?;
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
