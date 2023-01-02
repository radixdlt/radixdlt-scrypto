use crate::errors::*;
use radix_engine_interface::data::types::{ManifestBucket, ManifestProof};
use sbor::rust::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestIdAllocator {
    available: Range<u32>,
}

impl ManifestIdAllocator {
    pub fn new() -> Self {
        Self {
            available: 512..u32::MAX, // TODO: start from zero?
        }
    }

    fn next(&mut self) -> Result<u32, IdAllocationError> {
        if self.available.len() > 0 {
            let id = self.available.start;
            self.available.start += 1;
            Ok(id)
        } else {
            Err(IdAllocationError::OutOfID)
        }
    }

    pub fn new_bucket_id(&mut self) -> Result<ManifestBucket, IdAllocationError> {
        Ok(ManifestBucket(self.next()?))
    }

    pub fn new_proof_id(&mut self) -> Result<ManifestProof, IdAllocationError> {
        Ok(ManifestProof(self.next()?))
    }
}
