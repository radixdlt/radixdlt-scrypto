use crate::internal_prelude::*;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ValidatedNotarizedTransactionV2 {
    pub prepared: PreparedNotarizedTransactionV2,
    pub total_signature_validations: usize,
    pub overall_validity_range: OverallValidityRangeV2,
    pub transaction_intent_info: ValidatedIntentInformationV2,
    pub non_root_subintents_info: Vec<ValidatedIntentInformationV2>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ValidatedSignedPartialTransactionV2 {
    pub prepared: PreparedSignedPartialTransactionV2,
    pub total_signature_validations: usize,
    pub overall_validity_range: OverallValidityRangeV2,
    pub root_subintent_info: ValidatedIntentInformationV2,
    pub root_subintent_yield_to_parent_count: usize,
    pub non_root_subintents_info: Vec<ValidatedIntentInformationV2>,
}

pub struct ValidatedTransactionTreeV2 {
    pub overall_validity_range: OverallValidityRangeV2,
    pub total_signature_validations: usize,
    pub root_intent_info: ValidatedIntentInformationV2,
    pub root_yield_to_parent_count: usize,
    pub non_root_subintents_info: Vec<ValidatedIntentInformationV2>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct OverallValidityRangeV2 {
    pub epoch_range: EpochRange,
    pub proposer_timestamp_range: ProposerTimestampRange,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ValidatedIntentInformationV2 {
    pub encoded_instructions: Vec<u8>,
    pub signer_keys: IndexSet<PublicKey>,
    pub children_subintent_indices: Vec<SubintentIndex>,
}

impl HasTransactionIntentHash for ValidatedNotarizedTransactionV2 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        self.prepared.transaction_intent_hash()
    }
}

impl HasSignedTransactionIntentHash for ValidatedNotarizedTransactionV2 {
    fn signed_transaction_intent_hash(&self) -> SignedTransactionIntentHash {
        self.prepared.signed_transaction_intent_hash()
    }
}

impl HasNotarizedTransactionHash for ValidatedNotarizedTransactionV2 {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash {
        self.prepared.notarized_transaction_hash()
    }
}

impl HasNonRootSubintentHashes for ValidatedNotarizedTransactionV2 {
    fn non_root_subintent_hashes(&self) -> Vec<SubintentHash> {
        self.prepared.non_root_subintent_hashes()
    }
}

impl IntoExecutable for ValidatedNotarizedTransactionV2 {
    type Error = core::convert::Infallible;

    fn into_executable(
        self,
        _validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        Ok(self.create_executable())
    }
}

impl ValidatedNotarizedTransactionV2 {
    pub fn hashes(&self) -> UserTransactionHashes {
        self.prepared.hashes()
    }

    pub fn create_executable(self) -> ExecutableTransaction {
        let transaction_intent = self.prepared.signed_intent.transaction_intent;
        let transaction_intent_hash = transaction_intent.transaction_intent_hash();
        let transaction_header = transaction_intent.transaction_header.inner;
        let payload_size = self.prepared.summary.effective_length;
        let subintents = transaction_intent.non_root_subintents.subintents;

        let mut intent_hash_nullifications =
            Vec::with_capacity(self.non_root_subintents_info.len() + 1);
        intent_hash_nullifications.push(
            IntentHash::from(transaction_intent_hash).to_nullification(
                transaction_intent
                    .root_intent_core
                    .header
                    .inner
                    .end_epoch_exclusive,
            ),
        );
        for subintent in subintents.iter() {
            intent_hash_nullifications.push(
                IntentHash::from(subintent.subintent_hash())
                    .to_nullification(subintent.intent_core.header.inner.end_epoch_exclusive),
            )
        }
        let executable_transaction_intent = create_executable_intent(
            transaction_intent.root_intent_core,
            self.transaction_intent_info,
        );
        let executable_subintents = subintents
            .into_iter()
            .zip(self.non_root_subintents_info.into_iter())
            .map(|(subintent, info)| create_executable_intent(subintent.intent_core, info))
            .collect();

        ExecutableTransaction::new_v2(
            executable_transaction_intent,
            executable_subintents,
            ExecutionContext {
                unique_hash: transaction_intent_hash.0,
                intent_hash_nullifications,
                payload_size,
                num_of_signature_validations: self.total_signature_validations,
                costing_parameters: TransactionCostingParameters {
                    tip: TipSpecifier::BasisPoints(transaction_header.tip_basis_points),
                    free_credit_in_xrd: Decimal::ZERO,
                },
                pre_allocated_addresses: vec![],
                disable_limits_and_costing_modules: false,
                epoch_range: Some(self.overall_validity_range.epoch_range.clone()),
                proposer_timestamp_range: Some(
                    self.overall_validity_range.proposer_timestamp_range.clone(),
                ),
            },
        )
    }
}

fn create_executable_intent(
    core: PreparedIntentCoreV2,
    validated_info: ValidatedIntentInformationV2,
) -> ExecutableIntent {
    let signer_keys = validated_info.signer_keys;
    let auth_zone_init = AuthZoneInit::proofs(AuthAddresses::signer_set(&signer_keys));

    ExecutableIntent {
        encoded_instructions: validated_info.encoded_instructions,
        auth_zone_init,
        references: core.instructions.references,
        blobs: core.blobs.blobs_by_hash,
        children_subintent_indices: validated_info.children_subintent_indices,
    }
}
