use crate::internal_prelude::*;
use radix_common::constants::AuthAddresses;

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
    pub encoded_instructions: Rc<Vec<u8>>,
    pub signer_public_keys: Vec<PublicKey>,
    pub flags: PreviewFlags,
}

impl ValidatedPreviewIntent {
    pub fn get_executable(&self) -> ExecutableTransactionV1 {
        let intent = &self.intent;
        let flags = &self.flags;

        let mut simulate_every_proof_under_resources = BTreeSet::new();
        if self.flags.assume_all_signature_proofs {
            simulate_every_proof_under_resources.insert(SECP256K1_SIGNATURE_RESOURCE);
            simulate_every_proof_under_resources.insert(ED25519_SIGNATURE_RESOURCE);
        }

        let header = &intent.header.inner;
        let fee_payment = TransactionCostingParameters {
            tip: TipSpecifier::Percentage(header.tip_percentage),
            free_credit_in_xrd: if self.flags.use_free_credit {
                Decimal::try_from(PREVIEW_CREDIT_IN_XRD).unwrap()
            } else {
                Decimal::ZERO
            },
            abort_when_loan_repaid: false,
        };

        let mut initial_proofs = AuthAddresses::signer_set(&self.signer_public_keys);
        if header.notary_is_signatory {
            initial_proofs.insert(NonFungibleGlobalId::from_public_key(
                &header.notary_public_key,
            ));
        }

        let intent_hash = intent.transaction_intent_hash();

        ExecutableTransactionV1::new(
            self.encoded_instructions.clone(),
            intent.instructions.references.clone(),
            intent.blobs.blobs_by_hash.clone(),
            ExecutionContext {
                unique_hash: intent_hash.0,
                intent_hash_nullification: IntentHashNullification::TransactionIntent {
                    intent_hash,
                    expiry_epoch: intent.header.inner.end_epoch_exclusive,
                    ignore_duplicate: flags.skip_epoch_check,
                },
                epoch_range: if flags.skip_epoch_check {
                    None
                } else {
                    Some(EpochRange {
                        start_epoch_inclusive: intent.header.inner.start_epoch_inclusive,
                        end_epoch_exclusive: intent.header.inner.end_epoch_exclusive,
                    })
                },
                payload_size: self.intent.summary.effective_length,
                num_of_signature_validations: 0, // Accounted for by tests in `common_transformation_costs.rs`.
                auth_zone_init: AuthZoneInit::new(
                    initial_proofs,
                    simulate_every_proof_under_resources,
                ),
                costing_parameters: fee_payment,
                pre_allocated_addresses: vec![],
            },
            false,
        )
    }
}
