use crate::internal_prelude::*;

#[derive(ManifestSbor)]
pub struct TestTransaction {
    pub instructions: InstructionsV1,
    pub blobs: BlobsV1,
    pub nonce: u32,
}

#[derive(ManifestSbor)]
pub struct PreparedTestTransaction {
    pub encoded_instructions: Vec<u8>,
    pub references: IndexSet<Reference>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
    pub nonce: u32,
}

impl TestTransaction {
    pub fn new(manifest: TransactionManifestV1, nonce: u32) -> Self {
        let (instructions, blobs) = manifest.for_intent();
        Self {
            instructions,
            blobs,
            nonce,
        }
    }

    pub fn prepare(self) -> Result<PreparedTestTransaction, ConvertToPreparedError> {
        let prepared_instructions = self.instructions.prepare_partial()?;
        Ok(PreparedTestTransaction {
            encoded_instructions: manifest_encode(&prepared_instructions.inner.0)?,
            references: prepared_instructions.references,
            blobs: self.blobs.prepare_partial()?.blobs_by_hash,
            nonce: self.nonce,
        })
    }
}

impl PreparedTestTransaction {
    pub fn get_executable<'a>(
        &'a self,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> Executable<'a> {
        // Fake transaction hash
        let transaction_hash = hash(self.nonce.to_le_bytes());

        let auth_zone_params = AuthZoneParams {
            initial_proofs,
            virtual_resources: BTreeSet::new(),
        };

        Executable::new(
            &self.encoded_instructions,
            &self.references,
            &self.blobs,
            ExecutionContext {
                transaction_hash,
                payload_size: self.encoded_instructions.len(),
                auth_zone_params,
                fee_payment: FeePayment::User { tip_percentage: 0 },
                runtime_validations: vec![],
                pre_allocated_ids: BTreeSet::new(),
            },
        )
    }
}
