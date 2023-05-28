use crate::internal_prelude::*;

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
                transaction_hash: self.hash,
                payload_size: self.encoded_instructions.len(),
                auth_zone_params: AuthZoneParams {
                    initial_proofs,
                    virtual_resources: BTreeSet::new(),
                },
                fee_payment: FeePayment::User { tip_percentage: 0 },
                runtime_validations: vec![],
                pre_allocated_ids: index_set_new(),
            },
        )
    }
}
