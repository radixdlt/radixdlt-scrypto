use sbor::*;
use scrypto::rust::collections::*;
use scrypto::rust::vec::Vec;
use scrypto::types::Address;

use crate::model::Auth;

/// Represents an error when accessing a bucket.
#[derive(Debug, Clone)]
pub enum LazyMapError {
    UnauthorizedAccess,
}

/// A key-value map where keys and values are lazily loaded on-demand.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct LazyMap {
    map: HashMap<Vec<u8>, Vec<u8>>,
    auth: Address,
}

impl LazyMap {
    pub fn new(auth: Address) -> Self {
        Self {
            map: HashMap::new(),
            auth,
        }
    }

    // for diagnosis purpose only
    pub fn map(&self) -> &HashMap<Vec<u8>, Vec<u8>> {
        &self.map
    }

    pub fn get_entry(&self, key: &[u8], auth: Auth) -> Result<Option<&[u8]>, LazyMapError> {
        if auth.contains(self.auth) {
            Ok(self.map.get(key).map(|e| e.as_slice()))
        } else {
            Err(LazyMapError::UnauthorizedAccess)
        }
    }

    pub fn set_entry(
        &mut self,
        key: Vec<u8>,
        value: Vec<u8>,
        auth: Auth,
    ) -> Result<(), LazyMapError> {
        if auth.contains(self.auth) {
            self.map.insert(key, value);
            Ok(())
        } else {
            Err(LazyMapError::UnauthorizedAccess)
        }
    }
}
