use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::crypto::*;
use crate::engine::{api::*, call_engine, types::LazyMapId};
use crate::misc::*;
use crate::rust::fmt;
use crate::rust::marker::PhantomData;
use crate::rust::str::FromStr;
use crate::rust::vec;
use crate::types::*;

/// A scalable key-value map which loads entries on demand.
#[derive(Debug, PartialEq, Eq)]
pub struct LazyMap<K: Encode + Decode, V: Encode + Decode> {
    id: LazyMapId,
    key: PhantomData<K>,
    value: PhantomData<V>,
}

impl<K: Encode + Decode, V: Encode + Decode> LazyMap<K, V> {
    /// Creates a new lazy map.
    pub fn new() -> Self {
        let input = CreateLazyMapInput {};
        let output: CreateLazyMapOutput = call_engine(CREATE_LAZY_MAP, input);

        Self {
            id: output.lazy_map_id,
            key: PhantomData,
            value: PhantomData,
        }
    }

    /// Returns the value that is associated with the given key.
    pub fn get(&self, key: &K) -> Option<V> {
        let input = GetLazyMapEntryInput {
            lazy_map_id: self.id,
            key: scrypto_encode(key),
        };
        let output: GetLazyMapEntryOutput = call_engine(GET_LAZY_MAP_ENTRY, input);

        output.value.map(|v| scrypto_decode(&v).unwrap())
    }

    /// Inserts a new key-value pair into this map.
    pub fn insert(&self, key: K, value: V) {
        let input = PutLazyMapEntryInput {
            lazy_map_id: self.id,
            key: scrypto_encode(&key),
            value: scrypto_encode(&value),
        };
        let _: PutLazyMapEntryOutput = call_engine(PUT_LAZY_MAP_ENTRY, input);
    }
}

//========
// error
//========

#[derive(Debug, Clone)]
pub enum ParseLazyMapError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseLazyMapError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseLazyMapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl<K: Encode + Decode, V: Encode + Decode> TryFrom<&[u8]> for LazyMap<K, V> {
    type Error = ParseLazyMapError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            36 => Ok(Self {
                id: (
                    Hash(copy_u8_array(&slice[0..32])),
                    u32::from_le_bytes(copy_u8_array(&slice[32..])),
                ),
                key: PhantomData,
                value: PhantomData,
            }),
            _ => Err(ParseLazyMapError::InvalidLength(slice.len())),
        }
    }
}

impl<K: Encode + Decode, V: Encode + Decode> LazyMap<K, V> {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut v = self.id.0.to_vec();
        v.extend(self.id.1.to_le_bytes());
        v
    }
}

impl<K: Encode + Decode, V: Encode + Decode> TypeId for LazyMap<K, V> {
    #[inline]
    fn type_id() -> u8 {
        CustomType::LazyMap.id()
    }
}

impl<K: Encode + Decode, V: Encode + Decode> Encode for LazyMap<K, V> {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }
}

impl<K: Encode + Decode, V: Encode + Decode> Decode for LazyMap<K, V> {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(CustomType::LazyMap.id()))
    }
}

impl<K: Encode + Decode + Describe, V: Encode + Decode + Describe> Describe for LazyMap<K, V> {
    fn describe() -> Type {
        Type::Custom {
            name: CustomType::LazyMap.name(),
            generics: vec![K::describe(), V::describe()],
        }
    }
}

//======
// text
//======

impl<K: Encode + Decode, V: Encode + Decode> FromStr for LazyMap<K, V> {
    type Err = ParseLazyMapError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseLazyMapError::InvalidHex)?;
        Self::try_from(bytes.as_slice())
    }
}

impl<K: Encode + Decode, V: Encode + Decode> ToString for LazyMap<K, V> {
    fn to_string(&self) -> String {
        hex::encode(self.to_vec())
    }
}
