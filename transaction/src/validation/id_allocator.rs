use radix_engine_interface::{
    data::manifest::model::{ManifestBucket, ManifestProof},
    prelude::ManifestOwn,
};

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct ManifestIdAllocator {
    next_bucket_id: u32,
    next_proof_id: u32,
    next_own_id: u32,
}

impl ManifestIdAllocator {
    pub fn new() -> Self {
        Self::default()
    }

    // NOTE: overflow is practically impossible

    pub fn new_bucket_id(&mut self) -> ManifestBucket {
        let id = self.next_bucket_id;
        self.next_bucket_id += 1;
        ManifestBucket(id)
    }

    pub fn new_proof_id(&mut self) -> ManifestProof {
        let id = self.next_proof_id;
        self.next_proof_id += 1;
        ManifestProof(id)
    }

    pub fn new_own_id(&mut self) -> ManifestOwn {
        let id = self.next_own_id;
        self.next_own_id += 1;
        ManifestOwn(id)
    }
}
