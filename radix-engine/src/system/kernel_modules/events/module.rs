use crate::kernel::actor::ResolvedActor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::system::events::EventStoreSubstate;
use crate::{errors::RuntimeError, system::node::RENodeInit};
use radix_engine_interface::api::types::{RENodeId, RENodeType};
use sbor::rust::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct EventsModule {}

impl KernelModule for EventsModule {
    fn on_init<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let node_id = api.kernel_allocate_node_id(RENodeType::EventStore)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::EventStore(EventStoreSubstate::default()),
            BTreeMap::new(),
        )?;
        Ok(())
    }

    fn on_teardown<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        api.kernel_drop_node(RENodeId::EventStore)?;

        Ok(())
    }

    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _actor: &Option<ResolvedActor>,
        call_frame_update: &mut CallFrameUpdate,
    ) -> Result<(), RuntimeError> {
        call_frame_update
            .node_refs_to_copy
            .insert(RENodeId::EventStore);

        Ok(())
    }
}
