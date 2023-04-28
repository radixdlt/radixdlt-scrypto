use crate::hash_tree::{DbIndex, DbKey};
use crate::interface::DatabaseMapper;
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::types::{ModuleId, NodeId, SubstateKey};
use sbor::rust::vec;
use sbor::rust::vec::Vec;

pub struct JmtMapper;

impl DatabaseMapper for JmtMapper {
    fn map_to_db_index(node_id: &NodeId, module_id: ModuleId) -> DbIndex {
        let mut buffer = Vec::new();
        buffer.extend(node_id.as_ref());
        buffer.push(module_id.0);
        hash(buffer).0[(Hash::LENGTH - 26)..Hash::LENGTH].to_vec() // 26 bytes
    }

    fn map_to_db_key(key: &SubstateKey) -> DbKey {
        let bytes = match key {
            SubstateKey::Tuple(field) => {
                vec![*field]
            }
            SubstateKey::Map(key) => {
                hash(key).0[12..Hash::LENGTH].to_vec() // 20 bytes
            }
            SubstateKey::Sorted((bucket, key)) => {
                let mut bytes = bucket.to_be_bytes().to_vec();
                bytes.extend(hash(key).0[12..Hash::LENGTH].to_vec()); // 20 bytes
                bytes
            }
        };

        bytes
    }
}
