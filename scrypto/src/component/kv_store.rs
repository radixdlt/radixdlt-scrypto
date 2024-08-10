use super::*;
use crate::engine::scrypto_env::ScryptoVmV1Api;
use crate::runtime::Runtime;
use radix_common::data::scrypto::model::*;
use radix_common::data::scrypto::well_known_scrypto_custom_types::{
    own_key_value_store_type_data, OWN_KEY_VALUE_STORE_TYPE,
};
use radix_common::data::scrypto::*;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::prelude::{
    LocalKeyValueStoreDataSchema, KV_STORE_DATA_SCHEMA_VARIANT_LOCAL,
};
use sbor::rust::marker::PhantomData;
use sbor::*;

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
        let schema = LocalKeyValueStoreDataSchema::new_with_self_package_replacement::<K, V>(
            Runtime::package_address(),
            true,
        );

        let store_schema = LocalKeyValueStoreDataSchema {
            additional_schema: schema.additional_schema,
            key_type: schema.key_type,
            value_type: schema.value_type,
            allow_ownership: schema.allow_ownership,
        };
        Self {
            id: Own(ScryptoVmV1Api::kv_store_new(SborFixedEnumVariant::<
                KV_STORE_DATA_SCHEMA_VARIANT_LOCAL,
                LocalKeyValueStoreDataSchema,
            > {
                fields: store_schema,
            })),
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
        let substate: Option<V> = scrypto_decode(&raw_bytes).unwrap();
        match substate {
            Some(v) => Some(KeyValueEntryRef::new(handle, v)),
            None => {
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
        let substate: Option<V> = scrypto_decode(&raw_bytes).unwrap();
        match substate {
            Some(v) => Some(KeyValueEntryRefMut::new(handle, v)),
            None => {
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
    const TYPE_ID: RustTypeId = RustTypeId::WellKnown(OWN_KEY_VALUE_STORE_TYPE);

    fn type_data() -> sbor::TypeData<ScryptoCustomTypeKind, RustTypeId> {
        own_key_value_store_type_data()
    }
}
