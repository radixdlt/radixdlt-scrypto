use crate::kernel::module::KernelModule;
use radix_engine_interface::api::types::Vec;
use radix_engine_interface::events::EventTypeIdentifier;

#[derive(Debug, Clone)]
pub struct EventsModule(Vec<(EventTypeIdentifier, Vec<u8>)>);

impl Default for EventsModule {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl EventsModule {
    pub fn add_event(&mut self, identifier: EventTypeIdentifier, data: Vec<u8>) {
        self.0.push((identifier, data))
    }

    pub fn events(self) -> Vec<(EventTypeIdentifier, Vec<u8>)> {
        self.0
    }
}

impl KernelModule for EventsModule {}
