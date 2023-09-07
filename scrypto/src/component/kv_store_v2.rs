use super::*;
use crate::engine::scrypto_env::ScryptoVmV1Api;
use radix_engine_interface::api::field_api::LockFlags;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::well_known_scrypto_custom_types::{
    own_key_value_store_type_data, OWN_KEY_VALUE_STORE_TYPE,
};
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::prelude::{
    RemoteKeyValueStoreDataSchema, KV_STORE_DATA_SCHEMA_VARIANT_REMOTE,
};
use radix_engine_interface::types::RegisteredType;
use sbor::rust::marker::PhantomData;
use sbor::*;

/// A scalable key-value map which loads entries on demand.
///
/// Different from V1, this new version requires that both key and value types are registered under a blueprint.
///
/// This is to reduce WASM code size and avoid on-chain SBOR schema generation, which helps reduce transaction costs.
///
/// Example:
///
/// ```ignore
/// // You will need to add `#[types(u32, AnotherType)]` below the `#[blueprint]` line
/// let kv_store = KeyValueStoreV2::<RadiswapType, u32, AnotherType>::new();
/// let value: Option<AnotherType> = kv_store.get(1);
/// ```
pub struct KeyValueStoreV2<T, K: RegisteredType<T>, V: RegisteredType<T>> {
    pub id: Own,
    pub marker: PhantomData<T>,
    pub key: PhantomData<K>,
    pub value: PhantomData<V>,
}

impl<T, K: RegisteredType<T>, V: RegisteredType<T>> KeyValueStoreV2<T, K, V> {
    /// Creates a new key value store.
    pub fn new() -> Self {
        let store_schema = RemoteKeyValueStoreDataSchema {
            key_type: K::blueprint_type_identifier(),
            value_type: V::blueprint_type_identifier(),
            allow_ownership: true,
        };

        Self {
            id: Own(ScryptoVmV1Api::kv_store_new(FixedEnumVariant::<
                KV_STORE_DATA_SCHEMA_VARIANT_REMOTE,
                RemoteKeyValueStoreDataSchema,
            > {
                fields: store_schema,
            })),
            marker: PhantomData,
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
impl<T, K: RegisteredType<T>, V: RegisteredType<T>> Categorize<ScryptoCustomValueKind>
    for KeyValueStoreV2<T, K, V>
{
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<T, K: RegisteredType<T>, V: RegisteredType<T>, E: Encoder<ScryptoCustomValueKind>>
    Encode<ScryptoCustomValueKind, E> for KeyValueStoreV2<T, K, V>
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

impl<T, K: RegisteredType<T>, V: RegisteredType<T>, D: Decoder<ScryptoCustomValueKind>>
    Decode<ScryptoCustomValueKind, D> for KeyValueStoreV2<T, K, V>
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        let own = Own::decode_body_with_value_kind(decoder, value_kind)?;
        Ok(Self {
            id: own,
            marker: PhantomData,
            key: PhantomData,
            value: PhantomData,
        })
    }
}

impl<T, K: RegisteredType<T>, V: RegisteredType<T>> Describe<ScryptoCustomTypeKind>
    for KeyValueStoreV2<T, K, V>
{
    const TYPE_ID: RustTypeId = RustTypeId::WellKnown(OWN_KEY_VALUE_STORE_TYPE);

    fn type_data() -> sbor::TypeData<ScryptoCustomTypeKind, RustTypeId> {
        own_key_value_store_type_data()
    }
}
