use super::{ExecutionContext, FeePayment};
use crate::internal_prelude::*;
use crate::model::{AuthZoneParams, Executable};

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct SystemTransactionV1 {
    pub instructions: InstructionsV1,
    pub blobs: BlobsV1,
    // This is an IndexSet rather than a BTreeSet to better satisfy the round trip property
    // when encoding from a value model which may not respect the BTreeSet ordering
    pub pre_allocated_ids: IndexSet<NodeId>,
    pub hash_for_execution: Hash,
}

impl TransactionPayload for SystemTransactionV1 {
    type Versioned = SborFixedEnumVariant<{ TransactionDiscriminator::V1System as u8 }, Self>;
    type Prepared = PreparedSystemTransactionV1;
}

type PreparedPreAllocatedIds = SummarizedRawFullBody<IndexSet<NodeId>>;
type PreparedHash = SummarizedHash;

pub struct PreparedSystemTransactionV1 {
    pub encoded_instructions: Vec<u8>,
    pub references: IndexSet<Reference>,
    pub blobs: PreparedBlobsV1,
    pub pre_allocated_ids: PreparedPreAllocatedIds,
    pub hash_for_execution: PreparedHash,
    pub summary: Summary,
}

impl HasSystemTransactionHash for PreparedSystemTransactionV1 {
    fn system_transaction_hash(&self) -> SystemTransactionHash {
        SystemTransactionHash::from_hash(self.summary.hash)
    }
}

impl HasSummary for PreparedSystemTransactionV1 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionPayloadPreparable for PreparedSystemTransactionV1 {
    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let ((prepared_instructions, blobs, pre_allocated_ids, hash_for_execution), summary) =
            ConcatenatedDigest::prepare_from_transaction_payload_enum::<(
                PreparedInstructionsV1,
                PreparedBlobsV1,
                PreparedPreAllocatedIds,
                PreparedHash,
            )>(decoder, TransactionDiscriminator::V1System)?;
        Ok(Self {
            encoded_instructions: manifest_encode(&prepared_instructions.inner.0)?,
            references: prepared_instructions.references,
            blobs,
            pre_allocated_ids,
            hash_for_execution,
            summary,
        })
    }
}

impl TransactionFullChildPreparable for PreparedSystemTransactionV1 {
    fn prepare_as_full_body_child(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let ((prepared_instructions, blobs, pre_allocated_ids, hash_for_execution), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct::<(
                PreparedInstructionsV1,
                PreparedBlobsV1,
                PreparedPreAllocatedIds,
                PreparedHash,
            )>(decoder, TransactionDiscriminator::V1System)?;
        Ok(Self {
            encoded_instructions: manifest_encode(&prepared_instructions.inner.0)?,
            references: prepared_instructions.references,
            blobs,
            pre_allocated_ids,
            hash_for_execution,
            summary,
        })
    }
}

impl SystemTransactionV1 {
    pub fn new(manifest: TransactionManifestV1, hash_for_execution: Hash) -> Self {
        let (instructions, blobs) = manifest.for_intent();

        Self {
            instructions,
            blobs,
            pre_allocated_ids: indexset!(),
            hash_for_execution,
        }
    }
}

impl PreparedSystemTransactionV1 {
    pub fn get_executable<'a>(
        &'a self,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> Executable<'a> {
        Executable::new(
            &self.encoded_instructions,
            &self.references,
            &self.blobs.blobs_by_hash,
            ExecutionContext {
                transaction_hash: self.hash_for_execution.hash,
                payload_size: 0,
                auth_zone_params: AuthZoneParams {
                    initial_proofs,
                    virtual_resources: BTreeSet::new(),
                },
                fee_payment: FeePayment::NoFee,
                runtime_validations: vec![],
                pre_allocated_ids: self.pre_allocated_ids.inner.clone(),
            },
        )
    }
}
