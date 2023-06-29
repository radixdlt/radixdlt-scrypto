use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use sbor::rust::prelude::*;

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct KeyValueStoreInit<K: Ord, V> {
    pub data: BTreeMap<K, KeyValueStoreInitEntry<V>>,
}

impl<K: Ord, V> Default for KeyValueStoreInit<K, V> {
    fn default() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }
}

impl<K: Ord, V> KeyValueStoreInit<K, V> {
    pub fn new() -> Self {
        KeyValueStoreInit {
            data: BTreeMap::new(),
        }
    }

    pub fn set<E: Into<K>>(&mut self, key: E, value: V) {
        let entry = KeyValueStoreInitEntry {
            value: Some(value),
            lock: false,
        };
        self.data.insert(key.into(), entry);
    }

    pub fn set_and_lock<E: Into<K>>(&mut self, key: E, value: V) {
        let entry = KeyValueStoreInitEntry {
            value: Some(value),
            lock: true,
        };
        self.data.insert(key.into(), entry);
    }

    pub fn set_raw<E: Into<K>>(&mut self, key: E, value: Option<V>, lock: bool) {
        let entry = KeyValueStoreInitEntry { value, lock };
        self.data.insert(key.into(), entry);
    }

    pub fn lock_empty<E: Into<K>>(&mut self, key: E) {
        let entry = KeyValueStoreInitEntry {
            value: None,
            lock: true,
        };
        self.data.insert(key.into(), entry);
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct KeyValueStoreInitEntry<V> {
    pub value: Option<V>,
    pub lock: bool,
}

#[macro_export]
macro_rules! kv_store_init_set_entry {
    ($store:expr, $key:expr, $value:expr, updatable) => {{
        $store.set($key, $value);
    }};
    ($store:expr, $key:expr, $value:expr, locked) => {{
        $store.set_and_lock($key, $value);
    }};
}

#[macro_export]
macro_rules! kv_store_init {
    ( ) => ({
        radix_engine_interface::prelude::KeyValueStoreInit::new()
    });
    ( $($key:expr => $value:expr, $lock:ident;)* ) => ({
        let mut kv_store_init = radix_engine_interface::prelude::KeyValueStoreInit::new();
        $(
            kv_store_init_set_entry!(kv_store_init, $key, $value, $lock);
        )*
        kv_store_init
    });
}
