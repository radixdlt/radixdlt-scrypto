use crate::kernel::module::KernelModule;
use crate::types::*;
use radix_engine_interface::api::types::*;

#[derive(Debug, Default, Clone)]
pub struct EventsModule {
    events: HashMap<RENodeId, Vec<(EventTypeIdentifier, Vec<u8>, u64)>>,
    counter: u64,
}

impl EventsModule {
    pub fn add_event(&mut self, identifier: EventTypeIdentifier, data: Vec<u8>) {
        self.events
            .entry(identifier.node_id())
            .or_default()
            .push((identifier, data, self.counter));
        self.counter += 1;
    }

    pub fn remove_node_events(&mut self, node_id: &RENodeId) {
        self.events.remove(node_id);
    }

    pub fn events(self) -> Vec<(EventTypeIdentifier, Vec<u8>)> {
        let mut events = self
            .events
            .into_iter()
            .flat_map(|(_, events)| events)
            .collect::<Vec<(EventTypeIdentifier, Vec<u8>, u64)>>();
        events.sort_by(|(_, _, a), (_, _, b)| a.cmp(b));

        events
            .into_iter()
            .map(|(identifier, data, _)| (identifier, data))
            .collect()
    }
}

impl KernelModule for EventsModule {
    fn before_drop_node<
        Y: crate::kernel::kernel_api::KernelModuleApi<crate::errors::RuntimeError>,
    >(
        api: &mut Y,
        node_id: &RENodeId,
    ) -> Result<(), crate::errors::RuntimeError> {
        api.kernel_get_module_state()
            .events
            .remove_node_events(node_id);
        Ok(())
    }
}
