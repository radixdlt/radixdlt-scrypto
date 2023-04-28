use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::module::SystemModule;
use crate::types::*;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::types::*;

#[derive(Debug, Default, Clone)]
pub struct EventsModule {
    index: u32,
    event_store:
        IndexMap<(NodeId, ObjectModuleId), Vec<(Option<String>, LocalTypeIndex, Vec<u8>, u32)>>,
}

impl EventsModule {
    pub fn add_event(&mut self, identifier: EventTypeIdentifier, data: Vec<u8>) {
        let (node_id, module_id, blueprint_name, local_type_index) = match identifier {
            EventTypeIdentifier(Emitter::Method(node_id, module_id), local_type_index) => {
                (node_id, module_id, None, local_type_index)
            }
            EventTypeIdentifier(
                Emitter::Function(node_id, module_id, blueprint_name),
                local_type_index,
            ) => (node_id, module_id, Some(blueprint_name), local_type_index),
        };
        self.event_store
            .entry((node_id, module_id))
            .or_default()
            .push((blueprint_name, local_type_index, data, self.index));
        self.index += 1;
    }

    pub fn replace_key(&mut self, old: (NodeId, ObjectModuleId), new: (NodeId, ObjectModuleId)) {
        if let Some(value) = self.event_store.remove(&old) {
            self.event_store.insert(new, value);
        }
    }

    pub fn events(self) -> Vec<(EventTypeIdentifier, Vec<u8>)> {
        let mut events = self
            .event_store
            .into_iter()
            .flat_map(|((node_id, module_id), events)| {
                let mut resolved_events = Vec::new();
                for (blueprint_name, local_type_index, data, index) in events {
                    let event = if let Some(blueprint_name) = blueprint_name {
                        (
                            EventTypeIdentifier(
                                Emitter::Function(node_id, module_id, blueprint_name),
                                local_type_index,
                            ),
                            data,
                            index,
                        )
                    } else {
                        (
                            EventTypeIdentifier(
                                Emitter::Method(node_id, module_id),
                                local_type_index,
                            ),
                            data,
                            index,
                        )
                    };
                    resolved_events.push(event);
                }

                resolved_events
            })
            .collect::<Vec<(EventTypeIdentifier, Vec<u8>, u32)>>();
        events.sort_by(|(_, _, a), (_, _, b)| a.cmp(b));

        events
            .into_iter()
            .map(|(identifier, data, _)| (identifier, data))
            .collect()
    }
}

impl<K: KernelCallbackObject> SystemModule<K> for EventsModule {}
