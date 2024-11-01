use crate::internal_prelude::*;
use radix_common::constants::AuthAddresses;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValidatedNotarizedTransactionV1 {
    pub prepared: PreparedNotarizedTransactionV1,
    pub encoded_instructions: Vec<u8>,
    pub signer_keys: IndexSet<PublicKey>,
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
    pub fn hashes(&self) -> UserTransactionHashes {
        self.prepared.hashes()
    }

    pub fn create_executable(self) -> ExecutableTransaction {
        let intent = self.prepared.signed_intent.intent;
        let intent_hash = intent.transaction_intent_hash();
        let header = intent.header.inner;
        let summary = self.prepared.summary;

        ExecutableTransaction::new_v1(
            self.encoded_instructions,
            AuthZoneInit::proofs(AuthAddresses::signer_set(&self.signer_keys)),
            intent.instructions.references.clone(),
            intent.blobs.blobs_by_hash,
            ExecutionContext {
                unique_hash: intent_hash.0,
                intent_hash_nullifications: vec![IntentHashNullification::TransactionIntent {
                    intent_hash,
                    expiry_epoch: header.end_epoch_exclusive,
                }],
                epoch_range: Some(EpochRange {
                    start_epoch_inclusive: header.start_epoch_inclusive,
                    end_epoch_exclusive: header.end_epoch_exclusive,
                }),
                payload_size: summary.effective_length,
                num_of_signature_validations: self.num_of_signature_validations,
                costing_parameters: TransactionCostingParameters {
                    tip: TipSpecifier::Percentage(header.tip_percentage),
                    free_credit_in_xrd: Decimal::ZERO,
                },
                pre_allocated_addresses: vec![],
                disable_limits_and_costing_modules: false,
                proposer_timestamp_range: None,
            },
        )
    }
}
