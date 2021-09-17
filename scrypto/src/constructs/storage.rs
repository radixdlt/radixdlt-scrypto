use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::constants::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;
use crate::utils::*;

/// A scalable key-value storage.
#[derive(Debug, Encode, Decode)]
pub struct Storage {
    sid: SID,
}

impl From<SID> for Storage {
    fn from(sid: SID) -> Self {
        Self { sid }
    }
}

impl From<Storage> for SID {
    fn from(a: Storage) -> SID {
        a.sid
    }
}

impl Storage {
    pub fn new() -> Self {
        let input = CreateStorageInput {};
        let output: CreateStorageOutput = call_kernel(CREATE_STORAGE, input);

        output.storage.into()
    }

    pub fn get<K: Encode + ?Sized, V: Decode>(&self, key: &K) -> Option<V> {
        let input = GetStorageEntryInput {
            storage: self.sid,
            key: scrypto_encode(key),
        };
        let output: GetStorageEntryOutput = call_kernel(GET_STORAGE_ENTRY, input);

        output.value.map(|v| unwrap_light(scrypto_decode(&v)))
    }

    pub fn insert<K: Encode, V: Encode>(&self, key: K, value: V) {
        let input = PutStorageEntryInput {
            storage: self.sid,
            key: scrypto_encode(&key),
            value: scrypto_encode(&value),
        };
        let _: PutStorageEntryOutput = call_kernel(PUT_STORAGE_ENTRY, input);
    }

    pub fn sid(&self) -> SID {
        self.sid
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}

impl Describe for Storage {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_STORAGE.to_owned(),
        }
    }
}
