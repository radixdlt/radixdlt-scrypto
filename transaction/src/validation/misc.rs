use sbor::rust::collections::HashMap;
use scrypto::crypto::Hash;

pub trait IntentHashStore {
    fn allows(&self, hash: &Hash) -> bool;
}

pub enum HashStatus {
    Commited,
    Cancelled,
}

pub struct TestIntentHashStore {
    hash_status_map: HashMap<Hash, HashStatus>,
}

impl TestIntentHashStore {
    pub fn new() -> Self {
        Self {
            hash_status_map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, hash: Hash, status: HashStatus) {
        self.hash_status_map.insert(hash, status);
    }

    pub fn remove(&mut self, hash: &Hash) {
        self.hash_status_map.remove(hash);
    }
}

impl IntentHashStore for TestIntentHashStore {
    fn allows(&self, hash: &Hash) -> bool {
        !self.hash_status_map.contains_key(hash)
    }
}
