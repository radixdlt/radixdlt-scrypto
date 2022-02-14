use sbor::*;
use scrypto::engine::types::*;
use scrypto::rust::vec::Vec;

use crate::model::{Bucket, BucketError, Resource};

/// Represents an error when accessing a vault.
#[derive(Debug, Clone)]
pub enum VaultError {
    AccountingError(BucketError),
}

/// A persistent resource container on ledger state.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Vault {
    bucket: Bucket,
}

impl Vault {
    pub fn new(bucket: Bucket) -> Self {
        Self { bucket }
    }

    pub fn put(&mut self, other: Bucket) -> Result<(), VaultError> {
        self.bucket.put(other).map_err(VaultError::AccountingError)
    }

    pub fn take(&mut self, amount: Decimal) -> Result<Bucket, VaultError> {
        self.bucket
            .take(amount)
            .map_err(VaultError::AccountingError)
    }

    pub fn take_non_fungible(&mut self, key: &NonFungibleKey) -> Result<Bucket, VaultError> {
        self.bucket
            .take_non_fungible(key)
            .map_err(VaultError::AccountingError)
    }

    pub fn get_non_fungible_ids(&self) -> Result<Vec<NonFungibleKey>, VaultError> {
        self.bucket
            .get_non_fungible_keys()
            .map_err(VaultError::AccountingError)
    }

    pub fn resource(&self) -> Resource {
        self.bucket.resource()
    }

    pub fn amount(&self) -> Decimal {
        self.bucket.amount()
    }

    pub fn resource_def_ref(&self) -> ResourceDefRef {
        self.bucket.resource_def_ref()
    }
}
