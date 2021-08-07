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

    pub fn withdraw(&mut self, amount: U256, resource: Address) -> Result<Bucket, BucketError> {
        self.buckets
            .get_mut(&resource)
            .ok_or(BucketError::InsufficientBalance)?
            .take(amount)
    }

    pub fn deposit(&mut self, bucket: Bucket) -> Result<(), BucketError> {
        let resource = bucket.resource();

        self.buckets
            .entry(resource)
            .or_insert(Bucket::new(U256::zero(), resource))
            .put(bucket)
    }
}
