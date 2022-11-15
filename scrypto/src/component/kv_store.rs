use sbor::rust::borrow::ToOwned;
use sbor::rust::boxed::Box;
use sbor::rust::fmt;
use sbor::rust::marker::PhantomData;
use sbor::rust::str::FromStr;
use sbor::rust::string::*;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::buffer::*;
use crate::core::{DataRef, DataRefMut};
use crate::data::*;
use crate::engine::{api::*, types::*, utils::*};
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
        let input = RadixEngineInput::CreateNode(ScryptoRENode::KeyValueStore);
        let output: RENodeId = call_engine(input);

        Self {
            id: output.into(),
            key: PhantomData,
            value: PhantomData,
        }
    }

    /// Returns the value that is associated with the given key.
    pub fn get(&self, key: &K) -> Option<DataRef<V>> {
        let input = RadixEngineInput::LockSubstate(
            RENodeId::KeyValueStore(self.id),
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(scrypto_encode(key))),
            false,
        );
        let lock_handle: LockHandle = call_engine(input);

        let input = RadixEngineInput::Read(lock_handle);
        let value: KeyValueStoreEntrySubstate = call_engine(input);

        if value.0.is_none() {
            let input = RadixEngineInput::DropLock(lock_handle);
            let _: () = call_engine(input);
        }

        value
            .0
            .map(|raw| DataRef::new(lock_handle, scrypto_decode(&raw).unwrap()))
    }

    pub fn get_mut(&mut self, key: &K) -> Option<DataRefMut<V>> {
        let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(scrypto_encode(key)));
        let input =
            RadixEngineInput::LockSubstate(RENodeId::KeyValueStore(self.id), offset.clone(), true);
        let lock_handle: LockHandle = call_engine(input);

        let input = RadixEngineInput::Read(lock_handle);
        let value: KeyValueStoreEntrySubstate = call_engine(input);

        if value.0.is_none() {
            let input = RadixEngineInput::DropLock(lock_handle);
            let _: () = call_engine(input);
        }

        value
            .0
            .map(|raw| DataRefMut::new(lock_handle, offset, scrypto_decode(&raw).unwrap()))
    }

    /// Inserts a new key-value pair into this map.
    pub fn insert(&self, key: K, value: V) {
        let input = RadixEngineInput::LockSubstate(
            RENodeId::KeyValueStore(self.id),
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(scrypto_encode(&key))),
            true,
        );
        let lock_handle: LockHandle = call_engine(input);

        let substate = KeyValueStoreEntrySubstate(Some(scrypto_encode(&value)));
        let input = RadixEngineInput::Write(lock_handle, scrypto_encode(&substate));
        let _: () = call_engine(input);

        let input = RadixEngineInput::DropLock(lock_handle);
        let _: () = call_engine(input);
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
    fn type_id() -> ScryptoTypeId {
        SborTypeId::Custom(ScryptoCustomTypeId::KeyValueStore)
    }
}

impl<
        K: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
        V: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    > Encode<ScryptoCustomTypeId> for KeyValueStore<K, V>
{
    #[inline]
    fn encode_type_id(&self, encoder: &mut ScryptoEncoder) {
        encoder.write_type_id(Self::type_id());
    }

    #[inline]
    fn encode_value(&self, encoder: &mut ScryptoEncoder) {
        encoder.write_slice(&self.to_vec());
    }
}

impl<
        K: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
        V: Encode<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    > Decode<ScryptoCustomTypeId> for KeyValueStore<K, V>
{
    fn check_type_id(decoder: &mut ScryptoDecoder) -> Result<(), DecodeError> {
        decoder.check_type_id(Self::type_id())
    }

    fn decode_value(decoder: &mut ScryptoDecoder) -> Result<Self, DecodeError> {
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
