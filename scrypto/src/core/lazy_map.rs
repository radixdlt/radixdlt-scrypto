use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::rust::marker::PhantomData;
use crate::rust::vec;
use crate::types::*;
use crate::utils::*;

/// A scalable key-value map which loads values on demand.
#[derive(Debug, Clone)]
pub struct LazyMap<K: Encode + Decode, V: Encode + Decode> {
    mid: Mid,
    key: PhantomData<K>,
    value: PhantomData<V>,
}

impl<K: Encode + Decode, V: Encode + Decode> From<Mid> for LazyMap<K, V> {
    fn from(mid: Mid) -> Self {
        Self {
            mid,
            key: PhantomData,
            value: PhantomData,
        }
    }
}

impl<K: Encode + Decode, V: Encode + Decode> From<LazyMap<K, V>> for Mid {
    fn from(a: LazyMap<K, V>) -> Mid {
        a.mid
    }
}

impl<K: Encode + Decode, V: Encode + Decode> LazyMap<K, V> {
    pub fn new() -> Self {
        let input = CreateLazyMapInput {};
        let output: CreateLazyMapOutput = call_kernel(CREATE_LAZY_MAP, input);

        output.lazy_map.into()
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let input = GetLazyMapEntryInput {
            lazy_map: self.mid,
            key: scrypto_encode(key),
        };
        let output: GetLazyMapEntryOutput = call_kernel(GET_LAZY_MAP_ENTRY, input);

        output.value.map(|v| scrypto_unwrap(scrypto_decode(&v)))
    }

    pub fn insert(&self, key: K, value: V) {
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

impl<K: Encode + Decode, V: Encode + Decode> Default for LazyMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

//========
// SBOR
//========

impl<K: Encode + Decode, V: Encode + Decode> TypeId for LazyMap<K, V> {
    fn type_id() -> u8 {
        Mid::type_id()
    }
}

impl<K: Encode + Decode, V: Encode + Decode> Encode for LazyMap<K, V> {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.mid.encode_value(encoder);
    }
}

impl<K: Encode + Decode, V: Encode + Decode> Decode for LazyMap<K, V> {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Mid::decode_value(decoder).map(Into::into)
    }
}

impl<K: Encode + Decode + Describe, V: Encode + Decode + Describe> Describe for LazyMap<K, V> {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_LAZY_MAP.to_owned(),
            generics: vec![K::describe(), V::describe()],
        }
    }
}
