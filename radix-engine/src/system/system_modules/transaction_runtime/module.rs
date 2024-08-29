use crate::internal_prelude::*;
use crate::system::module::*;
use crate::system::system_callback::*;
use radix_common::crypto::Hash;
use radix_engine_interface::api::actor_api::EventFlags;
use radix_engine_interface::api::ModuleId;

#[derive(Debug, Clone)]
pub struct Event {
    pub type_identifier: EventTypeIdentifier,
    pub payload: Vec<u8>,
    pub flags: EventFlags,
}

/// Size of event flags when calculating event storage cost.
pub const EVENT_FLAGS_LEN: usize = 4;

impl Event {
    pub fn len(&self) -> usize {
        self.type_identifier.len() + self.payload.len() + EVENT_FLAGS_LEN
    }
}

#[derive(Debug, Clone)]
pub struct TransactionRuntimeModule {
    pub network_definition: NetworkDefinition,
    pub tx_hash: Hash,
    pub next_id: u32,
    pub logs: Vec<(Level, String)>,
    pub events: Vec<Event>,
    pub replacements: IndexMap<(NodeId, ModuleId), (NodeId, ModuleId)>,
}

impl TransactionRuntimeModule {
    pub fn new(network_definition: NetworkDefinition, tx_hash: Hash) -> Self {
        TransactionRuntimeModule {
            network_definition,
            tx_hash,
            next_id: 0,
            logs: Vec::new(),
            events: Vec::new(),
            replacements: index_map_new(),
        }
    }

    pub fn generate_ruid(&mut self) -> [u8; 32] {
        let mut bytes = [0u8; 36];
        (&mut bytes[..32]).copy_from_slice(self.tx_hash.as_slice());
        bytes[32..].copy_from_slice(&self.next_id.to_le_bytes());

        self.next_id += 1;

        hash(bytes).0
    }

    pub fn add_log(&mut self, level: Level, message: String) {
        self.logs.push((level, message))
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event)
    }

    pub fn add_replacement(&mut self, old: (NodeId, ModuleId), new: (NodeId, ModuleId)) {
        self.replacements.insert(old, new);
    }

    pub fn finalize(
        self,
        is_success: bool,
    ) -> (Vec<(EventTypeIdentifier, Vec<u8>)>, Vec<(Level, String)>) {
        let mut results = Vec::new();

        for Event {
            mut type_identifier,
            payload,
            flags,
        } in self.events.into_iter()
        {
            // Revert if failure
            if !flags.contains(EventFlags::FORCE_WRITE) && !is_success {
                continue;
            }

            // Apply replacements
            match &mut type_identifier {
                EventTypeIdentifier(Emitter::Method(node_id, module_id), _) => {
                    if let Some((new_node_id, new_module_id)) =
                        self.replacements.get(&(*node_id, *module_id))
                    {
                        *node_id = *new_node_id;
                        *module_id = *new_module_id;
                    }
                }
                _ => {}
            };

            // Add to results
            results.push((type_identifier, payload))
        }

        (results, self.logs)
    }
}

impl InitSystemModule for TransactionRuntimeModule {}
impl ResolvableSystemModule for TransactionRuntimeModule {
    #[inline]
    fn resolve_from_system(system: &mut impl HasModules) -> &mut Self {
        &mut system.modules_mut().transaction_runtime
    }
}
impl PrivilegedSystemModule for TransactionRuntimeModule {}
impl<ModuleApi: SystemModuleApiFor<Self>> SystemModule<ModuleApi> for TransactionRuntimeModule {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruid_gen() {
        let mut id = TransactionRuntimeModule {
            network_definition: NetworkDefinition::simulator(),
            tx_hash: Hash::from_str(
                "71f26aab5eec6679f67c71211aba9a3486cc8d24194d339385ee91ee5ca7b30d",
            )
            .unwrap(),
            next_id: 5,
            logs: Vec::new(),
            events: Vec::new(),
            replacements: index_map_new(),
        };
        assert_eq!(
            NonFungibleLocalId::ruid(id.generate_ruid()).to_string(),
            "{7b003d8e0b2c9e3a-516cf99882de64a1-f1cd6742ce3299e0-357f54f0333d25d0}"
        );

        let mut id = TransactionRuntimeModule {
            network_definition: NetworkDefinition::simulator(),
            tx_hash: Hash([0u8; 32]),
            next_id: 5,
            logs: Vec::new(),
            events: Vec::new(),
            replacements: index_map_new(),
        };
        assert_eq!(
            NonFungibleLocalId::ruid(id.generate_ruid()).to_string(),
            "{69f38caee99e9468-866032d1a68b4d2e-7931bb74aede4d0f-8043d3a87e9f2da3}"
        );

        let mut id = TransactionRuntimeModule {
            network_definition: NetworkDefinition::simulator(),
            tx_hash: Hash([255u8; 32]),
            next_id: 5,
            logs: Vec::new(),
            events: Vec::new(),
            replacements: index_map_new(),
        };
        assert_eq!(
            NonFungibleLocalId::ruid(id.generate_ruid()).to_string(),
            "{04660ebc8e2a2b36-44a6553bd6a17a3a-ef14ce1fae4cb5bc-000811f979007003}"
        );
    }
}
