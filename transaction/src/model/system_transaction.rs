use super::{ExecutionContext, FeePayment};
use crate::internal_prelude::*;
use crate::model::{AuthZoneParams, Executable};

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct SystemTransaction {
    pub instructions: InstructionsV1,
    pub blobs: BlobsV1,
    pub pre_allocated_ids: BTreeSet<NodeId>,
    pub nonce: u32,
}

pub struct PreparedSystemTransaction {
    pub encoded_instructions: Vec<u8>,
    pub references: IndexSet<Reference>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
    pub pre_allocated_ids: BTreeSet<NodeId>,
    pub nonce: u32,
}

impl SystemTransaction {
    pub fn new(manifest: TransactionManifestV1, nonce: u32) -> Self {
        let (instructions, blobs) = manifest.for_intent();

        Self {
            instructions,
            blobs,
            pre_allocated_ids: btreeset!(),
            nonce,
        }
    }

    pub fn prepare(self) -> Result<PreparedSystemTransaction, ConvertToPreparedError> {
        let prepared_instructions = self.instructions.prepare_partial()?;
        Ok(PreparedSystemTransaction {
            encoded_instructions: manifest_encode(&prepared_instructions.inner.0)?,
            references: prepared_instructions.references,
            blobs: self.blobs.prepare_partial()?.blobs_by_hash,
            pre_allocated_ids: self.pre_allocated_ids,
            nonce: self.nonce,
        })
    }
}

impl PreparedSystemTransaction {
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
                payload_size: 0,
                auth_zone_params,
                fee_payment: FeePayment::NoFee,
                runtime_validations: vec![],
                pre_allocated_ids: self.pre_allocated_ids.clone(),
            },
        )
    }
}
