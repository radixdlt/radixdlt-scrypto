use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::constants::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;
use crate::utils::*;

/// A scalable key-value lazy_map.
#[derive(Debug, TypeId, Encode, Decode)]
pub struct LazyMap {
    mid: MID,
}

impl From<MID> for LazyMap {
    fn from(mid: MID) -> Self {
        Self { mid }
    }
}

impl From<LazyMap> for MID {
    fn from(a: LazyMap) -> MID {
        a.mid
    }
}

impl LazyMap {
    pub fn new() -> Self {
        let input = CreateLazyMapInput {};
        let output: CreateLazyMapOutput = call_kernel(CREATE_LAZY_MAP, input);

        output.lazy_map.into()
    }

    pub fn get<K: Encode + ?Sized, V: Decode>(&self, key: &K) -> Option<V> {
        let input = GetLazyMapEntryInput {
            lazy_map: self.mid,
            key: scrypto_encode(key),
        };
        let output: GetLazyMapEntryOutput = call_kernel(GET_LAZY_MAP_ENTRY, input);

        output.value.map(|v| unwrap_light(scrypto_decode(&v)))
    }

    pub fn insert<K: Encode, V: Encode>(&self, key: K, value: V) {
        let input = PutLazyMapEntryInput {
            lazy_map: self.mid,
            key: scrypto_encode(&key),
            value: scrypto_encode(&value),
        };
        let _: PutLazyMapEntryOutput = call_kernel(PUT_LAZY_MAP_ENTRY, input);
    }

    pub fn mid(&self) -> MID {
        self.mid
    }
}

impl Default for LazyMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Describe for LazyMap {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_LAZY_MAP.to_owned(),
        }
    }
}
