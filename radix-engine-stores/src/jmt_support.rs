use crate::interface::DatabaseMapper;
use radix_engine_interface::crypto::hash;
use radix_engine_interface::types::{ModuleId, NodeId, SubstateKey};

pub struct JmtMapper;

impl DatabaseMapper for JmtMapper {
    fn map_to_index_id(node_id: &NodeId, module_id: ModuleId) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend(node_id.as_ref());
        buffer.push(module_id.0);
        buffer
    }

    fn map_to_db_key(key: SubstateKey) -> Vec<u8> {
        let bytes = match key {
            SubstateKey::Tuple(field) => {
                vec![field]
            }
            SubstateKey::Map(key) => hash(key).0[12..32].to_vec(),
            SubstateKey::Sorted(bucket, key) => {
                let mut bytes = bucket.to_be_bytes().to_vec();
                bytes.extend(hash(key).0[12..32].to_vec()); // 20 bytes
                bytes
            }
        };

        bytes
    }
}
