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
    pub message_validation: MessageValidationConfig,
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
            message_validation: MessageValidationConfig::default(),
        }
    }

    pub fn simulator() -> Self {
        Self::default(NetworkDefinition::simulator().id)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MessageValidationConfig {
    pub max_plaintext_message_length: usize,
    pub max_encrypted_message_length: usize,
    pub max_mime_type_length: usize,
    pub max_decryptors: usize,
}

impl Default for MessageValidationConfig {
    fn default() -> Self {
        Self {
            max_plaintext_message_length: 2048,
            max_mime_type_length: 128,
            max_encrypted_message_length: 2048 + 12 + 16, // Account for IV and MAC - see AesGcmPayload
            max_decryptors: 20,
        }
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

        self.validate_message_v1(&intent.message.inner)?;

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
                InstructionV1::AssertWorktopContainsAny { .. } => {}
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
                    id_validator.new_address_reservation();
                    id_validator.new_named_address();
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

    pub fn validate_message_v1(&self, message: &MessageV1) -> Result<(), InvalidMessageError> {
        let validation = &self.config.message_validation;
        match message {
            MessageV1::None => {}
            MessageV1::Plaintext(plaintext_message) => {
                let PlaintextMessageV1 { mime_type, message } = plaintext_message;
                if mime_type.len() > validation.max_mime_type_length {
                    return Err(InvalidMessageError::MimeTypeTooLong {
                        actual: mime_type.len(),
                        permitted: validation.max_mime_type_length,
                    });
                }
                if message.len() > validation.max_plaintext_message_length {
                    return Err(InvalidMessageError::PlaintextMessageTooLong {
                        actual: message.len(),
                        permitted: validation.max_plaintext_message_length,
                    });
                }
            }
            MessageV1::Encrypted(encrypted_message) => {
                let EncryptedMessageV1 {
                    encrypted,
                    decryptors_by_curve,
                } = encrypted_message;
                if encrypted.0.len() > validation.max_encrypted_message_length {
                    return Err(InvalidMessageError::EncryptedMessageTooLong {
                        actual: encrypted.0.len(),
                        permitted: validation.max_encrypted_message_length,
                    });
                }
                if decryptors_by_curve.len() == 0 {
                    return Err(InvalidMessageError::NoDecryptors);
                }
                let mut total_decryptors = 0;
                for (curve_type, decryptors) in decryptors_by_curve.iter() {
                    if decryptors.curve_type() != *curve_type {
                        return Err(InvalidMessageError::MismatchingDecryptorCurves {
                            actual: decryptors.curve_type(),
                            expected: *curve_type,
                        });
                    }
                    if decryptors.number_of_decryptors() == 0 {
                        return Err(InvalidMessageError::NoDecryptorsForCurveType {
                            curve_type: decryptors.curve_type(),
                        });
                    }
                    // Can't overflow because decryptor count << size of a transaction < 1MB < usize,
                    total_decryptors += decryptors.number_of_decryptors();
                }
                if total_decryptors > validation.max_decryptors {
                    return Err(InvalidMessageError::TooManyDecryptors {
                        actual: total_decryptors,
                        permitted: validation.max_decryptors,
                    });
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use radix_engine_interface::network::NetworkDefinition;

    use super::*;
    use crate::{
        builder::ManifestBuilder, builder::TransactionBuilder,
        signing::secp256k1::Secp256k1PrivateKey,
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
            (
                Epoch::zero(),
                Epoch::of(DEFAULT_MAX_EPOCH_RANGE + 1),
                5,
                vec![1],
                2
            )
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
                skip_epoch_check: false,
            },
        };

        let result = validator.validate_preview_intent_v1(preview_intent);

        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_messages() {
        // None
        {
            let message = MessageV1::None;
            let result = validate_default(&create_transaction_with_message(message));
            assert!(result.is_ok());
        }
        // Plaintext
        {
            let message = MessageV1::Plaintext(PlaintextMessageV1 {
                mime_type: "text/plain".to_owned(),
                message: MessageContentsV1::String("Hello world!".to_string()),
            });
            let result = validate_default(&create_transaction_with_message(message));
            assert!(result.is_ok());
        }
        // Encrypted
        {
            // Note - this isn't actually a validly encrypted message,
            // this just shows that a sufficiently valid encrypted message can pass validation
            let message = MessageV1::Encrypted(EncryptedMessageV1 {
                encrypted: AesGcmPayload(vec![]),
                decryptors_by_curve: indexmap!(
                    CurveType::Ed25519 => DecryptorsByCurve::Ed25519 {
                        dh_ephemeral_public_key: Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]),
                        decryptors: indexmap!(
                            PublicKeyFingerprint([0; PublicKeyFingerprint::LENGTH]) => AesWrapped128BitKey([0; AesWrapped128BitKey::LENGTH]),
                        ),
                    },
                    CurveType::Secp256k1 => DecryptorsByCurve::Secp256k1 {
                        dh_ephemeral_public_key: Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH]),
                        decryptors: indexmap!(
                            PublicKeyFingerprint([0; PublicKeyFingerprint::LENGTH]) => AesWrapped128BitKey([0; AesWrapped128BitKey::LENGTH]),
                            PublicKeyFingerprint([1; PublicKeyFingerprint::LENGTH]) => AesWrapped128BitKey([0; AesWrapped128BitKey::LENGTH]),
                        ),
                    },
                ),
            });
            let result = validate_default(&create_transaction_with_message(message));
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_invalid_message_errors() {
        // MimeTypeTooLong
        {
            let message = MessageV1::Plaintext(PlaintextMessageV1 {
                mime_type: "very long mimetype, very long mimetype, very long mimetype, very long mimetype, very long mimetype, very long mimetype, very long mimetype, very long mimetype, ".to_owned(),
                message: MessageContentsV1::String("Hello".to_string()),
            });
            let error =
                validate_default_expecting_message_error(&create_transaction_with_message(message));
            assert!(matches!(error, InvalidMessageError::MimeTypeTooLong { .. }))
        }

        // PlaintextMessageTooLong
        {
            let mut long_message: String = "".to_owned();
            while long_message.len() <= 2048 {
                long_message.push_str("more text please!");
            }
            let message = MessageV1::Plaintext(PlaintextMessageV1 {
                mime_type: "text/plain".to_owned(),
                message: MessageContentsV1::String(long_message),
            });
            let error =
                validate_default_expecting_message_error(&create_transaction_with_message(message));
            assert!(matches!(
                error,
                InvalidMessageError::PlaintextMessageTooLong { .. }
            ))
        }

        // EncryptedMessageTooLong
        {
            let mut message_which_is_too_long: String = "".to_owned();
            while message_which_is_too_long.len() <= 2048 + 50 {
                // Some more bytes for the AES padding
                message_which_is_too_long.push_str("more text please!");
            }
            let message = MessageV1::Encrypted(EncryptedMessageV1 {
                encrypted: AesGcmPayload(message_which_is_too_long.as_bytes().to_vec()),
                decryptors_by_curve: indexmap!(
                    CurveType::Ed25519 => DecryptorsByCurve::Ed25519 {
                        dh_ephemeral_public_key: Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]),
                        decryptors: indexmap!(
                            PublicKeyFingerprint([0; PublicKeyFingerprint::LENGTH]) => AesWrapped128BitKey([0; AesWrapped128BitKey::LENGTH]),
                        ),
                    }
                ),
            });
            let error =
                validate_default_expecting_message_error(&create_transaction_with_message(message));
            assert!(matches!(
                error,
                InvalidMessageError::EncryptedMessageTooLong { .. }
            ))
        }

        // NoDecryptors
        {
            let message = MessageV1::Encrypted(EncryptedMessageV1 {
                encrypted: AesGcmPayload(vec![]),
                decryptors_by_curve: indexmap!(),
            });
            let error =
                validate_default_expecting_message_error(&create_transaction_with_message(message));
            assert!(matches!(error, InvalidMessageError::NoDecryptors))
        }

        // NoDecryptorsForCurveType
        {
            let message = MessageV1::Encrypted(EncryptedMessageV1 {
                encrypted: AesGcmPayload(vec![]),
                decryptors_by_curve: indexmap!(
                    CurveType::Ed25519 => DecryptorsByCurve::Ed25519 {
                        dh_ephemeral_public_key: Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]),
                        decryptors: indexmap!(),
                    }
                ),
            });
            let error =
                validate_default_expecting_message_error(&create_transaction_with_message(message));
            assert!(matches!(
                error,
                InvalidMessageError::NoDecryptorsForCurveType {
                    curve_type: CurveType::Ed25519
                }
            ))
        }

        // MismatchingDecryptorCurves
        {
            let message = MessageV1::Encrypted(EncryptedMessageV1 {
                encrypted: AesGcmPayload(vec![]),
                decryptors_by_curve: indexmap!(
                    CurveType::Ed25519 => DecryptorsByCurve::Secp256k1 {
                        dh_ephemeral_public_key: Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH]),
                        decryptors: indexmap!(
                            PublicKeyFingerprint([0; PublicKeyFingerprint::LENGTH]) => AesWrapped128BitKey([0; AesWrapped128BitKey::LENGTH]),
                        ),
                    }
                ),
            });
            let error =
                validate_default_expecting_message_error(&create_transaction_with_message(message));
            assert!(matches!(
                error,
                InvalidMessageError::MismatchingDecryptorCurves {
                    actual: CurveType::Secp256k1,
                    expected: CurveType::Ed25519
                }
            ))
        }

        // TooManyDecryptors
        {
            let mut decryptors = IndexMap::<PublicKeyFingerprint, AesWrapped128BitKey>::default();
            for i in 0..30 {
                decryptors.insert(
                    PublicKeyFingerprint([0, 0, 0, 0, 0, 0, 0, i as u8]),
                    AesWrapped128BitKey([0; AesWrapped128BitKey::LENGTH]),
                );
            }
            let message = MessageV1::Encrypted(EncryptedMessageV1 {
                encrypted: AesGcmPayload(vec![]),
                decryptors_by_curve: indexmap!(
                    CurveType::Ed25519 => DecryptorsByCurve::Ed25519 {
                        dh_ephemeral_public_key: Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]),
                        decryptors,
                    }
                ),
            });
            let error =
                validate_default_expecting_message_error(&create_transaction_with_message(message));
            assert!(matches!(
                error,
                InvalidMessageError::TooManyDecryptors {
                    actual: 30,
                    permitted: 20
                }
            ))
        }
    }

    fn validate_default_expecting_message_error(
        transaction: &NotarizedTransactionV1,
    ) -> InvalidMessageError {
        match validate_default(transaction).expect_err("Expected validation error") {
            TransactionValidationError::InvalidMessage(error) => error,
            error => {
                panic!("Expected InvalidMessage error, got: {:?}", error)
            }
        }
    }

    fn validate_default(
        transaction: &NotarizedTransactionV1,
    ) -> Result<(), TransactionValidationError> {
        let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());
        validator
            .validate(transaction.prepare().unwrap())
            .map(|_| ())
    }

    fn create_transaction_with_message(message: MessageV1) -> NotarizedTransactionV1 {
        let sk_notary = Secp256k1PrivateKey::from_u64(1).unwrap();

        let mut builder = TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: Epoch::of(1),
                end_epoch_exclusive: Epoch::of(10),
                nonce: 0,
                notary_public_key: sk_notary.public_key().into(),
                notary_is_signatory: false,
                tip_percentage: 5,
            })
            .manifest(ManifestBuilder::new().clear_auth_zone().build())
            .message(message);

        builder = builder.notarize(&sk_notary);

        builder.build()
    }

    fn create_transaction(
        start_epoch: Epoch,
        end_epoch: Epoch,
        nonce: u32,
        signers: Vec<u64>,
        notary: u64,
    ) -> NotarizedTransactionV1 {
        let sk_notary = Secp256k1PrivateKey::from_u64(notary).unwrap();

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
            builder = builder.sign(&Secp256k1PrivateKey::from_u64(signer).unwrap());
        }
        builder = builder.notarize(&sk_notary);

        builder.build()
    }
}
