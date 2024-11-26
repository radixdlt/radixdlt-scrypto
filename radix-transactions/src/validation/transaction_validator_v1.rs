use crate::internal_prelude::*;

impl TransactionValidator {
    #[allow(deprecated)]
    pub fn validate_notarized_v1(
        &self,
        transaction: PreparedNotarizedTransactionV1,
    ) -> Result<ValidatedNotarizedTransactionV1, TransactionValidationError> {
        let transaction_intent = &transaction.signed_intent.intent;

        let signatures = AllPendingSignatureValidations::new_with_root(
            TransactionVersion::V1,
            &self.config,
            transaction_intent.transaction_intent_hash().into(),
            PendingIntentSignatureValidations::TransactionIntent {
                notary_is_signatory: transaction_intent.header.inner.notary_is_signatory,
                notary_public_key: transaction_intent.header.inner.notary_public_key,
                notary_signature: transaction.notary_signature.inner.0,
                notarized_hash: transaction.signed_transaction_intent_hash(),
                intent_signatures: transaction
                    .signed_intent
                    .intent_signatures
                    .inner
                    .signatures
                    .as_slice(),
                signed_hash: transaction_intent.transaction_intent_hash(),
            },
        )?;

        let aggregation = self
            .validate_intent_v1(&transaction.signed_intent.intent)
            .map_err(|err| {
                TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::RootTransactionIntent(
                        transaction_intent.transaction_intent_hash(),
                    ),
                    err,
                )
            })?;
        let _ = aggregation.finalize(&self.config)?; // Don't use the overall validity range in V1

        let encoded_instructions =
            manifest_encode(&transaction.signed_intent.intent.instructions.inner.0)?;

        let SignatureValidationSummary {
            root_signer_keys: signer_keys,
            non_root_signer_keys: _, // Not used in V1
            total_signature_validations: num_of_signature_validations,
        } = signatures.validate_all()?;

        Ok(ValidatedNotarizedTransactionV1 {
            prepared: transaction,
            encoded_instructions,
            signer_keys,
            num_of_signature_validations,
        })
    }

    #[allow(deprecated)]
    pub fn validate_preview_intent_v1(
        &self,
        preview_intent: PreviewIntentV1,
    ) -> Result<ValidatedPreviewIntent, TransactionValidationError> {
        let fake_intent_hash = SimulatedTransactionIntentNullification.transaction_intent_hash();
        let intent = preview_intent.intent.prepare(self.preparation_settings())?;

        let aggregation = self.validate_intent_v1(&intent).map_err(|err| {
            TransactionValidationError::IntentValidationError(
                TransactionValidationErrorLocation::RootTransactionIntent(fake_intent_hash),
                err,
            )
        })?;
        aggregation.finalize(&self.config)?;

        let encoded_instructions = manifest_encode(&intent.instructions.inner.0)?;

        Ok(ValidatedPreviewIntent {
            intent,
            encoded_instructions,
            signer_public_keys: preview_intent.signer_public_keys,
            flags: preview_intent.flags,
        })
    }

    // This method is public so it can be used by the toolkit.
    #[allow(deprecated)]
    pub fn validate_intent_v1(
        &self,
        intent: &PreparedIntentV1,
    ) -> Result<AcrossIntentAggregation, IntentValidationError> {
        let mut aggregation = AcrossIntentAggregation::start();
        self.validate_header_v1(&intent.header.inner)?;
        self.validate_message_v1(&intent.message.inner)?;
        aggregation.record_reference_count(intent.instructions.references.len(), &self.config)?;
        self.validate_instructions_v1(&intent.instructions.inner.0, &intent.blobs.blobs_by_hash)?;

        Ok(aggregation)
    }

    pub fn validate_instructions_v1(
        &self,
        instructions: &[InstructionV1],
        blobs: &IndexMap<Hash, Vec<u8>>,
    ) -> Result<(), IntentValidationError> {
        if instructions.len() > self.config.max_instructions {
            return Err(ManifestValidationError::TooManyInstructions.into());
        }

        match self.config.manifest_validation {
            ManifestValidationRuleset::BabylonBasicValidator => self
                .validate_instructions_basic_v1(instructions)
                .map_err(|err| err.into()),
            ManifestValidationRuleset::Interpreter(specifier) => StaticManifestInterpreter::new(
                ValidationRuleset::for_specifier(specifier),
                &EphemeralManifest::new_childless_transaction_manifest(instructions, blobs),
            )
            .validate()
            .map_err(|err| err.into()),
        }
    }

    pub fn validate_instructions_basic_v1(
        &self,
        instructions: &[InstructionV1],
    ) -> Result<(), ManifestBasicValidatorError> {
        let mut id_validator = BasicManifestValidator::new();
        for instruction in instructions {
            match instruction.effect() {
                ManifestInstructionEffect::CreateBucket { .. } => {
                    let _ = id_validator.new_bucket();
                }
                ManifestInstructionEffect::CreateProof { source_amount, .. } => {
                    let _ = id_validator.new_proof(source_amount.proof_kind())?;
                }
                ManifestInstructionEffect::ConsumeBucket {
                    consumed_bucket: bucket,
                    ..
                } => {
                    id_validator.drop_bucket(&bucket)?;
                }
                ManifestInstructionEffect::ConsumeProof {
                    consumed_proof: proof,
                    ..
                } => {
                    id_validator.drop_proof(&proof)?;
                }
                ManifestInstructionEffect::CloneProof { cloned_proof, .. } => {
                    let _ = id_validator.clone_proof(&cloned_proof)?;
                }
                ManifestInstructionEffect::DropManyProofs {
                    drop_all_named_proofs,
                    ..
                } => {
                    if drop_all_named_proofs {
                        id_validator.drop_all_named_proofs()?;
                    }
                }
                ManifestInstructionEffect::Invocation { args, .. } => {
                    id_validator.process_call_data(args)?;
                }
                ManifestInstructionEffect::CreateAddressAndReservation { .. } => {
                    let _ = id_validator.new_address_reservation();
                    id_validator.new_named_address();
                }
                ManifestInstructionEffect::ResourceAssertion { .. } => {}
                ManifestInstructionEffect::Verification { .. } => {
                    unreachable!("No InstructionV1 returns this effect");
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
        if let Some(required_network_id) = self.required_network_id {
            if header.network_id != required_network_id {
                return Err(HeaderValidationError::InvalidNetwork);
            }
        }

        // epoch
        if header.end_epoch_exclusive <= header.start_epoch_inclusive {
            return Err(HeaderValidationError::InvalidEpochRange);
        }
        let max_end_epoch = header
            .start_epoch_inclusive
            .after(self.config.max_epoch_range)
            .ok_or(HeaderValidationError::InvalidEpochRange)?;
        if header.end_epoch_exclusive > max_end_epoch {
            return Err(HeaderValidationError::InvalidEpochRange);
        }

        // tip percentage
        if header.tip_percentage < self.config.min_tip_percentage
            || header.tip_percentage > self.config.max_tip_percentage
        {
            return Err(HeaderValidationError::InvalidTip);
        }

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
    use crate::internal_prelude::*;

    macro_rules! assert_invalid_tx {
        ($result: pat, ($start_epoch: expr, $end_epoch: expr, $nonce: expr, $signers: expr, $notary: expr)) => {{
            let validator = TransactionValidator::new_for_latest_simulator();
            assert_matches!(
                create_transaction($start_epoch, $end_epoch, $nonce, $signers, $notary)
                    .prepare_and_validate(&validator)
                    .expect_err("Should be an error"),
                $result,
            );
        }};
    }

    #[test]
    fn test_invalid_header() {
        assert_invalid_tx!(
            TransactionValidationError::IntentValidationError(
                TransactionValidationErrorLocation::RootTransactionIntent(_),
                IntentValidationError::HeaderValidationError(
                    HeaderValidationError::InvalidEpochRange
                ),
            ),
            (Epoch::zero(), Epoch::zero(), 5, vec![1], 2)
        );
        assert_invalid_tx!(
            TransactionValidationError::IntentValidationError(
                TransactionValidationErrorLocation::RootTransactionIntent(_),
                IntentValidationError::HeaderValidationError(
                    HeaderValidationError::InvalidEpochRange
                ),
            ),
            (
                Epoch::zero(),
                Epoch::of(TransactionValidationConfig::latest().max_epoch_range + 1),
                5,
                vec![1],
                2
            )
        );
    }

    #[test]
    fn test_epoch_overflow() {
        assert_invalid_tx!(
            TransactionValidationError::IntentValidationError(
                TransactionValidationErrorLocation::RootTransactionIntent(_),
                IntentValidationError::HeaderValidationError(
                    HeaderValidationError::InvalidEpochRange
                ),
            ),
            (Epoch::of(u64::MAX - 5), Epoch::of(u64::MAX), 5, vec![1], 2)
        );
    }

    #[test]
    fn test_too_many_signatures() {
        assert_invalid_tx!(
            TransactionValidationError::SignatureValidationError(
                TransactionValidationErrorLocation::RootTransactionIntent(_),
                SignatureValidationError::TooManySignatures {
                    total: 19,
                    limit: 16,
                }
            ),
            (Epoch::zero(), Epoch::of(100), 5, (1..20).collect(), 2)
        );
    }

    #[test]
    fn test_duplicate_signers() {
        assert_invalid_tx!(
            TransactionValidationError::SignatureValidationError(
                TransactionValidationErrorLocation::RootTransactionIntent(_),
                SignatureValidationError::DuplicateSigner
            ),
            (Epoch::zero(), Epoch::of(100), 5, vec![1, 1], 2)
        );
    }

    #[test]
    fn test_valid_preview() {
        // Build the whole transaction but only really care about the intent
        let tx = create_transaction(Epoch::zero(), Epoch::of(100), 5, vec![1, 2], 2);

        let validator = TransactionValidator::new_for_latest_simulator();

        let preview_intent = PreviewIntentV1 {
            intent: tx.signed_intent.intent,
            signer_public_keys: Vec::new(),
            flags: PreviewFlags {
                use_free_credit: true,
                assume_all_signature_proofs: false,
                skip_epoch_check: false,
                disable_auth: false,
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
            assert_matches!(error, InvalidMessageError::MimeTypeTooLong { .. })
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
            assert_matches!(error, InvalidMessageError::PlaintextMessageTooLong { .. })
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
            assert_matches!(error, InvalidMessageError::EncryptedMessageTooLong { .. })
        }

        // NoDecryptors
        {
            let message = MessageV1::Encrypted(EncryptedMessageV1 {
                encrypted: AesGcmPayload(vec![]),
                decryptors_by_curve: indexmap!(),
            });
            let error =
                validate_default_expecting_message_error(&create_transaction_with_message(message));
            assert_matches!(error, InvalidMessageError::NoDecryptors)
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
            assert_matches!(
                error,
                InvalidMessageError::NoDecryptorsForCurveType {
                    curve_type: CurveType::Ed25519
                }
            )
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
            assert_matches!(
                error,
                InvalidMessageError::MismatchingDecryptorCurves {
                    actual: CurveType::Secp256k1,
                    expected: CurveType::Ed25519
                }
            )
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
            assert_matches!(
                error,
                InvalidMessageError::TooManyDecryptors {
                    actual: 30,
                    permitted: 20
                }
            )
        }
    }

    fn validate_default_expecting_message_error(
        transaction: &NotarizedTransactionV1,
    ) -> InvalidMessageError {
        match validate_default(transaction).expect_err("Expected validation error") {
            TransactionValidationError::IntentValidationError(
                _,
                IntentValidationError::InvalidMessage(error),
            ) => error,
            error => {
                panic!("Expected InvalidMessage error, got: {:?}", error)
            }
        }
    }

    fn validate_default(
        transaction: &NotarizedTransactionV1,
    ) -> Result<(), TransactionValidationError> {
        let validator = TransactionValidator::new_for_latest_simulator();
        transaction.prepare_and_validate(&validator).map(|_| ())
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
            .manifest(ManifestBuilder::new().drop_auth_zone_proofs().build())
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
        create_transaction_advanced(
            start_epoch,
            end_epoch,
            nonce,
            signers,
            notary,
            ManifestBuilder::new().drop_auth_zone_proofs().build(),
        )
    }

    fn create_transaction_advanced(
        start_epoch: Epoch,
        end_epoch: Epoch,
        nonce: u32,
        signers: Vec<u64>,
        notary: u64,
        manifest: TransactionManifestV1,
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
            .manifest(manifest);

        for signer in signers {
            builder = builder.sign(&Secp256k1PrivateKey::from_u64(signer).unwrap());
        }
        builder = builder.notarize(&sk_notary);

        builder.build()
    }

    #[test]
    fn test_drop_bucket_before_proof() {
        let transaction = create_transaction_advanced(
            Epoch::of(0),
            Epoch::of(40),
            123,
            vec![55],
            66,
            ManifestBuilder::new()
                .take_from_worktop(XRD, dec!(100), "bucket")
                .create_proof_from_bucket_of_amount("bucket", dec!(5), "proof1")
                .return_to_worktop("bucket")
                .drop_proof("proof1")
                .build_no_validate(),
        );
        let validator = TransactionValidator::new_for_latest_simulator();
        assert_matches!(
            transaction.prepare_and_validate(&validator),
            Err(TransactionValidationError::IntentValidationError(
                _,
                IntentValidationError::ManifestValidationError(
                    ManifestValidationError::BucketConsumedWhilstLockedByProof(
                        ManifestBucket(0),
                        _,
                    )
                )
            ))
        );
    }

    #[test]
    fn test_clone_invalid_proof() {
        let transaction = create_transaction_advanced(
            Epoch::of(0),
            Epoch::of(40),
            123,
            vec![55],
            66,
            ManifestBuilder::new()
                .take_from_worktop(XRD, dec!(100), "bucket")
                .create_proof_from_bucket_of_amount("bucket", dec!(5), "proof1")
                .then(|builder| {
                    let lookup = builder.name_lookup();
                    let proof_id = lookup.proof("proof1");

                    builder
                        .drop_proof("proof1")
                        .return_to_worktop("bucket")
                        .add_raw_instruction_ignoring_all_side_effects(CloneProof { proof_id })
                })
                .build_no_validate(),
        );
        let validator = TransactionValidator::new_for_latest_simulator();
        assert_matches!(
            transaction.prepare_and_validate(&validator),
            Err(TransactionValidationError::IntentValidationError(
                _,
                IntentValidationError::ManifestValidationError(
                    ManifestValidationError::ProofAlreadyUsed(ManifestProof(0), _,)
                )
            ))
        );
    }

    #[test]
    fn verify_call_direct_method_args_are_processed() {
        let transaction = create_transaction_advanced(
            Epoch::of(0),
            Epoch::of(40),
            123,
            vec![55],
            66,
            ManifestBuilder::new()
                .take_from_worktop(XRD, dec!(100), "bucket")
                .then(|builder| {
                    let lookup = builder.name_lookup();
                    builder
                        .call_direct_access_method(
                            InternalAddress::new_or_panic(
                                [EntityType::InternalFungibleVault as u8; NodeId::LENGTH],
                            ),
                            "test",
                            manifest_args!(lookup.bucket("bucket")),
                        )
                        .return_to_worktop("bucket")
                })
                .build_no_validate(),
        );
        let validator = TransactionValidator::new_for_latest_simulator();
        assert_matches!(
            transaction.prepare_and_validate(&validator),
            Err(TransactionValidationError::IntentValidationError(
                _,
                IntentValidationError::ManifestValidationError(
                    ManifestValidationError::BucketAlreadyUsed(ManifestBucket(0), _,)
                )
            ))
        );
    }
}
