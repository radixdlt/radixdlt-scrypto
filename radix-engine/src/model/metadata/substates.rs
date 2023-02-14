use crate::types::*;

/// A transient resource container.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct MetadataSubstate {
    pub metadata: BTreeMap<String, String>,
}

impl MetadataSubstate {
    pub fn insert(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}
