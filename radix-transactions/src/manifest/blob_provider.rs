use radix_common::prelude::{hash, Hash};
use sbor::prelude::*;

type Blob = Vec<u8>;
type BlobReference = Hash;

//========
// Traits
//========

pub trait IsBlobProvider {
    fn add_blob(&mut self, blob: Blob);

    fn get_blob(&self, blob_reference: &BlobReference) -> Option<Blob>;

    fn blobs(self) -> IndexMap<BlobReference, Blob>;
}

//=======================
// Default Blob Provider
//=======================

#[derive(Default, Debug, Clone)]
pub struct BlobProvider(IndexMap<BlobReference, Blob>);

impl BlobProvider {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_with_blobs(blobs: Vec<Blob>) -> Self {
        Self(blobs.into_iter().map(|blob| (hash(&blob), blob)).collect())
    }

    pub fn new_with_prehashed_blobs(blobs: IndexMap<BlobReference, Blob>) -> Self {
        Self(blobs)
    }
}

impl IsBlobProvider for BlobProvider {
    fn add_blob(&mut self, blob: Blob) {
        let hash = hash(&blob);
        self.0.insert(hash, blob);
    }

    fn get_blob(&self, blob_reference: &BlobReference) -> Option<Blob> {
        self.0.get(blob_reference).cloned()
    }

    fn blobs(self) -> IndexMap<BlobReference, Blob> {
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

impl IsBlobProvider for MockBlobProvider {
    fn add_blob(&mut self, _: Blob) {
        /* No OP */
    }

    fn get_blob(&self, _: &BlobReference) -> Option<Blob> {
        // All hashes are valid
        Some(vec![])
    }

    fn blobs(self) -> IndexMap<BlobReference, Blob> {
        Default::default()
    }
}
