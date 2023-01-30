use radix_engine_interface::api::types::{KeyValueStoreOffset, SubstateOffset};
use radix_engine_interface::api::{ClientSubstateApi, Invokable};
use radix_engine_interface::blueprints::kv_store::{
    KeyValueStoreCreateInvocation, KeyValueStoreGetMutInvocation, KeyValueStoreInsertInvocation,
};
use radix_engine_interface::data::types::Own;
use radix_engine_interface::data::*;
use sbor::rust::boxed::Box;
use sbor::rust::marker::PhantomData;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::{DataRef, DataRefMut};

/// A scalable key-value map which loads entries on demand.
pub struct KeyValueStore<K: ScryptoEncode + ScryptoDecode, V: ScryptoEncode + ScryptoDecode> {
    pub own: Own,
    pub key: PhantomData<K>,
    pub value: PhantomData<V>,
}

// TODO: de-duplication
#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq)]
pub struct KeyValueStoreEntrySubstate(pub Option<Vec<u8>>);

impl<K: ScryptoEncode + ScryptoDecode, V: ScryptoEncode + ScryptoDecode> KeyValueStore<K, V> {
    /// Creates a new key value store.
    pub fn new() -> Self {
        let mut env = ScryptoEnv;
        let own = env.invoke(KeyValueStoreCreateInvocation {}).unwrap();

        Self {
            own,
            key: PhantomData,
            value: PhantomData,
        }
    }

    /// Returns the value that is associated with the given key.
    pub fn get(&self, key: &K) -> Option<DataRef<V>> {
        let mut env = ScryptoEnv;
        let lock_handle = env
            .invoke(KeyValueStoreGetMutInvocation {
                receiver: self.own.kv_store_id(),
                key: scrypto_encode(key).unwrap(),
            })
            .unwrap();
        let raw_bytes = env.sys_read_substate(lock_handle).unwrap();
        let value: KeyValueStoreEntrySubstate = scrypto_decode(&raw_bytes).unwrap();

        // TODO: possibly wrong abstraction here
        if value.0.is_none() {
            env.sys_drop_lock(lock_handle).unwrap();
        }

        value
            .0
            .map(|raw| DataRef::new(lock_handle, scrypto_decode(&raw).unwrap()))
    }

    pub fn get_mut(&mut self, key: &K) -> Option<DataRefMut<V>> {
        let mut env = ScryptoEnv;
        let lock_handle = env
            .invoke(KeyValueStoreGetMutInvocation {
                receiver: self.own.kv_store_id(),
                key: scrypto_encode(key).unwrap(),
            })
            .unwrap();
        let raw_bytes = env.sys_read_substate(lock_handle).unwrap();
        let value: KeyValueStoreEntrySubstate = scrypto_decode(&raw_bytes).unwrap();

        // TODO: possibly wrong abstraction here
        if value.0.is_none() {
            env.sys_drop_lock(lock_handle).unwrap();
        }

        value.0.map(|raw| {
            DataRefMut::new(
                lock_handle,
                SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(
                    scrypto_encode(key).unwrap(),
                )),
                scrypto_decode(&raw).unwrap(),
            )
        })
    }

    /// Inserts a new key-value pair into this map.
    pub fn insert(&self, key: K, value: V) {
        let mut env = ScryptoEnv;
        env.invoke(KeyValueStoreInsertInvocation {
            receiver: self.own.kv_store_id(),
            key: scrypto_encode(&key).unwrap(),
            value: scrypto_encode(&value).unwrap(),
        })
        .unwrap();
    }
}

//========
// binary
//========
impl<K: ScryptoEncode + ScryptoDecode, V: ScryptoEncode + ScryptoDecode>
    Categorize<ScryptoCustomValueKind> for KeyValueStore<K, V>
{
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<
        K: ScryptoEncode + ScryptoDecode,
        V: ScryptoEncode + ScryptoDecode,
        E: Encoder<ScryptoCustomValueKind>,
    > Encode<ScryptoCustomValueKind, E> for KeyValueStore<K, V>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.own.encode_body(encoder)
    }
}

impl<
        K: ScryptoEncode + ScryptoDecode,
        V: ScryptoEncode + ScryptoDecode,
        D: Decoder<ScryptoCustomValueKind>,
    > Decode<ScryptoCustomValueKind, D> for KeyValueStore<K, V>
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let o = Own::decode_body_with_value_kind(decoder, value_kind)?;
        match o {
            Own::KeyValueStore(_) => Ok(Self {
                own: o,
                key: PhantomData,
                value: PhantomData,
            }),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl<
        K: ScryptoEncode + ScryptoDecode + LegacyDescribe,
        V: ScryptoEncode + ScryptoDecode + LegacyDescribe,
    > LegacyDescribe for KeyValueStore<K, V>
{
    fn describe() -> scrypto_abi::Type {
        Type::KeyValueStore {
            key_type: Box::new(K::describe()),
            value_type: Box::new(V::describe()),
        }
    }
}
