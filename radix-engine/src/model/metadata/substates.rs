use crate::model::convert;
use crate::types::*;

/// A transient resource container.
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct MetadataSubstate {
    pub metadata: HashMap<String, String>,
}

impl MetadataSubstate {
    pub fn insert(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}
