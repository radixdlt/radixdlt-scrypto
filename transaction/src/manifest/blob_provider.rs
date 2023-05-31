use radix_engine_common::prelude::{hash, Hash};
use sbor::prelude::*;

type Blob = Vec<u8>;
type BlobReference = Hash;

//========
// Traits
//========

pub trait BlobProvider {
    fn add_blob(&mut self, blob: Blob);

    fn get_blob(&self, blob_reference: &BlobReference) -> Option<Blob>;

    fn blobs(self) -> BTreeMap<BlobReference, Blob>;
}

//=======================
// Default Blob Provider
//=======================

#[derive(Default, Debug, Clone)]
pub struct DefaultBlobProvider(BTreeMap<BlobReference, Blob>);

impl DefaultBlobProvider {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_with_blobs(blobs: Vec<Blob>) -> Self {
        Self(blobs.into_iter().map(|blob| (hash(&blob), blob)).collect())
    }
}

impl BlobProvider for DefaultBlobProvider {
    fn add_blob(&mut self, blob: Blob) {
        let hash = hash(&blob);
        self.0.insert(hash, blob);
    }

    fn get_blob(&self, blob_reference: &BlobReference) -> Option<Blob> {
        self.0.get(blob_reference).cloned()
    }

    fn blobs(self) -> BTreeMap<BlobReference, Blob> {
        self.0
    }
}

//====================
// Mock Blob Provider
//====================

#[derive(Default, Debug, Clone)]
pub struct MockBlobProvider;

impl MockBlobProvider {
    pub fn new() -> Self {
        Default::default()
    }
}

impl BlobProvider for MockBlobProvider {
    fn add_blob(&mut self, _: Blob) {
        /* No OP */
    }

    fn get_blob(&self, _: &BlobReference) -> Option<Blob> {
        // All hashes are valid
        Some(vec![])
    }

    fn blobs(self) -> BTreeMap<BlobReference, Blob> {
        Default::default()
    }
}
