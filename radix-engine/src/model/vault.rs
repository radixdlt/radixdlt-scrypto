use sbor::*;
use scrypto::engine::types::*;

use crate::model::{Bucket, BucketError, ResourceAmount};

/// Represents an error when accessing a vault.
#[derive(Debug, Clone)]
pub enum VaultError {
    BucketError(BucketError),
}

/// A persistent resource container on ledger state.
#[derive(Debug, TypeId, Encode, Decode)]
pub struct Vault {
    bucket: Bucket,
}

impl Vault {
    pub fn new(bucket: Bucket) -> Self {
        Self { bucket }
    }

    pub fn put(&mut self, other: Bucket) -> Result<(), VaultError> {
        self.bucket.put(other).map_err(VaultError::BucketError)
    }

    pub fn take(&mut self, amount: Decimal) -> Result<Bucket, VaultError> {
        self.bucket.take(amount).map_err(VaultError::BucketError)
    }

    pub fn take_non_fungible(&mut self, key: &NonFungibleId) -> Result<Bucket, VaultError> {
        self.bucket
            .take_non_fungible(key)
            .map_err(VaultError::BucketError)
    }

    pub fn liquid_amount(&self) -> ResourceAmount {
        self.bucket.liquid_amount()
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        self.bucket.resource_def_id()
    }

    pub fn resource_type(&self) -> ResourceType {
        self.bucket.resource_type()
    }
}
