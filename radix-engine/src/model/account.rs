use sbor::*;
use scrypto::rust::collections::*;
use scrypto::types::*;

/// An account accounts for buckets owned by a package, component or public key.
#[derive(Debug, Clone, Encode, Decode)]
pub struct Account {
    buckets: HashMap<Address, BID>,
}

impl Account {
    pub fn new() -> Account {
        Account {
            buckets: HashMap::new(),
        }
    }

    pub fn get_bucket(&self, resource: Address) -> Option<&BID> {
        self.buckets.get(&resource)
    }

    pub fn insert_bucket(&mut self, resource: Address, bid: BID) {
        assert!(bid.is_persisted());

        self.buckets.insert(resource, bid);
    }

    pub fn buckets(&self) -> &HashMap<Address, BID> {
        &self.buckets
    }
}
