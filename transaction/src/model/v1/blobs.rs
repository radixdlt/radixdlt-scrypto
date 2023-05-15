use radix_engine_constants::MAX_NUMBER_OF_BLOBS;

use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
#[sbor(transparent)]
#[repr(transparent)]
pub struct BlobV1(pub Vec<u8>);

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
#[sbor(transparent)]
#[repr(transparent)]
pub struct BlobsV1 {
    pub blobs: Vec<BlobV1>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedBlobsV1 {
    blobs_by_hash: IndexMap<Hash, BlobV1>,
    summary: Summary,
}

impl HasSummary for PreparedBlobsV1 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionFullChildPreparable for PreparedBlobsV1 {
    fn prepare_as_full_body_child(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let (blobs, summary) = ConcatenatedDigest::prepare_from_sbor_array::<
            Vec<SummarizedRawInnerBodyRawBytes>,
            MAX_NUMBER_OF_BLOBS,
        >(decoder, HashAccumulator::new(), ValueType::Blob)?;

        let mut blobs_by_hash = index_map_with_capacity(blobs.len());
        for blob in blobs {
            blobs_by_hash.insert(blob.summary.hash, BlobV1(blob.inner));
        }

        Ok(PreparedBlobsV1 {
            blobs_by_hash,
            summary,
        })
    }
}
