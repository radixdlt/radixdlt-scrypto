use crate::system::module::SystemModule;
use crate::system::system_callback::SystemCallback;
use crate::types::*;
use crate::vm::wasm::WasmEngine;
use radix_engine_interface::types::*;

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

impl<'g, W: WasmEngine + 'g> SystemModule<SystemCallback<'g, W>> for EventsModule {}
