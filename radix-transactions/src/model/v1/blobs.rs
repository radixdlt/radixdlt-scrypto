use radix_common::constants::MAX_NUMBER_OF_BLOBS;

use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct BlobV1(pub Vec<u8>);

#[derive(Default, Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct BlobsV1 {
    pub blobs: Vec<BlobV1>,
}

impl TransactionPartialPrepare for BlobsV1 {
    type Prepared = PreparedBlobsV1;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedBlobsV1 {
    pub blobs_by_hash: Rc<IndexMap<Hash, Vec<u8>>>,
    pub summary: Summary,
}

impl_has_summary!(PreparedBlobsV1);

#[allow(deprecated)]
impl TransactionPreparableFromValue for PreparedBlobsV1 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let (blobs, summary) = ConcatenatedDigest::prepare_from_sbor_array_full_value::<
            Vec<SummarizedRawValueBodyRawBytes>,
            MAX_NUMBER_OF_BLOBS,
        >(decoder, ValueType::Blob)?;

        let mut blobs_by_hash = index_map_with_capacity(blobs.len());
        for blob in blobs {
            blobs_by_hash.insert(blob.summary.hash, blob.inner);
        }

        Ok(PreparedBlobsV1 {
            blobs_by_hash: Rc::new(blobs_by_hash),
            summary,
        })
    }
}
