use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::module::SystemModule;
use crate::types::*;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::crypto::Hash;

#[derive(Debug, Clone)]
pub struct TransactionRuntimeModule {
    pub tx_hash: Hash,
    pub next_id: u32,
    pub logs: Vec<(Level, String)>,
    pub events: Vec<(EventTypeIdentifier, Vec<u8>)>,
    pub replacements: IndexMap<(NodeId, ObjectModuleId), (NodeId, ObjectModuleId)>,
}

impl TransactionRuntimeModule {
    pub fn transaction_hash(&self) -> Hash {
        self.tx_hash
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

    pub fn finalize(
        self,
        is_success: bool,
    ) -> (Vec<(EventTypeIdentifier, Vec<u8>)>, Vec<(Level, String)>) {
        if !is_success {
            return (Vec::new(), self.logs);
        }

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

        (events, self.logs)
    }
}

impl<K: KernelCallbackObject> SystemModule<K> for TransactionRuntimeModule {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruid_gen() {
        let mut id = TransactionRuntimeModule {
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
