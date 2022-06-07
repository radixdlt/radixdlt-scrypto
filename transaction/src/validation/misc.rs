use sbor::rust::collections::HashMap;
use scrypto::crypto::Hash;

pub trait IntentHashManager {
    fn allows(&self, hash: &Hash) -> bool;
}

pub trait EpochManager {
    fn current_epoch(&self) -> u64;
}

pub struct TestEpochManager {
    current_epoch: u64,
}

pub enum HashStatus {
    Commited,
    Cancelled,
}

pub struct TestIntentHashManager {
    hash_status_map: HashMap<Hash, HashStatus>,
}

impl TestEpochManager {
    pub fn new(current_epoch: u64) -> Self {
        Self { current_epoch }
    }
    pub fn update_epoch(&mut self, new_epoch: u64) {
        self.current_epoch = new_epoch;
    }
}

impl EpochManager for TestEpochManager {
    fn current_epoch(&self) -> u64 {
        self.current_epoch
    }
}

impl TestIntentHashManager {
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

impl IntentHashManager for TestIntentHashManager {
    fn allows(&self, hash: &Hash) -> bool {
        !self.hash_status_map.contains_key(hash)
    }
}
