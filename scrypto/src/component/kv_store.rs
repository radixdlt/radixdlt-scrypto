use radix_engine_interface::api::api::EngineApi;
use radix_engine_interface::api::types::{
    KeyValueStoreId, KeyValueStoreOffset, RENodeId, ScryptoRENode, SubstateOffset,
};
use radix_engine_interface::data::*;

use sbor::rust::borrow::ToOwned;
use sbor::rust::boxed::Box;
use sbor::rust::fmt;
use sbor::rust::marker::PhantomData;
use sbor::rust::str::FromStr;
use sbor::rust::string::*;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

use crate::abi::*;
use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::{DataRef, DataRefMut};

/// A scalable key-value map which loads entries on demand.
pub struct KeyValueStore<K: ScryptoEncode + ScryptoDecode, V: ScryptoEncode + ScryptoDecode> {
    pub id: KeyValueStoreId,
    pub key: PhantomData<K>,
    pub value: PhantomData<V>,
}

// TODO: de-duplication
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct KeyValueStoreEntrySubstate(pub Option<Vec<u8>>);

impl<K: ScryptoEncode + ScryptoDecode, V: ScryptoEncode + ScryptoDecode> KeyValueStore<K, V> {
    /// Creates a new key value store.
    pub fn new() -> Self {
        let mut env = ScryptoEnv;
        let id = env.sys_create_node(ScryptoRENode::KeyValueStore).unwrap();

        Self {
            id: id.into(),
            key: PhantomData,
            value: PhantomData,
        }
    }

    /// Returns the value that is associated with the given key.
    pub fn get(&self, key: &K) -> Option<DataRef<V>> {
        let mut env = ScryptoEnv;
        let offset =
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(scrypto_encode(key).unwrap()));
        let lock_handle = env
            .sys_lock_substate(RENodeId::KeyValueStore(self.id), offset, false)
            .unwrap();
        let raw_bytes = env.sys_read(lock_handle).unwrap();
        let value: KeyValueStoreEntrySubstate = scrypto_decode(&raw_bytes).unwrap();

        if value.0.is_none() {
            env.sys_drop_lock(lock_handle).unwrap();
        }

        value
            .0
            .map(|raw| DataRef::new(lock_handle, scrypto_decode(&raw).unwrap()))
    }

    pub fn get_mut(&mut self, key: &K) -> Option<DataRefMut<V>> {
        let mut env = ScryptoEnv;
        let offset =
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(scrypto_encode(key).unwrap()));
        let lock_handle = env
            .sys_lock_substate(RENodeId::KeyValueStore(self.id), offset.clone(), true)
            .unwrap();
        let raw_bytes = env.sys_read(lock_handle).unwrap();
        let value: KeyValueStoreEntrySubstate = scrypto_decode(&raw_bytes).unwrap();

        if value.0.is_none() {
            env.sys_drop_lock(lock_handle).unwrap();
        }

        value
            .0
            .map(|raw| DataRefMut::new(lock_handle, offset, scrypto_decode(&raw).unwrap()))
    }

    /// Inserts a new key-value pair into this map.
    pub fn insert(&self, key: K, value: V) {
        let mut env = ScryptoEnv;
        let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(
            scrypto_encode(&key).unwrap(),
        ));
        let lock_handle = env
            .sys_lock_substate(RENodeId::KeyValueStore(self.id), offset.clone(), true)
            .unwrap();
        let substate = KeyValueStoreEntrySubstate(Some(scrypto_encode(&value).unwrap()));
        env.sys_write(lock_handle, scrypto_encode(&substate).unwrap())
            .unwrap();
        env.sys_drop_lock(lock_handle).unwrap();
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

impl<K: ScryptoEncode + ScryptoDecode, V: ScryptoEncode + ScryptoDecode> TryFrom<&[u8]>
    for KeyValueStore<K, V>
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

impl<K: ScryptoEncode + ScryptoDecode, V: ScryptoEncode + ScryptoDecode> KeyValueStore<K, V> {
    pub fn to_vec(&self) -> Vec<u8> {
        self.id.to_vec()
    }
}

// TODO: extend scrypto_type! macro to support generics

impl<K: ScryptoEncode + ScryptoDecode, V: ScryptoEncode + ScryptoDecode> TypeId<ScryptoCustomTypeId>
    for KeyValueStore<K, V>
{
    #[inline]
    fn type_id() -> ScryptoSborTypeId {
        SborTypeId::Custom(ScryptoCustomTypeId::KeyValueStore)
    }
}

impl<
        K: ScryptoEncode + ScryptoDecode,
        V: ScryptoEncode + ScryptoDecode,
        E: Encoder<ScryptoCustomTypeId>,
    > Encode<ScryptoCustomTypeId, E> for KeyValueStore<K, V>
{
    #[inline]
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_type_id(Self::type_id())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_slice(&self.to_vec())
    }
}

impl<
        K: ScryptoEncode + ScryptoDecode,
        V: ScryptoEncode + ScryptoDecode,
        D: Decoder<ScryptoCustomTypeId>,
    > Decode<ScryptoCustomTypeId, D> for KeyValueStore<K, V>
{
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: ScryptoSborTypeId,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_type_id(type_id, Self::type_id())?;
        let slice = decoder.read_slice(36)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomValue)
    }
}

impl<K: ScryptoEncode + ScryptoDecode + Describe, V: ScryptoEncode + ScryptoDecode + Describe>
    Describe for KeyValueStore<K, V>
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

impl<K: ScryptoEncode + ScryptoDecode, V: ScryptoEncode + ScryptoDecode> FromStr
    for KeyValueStore<K, V>
{
    type Err = ParseKeyValueStoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseKeyValueStoreError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl<K: ScryptoEncode + ScryptoDecode, V: ScryptoEncode + ScryptoDecode> fmt::Display
    for KeyValueStore<K, V>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl<K: ScryptoEncode + ScryptoDecode, V: ScryptoEncode + ScryptoDecode> fmt::Debug
    for KeyValueStore<K, V>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
