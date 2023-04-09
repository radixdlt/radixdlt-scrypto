use crate::kernel::module::KernelModule;
use crate::types::*;
use radix_engine_interface::types::*;
use crate::system::system::SystemUpstream;
use crate::wasm::WasmEngine;

#[derive(Debug, Default, Clone)]
pub struct EventsModule(Vec<(EventTypeIdentifier, Vec<u8>)>);

impl EventsModule {
    pub fn add_event(&mut self, identifier: EventTypeIdentifier, data: Vec<u8>) {
        self.0.push((identifier, data))
    }

    pub fn events(self) -> Vec<(EventTypeIdentifier, Vec<u8>)> {
        self.0
    }
}

impl<'g, W: WasmEngine + 'g> KernelModule<SystemUpstream<'g, W>> for EventsModule {}
