use crate::internal_prelude::*;

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Default)]
pub struct PreviewFlags {
    pub use_free_credit: bool,
    pub assume_all_signature_proofs: bool,
    pub skip_epoch_check: bool,
    pub disable_auth: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct PreviewIntentV1 {
    pub intent: IntentV1,
    pub signer_public_keys: Vec<PublicKey>,
    pub flags: PreviewFlags,
}

pub struct ValidatedPreviewIntent {
    pub intent: PreparedIntentV1,
    pub encoded_instructions: Vec<u8>,
    pub signer_public_keys: Vec<PublicKey>,
    pub flags: PreviewFlags,
}

#[allow(deprecated)]
impl ValidatedPreviewIntent {
    pub fn create_executable(self) -> ExecutableTransaction {
        let intent = self.intent;
        let flags = self.flags;

        let mut simulate_every_proof_under_resources = BTreeSet::new();
        if flags.assume_all_signature_proofs {
            simulate_every_proof_under_resources.insert(SECP256K1_SIGNATURE_RESOURCE);
            simulate_every_proof_under_resources.insert(ED25519_SIGNATURE_RESOURCE);
        }

        let header = &intent.header.inner;
        let fee_payment = TransactionCostingParameters {
            tip: TipSpecifier::Percentage(header.tip_percentage),
            free_credit_in_xrd: if flags.use_free_credit {
                Decimal::try_from(PREVIEW_CREDIT_IN_XRD).unwrap()
            } else {
                Decimal::ZERO
            },
        };

        let mut initial_proofs = AuthAddresses::signer_set(&self.signer_public_keys);
        if header.notary_is_signatory {
            initial_proofs.insert(NonFungibleGlobalId::from_public_key(
                &header.notary_public_key,
            ));
        }

        let intent_hash = intent.transaction_intent_hash();

        let nullification = if flags.skip_epoch_check {
            IntentHashNullification::SimulatedTransactionIntent {
                simulated: SimulatedTransactionIntentNullification,
            }
        } else {
            IntentHashNullification::TransactionIntent {
                intent_hash,
                expiry_epoch: intent.header.inner.end_epoch_exclusive,
            }
        };

        ExecutableTransaction::new_v1(
            self.encoded_instructions,
            AuthZoneInit::new(initial_proofs, simulate_every_proof_under_resources),
            intent.instructions.references,
            intent.blobs.blobs_by_hash,
            ExecutionContext {
                unique_hash: intent_hash.0,
                intent_hash_nullifications: vec![nullification],
                epoch_range: if flags.skip_epoch_check {
                    None
                } else {
                    Some(EpochRange {
                        start_epoch_inclusive: intent.header.inner.start_epoch_inclusive,
                        end_epoch_exclusive: intent.header.inner.end_epoch_exclusive,
                    })
                },
                payload_size: intent.summary.effective_length,
                num_of_signature_validations: 0, // Accounted for by tests in `common_transformation_costs.rs`.
                costing_parameters: fee_payment,
                pre_allocated_addresses: vec![],
                disable_limits_and_costing_modules: false,
                proposer_timestamp_range: None,
            },
        )
    }
}
