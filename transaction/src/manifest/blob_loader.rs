use sbor::rust::collections::HashMap;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

pub trait BlobLoader {
    fn load(&self, key: &str) -> Option<Vec<u8>>;
}

#[derive(Default)]
pub struct InMemoryBlobLoader {
    blobs: HashMap<String, Vec<u8>>,
}

impl InMemoryBlobLoader {
    pub fn insert(&mut self, key: String, blob: Vec<u8>) {
        self.blobs.insert(key, blob);
    }
}

impl BlobLoader for InMemoryBlobLoader {
    fn load(&self, key: &str) -> Option<Vec<u8>> {
        self.blobs.get(key).cloned()
    }
}
