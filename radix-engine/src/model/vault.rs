use sbor::*;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::model::{Auth, Bucket, BucketError};

/// Represents an error when accessing a vault.
#[derive(Debug, Clone)]
pub enum VaultError {
    AccountingError(BucketError),
    UnauthorizedAccess,
}

/// A persistent resource container on ledger state.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Vault {
    bucket: Bucket,
    auth: Address,
}

impl Vault {
    pub fn new(bucket: Bucket, auth: Address) -> Self {
        Self { bucket, auth }
    }

    pub fn put(&mut self, other: Bucket, auth: Auth) -> Result<(), VaultError> {
        if auth.contains(self.auth) {
            self.bucket.put(other).map_err(VaultError::AccountingError)
        } else {
            Err(VaultError::UnauthorizedAccess)
        }
    }

    pub fn take(&mut self, amount: Decimal, auth: Auth) -> Result<Bucket, VaultError> {
        if auth.contains(self.auth) {
            self.bucket
                .take(amount)
                .map_err(VaultError::AccountingError)
        } else {
            Err(VaultError::UnauthorizedAccess)
        }
    }

    pub fn take_nft(&mut self, id: u64, auth: Auth) -> Result<Bucket, VaultError> {
        if auth.contains(self.auth) {
            self.bucket
                .take_nft(id)
                .map_err(VaultError::AccountingError)
        } else {
            Err(VaultError::UnauthorizedAccess)
        }
    }

    pub fn get_nft_ids(&self, auth: Auth) -> Result<Vec<u64>, VaultError> {
        if auth.contains(self.auth) {
            self.bucket
                .get_nft_ids()
                .map_err(VaultError::AccountingError)
        } else {
            Err(VaultError::UnauthorizedAccess)
        }
    }

    pub fn get_nft(&self, id: u64, auth: Auth) -> Result<Vec<u8>, VaultError> {
        if auth.contains(self.auth) {
            self.bucket.get_nft(id).map_err(VaultError::AccountingError)
        } else {
            Err(VaultError::UnauthorizedAccess)
        }
    }

    pub fn amount(&self, auth: Auth) -> Result<Decimal, VaultError> {
        if auth.contains(self.auth) {
            Ok(self.bucket.amount())
        } else {
            Err(VaultError::UnauthorizedAccess)
        }
    }

    pub fn resource_def(&self, auth: Auth) -> Result<Address, VaultError> {
        if auth.contains(self.auth) {
            Ok(self.bucket.resource_def())
        } else {
            Err(VaultError::UnauthorizedAccess)
        }
    }
}
