use hashbrown::HashMap;
use sbor::*;
use scrypto::types::*;

use crate::model::*;

#[derive(Debug, Clone, Encode, Decode)]
pub struct Account {
    buckets: HashMap<Address, Bucket>,
}

impl Account {
    pub fn new() -> Account {
        Account {
            buckets: HashMap::new(),
        }
    }

    pub fn withdraw_tokens(&mut self, amount: U256, resource: Address) -> Bucket {
        self.buckets
            .get_mut(&resource)
            .expect("No such resource in account")
            .take(amount)
    }

    pub fn deposit_tokens(&mut self, bucket: Bucket) {
        let resource = bucket.resource();

        self.buckets
            .entry(resource)
            .or_insert(Bucket::new(U256::zero(), resource))
            .put(bucket);
    }

    pub fn withdraw_badges(&mut self, amount: U256, resource: Address) -> Bucket {
        self.buckets
            .get_mut(&resource)
            .expect("No such resource in account")
            .take(amount)
    }

    pub fn deposit_badges(&mut self, bucket: Bucket) {
        let resource = bucket.resource();

        self.buckets
            .entry(resource)
            .or_insert(Bucket::new(U256::zero(), resource))
            .put(bucket);
    }
}
