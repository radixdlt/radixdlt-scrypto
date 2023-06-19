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

    pub fn generate_uuid(&mut self) -> u128 {
        // Take the lower 16 bytes
        let mut temp: [u8; 16] = self.tx_hash.lower_bytes();

        // Put TX runtime counter to the last 4 bytes.
        temp[12..16].copy_from_slice(&self.next_id.to_be_bytes());

        // Construct UUID v4 variant 1
        let uuid = (u128::from_be_bytes(temp) & 0xffffffff_ffff_0fff_3fff_ffffffffffffu128)
            | 0x00000000_0000_4000_8000_000000000000u128;

        self.next_id += 1;

        uuid
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
    fn test_uuid_gen() {
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
            NonFungibleLocalId::uuid(id.generate_uuid())
                .unwrap()
                .to_string(),
            "{86cc8d24-194d-4393-85ee-91ee00000005}"
        );

        let mut id = TransactionRuntimeModule {
            tx_hash: Hash([0u8; 32]),
            next_id: 5,
            logs: Vec::new(),
            events: Vec::new(),
            replacements: index_map_new(),
        };
        assert_eq!(
            NonFungibleLocalId::uuid(id.generate_uuid())
                .unwrap()
                .to_string(),
            "{00000000-0000-4000-8000-000000000005}"
        );

        let mut id = TransactionRuntimeModule {
            tx_hash: Hash([255u8; 32]),
            next_id: 5,
            logs: Vec::new(),
            events: Vec::new(),
            replacements: index_map_new(),
        };
        assert_eq!(
            NonFungibleLocalId::uuid(id.generate_uuid())
                .unwrap()
                .to_string(),
            "{ffffffff-ffff-4fff-bfff-ffff00000005}"
        );
    }
}
