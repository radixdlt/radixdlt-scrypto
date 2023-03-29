use radix_engine_interface::crypto::Hash;
use sbor::rust::collections::NonIterMap;

pub trait IntentHashManager {
    fn allows(&self, hash: &Hash) -> bool;
}

pub enum HashStatus {
    Committed,
    Cancelled,
}

pub struct TestIntentHashManager {
    hash_status_map: NonIterMap<Hash, HashStatus>,
}

impl TestIntentHashManager {
    pub fn new() -> Self {
        Self {
            hash_status_map: NonIterMap::new(),
        }
    }

    pub fn insert(&mut self, hash: Hash, status: HashStatus) {
        self.hash_status_map.insert(hash, status);
    }

    pub fn remove(&mut self, hash: &Hash) {
        self.hash_status_map.remove(hash);
    }
}

impl IntentHashManager for TestIntentHashManager {
    fn allows(&self, hash: &Hash) -> bool {
        !self.hash_status_map.contains_key(hash)
    }
}
