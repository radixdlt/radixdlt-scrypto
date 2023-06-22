use crate::api::ObjectModuleId;
use crate::types::*;
use crate::*;
use core::fmt::Formatter;
use radix_engine_common::address::{AddressDisplayContext, NO_NETWORK};
use radix_engine_common::types::*;
use sbor::rust::prelude::*;
use sbor::rust::string::String;
use utils::ContextualDisplay;

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
#[sbor(transparent)]
pub struct LockableKeyValueStoreInit<K: Ord, V> {
    pub data: BTreeMap<K, V>,
}

impl<K: Ord, V> LockableKeyValueStoreInit<K, V> {
    pub fn insert(&mut self, key: K, value: V) {
        self.data.insert(key, value);
    }
}