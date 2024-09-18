use crate::internal_prelude::*;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ValidatedNotarizedTransactionV2 {
    pub prepared: PreparedNotarizedTransactionV2,
    pub overall_epoch_range: EpochRange,
    pub overall_start_timestamp_inclusive: Option<Instant>,
    pub overall_end_timestamp_exclusive: Option<Instant>,
    pub transaction_intent_info: ValidatedIntentInformationV2,
    pub subintents_info: Vec<ValidatedIntentInformationV2>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ValidatedIntentInformationV2 {
    pub encoded_instructions: Vec<u8>,
    pub signature_validations: SignatureValidations,
    pub children_subintent_indices: Vec<SubintentIndex>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
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
    pub fn create_executable(self) -> ExecutableTransaction {
        let transaction_intent = self.prepared.signed_intent.root_intent;
        let transaction_intent_hash = transaction_intent.transaction_intent_hash();
        let transaction_header = transaction_intent.root_header.inner;
        let summary = self.prepared.summary;
        let subintents = transaction_intent.subintents.subintents;

        let mut intent_hash_nullifications = Vec::with_capacity(self.subintents_info.len() + 1);
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
            .zip(self.subintents_info.into_iter())
            .map(|(subintent, info)| create_executable_intent(subintent.intent_core, info))
            .collect();

        ExecutableTransaction::new_v2(
            executable_transaction_intent,
            executable_subintents,
            ExecutionContext {
                unique_hash: transaction_intent_hash.0,
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

fn create_executable_intent(
    core: PreparedIntentCoreV2,
    validated_info: ValidatedIntentInformationV2,
) -> ExecutableIntent {
    // FIX ME when we implement delegated signature checking
    let signer_keys = match validated_info.signature_validations {
        SignatureValidations::Validated {
            signer_keys,
            num_validations: _,
        } => signer_keys,
        SignatureValidations::Unvalidated { signatures: _ } => {
            unimplemented!()
        }
    };
    let auth_zone_init = AuthZoneInit::proofs(AuthAddresses::signer_set(&signer_keys));

    ExecutableIntent {
        encoded_instructions: Rc::new(validated_info.encoded_instructions),
        auth_zone_init,
        references: core.instructions.references.clone(),
        blobs: core.blobs.blobs_by_hash.clone(),
        children_subintent_indices: validated_info.children_subintent_indices.clone(),
    }
}

#[deprecated]
fn move_sig_validations_to_be_lazy_and_bill_correctly() -> usize {
    0
}
