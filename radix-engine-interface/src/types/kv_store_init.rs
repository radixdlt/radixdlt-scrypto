use crate::*;
use core::fmt::Formatter;
use radix_engine_common::address::{AddressDisplayContext, NO_NETWORK};
use sbor::rust::prelude::*;
use sbor::rust::string::String;
use utils::ContextualDisplay;

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct LockableKeyValueStoreInit<K: Ord, V> {
    pub data: BTreeMap<K, LockableEntry<V>>,
}

impl<K: Ord, V> LockableKeyValueStoreInit<K, V> {
    pub fn new() -> Self {
        LockableKeyValueStoreInit {
            data: BTreeMap::new()
        }
    }

    pub fn set(&mut self, key: K, value: V) {
        let entry = LockableEntry {
            value,
            lock: false,
        };
        self.data.insert(key, entry);
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct LockableEntry<V> {
    pub value: V,
    pub lock: bool,
}