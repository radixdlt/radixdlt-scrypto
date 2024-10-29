use crate::internal_prelude::*;

/// A [`PreviewTransactionV2`] is the payload for V2 preview requests.
///
/// This model is similar to [`SignedTransactionIntentV2`], except it doesn't
/// require signatures, and instead allows just using public keys.
///
/// It can be currently constructed from a [`TransactionV2Builder`].
/// In future we may also support a `PreviewSubintentV2Builder` and
/// `PreviewTransactionV2Builder` which don't require the subintents
/// to be signed properly, and instead just allow the public keys to
/// be specified. For now, if you wish to support that paradigm, just
/// add the public keys manually.
///
/// Unlike with V1 preview, the V2 preview API (at least at launch) will take
/// a raw payload of this type, rather than a JSON model. This will be more
/// consistent with the transaction submit API, and avoid UX issues with
/// ensuring the subintent hashes in the `USE_CHILD` manifest instructions
/// survive encoding to JSON.
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe, ScryptoSborAssertion)]
#[sbor_assert(
    fixed("FILE:preview_transaction_v2_schema.bin"),
    settings(allow_name_changes)
)]
pub struct PreviewTransactionV2 {
    pub transaction_intent: TransactionIntentV2,
    pub root_signer_public_keys: IndexSet<PublicKey>,
    pub non_root_subintent_signer_public_keys: Vec<Vec<PublicKey>>,
}

impl PreviewTransactionV2 {
    pub fn prepare_and_validate(
        &self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedPreviewTransactionV2, TransactionValidationError> {
        self.prepare(validator.preparation_settings())?
            .validate(validator)
    }
}

define_transaction_payload!(
    PreviewTransactionV2,
    RawPreviewTransaction,
    PreparedPreviewTransactionV2 {
        transaction_intent: PreparedTransactionIntentV2,
        root_subintent_signatures: SummarizedRawValueBody<Vec<PublicKey>>,
        non_root_subintent_signatures: SummarizedRawValueBody<Vec<Vec<PublicKey>>>,
    },
    TransactionDiscriminator::V2PreviewTransaction,
);

impl PreparedPreviewTransactionV2 {
    pub fn validate(
        self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedPreviewTransactionV2, TransactionValidationError> {
        validator.validate_preview_transaction_v2(self)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ValidatedPreviewTransactionV2 {
    pub prepared: PreparedPreviewTransactionV2,
    pub overall_validity_range: OverallValidityRangeV2,
    /// This would be the expected number of signature validations, if the
    /// given public keys (and the notary) signed the transaction
    pub total_expected_signature_validations: usize,
    pub transaction_intent_info: ValidatedIntentInformationV2,
    pub non_root_subintents_info: Vec<ValidatedIntentInformationV2>,
}

impl ValidatedPreviewTransactionV2 {
    pub fn create_executable(self, flags: PreviewFlags) -> ExecutableTransaction {
        let transaction_intent = self.prepared.transaction_intent;
        let transaction_intent_hash = transaction_intent.transaction_intent_hash();
        let transaction_header = transaction_intent.transaction_header.inner;
        let subintents = transaction_intent.non_root_subintents.subintents;

        // NOTE: Ideally we'd use a slightly more accurate estimation for the notarized transaction
        // payload size here, by estimating the additional size of the signatures and notarization models.
        // This can be improved in future.
        let payload_size = transaction_intent.summary.effective_length;

        let mut simulate_every_proof_under_resources = BTreeSet::new();
        if flags.assume_all_signature_proofs {
            simulate_every_proof_under_resources.insert(SECP256K1_SIGNATURE_RESOURCE);
            simulate_every_proof_under_resources.insert(ED25519_SIGNATURE_RESOURCE);
        }

        let costing_parameters = TransactionCostingParameters {
            tip: TipSpecifier::BasisPoints(transaction_header.tip_basis_points),
            free_credit_in_xrd: if flags.use_free_credit {
                Decimal::try_from(PREVIEW_CREDIT_IN_XRD).unwrap()
            } else {
                Decimal::ZERO
            },
        };

        let mut intent_hash_nullifications =
            Vec::with_capacity(self.non_root_subintents_info.len() + 1);
        {
            let expiry_epoch = transaction_intent
                .root_intent_core
                .header
                .inner
                .end_epoch_exclusive;
            intent_hash_nullifications
                .push(IntentHash::from(transaction_intent_hash).to_nullification(expiry_epoch));
        }
        for subintent in subintents.iter() {
            let expiry_epoch = subintent.intent_core.header.inner.end_epoch_exclusive;
            intent_hash_nullifications
                .push(IntentHash::from(subintent.subintent_hash()).to_nullification(expiry_epoch))
        }

        let executable_transaction_intent = create_executable_intent(
            transaction_intent.root_intent_core,
            self.transaction_intent_info,
            &flags,
        );

        let executable_subintents = subintents
            .into_iter()
            .zip(self.non_root_subintents_info.into_iter())
            .map(|(subintent, info)| create_executable_intent(subintent.intent_core, info, &flags))
            .collect();

        ExecutableTransaction::new_v2(
            executable_transaction_intent,
            executable_subintents,
            ExecutionContext {
                unique_hash: transaction_intent_hash.0,
                intent_hash_nullifications,
                payload_size,
                num_of_signature_validations: self.total_expected_signature_validations,
                costing_parameters,
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
    flags: &PreviewFlags,
) -> ExecutableIntent {
    let signer_keys = validated_info.signer_keys;

    let mut simulate_every_proof_under_resources = BTreeSet::new();
    if flags.assume_all_signature_proofs {
        simulate_every_proof_under_resources.insert(SECP256K1_SIGNATURE_RESOURCE);
        simulate_every_proof_under_resources.insert(ED25519_SIGNATURE_RESOURCE);
    }

    let auth_zone_init = AuthZoneInit::new(
        AuthAddresses::signer_set(&signer_keys),
        simulate_every_proof_under_resources,
    );

    ExecutableIntent {
        encoded_instructions: validated_info.encoded_instructions,
        auth_zone_init,
        references: core.instructions.references,
        blobs: core.blobs.blobs_by_hash,
        children_subintent_indices: validated_info.children_subintent_indices,
    }
}
