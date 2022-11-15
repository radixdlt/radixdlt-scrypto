use radix_engine_lib::engine::api::Syscalls;
use radix_engine_lib::engine::types::{
    KeyValueStoreId, KeyValueStoreOffset, RENodeId, ScryptoRENode, SubstateOffset,
};
use sbor::rust::borrow::ToOwned;
use sbor::rust::boxed::Box;
use sbor::rust::fmt;
use sbor::rust::marker::PhantomData;
use sbor::rust::str::FromStr;
use sbor::rust::string::*;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::misc::copy_u8_array;

use crate::abi::*;
use crate::buffer::*;
use crate::engine::scrypto_env::ScryptoEnv;
use radix_engine_lib::data::*;
use radix_engine_lib::crypto::*;
use crate::core::{DataRef, DataRefMut};
use crate::misc::*;

/// A scalable key-value map which loads entries on demand.
pub struct KeyValueStore<
    K: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    V: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
> {
    pub id: KeyValueStoreId,
    pub key: PhantomData<K>,
    pub value: PhantomData<V>,
}

// TODO: de-duplication
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct KeyValueStoreEntrySubstate(pub Option<Vec<u8>>);

impl<
        K: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
        V: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    > KeyValueStore<K, V>
{
    /// Creates a new key value store.
    pub fn new() -> Self {
        let mut syscalls = ScryptoEnv;
        let id = syscalls
            .sys_create_node(ScryptoRENode::KeyValueStore)
            .unwrap();

        Self {
            id: id.into(),
            key: PhantomData,
            value: PhantomData,
        }
    }

    /// Returns the value that is associated with the given key.
    pub fn get(&self, key: &K) -> Option<DataRef<V>> {
        let mut syscalls = ScryptoEnv;
        let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(scrypto_encode(key)));
        let lock_handle = syscalls
            .sys_lock_substate(RENodeId::KeyValueStore(self.id), offset, false)
            .unwrap();
        let raw_bytes = syscalls.sys_read(lock_handle).unwrap();
        let value: KeyValueStoreEntrySubstate = scrypto_decode(&raw_bytes).unwrap();

        if value.0.is_none() {
            syscalls.sys_drop_lock(lock_handle).unwrap();
        }

        value
            .0
            .map(|raw| DataRef::new(lock_handle, scrypto_decode(&raw).unwrap()))
    }

    pub fn get_mut(&mut self, key: &K) -> Option<DataRefMut<V>> {
        let mut syscalls = ScryptoEnv;
        let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(scrypto_encode(key)));
        let lock_handle = syscalls
            .sys_lock_substate(RENodeId::KeyValueStore(self.id), offset.clone(), true)
            .unwrap();
        let raw_bytes = syscalls.sys_read(lock_handle).unwrap();
        let value: KeyValueStoreEntrySubstate = scrypto_decode(&raw_bytes).unwrap();

        if value.0.is_none() {
            syscalls.sys_drop_lock(lock_handle).unwrap();
        }

        value
            .0
            .map(|raw| DataRefMut::new(lock_handle, offset, scrypto_decode(&raw).unwrap()))
    }

    /// Inserts a new key-value pair into this map.
    pub fn insert(&self, key: K, value: V) {
        let mut syscalls = ScryptoEnv;
        let offset =
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(scrypto_encode(&key)));
        let lock_handle = syscalls
            .sys_lock_substate(RENodeId::KeyValueStore(self.id), offset.clone(), true)
            .unwrap();
        let substate = KeyValueStoreEntrySubstate(Some(scrypto_encode(&value)));
        syscalls
            .sys_write(lock_handle, scrypto_encode(&substate))
            .unwrap();
        syscalls.sys_drop_lock(lock_handle).unwrap();
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

impl<
        K: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
        V: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    > TryFrom<&[u8]> for KeyValueStore<K, V>
{
    type Error = ParseKeyValueStoreError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            36 => Ok(Self {
                id: copy_u8_array(slice),
                key: PhantomData,
                value: PhantomData,
            }),
            _ => Err(ParseKeyValueStoreError::InvalidLength(slice.len())),
        }
    }
}

impl<
        K: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
        V: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    > KeyValueStore<K, V>
{
    pub fn to_vec(&self) -> Vec<u8> {
        self.id.to_vec()
    }
}

// TODO: extend scrypto_type! macro to support generics

impl<
        K: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
        V: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    > TypeId<ScryptoCustomTypeId> for KeyValueStore<K, V>
{
    #[inline]
    fn type_id() -> SborTypeId<ScryptoCustomTypeId> {
        SborTypeId::Custom(ScryptoCustomTypeId::KeyValueStore)
    }
}

impl<
        K: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
        V: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    > Encode<ScryptoCustomTypeId> for KeyValueStore<K, V>
{
    #[inline]
    fn encode_type_id(encoder: &mut Encoder<ScryptoCustomTypeId>) {
        encoder.write_type_id(Self::type_id());
    }

    #[inline]
    fn encode_value(&self, encoder: &mut Encoder<ScryptoCustomTypeId>) {
        encoder.write_slice(&self.to_vec());
    }
}

impl<
        K: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
        V: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    > Decode<ScryptoCustomTypeId> for KeyValueStore<K, V>
{
    fn check_type_id(decoder: &mut Decoder<ScryptoCustomTypeId>) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }

    fn decode_value(decoder: &mut Decoder<ScryptoCustomTypeId>) -> Result<Self, DecodeError> {
        let slice = decoder.read_slice(36)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomValue)
    }
}

impl<
        K: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId> + Describe,
        V: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId> + Describe,
    > Describe for KeyValueStore<K, V>
{
    fn describe() -> Type {
        Type::KeyValueStore {
            key_type: Box::new(K::describe()),
            value_type: Box::new(V::describe()),
        }
    }
}

//======
// text
//======

impl<
        K: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
        V: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    > FromStr for KeyValueStore<K, V>
{
    type Err = ParseKeyValueStoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseKeyValueStoreError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl<
        K: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
        V: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    > fmt::Display for KeyValueStore<K, V>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl<
        K: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
        V: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    > fmt::Debug for KeyValueStore<K, V>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
