use crate::internal_prelude::*;
use crate::model::*;
use radix_common::crypto::hash;
use radix_common::data::manifest::*;
use radix_common::types::NonFungibleGlobalId;
use std::collections::BTreeSet;
use std::ops::Deref;

#[derive(ManifestSbor)]
pub enum TestTransaction {
    V1(TestIntentV1),
    V2(Vec<TestIntentV2>),
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
    pub children_intent_indices: Vec<usize>,
}

#[derive(ManifestSbor)]
pub enum PreparedTestTransaction {
    V1(PreparedTestIntent),
    V2(Vec<PreparedTestIntent>),
}

#[derive(ManifestSbor)]
pub struct PreparedTestIntent {
    pub encoded_instructions: Rc<Vec<u8>>,
    pub references: IndexSet<Reference>,
    pub blobs: Rc<IndexMap<Hash, Vec<u8>>>,
    pub hash: Hash,
    pub children_intent_indices: Vec<usize>,
}

impl TestTransaction {
    /// The nonce needs to be globally unique amongst test transactions on your ledger
    pub fn new_v1_from_nonce(manifest: TransactionManifestV1, nonce: u32) -> Self {
        Self::new_v1(manifest, hash(format!("Test transaction: {}", nonce)))
    }

    pub fn new_v1(manifest: TransactionManifestV1, hash: Hash) -> Self {
        let (instructions, blobs) = manifest.for_intent();
        Self::V1(TestIntentV1 {
            instructions,
            blobs,
            hash,
        })
    }

    pub fn new_v2_from_nonce(intents: Vec<(TransactionManifestV2, u32, Vec<usize>)>) -> Self {
        let intents = intents
            .into_iter()
            .map(|(manifest, nonce, children_intent_indices)| {
                (
                    manifest,
                    hash(format!("Test transaction: {}", nonce)),
                    children_intent_indices,
                )
            })
            .collect();
        Self::new_v2(intents)
    }

    pub fn new_v2(intents: Vec<(TransactionManifestV2, Hash, Vec<usize>)>) -> Self {
        let intents = intents
            .into_iter()
            .map(|(manifest, hash, children_intent_indices)| {
                let (instructions, blobs, ..) = manifest.for_intent();
                TestIntentV2 {
                    instructions,
                    blobs,
                    hash,
                    children_intent_indices,
                }
            })
            .collect();

        Self::V2(intents)
    }

    #[allow(deprecated)]
    pub fn prepare(self) -> Result<PreparedTestTransaction, PrepareError> {
        match self {
            Self::V1(intent) => {
                let prepared_instructions = intent.instructions.prepare_partial()?;
                Ok(PreparedTestTransaction::V1(PreparedTestIntent {
                    encoded_instructions: Rc::new(manifest_encode(&prepared_instructions.inner.0)?),
                    references: prepared_instructions.references,
                    blobs: intent.blobs.prepare_partial()?.blobs_by_hash,
                    hash: intent.hash,
                    children_intent_indices: vec![],
                }))
            }
            Self::V2(intents) => {
                let mut prepared = vec![];
                for intent in intents {
                    let prepared_instructions = intent.instructions.prepare_partial()?;
                    prepared.push(PreparedTestIntent {
                        encoded_instructions: Rc::new(manifest_encode(
                            &prepared_instructions.inner.0,
                        )?),
                        references: prepared_instructions.references.deref().clone(),
                        blobs: intent.blobs.prepare_partial()?.blobs_by_hash,
                        hash: intent.hash,
                        children_intent_indices: intent.children_intent_indices,
                    });
                }

                Ok(PreparedTestTransaction::V2(prepared))
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
            PreparedTestTransaction::V2(intents) => {
                let payload_size = intents
                    .iter()
                    .map(|intent| {
                        intent.encoded_instructions.len()
                            + intent.blobs.values().map(|x| x.len()).sum::<usize>()
                    })
                    .sum();
                let num_of_signature_validations = initial_proofs.len() + 1;
                let context = ExecutionContext {
                    unique_hash: intents.get(0).unwrap().hash,
                    intent_hash_nullifications: vec![],
                    epoch_range: None,
                    payload_size,
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
                };
                let mut root_auth_zone = Some(AuthZoneInit::proofs(initial_proofs));
                let intents = intents
                    .iter()
                    .map(|intent| {
                        // TODO
                        let auth_zone_init = root_auth_zone
                            .take()
                            .unwrap_or(AuthZoneInit::proofs(Default::default()));

                        ExecutableIntent {
                            encoded_instructions: intent.encoded_instructions.clone(),
                            auth_zone_init,
                            references: intent.references.clone(),
                            blobs: intent.blobs.clone(),
                            children_intent_indices: intent.children_intent_indices.clone(),
                        }
                    })
                    .collect();

                ExecutableTransaction::new_v2(intents, context)
            }
        }
    }
}
