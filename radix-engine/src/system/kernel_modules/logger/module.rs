use crate::{
    blueprints::logger::LoggerSubstate, errors::RuntimeError,
    kernel::kernel_api::KernelSubstateApi, kernel::*, system::node::RENodeInit,
};
use radix_engine_interface::api::types::{RENodeId, RENodeType};
use radix_engine_interface::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::vec::Vec;

#[derive(ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LoggerModule;

impl KernelModuleState for LoggerModule {
    const ID: u8 = KernelModuleId::Logger as u8;
}

impl KernelModule for LoggerModule {
    fn on_init<Y: KernelNodeApi + KernelSubstateApi>(api: &mut Y) -> Result<(), RuntimeError> {
        if api.get_module_state::<LoggerModule>().is_none() {
            return Ok(());
        }

        let logger = LoggerSubstate { logs: Vec::new() };
        let node_id = api.allocate_node_id(RENodeType::Logger)?;
        api.create_node(node_id, RENodeInit::Logger(logger), BTreeMap::new())?;
        Ok(())
    }

    fn on_teardown<Y: KernelNodeApi + KernelSubstateApi>(api: &mut Y) -> Result<(), RuntimeError> {
        if api.get_module_state::<LoggerModule>().is_none() {
            return Ok(());
        }

        api.drop_node(RENodeId::Logger)?;

        Ok(())
    }

    fn before_create_frame<Y: KernelNodeApi + KernelSubstateApi>(
        api: &mut Y,
        _actor: &ResolvedActor,
        call_frame_update: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        if api.get_module_state::<LoggerModule>().is_none() {
            return Ok(());
        }

        if api.get_visible_node_data(RENodeId::Logger).is_ok() {
            call_frame_update.node_refs_to_copy.insert(RENodeId::Logger);
        }

        Ok(())
    }
}
