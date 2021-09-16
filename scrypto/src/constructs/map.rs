use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::constants::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;
use crate::utils::*;

/// A scalable key-value storage.
#[derive(Debug, Encode, Decode)]
pub struct Map {
    mid: MID,
}

impl From<MID> for Map {
    fn from(mid: MID) -> Self {
        Self { mid }
    }
}

impl From<Map> for MID {
    fn from(a: Map) -> MID {
        a.mid
    }
}

impl Map {
    pub fn new() -> Self {
        let input = CreateMapInput {};
        let output: CreateMapOutput = call_kernel(CREATE_MAP, input);

        output.map.into()
    }

    pub fn get<K: Encode + ?Sized, V: Decode>(&self, key: &K) -> Option<V> {
        let input = GetMapEntryInput {
            map: self.mid,
            key: scrypto_encode(key),
        };
        let output: GetMapEntryOutput = call_kernel(GET_MAP_ENTRY, input);

        output.value.map(|v| unwrap_light(scrypto_decode(&v)))
    }

    pub fn insert<K: Encode, V: Encode>(&self, key: K, value: V) {
        let input = PutMapEntryInput {
            map: self.mid,
            key: scrypto_encode(&key),
            value: scrypto_encode(&value),
        };
        let _: PutMapEntryOutput = call_kernel(PUT_MAP_ENTRY, input);
    }

    pub fn mid(&self) -> MID {
        self.mid
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Describe for Map {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_MAP.to_owned(),
        }
    }
}
