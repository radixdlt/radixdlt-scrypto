use core::iter;
use std::ops::Deref;

use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValidatedNotarizedTransactionV2 {
    pub prepared: PreparedNotarizedTransactionV2,
    pub subintent_lookup: IndexSet<SubintentHash>,
    pub overall_epoch_range: EpochRange,
    pub overall_start_timestamp_inclusive: Option<Instant>,
    pub overall_end_timestamp_exclusive: Option<Instant>,
    pub root_intent: ValidatedIntentInformationV2,
    pub subintents: Vec<ValidatedIntentInformationV2>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValidatedIntentInformationV2 {
    pub encoded_instructions: Rc<Vec<u8>>,
    pub signature_validations: SignatureValidations,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SignatureValidations {
    Validated {
        /// This could be one more than signer_keys due to notary not being a signer
        num_validations: usize,
        signer_keys: Vec<PublicKey>,
    },
    Unvalidated {
        signatures: IntentSignaturesV2,
    },
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

impl ValidatedNotarizedTransactionV2 {
    pub fn get_executable(&self) -> ExecutableTransaction {
        let transaction_intent = &self.prepared.signed_intent.root_intent;
        let transaction_header = &transaction_intent.root_header.inner;
        let root_intent_hash = transaction_intent.transaction_intent_hash();
        let summary = &self.prepared.summary;
        let root_intent_core = &transaction_intent.root_intent_core;

        let root_intent_info = (
            IntentHash::Transaction(root_intent_hash),
            root_intent_core,
            &self.root_intent,
        );
        let subintent_infos = transaction_intent
            .subintents
            .subintents_by_hash
            .iter()
            .zip(self.subintents.iter())
            .map(|((si_hash, su), info)| (IntentHash::Sub(*si_hash), &su.intent_core, info));

        let (executable_intents, intent_hash_nullifications) = iter::once(root_intent_info)
            .chain(subintent_infos)
            .map(|(intent_hash, core, validated_info)| {
                let header = &core.header.inner;

                // FIX ME when we implement delegated signature checking
                let signer_keys = match &validated_info.signature_validations {
                    SignatureValidations::Validated {
                        signer_keys,
                        num_validations: _,
                    } => signer_keys,
                    SignatureValidations::Unvalidated { signatures: _ } => {
                        unimplemented!()
                    }
                };
                let auth_zone_init = AuthZoneInit::proofs(AuthAddresses::signer_set(&signer_keys));

                let executable_intent = ExecutableIntent {
                    encoded_instructions: validated_info.encoded_instructions.clone(),
                    auth_zone_init,
                    references: core.instructions.references.deref().clone(),
                    blobs: core.blobs.blobs_by_hash.clone(),
                    children_intent_indices: core
                        .children
                        .children
                        .iter()
                        .map(|c| {
                            self.subintent_lookup
                                .get_index_of(&c.hash)
                                .expect("Hash couldn't be found in lookup")
                        })
                        .collect(),
                };
                let intent_hash_nullification =
                    intent_hash.to_nullification(header.end_epoch_exclusive);

                (executable_intent, intent_hash_nullification)
            })
            .unzip();

        ExecutableTransaction::new_v2(
            executable_intents,
            ExecutionContext {
                unique_hash: root_intent_hash.0,
                intent_hash_nullifications,
                payload_size: summary.effective_length,
                num_of_signature_validations: move_sig_validations_to_be_lazy_and_bill_correctly(),
                costing_parameters: TransactionCostingParameters {
                    tip: TipSpecifier::BasisPoints(transaction_header.tip_basis_points),
                    free_credit_in_xrd: Decimal::ZERO,
                    abort_when_loan_repaid: false,
                },
                pre_allocated_addresses: vec![],
                disable_limits_and_costing_modules: false,
                epoch_range: Some(self.overall_epoch_range.clone()),
                start_timestamp_inclusive: self.overall_start_timestamp_inclusive,
                end_timestamp_exclusive: self.overall_end_timestamp_exclusive,
            },
        )
    }
}

#[deprecated]
fn move_sig_validations_to_be_lazy_and_bill_correctly() -> usize {
    0
}
