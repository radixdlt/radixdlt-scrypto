use radix_engine_interface::api::api::EngineApi;
use radix_engine_interface::api::types::{
    KeyValueStoreId, KeyValueStoreOffset, RENodeId, ScryptoRENode, SubstateOffset,
};
use radix_engine_interface::data::*;

use radix_engine_interface::data::types::Own;
use sbor::rust::boxed::Box;
use sbor::rust::marker::PhantomData;
use sbor::rust::vec::Vec;
use sbor::*;

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
// binary
//========
impl<K: ScryptoEncode + ScryptoDecode, V: ScryptoEncode + ScryptoDecode> TypeId<ScryptoCustomTypeId>
    for KeyValueStore<K, V>
{
    #[inline]
    fn type_id() -> SborTypeId<ScryptoCustomTypeId> {
        SborTypeId::Custom(ScryptoCustomTypeId::Own)
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
        Own::KeyValueStore(self.id).encode_body(encoder)
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
        type_id: SborTypeId<ScryptoCustomTypeId>,
    ) -> Result<Self, DecodeError> {
        let o = Own::decode_body_with_type_id(decoder, type_id)?;
        match o {
            Own::KeyValueStore(kv_store_id) => Ok(Self {
                id: kv_store_id,
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
