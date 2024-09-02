use crate::internal_prelude::*;
use radix_common::constants::AuthAddresses;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValidatedNotarizedTransactionV1 {
    pub prepared: PreparedNotarizedTransactionV1,
    pub encoded_instructions: Rc<Vec<u8>>,
    pub signer_keys: Vec<PublicKey>,
    pub num_of_signature_validations: usize,
}

impl HasTransactionIntentHash for ValidatedNotarizedTransactionV1 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        self.prepared.transaction_intent_hash()
    }
}

impl HasSignedTransactionIntentHash for ValidatedNotarizedTransactionV1 {
    fn signed_transaction_intent_hash(&self) -> SignedTransactionIntentHash {
        self.prepared.signed_transaction_intent_hash()
    }
}

impl HasNotarizedTransactionHash for ValidatedNotarizedTransactionV1 {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash {
        self.prepared.notarized_transaction_hash()
    }
}

#[allow(deprecated)]
impl ValidatedNotarizedTransactionV1 {
    pub fn get_executable(&self) -> ExecutableTransactionV1 {
        let intent = &self.prepared.signed_intent.intent;
        let header = &intent.header.inner;
        let intent_hash = intent.transaction_intent_hash();
        let summary = &self.prepared.summary;

        ExecutableTransactionV1::new(
            self.encoded_instructions.clone(),
            AuthZoneInit::proofs(AuthAddresses::signer_set(&self.signer_keys)),
            intent.instructions.references.clone(),
            intent.blobs.blobs_by_hash.clone(),
            ExecutionContext {
                unique_hash: intent_hash.0,
                intent_hash_nullifications: vec![IntentHashNullification::TransactionIntent {
                    intent_hash,
                    expiry_epoch: header.end_epoch_exclusive,
                    ignore_duplicate: false,
                }],
                epoch_range: Some(EpochRange {
                    start_epoch_inclusive: header.start_epoch_inclusive,
                    end_epoch_exclusive: header.end_epoch_exclusive,
                }),
                payload_size: summary.effective_length,
                num_of_signature_validations: self.num_of_signature_validations,
                costing_parameters: TransactionCostingParameters {
                    tip: TipSpecifier::Percentage(intent.header.inner.tip_percentage),
                    free_credit_in_xrd: Decimal::ZERO,
                    abort_when_loan_repaid: false,
                },
                pre_allocated_addresses: vec![],
                disable_limits_and_costing_modules: false,
                start_timestamp_inclusive: None,
                end_timestamp_exclusive: None,
            },
            false,
        )
    }
}
