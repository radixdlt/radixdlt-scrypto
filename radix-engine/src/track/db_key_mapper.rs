use radix_engine_common::data::scrypto::{scrypto_decode, ScryptoDecode};
use radix_engine_interface::crypto::{hash, Hash};
use radix_engine_interface::types::{ModuleId, NodeId, SubstateKey};
use radix_engine_store_interface::interface::SubstateDatabase;
use sbor::rust::prelude::*;

pub trait DatabaseMapper {
    fn map_to_db_index(node_id: &NodeId, module_id: ModuleId) -> Vec<u8>;
    fn map_to_db_key(key: &SubstateKey) -> Vec<u8>;
}

pub struct JmtMapper;

impl DatabaseMapper for JmtMapper {
    fn map_to_db_index(node_id: &NodeId, module_id: ModuleId) -> Vec<u8> {
        let mut buffer = Vec::new();
        buffer.extend(node_id.as_ref());
        buffer.push(module_id.0);
        hash(buffer).0[(Hash::LENGTH - 26)..Hash::LENGTH].to_vec() // 26 bytes
    }

    fn map_to_db_key(key: &SubstateKey) -> Vec<u8> {
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

/// Convenience methods for direct `SubstateDatabase` readers.
pub trait MappedSubstateDatabase {
    fn get_mapped_substate<M: DatabaseMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<D>;

    fn list_mapped_substates<M: DatabaseMapper>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + '_>;
}

impl<S: SubstateDatabase> MappedSubstateDatabase for S {
    fn get_mapped_substate<M: DatabaseMapper, D: ScryptoDecode>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
        substate_key: &SubstateKey,
    ) -> Option<D> {
        self.get_substate(
            &M::map_to_db_index(node_id, module_id),
            &M::map_to_db_key(&substate_key),
        )
        .map(|buf| scrypto_decode(&buf).unwrap())
    }

    fn list_mapped_substates<M: DatabaseMapper>(
        &self,
        node_id: &NodeId,
        module_id: ModuleId,
    ) -> Box<dyn Iterator<Item = (Vec<u8>, Vec<u8>)> + '_> {
        self.list_substates(&M::map_to_db_index(node_id, module_id))
    }
}
