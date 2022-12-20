use crate::state_manager::{
    StagedSubstateStoreKey, StagedSubstateStoreNodeKey, StagedSubstateStoreVisitor,
};

use core::hash::Hash;
use sbor::rust::collections::HashMap;
use slotmap::SecondaryMap;

pub struct ExecutionCache<H> {
    root_accumulator_hash: H,
    accumulator_hash_to_key: HashMap<H, StagedSubstateStoreNodeKey>,
    key_to_accumulator_hash: SecondaryMap<StagedSubstateStoreNodeKey, H>,
}

impl<H> StagedSubstateStoreVisitor for ExecutionCache<H>
where
    H: Eq + Hash,
{
    fn remove_node(&mut self, key: &StagedSubstateStoreNodeKey) {
        match self.key_to_accumulator_hash.get(*key) {
            None => {}
            Some(accumulator_hash) => {
                self.accumulator_hash_to_key.remove(accumulator_hash);
            }
        };
    }
}

impl<H> ExecutionCache<H>
where
    H: Eq + Hash + Copy,
{
    pub fn new(root_accumulator_hash: H) -> Self {
        ExecutionCache {
            root_accumulator_hash,
            accumulator_hash_to_key: HashMap::new(),
            key_to_accumulator_hash: SecondaryMap::new(),
        }
    }

    pub fn get(&self, accumulator_hash: &H) -> Option<StagedSubstateStoreKey> {
        match self.accumulator_hash_to_key.get(accumulator_hash) {
            None => {
                if *accumulator_hash == self.root_accumulator_hash {
                    return Some(StagedSubstateStoreKey::RootStoreKey);
                }
                None
            }
            Some(node_key) => Some(StagedSubstateStoreKey::InternalNodeStoreKey(
                node_key.clone(),
            )),
        }
    }

    pub fn set(&mut self, accumulator_hash: &H, key: StagedSubstateStoreKey) {
        match key {
            StagedSubstateStoreKey::RootStoreKey => {
                self.root_accumulator_hash = *accumulator_hash;
            }
            StagedSubstateStoreKey::InternalNodeStoreKey(node_key) => {
                self.key_to_accumulator_hash[node_key] = *accumulator_hash;
                self.accumulator_hash_to_key
                    .insert(*accumulator_hash, node_key);
            }
        }
    }
}
