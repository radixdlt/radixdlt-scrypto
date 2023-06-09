use crate::internal_prelude::*;
use crate::validation::*;

pub trait TransactionValidator<Prepared: TransactionPayloadPreparable> {
    type Validated;

    fn prepare_from_raw(
        &self,
        raw: &Prepared::Raw,
    ) -> Result<Prepared, TransactionValidationError> {
        self.prepare_from_payload_bytes(raw.as_slice())
    }

    fn prepare_from_payload_bytes(
        &self,
        raw_payload_bytes: &[u8],
    ) -> Result<Prepared, TransactionValidationError> {
        if raw_payload_bytes.len() > self.max_payload_length() {
            return Err(TransactionValidationError::TransactionTooLarge);
        }

        Ok(Prepared::prepare_from_payload(raw_payload_bytes)?)
    }

    fn validate_from_raw(
        &self,
        raw: &Prepared::Raw,
    ) -> Result<Self::Validated, TransactionValidationError> {
        self.validate_from_payload_bytes(raw.as_slice())
    }

    fn validate_from_payload_bytes(
        &self,
        payload_bytes: &[u8],
    ) -> Result<Self::Validated, TransactionValidationError> {
        let prepared = self.prepare_from_payload_bytes(payload_bytes)?;
        self.validate(prepared)
    }

    fn max_payload_length(&self) -> usize;

    fn validate(
        &self,
        transaction: Prepared,
    ) -> Result<Self::Validated, TransactionValidationError>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ValidationConfig {
    pub network_id: u8,
    pub max_notarized_payload_size: usize,
    pub min_cost_unit_limit: u32,
    pub max_cost_unit_limit: u32,
    pub min_tip_percentage: u16,
    pub max_tip_percentage: u16,
    pub max_epoch_range: u64,
}

impl ValidationConfig {
    pub fn default(network_id: u8) -> Self {
        Self {
            network_id,
            max_notarized_payload_size: DEFAULT_MAX_TRANSACTION_SIZE,
            min_cost_unit_limit: DEFAULT_MIN_COST_UNIT_LIMIT,
            max_cost_unit_limit: DEFAULT_MAX_COST_UNIT_LIMIT,
            min_tip_percentage: DEFAULT_MIN_TIP_PERCENTAGE,
            max_tip_percentage: DEFAULT_MAX_TIP_PERCENTAGE,
            max_epoch_range: DEFAULT_MAX_EPOCH_RANGE,
        }
    }

    pub fn simulator() -> Self {
        Self::default(NetworkDefinition::simulator().id)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NotarizedTransactionValidator {
    config: ValidationConfig,
}

impl TransactionValidator<PreparedNotarizedTransactionV1> for NotarizedTransactionValidator {
    type Validated = ValidatedNotarizedTransactionV1;

    fn max_payload_length(&self) -> usize {
        self.config.max_notarized_payload_size
    }

    fn validate(
        &self,
        transaction: PreparedNotarizedTransactionV1,
    ) -> Result<Self::Validated, TransactionValidationError> {
        self.validate_intent_v1(&transaction.signed_intent.intent)?;

        let encoded_instructions =
            manifest_encode(&transaction.signed_intent.intent.instructions.inner.0)?;

        let signer_keys = self
            .validate_signatures_v1(&transaction)
            .map_err(TransactionValidationError::SignatureValidationError)?;

        Ok(ValidatedNotarizedTransactionV1 {
            prepared: transaction,
            encoded_instructions,
            signer_keys,
        })
    }
}

impl NotarizedTransactionValidator {
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    pub fn validate_preview_intent_v1(
        &self,
        preview_intent: PreviewIntentV1,
    ) -> Result<ValidatedPreviewIntent, TransactionValidationError> {
        let intent = preview_intent.intent.prepare()?;

        self.validate_intent_v1(&intent)?;

        let encoded_instructions = manifest_encode(&intent.instructions.inner.0)?;

        Ok(ValidatedPreviewIntent {
            intent,
            encoded_instructions,
            signer_public_keys: preview_intent.signer_public_keys,
            flags: preview_intent.flags,
        })
    }

    pub fn validate_intent_v1(
        &self,
        intent: &PreparedIntentV1,
    ) -> Result<(), TransactionValidationError> {
        self.validate_header_v1(&intent.header.inner)
            .map_err(TransactionValidationError::HeaderValidationError)?;

        Self::validate_instructions_v1(&intent.instructions.inner.0)?;

        return Ok(());
    }

    pub fn validate_instructions_v1(
        instructions: &[InstructionV1],
    ) -> Result<(), TransactionValidationError> {
        // semantic analysis
        let mut id_validator = ManifestValidator::new();
        for inst in instructions {
            match inst {
                InstructionV1::TakeAllFromWorktop { .. } => {
                    id_validator.new_bucket();
                }
                InstructionV1::TakeFromWorktop { .. } => {
                    id_validator.new_bucket();
                }
                InstructionV1::TakeNonFungiblesFromWorktop { .. } => {
                    id_validator.new_bucket();
                }
                InstructionV1::ReturnToWorktop { bucket_id } => {
                    id_validator
                        .drop_bucket(&bucket_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::AssertWorktopContains { .. } => {}
                InstructionV1::AssertWorktopContainsNonFungibles { .. } => {}
                InstructionV1::PopFromAuthZone => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::PushToAuthZone { proof_id } => {
                    id_validator
                        .drop_proof(&proof_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::ClearAuthZone => {}
                InstructionV1::CreateProofFromAuthZone { .. } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::CreateProofFromAuthZoneOfAmount { .. } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::CreateProofFromAuthZoneOfNonFungibles { .. } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::CreateProofFromAuthZoneOfAll { .. } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::CreateProofFromBucket { bucket_id } => {
                    id_validator
                        .new_proof(ProofKind::BucketProof(bucket_id.clone()))
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::CreateProofFromBucketOfAmount { bucket_id, .. } => {
                    id_validator
                        .new_proof(ProofKind::BucketProof(bucket_id.clone()))
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::CreateProofFromBucketOfNonFungibles { bucket_id, .. } => {
                    id_validator
                        .new_proof(ProofKind::BucketProof(bucket_id.clone()))
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::CreateProofFromBucketOfAll { bucket_id, .. } => {
                    id_validator
                        .new_proof(ProofKind::BucketProof(bucket_id.clone()))
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::CloneProof { proof_id } => {
                    id_validator
                        .clone_proof(&proof_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::DropProof { proof_id } => {
                    id_validator
                        .drop_proof(&proof_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::DropAllProofs => {
                    id_validator
                        .drop_all_proofs()
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::ClearSignatureProofs => {}
                InstructionV1::CallFunction { args, .. }
                | InstructionV1::CallMethod { args, .. }
                | InstructionV1::CallRoyaltyMethod { args, .. }
                | InstructionV1::CallMetadataMethod { args, .. }
                | InstructionV1::CallAccessRulesMethod { args, .. } => {
                    // TODO: decode into Value
                    Self::validate_call_args(&args, &mut id_validator)
                        .map_err(TransactionValidationError::CallDataValidationError)?;
                }
                InstructionV1::BurnResource { bucket_id } => {
                    id_validator
                        .drop_bucket(&bucket_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                InstructionV1::CallDirectVaultMethod { .. } => {}
                InstructionV1::AllocateGlobalAddress { .. } => {
                    id_validator.new_reservation();
                    id_validator.new_allocated_address();
                }
            }
        }

        Ok(())
    }

    pub fn validate_header_v1(
        &self,
        header: &TransactionHeaderV1,
    ) -> Result<(), HeaderValidationError> {
        // network
        if header.network_id != self.config.network_id {
            return Err(HeaderValidationError::InvalidNetwork);
        }

        // epoch
        if header.end_epoch_exclusive <= header.start_epoch_inclusive {
            return Err(HeaderValidationError::InvalidEpochRange);
        }
        let max_end_epoch = header
            .start_epoch_inclusive
            .after(self.config.max_epoch_range);
        if header.end_epoch_exclusive > max_end_epoch {
            return Err(HeaderValidationError::EpochRangeTooLarge);
        }

        // tip percentage
        if header.tip_percentage < self.config.min_tip_percentage
            || header.tip_percentage > self.config.max_tip_percentage
        {
            return Err(HeaderValidationError::InvalidTipPercentage);
        }

        Ok(())
    }

    pub fn validate_signatures_v1(
        &self,
        transaction: &PreparedNotarizedTransactionV1,
    ) -> Result<Vec<PublicKey>, SignatureValidationError> {
        // TODO: split into static validation part and runtime validation part to support more signatures
        if transaction
            .signed_intent
            .intent_signatures
            .inner
            .signatures
            .len()
            > MAX_NUMBER_OF_INTENT_SIGNATURES
        {
            return Err(SignatureValidationError::TooManySignatures);
        }

        // verify intent signature
        let mut signers = index_set_new();
        let intent_hash = transaction.intent_hash().into_hash();
        for intent_signature in &transaction.signed_intent.intent_signatures.inner.signatures {
            let public_key = recover(&intent_hash, &intent_signature.0)
                .ok_or(SignatureValidationError::InvalidIntentSignature)?;

            if !verify(&intent_hash, &public_key, &intent_signature.0.signature()) {
                return Err(SignatureValidationError::InvalidIntentSignature);
            }

            if !signers.insert(public_key) {
                return Err(SignatureValidationError::DuplicateSigner);
            }
        }

        let header = &transaction.signed_intent.intent.header.inner;

        if header.notary_is_signatory {
            signers.insert(header.notary_public_key);
        }

        // verify notary signature
        let signed_intent_hash = transaction.signed_intent_hash().into_hash();
        if !verify(
            &signed_intent_hash,
            &header.notary_public_key,
            &transaction.notary_signature.inner.0,
        ) {
            return Err(SignatureValidationError::InvalidNotarySignature);
        }

        Ok(signers.into_iter().collect())
    }

    pub fn validate_call_args(
        value: &ManifestValue,
        id_validator: &mut ManifestValidator,
    ) -> Result<(), CallDataValidationError> {
        id_validator
            .process_call_data(&value)
            .map_err(CallDataValidationError::IdValidationError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use radix_engine_interface::network::NetworkDefinition;

    use super::*;
    use crate::{
        builder::ManifestBuilder, builder::TransactionBuilder,
        ecdsa_secp256k1::EcdsaSecp256k1PrivateKey,
    };

    macro_rules! assert_invalid_tx {
        ($result: expr, ($start_epoch: expr, $end_epoch: expr, $nonce: expr, $signers: expr, $notary: expr)) => {{
            let config: ValidationConfig = ValidationConfig::simulator();
            let validator = NotarizedTransactionValidator::new(config);
            assert_eq!(
                $result,
                validator
                    .validate(
                        create_transaction($start_epoch, $end_epoch, $nonce, $signers, $notary)
                            .prepare()
                            .unwrap()
                    )
                    .expect_err("Should be an error")
            );
        }};
    }

    #[test]
    fn test_invalid_header() {
        assert_invalid_tx!(
            TransactionValidationError::HeaderValidationError(
                HeaderValidationError::InvalidEpochRange
            ),
            (Epoch::zero(), Epoch::zero(), 5, vec![1], 2)
        );
        assert_invalid_tx!(
            TransactionValidationError::HeaderValidationError(
                HeaderValidationError::EpochRangeTooLarge
            ),
            (Epoch::zero(), Epoch::of(1000), 5, vec![1], 2)
        );
    }

    #[test]
    fn test_invalid_signatures() {
        assert_invalid_tx!(
            TransactionValidationError::SignatureValidationError(
                SignatureValidationError::TooManySignatures
            ),
            (Epoch::zero(), Epoch::of(100), 5, (1..20).collect(), 2)
        );
        assert_invalid_tx!(
            TransactionValidationError::SignatureValidationError(
                SignatureValidationError::DuplicateSigner
            ),
            (Epoch::zero(), Epoch::of(100), 5, vec![1, 1], 2)
        );
    }

    #[test]
    fn test_valid_preview() {
        // Build the whole transaction but only really care about the intent
        let tx = create_transaction(Epoch::zero(), Epoch::of(100), 5, vec![1, 2], 2);

        let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());

        let preview_intent = PreviewIntentV1 {
            intent: tx.signed_intent.intent,
            signer_public_keys: Vec::new(),
            flags: PreviewFlags {
                use_free_credit: true,
                assume_all_signature_proofs: false,
                permit_invalid_header_epoch: false,
                permit_duplicate_intent_hash: false,
            },
        };

        let result = validator.validate_preview_intent_v1(preview_intent);

        assert!(result.is_ok());
    }

    fn create_transaction(
        start_epoch: Epoch,
        end_epoch: Epoch,
        nonce: u32,
        signers: Vec<u64>,
        notary: u64,
    ) -> NotarizedTransactionV1 {
        let sk_notary = EcdsaSecp256k1PrivateKey::from_u64(notary).unwrap();

        let mut builder = TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: start_epoch,
                end_epoch_exclusive: end_epoch,
                nonce,
                notary_public_key: sk_notary.public_key().into(),
                notary_is_signatory: false,
                tip_percentage: 5,
            })
            .manifest(ManifestBuilder::new().clear_auth_zone().build());

        for signer in signers {
            builder = builder.sign(&EcdsaSecp256k1PrivateKey::from_u64(signer).unwrap());
        }
        builder = builder.notarize(&sk_notary);

        builder.build()
    }
}
