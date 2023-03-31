use crate::errors::*;
use radix_engine_interface::data::manifest::model::{ManifestBucket, ManifestProof};
use sbor::rust::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestIdAllocator {
    available: Range<u32>,
}

impl ManifestIdAllocator {
    pub fn new() -> Self {
        Self {
            available: 0..u32::MAX,
        }
    }

    fn next(&mut self) -> Result<u32, ManifestIdAllocationError> {
        if self.available.len() > 0 {
            let id = self.available.start;
            self.available.start += 1;
            Ok(id)
        } else {
            Err(ManifestIdAllocationError::OutOfID)
        }
    }

    pub fn new_bucket_id(&mut self) -> Result<ManifestBucket, ManifestIdAllocationError> {
        Ok(ManifestBucket(self.next()?))
    }

    pub fn new_proof_id(&mut self) -> Result<ManifestProof, ManifestIdAllocationError> {
        Ok(ManifestProof(self.next()?))
    }
}
