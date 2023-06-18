use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::module::SystemModule;
use crate::types::*;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::types::*;

#[derive(Debug, Default, Clone)]
pub struct TransactionEventsModule {
    events: Vec<(EventTypeIdentifier, Vec<u8>)>,
    replacements: IndexMap<(NodeId, ObjectModuleId), (NodeId, ObjectModuleId)>,
}

impl TransactionEventsModule {
    pub fn add_event(&mut self, identifier: EventTypeIdentifier, data: Vec<u8>) {
        self.events.push((identifier, data))
    }

    pub fn add_replacement(
        &mut self,
        old: (NodeId, ObjectModuleId),
        new: (NodeId, ObjectModuleId),
    ) {
        self.replacements.insert(old, new);
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.replacements.clear();
    }

    pub fn finalize(self) -> Vec<(EventTypeIdentifier, Vec<u8>)> {
        let mut events = self.events;

        for (event_identifier, _) in events.iter_mut() {
            // Apply replacements
            let (node_id, module_id) = match event_identifier {
                EventTypeIdentifier(Emitter::Method(node_id, module_id), _) => (node_id, module_id),
                EventTypeIdentifier(Emitter::Function(node_id, module_id, _), _) => {
                    (node_id, module_id)
                }
            };
            if let Some((new_node_id, new_module_id)) =
                self.replacements.get(&(*node_id, *module_id))
            {
                *node_id = *new_node_id;
                *module_id = *new_module_id;
            }
        }

        events
    }
}

impl<K: KernelCallbackObject> SystemModule<K> for TransactionEventsModule {}
