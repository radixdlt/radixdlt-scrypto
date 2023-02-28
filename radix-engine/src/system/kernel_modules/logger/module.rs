use crate::kernel::actor::Actor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::type_info::TypeInfoSubstate;
use crate::{blueprints::logger::LoggerSubstate, errors::RuntimeError, system::node::RENodeInit};
use radix_engine_interface::api::types::{NodeModuleId, RENodeId, RENodeType};
use radix_engine_interface::blueprints::logger::LOGGER_BLUEPRINT;
use radix_engine_interface::constants::LOGGER_PACKAGE;
use radix_engine_interface::data::ScryptoValue;
use sbor::btreemap;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone)]
pub struct LoggerModule {}

impl KernelModule for LoggerModule {
    fn on_init<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let logger = LoggerSubstate { logs: Vec::new() };
        let node_id = api.kernel_allocate_node_id(RENodeType::Logger)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::Logger(logger),
            btreemap!(
                NodeModuleId::TypeInfo => RENodeModuleInit::TypeInfo(TypeInfoSubstate {
                    package_address: LOGGER_PACKAGE,
                    blueprint_name: LOGGER_BLUEPRINT.to_string(),
                    global: false,
                })
            ),
        )?;

        Ok(())
    }

    fn on_teardown<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        api.kernel_drop_node(RENodeId::Logger)?;

        Ok(())
    }

    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _actor: &Option<Actor>,
        call_frame_update: &mut CallFrameUpdate,
        _args: &ScryptoValue,
    ) -> Result<(), RuntimeError> {
        call_frame_update.node_refs_to_copy.insert(RENodeId::Logger);

        Ok(())
    }
}
