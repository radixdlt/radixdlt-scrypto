use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::types::*;
use crate::utils::*;

/// A scalable key-value map which loads values on demand.
#[derive(Debug)]
pub struct LazyMap {
    mid: Mid,
}

impl From<Mid> for LazyMap {
    fn from(mid: Mid) -> Self {
        Self { mid }
    }
}

impl From<LazyMap> for Mid {
    fn from(a: LazyMap) -> Mid {
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

    pub fn mid(&self) -> Mid {
        self.mid
    }
}

impl Default for LazyMap {
    fn default() -> Self {
        Self::new()
    }
}

//========
// SBOR
//========

impl TypeId for LazyMap {
    fn type_id() -> u8 {
        Mid::type_id()
    }
}

impl Encode for LazyMap {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.mid.encode_value(encoder);
    }
}

impl Decode for LazyMap {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Mid::decode_value(decoder).map(Into::into)
    }
}

impl Describe for LazyMap {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_LAZY_MAP.to_owned(),
        }
    }
}
