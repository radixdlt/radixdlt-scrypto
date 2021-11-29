use sbor::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::types::*;

use crate::model::{Auth, Bucket, BucketError, Supply};

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
    owner: Address,
}

impl Vault {
    pub fn new(bucket: Bucket, owner: Address) -> Self {
        Self { bucket, owner }
    }

    pub fn put(&mut self, other: Bucket, auth: Auth) -> Result<(), VaultError> {
        if auth.check(self.owner) {
            self.bucket.put(other).map_err(VaultError::AccountingError)
        } else {
            Err(VaultError::UnauthorizedAccess)
        }
    }

    pub fn take(&mut self, amount: Decimal, auth: Auth) -> Result<Bucket, VaultError> {
        if auth.check(self.owner) {
            self.bucket
                .take(amount)
                .map_err(VaultError::AccountingError)
        } else {
            Err(VaultError::UnauthorizedAccess)
        }
    }

    pub fn take_nft(&mut self, id: u128, auth: Auth) -> Result<Bucket, VaultError> {
        if auth.check(self.owner) {
            self.bucket
                .take_nft(id)
                .map_err(VaultError::AccountingError)
        } else {
            Err(VaultError::UnauthorizedAccess)
        }
    }

    pub fn get_nft_ids(&self, auth: Auth) -> Result<BTreeSet<u128>, VaultError> {
        if auth.check(self.owner) {
            self.bucket
                .get_nft_ids()
                .map_err(VaultError::AccountingError)
        } else {
            Err(VaultError::UnauthorizedAccess)
        }
    }

    pub fn total_supply(&self, auth: Auth) -> Result<Supply, VaultError> {
        if auth.check(self.owner) {
            Ok(self.bucket.supply())
        } else {
            Err(VaultError::UnauthorizedAccess)
        }
    }

    pub fn amount(&self, auth: Auth) -> Result<Decimal, VaultError> {
        if auth.check(self.owner) {
            Ok(self.bucket.amount())
        } else {
            Err(VaultError::UnauthorizedAccess)
        }
    }

    pub fn resource_address(&self, auth: Auth) -> Result<Address, VaultError> {
        if auth.check(self.owner) {
            Ok(self.bucket.resource_address())
        } else {
            Err(VaultError::UnauthorizedAccess)
        }
    }
}
