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

impl BlobsV1 {
    pub fn none() -> Self {
        Self { blobs: Vec::new() }
    }
}

impl From<IndexMap<Hash, Vec<u8>>> for BlobsV1 {
    fn from(blobs: IndexMap<Hash, Vec<u8>>) -> Self {
        let blobs = blobs
            .into_values()
            .into_iter()
            .map(|blob| BlobV1(blob))
            .collect();
        Self { blobs }
    }
}

impl From<BlobsV1> for IndexMap<Hash, Vec<u8>> {
    fn from(value: BlobsV1) -> Self {
        let mut blobs = IndexMap::default();
        for blob in value.blobs {
            let content = blob.0;
            blobs.insert(hash(&content), content);
        }
        blobs
    }
}

impl TransactionPartialPrepare for BlobsV1 {
    type Prepared = PreparedBlobsV1;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedBlobsV1 {
    pub blobs_by_hash: IndexMap<Hash, Vec<u8>>,
    pub summary: Summary,
}

impl_has_summary!(PreparedBlobsV1);

#[allow(deprecated)]
impl TransactionPreparableFromValue for PreparedBlobsV1 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let max_blobs = decoder.settings().max_blobs;
        let (blobs, summary) = ConcatenatedDigest::prepare_from_sbor_array_full_value::<
            Vec<SummarizedRawValueBodyRawBytes>,
        >(decoder, ValueType::Blob, max_blobs)?;

        let mut blobs_by_hash = index_map_with_capacity(blobs.len());
        for blob in blobs {
            blobs_by_hash.insert(blob.summary.hash, blob.inner);
        }

        Ok(PreparedBlobsV1 {
            blobs_by_hash,
            summary,
        })
    }
}
