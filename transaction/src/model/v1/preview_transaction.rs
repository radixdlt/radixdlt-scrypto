use crate::internal_prelude::*;
use radix_engine_interface::{
    api::node_modules::auth::AuthAddresses, blueprints::transaction_processor::RuntimeValidation,
};

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Default)]
pub struct PreviewFlags {
    pub use_free_credit: bool,
    pub assume_all_signature_proofs: bool,
    pub permit_duplicate_intent_hash: bool,
    pub permit_invalid_header_epoch: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
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

impl ValidatedPreviewIntent {
    pub fn get_executable<'a>(&'a self) -> Executable<'a> {
        let intent = &self.intent;
        let flags = &self.flags;

        let mut virtual_resources = BTreeSet::new();
        if self.flags.assume_all_signature_proofs {
            virtual_resources.insert(ECDSA_SECP256K1_SIGNATURE_VIRTUAL_BADGE);
            virtual_resources.insert(EDDSA_ED25519_SIGNATURE_VIRTUAL_BADGE);
        }

        let header = &intent.header.inner;
        let fee_payment = FeePayment {
            tip_percentage: header.tip_percentage,
            free_credit_in_xrd: if self.flags.use_free_credit {
                DEFAULT_FREE_CREDIT_IN_XRD
            } else {
                0
            },
        };
        let initial_proofs = AuthAddresses::signer_set(&self.signer_public_keys);

        let intent_hash = intent.intent_hash();

        Executable::new(
            &self.encoded_instructions,
            &intent.instructions.references,
            &intent.blobs.blobs_by_hash,
            ExecutionContext {
                transaction_hash: intent_hash.into_hash(),
                payload_size: 0,
                auth_zone_params: AuthZoneParams {
                    initial_proofs,
                    virtual_resources,
                },
                fee_payment,
                runtime_validations: vec![
                    RuntimeValidation::IntentHashUniqueness {
                        intent_hash: intent_hash.into_hash(),
                    }
                    .with_skipped_assertion_if(flags.permit_duplicate_intent_hash),
                    RuntimeValidation::WithinEpochRange {
                        start_epoch_inclusive: header.start_epoch_inclusive,
                        end_epoch_exclusive: header.end_epoch_exclusive,
                    }
                    .with_skipped_assertion_if(flags.permit_invalid_header_epoch),
                ],
                pre_allocated_addresses: vec![],
            },
        )
    }
}
