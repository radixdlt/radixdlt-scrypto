use crate::internal_prelude::*;
use crate::model::*;
use radix_common::crypto::hash;
use radix_common::data::manifest::*;
use radix_common::types::NonFungibleGlobalId;
use std::collections::BTreeSet;

#[derive(ManifestSbor)]
pub enum TestTransaction {
    V1(TestIntentV1)
}

#[derive(ManifestSbor)]
pub struct TestIntentV1 {
    pub instructions: InstructionsV1,
    pub blobs: BlobsV1,
    pub hash: Hash,
}

#[derive(ManifestSbor)]
pub struct TestIntentV2 {
    pub instructions: InstructionsV2,
    pub blobs: BlobsV1,
    pub hash: Hash,
}

#[derive(ManifestSbor)]
pub enum PreparedTestTransaction {
    V1(PreparedTestIntentV1)
}

#[derive(ManifestSbor)]
pub struct PreparedTestIntentV1 {
    pub encoded_instructions: Rc<Vec<u8>>,
    pub references: IndexSet<Reference>,
    pub blobs: Rc<IndexMap<Hash, Vec<u8>>>,
    pub hash: Hash,
}

impl TestTransaction {
    /// The nonce needs to be globally unique amongst test transactions on your ledger
    pub fn new_from_nonce(manifest: TransactionManifestV1, nonce: u32) -> Self {
        Self::new(manifest, hash(format!("Test transaction: {}", nonce)))
    }

    pub fn new(manifest: TransactionManifestV1, hash: Hash) -> Self {
        let (instructions, blobs) = manifest.for_intent();
        Self::V1(TestIntentV1 {
            instructions,
            blobs,
            hash,
        })
    }

    #[allow(deprecated)]
    pub fn prepare(self) -> Result<PreparedTestTransaction, PrepareError> {
        match self {
            Self::V1(intent) => {
                let prepared_instructions = intent.instructions.prepare_partial()?;
                Ok(PreparedTestTransaction::V1(PreparedTestIntentV1 {
                    encoded_instructions: Rc::new(manifest_encode(&prepared_instructions.inner.0)?),
                    references: prepared_instructions.references,
                    blobs: intent.blobs.prepare_partial()?.blobs_by_hash,
                    hash: intent.hash,
                }))
            }
        }
    }
}

impl PreparedTestTransaction {
    pub fn get_executable(
        &self,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> ExecutableTransaction {
        match self {
            PreparedTestTransaction::V1(intent) => {
                let num_of_signature_validations = initial_proofs.len() + 1;
                ExecutableTransaction::new_v1(
                    intent.encoded_instructions.clone(),
                    AuthZoneInit::proofs(initial_proofs),
                    intent.references.clone(),
                    intent.blobs.clone(),
                    ExecutionContext {
                        unique_hash: intent.hash,
                        intent_hash_nullifications: vec![],
                        epoch_range: None,
                        payload_size: intent.encoded_instructions.len()
                            + intent.blobs.values().map(|x| x.len()).sum::<usize>(),
                        // For testing purpose, assume `num_of_signature_validations = num_of_initial_proofs + 1`
                        num_of_signature_validations,
                        costing_parameters: TransactionCostingParameters {
                            tip: TipSpecifier::None,
                            free_credit_in_xrd: Decimal::ZERO,
                            abort_when_loan_repaid: false,
                        },
                        pre_allocated_addresses: vec![],
                        disable_limits_and_costing_modules: false,
                        start_timestamp_inclusive: None,
                        end_timestamp_exclusive: None,
                    },
                )
            }
        }
    }
}
