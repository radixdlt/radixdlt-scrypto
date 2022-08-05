use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::marker::PhantomData;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::core::ScryptoRENode;

use crate::abi::*;
use crate::buffer::*;
use crate::core::{DataRef, DataRefMut};
use crate::crypto::*;
use crate::engine::types::{RENodeId, SubstateId};
use crate::engine::{api::*, call_engine, types::KeyValueStoreId};
use crate::misc::*;

/// A scalable key-value map which loads entries on demand.
pub struct KeyValueStore<K: Encode + Decode, V: 'static + Encode + Decode + TypeId> {
    pub id: KeyValueStoreId,
    pub key: PhantomData<K>,
    pub value: PhantomData<V>,
}

impl<K: Encode + Decode, V: 'static + Encode + Decode + TypeId> KeyValueStore<K, V> {
    /// Creates a new key value store.
    pub fn new() -> Self {
        let input = RadixEngineInput::RENodeCreate(ScryptoRENode::KeyValueStore);
        let output: RENodeId = call_engine(input);

        Self {
            id: output.into(),
            key: PhantomData,
            value: PhantomData,
        }
    }

    /// Returns the value that is associated with the given key.
    pub fn get(&self, key: &K) -> Option<DataRef<V>> {
        let substate_id = SubstateId::KeyValueStoreEntry(self.id, scrypto_encode(key));
        let input = RadixEngineInput::SubstateRead(substate_id.clone());
        let value: Option<V> = call_engine(input);
        value.map(|value| DataRef::new(value))
    }

    pub fn get_mut(&mut self, key: &K) -> Option<DataRefMut<V>> {
        let substate_id = SubstateId::KeyValueStoreEntry(self.id, scrypto_encode(key));
        let input = RadixEngineInput::SubstateRead(substate_id.clone());
        let value: Option<V> = call_engine(input);
        value.map(|value| DataRefMut::new(substate_id, value))
    }

    /// Inserts a new key-value pair into this map.
    pub fn insert(&self, key: K, value: V) {
        let substate_id = SubstateId::KeyValueStoreEntry(self.id, scrypto_encode(&key));
        let input = RadixEngineInput::SubstateWrite(substate_id, scrypto_encode(&value));
        call_engine(input)
    }
}

//========
// error
//========

/// Represents an error when decoding key value store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseKeyValueStoreError {
    InvalidHex(String),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseKeyValueStoreError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseKeyValueStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl<K: Encode + Decode, V: 'static + Encode + Decode + TypeId> TryFrom<&[u8]>
    for KeyValueStore<K, V>
{
    type Error = ParseKeyValueStoreError;

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
            _ => Err(ParseKeyValueStoreError::InvalidLength(slice.len())),
        }
    }
}

impl<K: Encode + Decode, V: 'static + Encode + Decode + TypeId> KeyValueStore<K, V> {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut v = self.id.0.to_vec();
        v.extend(self.id.1.to_le_bytes());
        v
    }
}

impl<K: Encode + Decode, V: 'static + Encode + Decode + TypeId> TypeId for KeyValueStore<K, V> {
    #[inline]
    fn type_id() -> u8 {
        ScryptoType::KeyValueStore.id()
    }
}

impl<K: Encode + Decode, V: 'static + Encode + Decode + TypeId> Encode for KeyValueStore<K, V> {
    #[inline]
    fn encode_type_id(&self, encoder: &mut Encoder) {
        encoder.write_type_id(Self::type_id());
    }

    #[inline]
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_dynamic_size(bytes.len());
        encoder.write_slice(&bytes);
    }
}

impl<K: Encode + Decode, V: 'static + Encode + Decode + TypeId> Decode for KeyValueStore<K, V> {
    fn check_type_id(decoder: &mut Decoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }

    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_dynamic_size()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice)
            .map_err(|_| DecodeError::CustomError("Failed to decode KeyValueStore".to_string()))
    }
}

impl<K: Encode + Decode + Describe, V: 'static + Encode + Decode + TypeId + Describe> Describe
    for KeyValueStore<K, V>
{
    fn describe() -> Type {
        Type::Custom {
            type_id: ScryptoType::KeyValueStore.id(),
            generics: vec![K::describe(), V::describe()],
        }
    }
}

//======
// text
//======

impl<K: Encode + Decode, V: Encode + Decode + TypeId> FromStr for KeyValueStore<K, V> {
    type Err = ParseKeyValueStoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseKeyValueStoreError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl<K: Encode + Decode, V: Encode + Decode + TypeId> fmt::Display for KeyValueStore<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl<K: Encode + Decode, V: Encode + Decode + TypeId> fmt::Debug for KeyValueStore<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
