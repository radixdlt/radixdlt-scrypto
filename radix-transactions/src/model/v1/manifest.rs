use super::*;
use crate::internal_prelude::*;
use std::ops::Deref;

//=================================================================================
// NOTE:
// This isn't actually embedded as a model - it's just a useful model which we use
// in eg the manifest builder
//=================================================================================

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct TransactionManifestV1 {
    pub instructions: Vec<InstructionV1>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
}

impl TransactionManifestV1 {
    pub fn from_intent(intent: &IntentV1) -> Self {
        Self {
            instructions: intent.instructions.0.deref().clone(),
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
            InstructionsV1(Rc::new(self.instructions)),
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

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct TransactionManifestV2 {
    pub instructions: Vec<InstructionV2>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
    pub children: Vec<SubintentHash>,
}

impl TransactionManifestV2 {
    pub fn from_intent(intent: &IntentCoreV2) -> Self {
        Self {
            instructions: intent.instructions.0.deref().clone(),
            blobs: intent
                .blobs
                .blobs
                .iter()
                .map(|blob| (hash(&blob.0), blob.0.clone()))
                .collect(),
            children: intent.children.children.clone(),
        }
    }

    pub fn for_intent(self) -> (InstructionsV2, BlobsV1, ChildIntentsV2) {
        (
            InstructionsV2(Rc::new(self.instructions)),
            BlobsV1 {
                blobs: self
                    .blobs
                    .into_values()
                    .into_iter()
                    .map(|blob| BlobV1(blob))
                    .collect(),
            },
            ChildIntentsV2 {
                children: self.children,
            },
        )
    }
}
