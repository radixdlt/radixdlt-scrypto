use crate::*;
use sbor::rust::prelude::*;

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct KeyValueStoreInit<K: Ord, V> {
    pub data: BTreeMap<K, KeyValueStoreInitEntry<V>>,
}

impl<K: Ord, V> KeyValueStoreInit<K, V> {
    pub fn new() -> Self {
        KeyValueStoreInit {
            data: BTreeMap::new()
        }
    }

    pub fn set(&mut self, key: K, value: V) {
        let entry = KeyValueStoreInitEntry {
            value,
            lock: false,
        };
        self.data.insert(key, entry);
    }

    pub fn set_and_lock(&mut self, key: K, value: V) {
        let entry = KeyValueStoreInitEntry {
            value,
            lock: true,
        };
        self.data.insert(key, entry);
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct KeyValueStoreInitEntry<V> {
    pub value: V,
    pub lock: bool,
}

#[macro_export]
macro_rules! kv_store_init_set_entry {
    ($store:expr, $key:expr, $value:expr, updatable) => ({
        $store.set($key, $value);
    });
    ($store:expr, $key:expr, $value:expr, locked) => ({
        $store.set_and_lock($key, $value);
    });
}

#[macro_export]
macro_rules! kv_store_init {
    ( ) => ({
        KeyValueStoreInit::new()
    });
    ( $($key:expr => $value:expr, $lock:ident;)* ) => ({
        let mut kv_store_init = KeyValueStoreInit::new();
        $(
            kv_store_init_set_entry!(kv_store_init, $key, $value, $lock);
        )*
        kv_store_init
    });
}