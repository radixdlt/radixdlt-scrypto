use crate::internal_prelude::*;
use crate::model::*;
use radix_common::crypto::hash;
use radix_common::data::manifest::*;
use radix_common::types::NonFungibleGlobalId;
use std::collections::BTreeSet;

#[derive(ManifestSbor)]
pub struct TestTransaction {
    pub instructions: InstructionsV1,
    pub blobs: BlobsV1,
    pub hash: Hash,
}

#[derive(ManifestSbor)]
pub struct PreparedTestTransaction {
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
        Self {
            instructions,
            blobs,
            hash,
        }
    }

    #[allow(deprecated)]
    pub fn prepare(self) -> Result<PreparedTestTransaction, PrepareError> {
        let prepared_instructions = self.instructions.prepare_partial()?;
        Ok(PreparedTestTransaction {
            encoded_instructions: Rc::new(manifest_encode(&prepared_instructions.inner.0)?),
            references: prepared_instructions.references,
            blobs: self.blobs.prepare_partial()?.blobs_by_hash,
            hash: self.hash,
        })
    }
}

impl PreparedTestTransaction {
    pub fn get_executable(
        &self,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> ExecutableTransactionV1 {
        ExecutableTransactionV1::new(
            self.encoded_instructions.clone(),
            self.references.clone(),
            self.blobs.clone(),
            ExecutionContext {
                unique_hash: self.hash,
                intent_hash_nullifications: vec![],
                epoch_range: None,
                payload_size: self.encoded_instructions.len()
                    + self.blobs.values().map(|x| x.len()).sum::<usize>(),
                // For testing purpose, assume `num_of_signature_validations = num_of_initial_proofs + 1`
                num_of_signature_validations: initial_proofs.len() + 1,
                auth_zone_init: AuthZoneInit::proofs(initial_proofs),
                costing_parameters: TransactionCostingParameters {
                    tip: TipSpecifier::None,
                    free_credit_in_xrd: Decimal::ZERO,
                    abort_when_loan_repaid: false,
                },
                pre_allocated_addresses: vec![],
            },
            false,
        )
    }
}
