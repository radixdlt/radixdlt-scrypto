use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::api::key_value_entry_api::KeyValueEntryHandle;
use radix_engine_interface::api::key_value_store_api::KeyValueStoreGenericArgs;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::well_known_scrypto_custom_types::{
    own_key_value_store_type_data, OWN_KEY_VALUE_STORE_TYPE,
};
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::types::SubstateHandle;
use sbor::rust::fmt;
use sbor::rust::marker::PhantomData;
use sbor::rust::ops::{Deref, DerefMut};
use sbor::*;

use crate::engine::scrypto_env::ScryptoVmV1Api;
use crate::runtime::Runtime;

// TODO: optimize `rust_value -> bytes -> scrypto_value` conversion.

/// A scalable key-value map which loads entries on demand.
pub struct KeyValueStore<
    K: ScryptoEncode + ScryptoDecode + ScryptoDescribe,
    V: ScryptoEncode + ScryptoDecode + ScryptoDescribe,
> {
    pub id: Own,
    pub key: PhantomData<K>,
    pub value: PhantomData<V>,
}

impl<
        K: ScryptoEncode + ScryptoDecode + ScryptoDescribe,
        V: ScryptoEncode + ScryptoDecode + ScryptoDescribe,
    > KeyValueStore<K, V>
{
    /// Creates a new key value store.
    pub fn new() -> Self {
        let store_schema = KeyValueStoreGenericArgs::new_with_self_package::<K, V>(
            true,
            Runtime::package_address(),
        );

        Self {
            id: Own(ScryptoVmV1Api::kv_store_new(store_schema)),
            key: PhantomData,
            value: PhantomData,
        }
    }

    /// Returns the value that is associated with the given key.
    pub fn get(&self, key: &K) -> Option<KeyValueEntryRef<'_, V>> {
        let key_payload = scrypto_encode(key).unwrap();
        let handle = ScryptoVmV1Api::kv_store_open_entry(
            self.id.as_node_id(),
            &key_payload,
            LockFlags::read_only(),
        );
        let raw_bytes = ScryptoVmV1Api::kv_entry_read(handle);

        // Decode and create Ref
        let substate: Option<ScryptoValue> = scrypto_decode(&raw_bytes).unwrap();
        match substate {
            Option::Some(value) => Some(KeyValueEntryRef::new(
                handle,
                scrypto_decode(&scrypto_encode(&value).unwrap()).unwrap(),
            )),
            Option::None => {
                ScryptoVmV1Api::kv_entry_close(handle);
                None
            }
        }
    }

    pub fn get_mut(&mut self, key: &K) -> Option<KeyValueEntryRefMut<'_, V>> {
        let key_payload = scrypto_encode(key).unwrap();
        let handle = ScryptoVmV1Api::kv_store_open_entry(
            self.id.as_node_id(),
            &key_payload,
            LockFlags::MUTABLE,
        );
        let raw_bytes = ScryptoVmV1Api::kv_entry_read(handle);

        // Decode and create RefMut
        let substate: Option<ScryptoValue> = scrypto_decode(&raw_bytes).unwrap();
        match substate {
            Option::Some(value) => {
                let rust_value = scrypto_decode(&scrypto_encode(&value).unwrap()).unwrap();
                Some(KeyValueEntryRefMut::new(handle, rust_value))
            }
            Option::None => {
                ScryptoVmV1Api::kv_entry_close(handle);
                None
            }
        }
    }

    /// Inserts a new key-value pair into this map.
    pub fn insert(&self, key: K, value: V) {
        let key_payload = scrypto_encode(&key).unwrap();
        let handle = ScryptoVmV1Api::kv_store_open_entry(
            self.id.as_node_id(),
            &key_payload,
            LockFlags::MUTABLE,
        );
        let value_payload = scrypto_encode(&value).unwrap();

        ScryptoVmV1Api::kv_entry_write(handle, value_payload);
        ScryptoVmV1Api::kv_entry_close(handle);
    }

    /// Remove an entry from the map and return the original value if it exists
    pub fn remove(&self, key: &K) -> Option<V> {
        let key_payload = scrypto_encode(&key).unwrap();
        let rtn = ScryptoVmV1Api::kv_store_remove_entry(self.id.as_node_id(), &key_payload);

        scrypto_decode(&rtn).unwrap()
    }
}

//========
// binary
//========
impl<
        K: ScryptoEncode + ScryptoDecode + ScryptoDescribe,
        V: ScryptoEncode + ScryptoDecode + ScryptoDescribe,
    > Categorize<ScryptoCustomValueKind> for KeyValueStore<K, V>
{
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<
        K: ScryptoEncode + ScryptoDecode + ScryptoDescribe,
        V: ScryptoEncode + ScryptoDecode + ScryptoDescribe,
        E: Encoder<ScryptoCustomValueKind>,
    > Encode<ScryptoCustomValueKind, E> for KeyValueStore<K, V>
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.id.encode_body(encoder)
    }
}

impl<
        K: ScryptoEncode + ScryptoDecode + ScryptoDescribe,
        V: ScryptoEncode + ScryptoDecode + ScryptoDescribe,
        D: Decoder<ScryptoCustomValueKind>,
    > Decode<ScryptoCustomValueKind, D> for KeyValueStore<K, V>
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let own = Own::decode_body_with_value_kind(decoder, value_kind)?;
        Ok(Self {
            id: own,
            key: PhantomData,
            value: PhantomData,
        })
    }
}

impl<
        K: ScryptoEncode + ScryptoDecode + ScryptoDescribe,
        V: ScryptoEncode + ScryptoDecode + ScryptoDescribe,
    > Describe<ScryptoCustomTypeKind> for KeyValueStore<K, V>
{
    const TYPE_ID: GlobalTypeId = GlobalTypeId::WellKnown(OWN_KEY_VALUE_STORE_TYPE);

    fn type_data() -> sbor::TypeData<ScryptoCustomTypeKind, GlobalTypeId> {
        own_key_value_store_type_data()
    }
}

pub struct KeyValueEntryRef<'a, V: ScryptoEncode> {
    lock_handle: KeyValueEntryHandle,
    value: V,
    phantom: PhantomData<&'a ()>,
}

impl<'a, V: fmt::Display + ScryptoEncode> fmt::Display for KeyValueEntryRef<'a, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<'a, V: ScryptoEncode> KeyValueEntryRef<'a, V> {
    pub fn new(lock_handle: KeyValueEntryHandle, value: V) -> KeyValueEntryRef<'a, V> {
        KeyValueEntryRef {
            lock_handle,
            value,
            phantom: PhantomData::default(),
        }
    }
}

impl<'a, V: ScryptoEncode> Deref for KeyValueEntryRef<'a, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, V: ScryptoEncode> Drop for KeyValueEntryRef<'a, V> {
    fn drop(&mut self) {
        ScryptoVmV1Api::kv_entry_close(self.lock_handle);
    }
}

pub struct KeyValueEntryRefMut<'a, V: ScryptoEncode> {
    handle: KeyValueEntryHandle,
    value: V,
    phantom: PhantomData<&'a ()>,
}

impl<V: fmt::Display + ScryptoEncode> fmt::Display for KeyValueEntryRefMut<'_, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<'a, V: ScryptoEncode> KeyValueEntryRefMut<'a, V> {
    pub fn new(lock_handle: SubstateHandle, value: V) -> KeyValueEntryRefMut<'a, V> {
        KeyValueEntryRefMut {
            handle: lock_handle,
            value,
            phantom: PhantomData::default(),
        }
    }
}

impl<'a, V: ScryptoEncode> Drop for KeyValueEntryRefMut<'a, V> {
    fn drop(&mut self) {
        let value = scrypto_encode(&self.value).unwrap();
        ScryptoVmV1Api::kv_entry_write(self.handle, value);
        ScryptoVmV1Api::kv_entry_close(self.handle);
    }
}

impl<'a, V: ScryptoEncode> Deref for KeyValueEntryRefMut<'a, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<'a, V: ScryptoEncode> DerefMut for KeyValueEntryRefMut<'a, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
