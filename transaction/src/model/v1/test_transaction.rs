use crate::internal_prelude::*;
use crate::model::*;
use radix_engine_interface::blueprints::resource::NonFungibleGlobalId;
use radix_engine_interface::crypto::hash;
use radix_engine_interface::data::manifest::*;
use radix_engine_interface::*;
use std::collections::BTreeSet;

#[derive(ManifestSbor)]
pub struct TestTransaction {
    pub instructions: InstructionsV1,
    pub blobs: BlobsV1,
    pub hash: Hash,
}

#[derive(ManifestSbor)]
pub struct PreparedTestTransaction {
    pub encoded_instructions: Vec<u8>,
    pub references: IndexSet<Reference>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
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

    pub fn prepare(self) -> Result<PreparedTestTransaction, PrepareError> {
        let prepared_instructions = self.instructions.prepare_partial()?;
        Ok(PreparedTestTransaction {
            encoded_instructions: manifest_encode(&prepared_instructions.inner.0)?,
            references: prepared_instructions.references,
            blobs: self.blobs.prepare_partial()?.blobs_by_hash,
            hash: self.hash,
        })
    }
}

impl PreparedTestTransaction {
    pub fn get_executable<'a>(
        &'a self,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> Executable<'a> {
        Executable::new(
            &self.encoded_instructions,
            &self.references,
            &self.blobs,
            ExecutionContext {
                intent_hash: TransactionIntentHash::NotToCheck {
                    intent_hash: self.hash,
                },
                epoch_range: None,
                payload_size: self.encoded_instructions.len()
                    + self.blobs.values().map(|x| x.len()).sum::<usize>(),
                auth_zone_params: AuthZoneParams {
                    initial_proofs,
                    virtual_resources: BTreeSet::new(),
                },
                fee_payment: FeePayment {
                    tip_percentage: DEFAULT_TIP_PERCENTAGE,
                    free_credit_in_xrd: Decimal::ZERO,
                },
                pre_allocated_addresses: vec![],
            },
        )
    }
}
