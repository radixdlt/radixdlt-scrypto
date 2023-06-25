use super::{ExecutionContext, FeePayment};
use crate::internal_prelude::*;
use crate::model::{AuthZoneParams, Executable};

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct SystemTransactionV1 {
    pub instructions: InstructionsV1,
    pub blobs: BlobsV1,
    pub pre_allocated_addresses: Vec<PreAllocatedAddress>,
    pub hash_for_execution: Hash,
}

impl TransactionPayload for SystemTransactionV1 {
    type Versioned = SborFixedEnumVariant<{ TransactionDiscriminator::V1System as u8 }, Self>;
    type Prepared = PreparedSystemTransactionV1;
    type Raw = RawSystemTransaction;
}

type PreparedPreAllocatedAddresses = SummarizedRawFullBody<Vec<PreAllocatedAddress>>;
type PreparedHash = SummarizedHash;

pub struct PreparedSystemTransactionV1 {
    pub encoded_instructions: Vec<u8>,
    pub references: IndexSet<Reference>,
    pub blobs: PreparedBlobsV1,
    pub pre_allocated_addresses: PreparedPreAllocatedAddresses,
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
    type Raw = RawSystemTransaction;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let ((prepared_instructions, blobs, pre_allocated_addresses, hash_for_execution), summary) =
            ConcatenatedDigest::prepare_from_transaction_payload_enum::<(
                PreparedInstructionsV1,
                PreparedBlobsV1,
                PreparedPreAllocatedAddresses,
                PreparedHash,
            )>(decoder, TransactionDiscriminator::V1System)?;
        Ok(Self {
            encoded_instructions: manifest_encode(&prepared_instructions.inner.0)?,
            references: prepared_instructions.references,
            blobs,
            pre_allocated_addresses,
            hash_for_execution,
            summary,
        })
    }
}

impl TransactionFullChildPreparable for PreparedSystemTransactionV1 {
    fn prepare_as_full_body_child(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let ((prepared_instructions, blobs, pre_allocated_addresses, hash_for_execution), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct::<(
                PreparedInstructionsV1,
                PreparedBlobsV1,
                PreparedPreAllocatedAddresses,
                PreparedHash,
            )>(decoder, TransactionDiscriminator::V1System)?;
        Ok(Self {
            encoded_instructions: manifest_encode(&prepared_instructions.inner.0)?,
            references: prepared_instructions.references,
            blobs,
            pre_allocated_addresses,
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
            pre_allocated_addresses: vec![],
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
                intent_hash: TransactionIntentHash::NotToCheck {
                    intent_hash: self.hash_for_execution.hash,
                },
                epoch_range: None,
                payload_size: 0,
                auth_zone_params: AuthZoneParams {
                    initial_proofs,
                    virtual_resources: BTreeSet::new(),
                },
                fee_payment: FeePayment {
                    tip_percentage: 0,
                    free_credit_in_xrd: Decimal::ZERO,
                },
                pre_allocated_addresses: self.pre_allocated_addresses.inner.clone(),
            },
        )
    }
}
