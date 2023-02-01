use radix_engine_interface::api::types::{KeyValueStoreOffset, RENodeId, SubstateOffset};
use radix_engine_interface::api::{ClientNativeInvokeApi, ClientSubstateApi};
use radix_engine_interface::blueprints::kv_store::{
    KeyValueStoreCreateInvocation, KeyValueStoreEntrySubstate, KeyValueStoreInsertInvocation,
};
use radix_engine_interface::data::types::Own;
use radix_engine_interface::data::*;
use sbor::rust::boxed::Box;
use sbor::rust::marker::PhantomData;
use sbor::*;

use crate::abi::*;
use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::{DataRef, DataRefMut, OriginalData};

/// A scalable key-value map which loads entries on demand.
pub struct KeyValueStore<K: ScryptoEncode + ScryptoDecode, V: ScryptoEncode + ScryptoDecode> {
    pub own: Own,
    pub key: PhantomData<K>,
    pub value: PhantomData<V>,
}

impl<K: ScryptoEncode + ScryptoDecode, V: ScryptoEncode + ScryptoDecode> KeyValueStore<K, V> {
    /// Creates a new key value store.
    pub fn new() -> Self {
        let mut env = ScryptoEnv;
        let own = env.call_native(KeyValueStoreCreateInvocation {}).unwrap();

        Self {
            own,
            key: PhantomData,
            value: PhantomData,
        }
    }

    /// Returns the value that is associated with the given key.
    pub fn get(&self, key: &K) -> Option<DataRef<V>> {
        let mut env = ScryptoEnv;
        let key_payload = scrypto_encode(key).unwrap();
        let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key_payload));
        let handle = env
            .sys_lock_substate(
                RENodeId::KeyValueStore(self.own.kv_store_id()),
                offset,
                false,
            )
            .unwrap();
        let raw_bytes = env.sys_read_substate(handle).unwrap();

        // Decode and create Ref
        let substate: KeyValueStoreEntrySubstate = scrypto_decode(&raw_bytes).unwrap();
        match substate {
            KeyValueStoreEntrySubstate::Some(_, value) => Some(DataRef::new(
                handle,
                scrypto_decode(&scrypto_encode(&value).unwrap()).unwrap(),
            )),
            KeyValueStoreEntrySubstate::None => {
                env.sys_drop_lock(handle).unwrap();
                None
            }
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<DataRefMut<V>> {
        let mut env = ScryptoEnv;
        let key_payload = scrypto_encode(key).unwrap();
        let offset = SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(key_payload));
        let handle = env
            .sys_lock_substate(
                RENodeId::KeyValueStore(self.own.kv_store_id()),
                offset.clone(),
                true,
            )
            .unwrap();
        let raw_bytes = env.sys_read_substate(handle).unwrap();

        // Decode and create RefMut
        let substate: KeyValueStoreEntrySubstate = scrypto_decode(&raw_bytes).unwrap();
        match substate {
            KeyValueStoreEntrySubstate::Some(key, value) => {
                let rust_value = scrypto_decode(&scrypto_encode(&value).unwrap()).unwrap();
                Some(DataRefMut::new(
                    handle,
                    OriginalData::KeyValueStoreEntry(key, value),
                    rust_value,
                ))
            }
            KeyValueStoreEntrySubstate::None => {
                env.sys_drop_lock(handle).unwrap();
                None
            }
        }
    }

    /// Inserts a new key-value pair into this map.
    pub fn insert(&self, key: K, value: V) {
        let mut env = ScryptoEnv;
        let key_payload = scrypto_encode(&key).unwrap();
        let value_payload = scrypto_encode(&value).unwrap();
        env.call_native(KeyValueStoreInsertInvocation {
            receiver: self.own.kv_store_id(),
            entry_id: key_payload.clone(),
            key: scrypto_decode(&key_payload).unwrap(), // TODO: remove encoding & decoding
            value: scrypto_decode(&value_payload).unwrap(),
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
