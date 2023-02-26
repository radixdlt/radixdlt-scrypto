use crate::kernel::actor::ResolvedActor;
use crate::kernel::call_frame::CallFrameUpdate;
use crate::kernel::kernel_api::KernelModuleApi;
use crate::kernel::module::KernelModule;
use crate::system::events::EventStoreSubstate;
use crate::{errors::RuntimeError, system::node::RENodeInit};
use radix_engine_interface::api::types::{
    EventStoreOffset, NodeModuleId, RENodeId, RENodeType, SubstateOffset,
};
use radix_engine_interface::api::LockFlags;
use radix_engine_interface::data::ScryptoValue;
use radix_engine_interface::events::EventTypeIdentifier;

#[derive(Debug, Clone, Default)]
pub struct EventsModule(pub Vec<(EventTypeIdentifier, Vec<u8>)>);

impl KernelModule for EventsModule {
    fn on_init<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        let node_id = api.kernel_allocate_node_id(RENodeType::EventStore)?;
        api.kernel_create_node(
            node_id,
            RENodeInit::EventStore(EventStoreSubstate::default()),
            Default::default(),
        )?;
        Ok(())
    }

    fn on_teardown<Y: KernelModuleApi<RuntimeError>>(api: &mut Y) -> Result<(), RuntimeError> {
        // Read all of the events stored in the RENode and store them in the module state. This is
        // done to allow for the inclusion of events into receipts.
        let events = {
            let handle = api.kernel_lock_substate(
                RENodeId::EventStore,
                NodeModuleId::SELF,
                SubstateOffset::EventStore(EventStoreOffset::EventStore),
                LockFlags::read_only(),
            )?;
            let event_store = api.kernel_get_substate_ref::<EventStoreSubstate>(handle)?;
            let events = event_store.0.clone();
            api.kernel_drop_lock(handle)?;
            events
        };
        api.kernel_get_module_state().events.0 = events;

        // Drop the RENode that stored the events; they're now all stored in the kernel module
        // state.
        api.kernel_drop_node(RENodeId::EventStore)?;

        Ok(())
    }

    fn before_push_frame<Y: KernelModuleApi<RuntimeError>>(
        _api: &mut Y,
        _actor: &Option<ResolvedActor>,
        down_movement: &mut CallFrameUpdate,
        _args: &ScryptoValue,
    ) -> Result<(), RuntimeError> {
        down_movement.node_refs_to_copy.insert(RENodeId::EventStore);

        Ok(())
    }
}
