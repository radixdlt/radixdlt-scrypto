use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct SystemTransactionV1 {
    pub instructions: InstructionsV1,
    pub blobs: BlobsV1,
    pub pre_allocated_addresses: Vec<PreAllocatedAddress>,
    pub hash_for_execution: Hash,
}

impl TransactionPayload for SystemTransactionV1 {
    type Prepared = PreparedSystemTransactionV1;
    type Raw = RawSystemTransaction;
}

#[allow(deprecated)]
type PreparedPreAllocatedAddresses = SummarizedRawFullValue<Vec<PreAllocatedAddress>>;
type PreparedHash = RawHash;

pub struct PreparedSystemTransactionV1 {
    pub encoded_instructions: Rc<Vec<u8>>,
    pub references: IndexSet<Reference>,
    pub blobs: PreparedBlobsV1,
    pub pre_allocated_addresses: PreparedPreAllocatedAddresses,
    pub hash_for_execution: PreparedHash,
    pub summary: Summary,
}

impl_has_summary!(PreparedSystemTransactionV1);

impl HasSystemTransactionHash for PreparedSystemTransactionV1 {
    fn system_transaction_hash(&self) -> SystemTransactionHash {
        SystemTransactionHash::from_hash(self.summary.hash)
    }
}

#[allow(deprecated)]
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
            encoded_instructions: Rc::new(manifest_encode(&prepared_instructions.inner.0)?),
            references: prepared_instructions.references,
            blobs,
            pre_allocated_addresses,
            hash_for_execution,
            summary,
        })
    }
}

#[allow(deprecated)]
impl TransactionPreparableFromValue for PreparedSystemTransactionV1 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let ((prepared_instructions, blobs, pre_allocated_addresses, hash_for_execution), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct::<(
                PreparedInstructionsV1,
                PreparedBlobsV1,
                PreparedPreAllocatedAddresses,
                PreparedHash,
            )>(decoder, TransactionDiscriminator::V1System)?;
        Ok(Self {
            encoded_instructions: Rc::new(manifest_encode(&prepared_instructions.inner.0)?),
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

#[allow(deprecated)]
impl PreparedSystemTransactionV1 {
    pub fn get_executable(
        &self,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> ExecutableTransactionV1 {
        ExecutableTransactionV1::new(
            self.encoded_instructions.clone(),
            self.references.clone(),
            self.blobs.blobs_by_hash.clone(),
            ExecutionContext {
                unique_hash: self.hash_for_execution.hash,
                intent_hash_nullifications: vec![
                    IntentHashNullification::System
                ],
                epoch_range: None,
                payload_size: 0,
                num_of_signature_validations: 0,
                auth_zone_init: AuthZoneInit::proofs(initial_proofs),
                costing_parameters: TransactionCostingParameters {
                    tip: TipSpecifier::None,
                    free_credit_in_xrd: Decimal::ZERO,
                    abort_when_loan_repaid: false,
                },
                pre_allocated_addresses: self.pre_allocated_addresses.inner.clone(),
            },
            true,
        )
    }
}
