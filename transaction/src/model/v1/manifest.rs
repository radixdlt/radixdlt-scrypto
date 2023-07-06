use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// This isn't actually embedded as a model - it's just a useful model which we use
// in eg the manifest builder
//=================================================================================

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor)]
pub struct TransactionManifestV1 {
    pub instructions: Vec<InstructionV1>,
    pub blobs: BTreeMap<Hash, Vec<u8>>,
}

impl TransactionManifestV1 {
    pub fn from_intent(intent: &IntentV1) -> Self {
        Self {
            instructions: intent.instructions.0.clone(),
            blobs: intent
                .blobs
                .blobs
                .iter()
                .map(|blob| (hash(&blob.0), blob.0.clone()))
                .collect(),
        }
    }

    pub fn for_intent(self) -> (InstructionsV1, BlobsV1) {
        (
            InstructionsV1(self.instructions),
            BlobsV1 {
                blobs: self
                    .blobs
                    .into_values()
                    .into_iter()
                    .map(|blob| BlobV1(blob))
                    .collect(),
            },
        )
    }
}
