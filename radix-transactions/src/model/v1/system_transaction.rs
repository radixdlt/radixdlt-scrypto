use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct SystemTransactionV1 {
    pub instructions: InstructionsV1,
    pub blobs: BlobsV1,
    pub pre_allocated_addresses: Vec<PreAllocatedAddress>,
    pub hash_for_execution: Hash,
}

impl SystemTransactionV1 {
    pub fn with_proofs_ref(
        &self,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> SystemTransactionV1WithProofs {
        SystemTransactionV1WithProofs {
            initial_proofs,
            transaction: Cow::Borrowed(self),
        }
    }

    pub fn with_proofs(
        self,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> SystemTransactionV1WithProofs<'static> {
        SystemTransactionV1WithProofs {
            initial_proofs,
            transaction: Cow::Owned(self),
        }
    }
}

/// This is mostly so that you can create executables easily.
/// We can't update SystemTransaction to include these proofs, because
/// it's already used in genesis.
pub struct SystemTransactionV1WithProofs<'a> {
    initial_proofs: BTreeSet<NonFungibleGlobalId>,
    transaction: Cow<'a, SystemTransactionV1>,
}

impl<'a> IntoExecutable for SystemTransactionV1WithProofs<'a> {
    type Error = PrepareError;

    fn into_executable(
        self,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        let executable = self
            .transaction
            .prepare(validator.preparation_settings())?
            .create_executable(self.initial_proofs);
        Ok(executable)
    }
}

#[allow(deprecated)]
type PreparedPreAllocatedAddresses = SummarizedRawFullValue<Vec<PreAllocatedAddress>>;
type PreparedHash = RawHash;

impl TransactionPayload for SystemTransactionV1 {
    type Prepared = PreparedSystemTransactionV1;
    type Raw = RawSystemTransaction;
}

pub struct PreparedSystemTransactionV1 {
    pub encoded_instructions: Vec<u8>,
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
impl PreparedTransaction for PreparedSystemTransactionV1 {
    type Raw = RawSystemTransaction;

    fn prepare_from_transaction_enum(
        decoder: &mut TransactionDecoder,
    ) -> Result<Self, PrepareError> {
        let ((prepared_instructions, blobs, pre_allocated_addresses, hash_for_execution), summary) =
            ConcatenatedDigest::prepare_transaction_payload::<(
                PreparedInstructionsV1,
                PreparedBlobsV1,
                PreparedPreAllocatedAddresses,
                PreparedHash,
            )>(
                decoder,
                TransactionDiscriminator::V1System,
                ExpectedHeaderKind::EnumWithValueKind,
            )?;
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

#[allow(deprecated)]
impl TransactionPreparableFromValue for PreparedSystemTransactionV1 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let ((prepared_instructions, blobs, pre_allocated_addresses, hash_for_execution), summary) =
            ConcatenatedDigest::prepare_transaction_payload::<(
                PreparedInstructionsV1,
                PreparedBlobsV1,
                PreparedPreAllocatedAddresses,
                PreparedHash,
            )>(
                decoder,
                TransactionDiscriminator::V1System,
                ExpectedHeaderKind::TupleWithValueKind,
            )?;
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

#[allow(deprecated)]
impl PreparedSystemTransactionV1 {
    pub fn create_executable(
        self,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> ExecutableTransaction {
        ExecutableTransaction::new_v1(
            self.encoded_instructions,
            AuthZoneInit::proofs(initial_proofs),
            self.references,
            self.blobs.blobs_by_hash,
            ExecutionContext {
                unique_hash: self.hash_for_execution.hash,
                intent_hash_nullifications: vec![],
                epoch_range: None,
                payload_size: 0,
                num_of_signature_validations: 0,
                costing_parameters: TransactionCostingParameters {
                    tip: TipSpecifier::None,
                    free_credit_in_xrd: Decimal::ZERO,
                },
                pre_allocated_addresses: self.pre_allocated_addresses.inner,
                disable_limits_and_costing_modules: true,
                proposer_timestamp_range: None,
            },
        )
    }
}
