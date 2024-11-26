use crate::internal_prelude::*;

pub fn verify_and_recover(
    signed_hash: &Hash,
    signature: &SignatureWithPublicKeyV1,
) -> Option<PublicKey> {
    match signature {
        SignatureWithPublicKeyV1::Secp256k1 { signature } => {
            verify_and_recover_secp256k1(signed_hash, signature).map(Into::into)
        }
        SignatureWithPublicKeyV1::Ed25519 {
            public_key,
            signature,
        } => {
            if verify_ed25519(&signed_hash, public_key, signature) {
                Some(public_key.clone().into())
            } else {
                None
            }
        }
    }
}

pub fn verify(signed_hash: &Hash, public_key: &PublicKey, signature: &SignatureV1) -> bool {
    match (public_key, signature) {
        (PublicKey::Secp256k1(public_key), SignatureV1::Secp256k1(signature)) => {
            verify_secp256k1(&signed_hash, public_key, signature)
        }
        (PublicKey::Ed25519(public_key), SignatureV1::Ed25519(signature)) => {
            verify_ed25519(&signed_hash, public_key, signature)
        }
        _ => false,
    }
}

pub trait SignedIntentTreeStructure {
    type IntentTree: IntentTreeStructure;
    fn root_signatures(&self) -> PendingIntentSignatureValidations;
    fn non_root_subintent_signatures(
        &self,
    ) -> impl ExactSizeIterator<Item = PendingSubintentSignatureValidations>;
    fn intent_tree(&self) -> &Self::IntentTree;
    fn transaction_version(&self) -> TransactionVersion;

    fn construct_pending_signature_validations<'a>(
        &'a self,
        config: &'a TransactionValidationConfig,
    ) -> Result<AllPendingSignatureValidations<'a>, TransactionValidationError> {
        let mut pending_signatures = AllPendingSignatureValidations::new_with_root(
            self.transaction_version(),
            config,
            self.intent_tree().root().intent_hash(),
            self.root_signatures(),
        )?;

        let non_root_subintents = self.intent_tree().non_root_subintents();
        let non_root_subintent_signatures = self.non_root_subintent_signatures();
        if non_root_subintents.len() != non_root_subintent_signatures.len() {
            return Err(
                SignatureValidationError::IncorrectNumberOfSubintentSignatureBatches
                    .located(TransactionValidationErrorLocation::AcrossTransaction),
            );
        }
        for (index, (subintent, signatures)) in non_root_subintents
            .zip(non_root_subintent_signatures)
            .enumerate()
        {
            pending_signatures.add_non_root(
                SubintentIndex(index),
                subintent.subintent_hash(),
                signatures.for_subintent(subintent.subintent_hash()),
            )?;
        }

        Ok(pending_signatures)
    }
}

#[must_use]
pub struct AllPendingSignatureValidations<'a> {
    transaction_version: TransactionVersion,
    config: &'a TransactionValidationConfig,
    root: (
        PendingIntentSignatureValidations<'a>,
        TransactionValidationErrorLocation,
    ),
    non_roots: Vec<(
        PendingIntentSignatureValidations<'a>,
        TransactionValidationErrorLocation,
    )>,
    total_signature_validations: usize,
}

pub enum PendingSubintentSignatureValidations<'a> {
    Subintent {
        intent_signatures: &'a [IntentSignatureV1],
    },
    PreviewSubintent {
        intent_public_keys: &'a [PublicKey],
    },
}

impl<'a> PendingSubintentSignatureValidations<'a> {
    fn for_subintent(self, signed_hash: SubintentHash) -> PendingIntentSignatureValidations<'a> {
        match self {
            PendingSubintentSignatureValidations::Subintent { intent_signatures } => {
                PendingIntentSignatureValidations::Subintent {
                    intent_signatures,
                    signed_hash,
                }
            }
            PendingSubintentSignatureValidations::PreviewSubintent { intent_public_keys } => {
                PendingIntentSignatureValidations::PreviewSubintent { intent_public_keys }
            }
        }
    }
}

/// This can assume that the signature counts are within checked limits,
/// so calculations cannot overflow.
pub enum PendingIntentSignatureValidations<'a> {
    TransactionIntent {
        notary_is_signatory: bool,
        notary_public_key: PublicKey,
        notary_signature: SignatureV1,
        notarized_hash: SignedTransactionIntentHash,
        intent_signatures: &'a [IntentSignatureV1],
        signed_hash: TransactionIntentHash,
    },
    PreviewTransactionIntent {
        notary_is_signatory: bool,
        notary_public_key: PublicKey,
        intent_public_keys: &'a [PublicKey],
    },
    Subintent {
        intent_signatures: &'a [IntentSignatureV1],
        signed_hash: SubintentHash,
    },
    PreviewSubintent {
        intent_public_keys: &'a [PublicKey],
    },
}

impl<'a> AllPendingSignatureValidations<'a> {
    pub(crate) fn new_with_root(
        transaction_version: TransactionVersion,
        config: &'a TransactionValidationConfig,
        root_intent_hash: IntentHash,
        signatures: PendingIntentSignatureValidations<'a>,
    ) -> Result<Self, TransactionValidationError> {
        let intent_signature_validations = signatures.intent_signature_validations();
        let error_location = TransactionValidationErrorLocation::for_root(root_intent_hash);
        if intent_signature_validations > config.max_signer_signatures_per_intent {
            return Err(TransactionValidationError::SignatureValidationError(
                error_location,
                SignatureValidationError::TooManySignatures {
                    total: intent_signature_validations,
                    limit: config.max_signer_signatures_per_intent,
                },
            ));
        }
        let notary_signature_validations = signatures.notary_signature_validations();

        Ok(Self {
            transaction_version,
            config,
            root: (signatures, error_location),
            non_roots: Default::default(),
            total_signature_validations: intent_signature_validations
                + notary_signature_validations,
        })
    }

    fn add_non_root(
        &mut self,
        subintent_index: SubintentIndex,
        subintent_hash: SubintentHash,
        signatures: PendingIntentSignatureValidations<'a>,
    ) -> Result<(), TransactionValidationError> {
        let intent_signature_validations = signatures.intent_signature_validations();
        let error_location =
            TransactionValidationErrorLocation::NonRootSubintent(subintent_index, subintent_hash);
        if intent_signature_validations > self.config.max_signer_signatures_per_intent {
            return Err(TransactionValidationError::SignatureValidationError(
                error_location,
                SignatureValidationError::TooManySignatures {
                    total: intent_signature_validations,
                    limit: self.config.max_signer_signatures_per_intent,
                },
            ));
        }

        self.non_roots.push((signatures, error_location));
        self.total_signature_validations += intent_signature_validations;
        Ok(())
    }

    pub(crate) fn validate_all(
        self,
    ) -> Result<SignatureValidationSummary, TransactionValidationError> {
        if self.total_signature_validations > self.config.max_total_signature_validations {
            return Err(TransactionValidationError::SignatureValidationError(
                TransactionValidationErrorLocation::AcrossTransaction,
                SignatureValidationError::TooManySignatures {
                    total: self.total_signature_validations,
                    limit: self.config.max_total_signature_validations,
                },
            ));
        }
        let config = self.config;
        let transaction_version = self.transaction_version;
        let root_signer_keys = Self::validate_signatures(self.root.0, config, transaction_version)
            .map_err(|err| {
                TransactionValidationError::SignatureValidationError(self.root.1, err)
            })?;

        let non_root_signer_keys = self
            .non_roots
            .into_iter()
            .map(|non_root| {
                Self::validate_signatures(non_root.0, config, transaction_version).map_err(|err| {
                    TransactionValidationError::SignatureValidationError(non_root.1, err)
                })
            })
            .collect::<Result<_, _>>()?;

        Ok(SignatureValidationSummary {
            root_signer_keys,
            non_root_signer_keys,
            total_signature_validations: self.total_signature_validations,
        })
    }

    fn validate_signatures(
        signatures: PendingIntentSignatureValidations,
        config: &TransactionValidationConfig,
        transaction_version: TransactionVersion,
    ) -> Result<IndexSet<PublicKey>, SignatureValidationError> {
        let public_keys = match signatures {
            PendingIntentSignatureValidations::TransactionIntent {
                notary_is_signatory,
                notary_public_key,
                notary_signature,
                notarized_hash,
                intent_signatures,
                signed_hash,
            } => {
                let mut intent_public_keys: IndexSet<PublicKey> = Default::default();
                for signature in intent_signatures {
                    let public_key = verify_and_recover(signed_hash.as_hash(), &signature.0)
                        .ok_or(SignatureValidationError::InvalidIntentSignature)?;

                    if !intent_public_keys.insert(public_key) {
                        return Err(SignatureValidationError::DuplicateSigner);
                    }
                }

                if !verify(
                    notarized_hash.as_hash(),
                    &notary_public_key,
                    &notary_signature,
                ) {
                    return Err(SignatureValidationError::InvalidNotarySignature);
                }

                if notary_is_signatory {
                    if !intent_public_keys.insert(notary_public_key)
                        && !config.allow_notary_to_duplicate_signer(transaction_version)
                    {
                        return Err(
                            SignatureValidationError::NotaryIsSignatorySoShouldNotAlsoBeASigner,
                        );
                    }
                }

                intent_public_keys
            }
            PendingIntentSignatureValidations::PreviewTransactionIntent {
                notary_is_signatory,
                notary_public_key,
                intent_public_keys,
            } => {
                let mut checked_intent_public_keys: IndexSet<PublicKey> = Default::default();
                for key in intent_public_keys {
                    if !checked_intent_public_keys.insert(key.clone()) {
                        return Err(SignatureValidationError::DuplicateSigner);
                    }
                }
                if notary_is_signatory {
                    if !checked_intent_public_keys.insert(notary_public_key)
                        && !config.allow_notary_to_duplicate_signer(transaction_version)
                    {
                        return Err(
                            SignatureValidationError::NotaryIsSignatorySoShouldNotAlsoBeASigner,
                        );
                    }
                }
                checked_intent_public_keys
            }
            PendingIntentSignatureValidations::Subintent {
                intent_signatures,
                signed_hash,
            } => {
                let mut intent_public_keys: IndexSet<PublicKey> = Default::default();
                for signature in intent_signatures {
                    let public_key = verify_and_recover(signed_hash.as_hash(), &signature.0)
                        .ok_or(SignatureValidationError::InvalidIntentSignature)?;

                    if !intent_public_keys.insert(public_key) {
                        return Err(SignatureValidationError::DuplicateSigner);
                    }
                }
                intent_public_keys
            }
            PendingIntentSignatureValidations::PreviewSubintent { intent_public_keys } => {
                let mut checked_intent_public_keys: IndexSet<PublicKey> = Default::default();
                for key in intent_public_keys {
                    if !checked_intent_public_keys.insert(key.clone()) {
                        return Err(SignatureValidationError::DuplicateSigner);
                    }
                }
                checked_intent_public_keys
            }
        };

        Ok(public_keys)
    }
}

pub(crate) struct SignatureValidationSummary {
    pub root_signer_keys: IndexSet<PublicKey>,
    pub non_root_signer_keys: Vec<IndexSet<PublicKey>>,
    pub total_signature_validations: usize,
}

impl<'a> PendingIntentSignatureValidations<'a> {
    fn intent_signature_validations(&self) -> usize {
        match self {
            PendingIntentSignatureValidations::TransactionIntent {
                intent_signatures, ..
            } => intent_signatures.len(),
            PendingIntentSignatureValidations::PreviewTransactionIntent {
                intent_public_keys,
                ..
            } => intent_public_keys.len(),
            PendingIntentSignatureValidations::Subintent {
                intent_signatures, ..
            } => intent_signatures.len(),
            PendingIntentSignatureValidations::PreviewSubintent { intent_public_keys } => {
                intent_public_keys.len()
            }
        }
    }

    fn notary_signature_validations(&self) -> usize {
        match self {
            PendingIntentSignatureValidations::TransactionIntent { .. }
            | PendingIntentSignatureValidations::PreviewTransactionIntent { .. } => 1,
            PendingIntentSignatureValidations::Subintent { .. }
            | PendingIntentSignatureValidations::PreviewSubintent { .. } => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::internal_prelude::*;

    #[test]
    fn test_demonstrate_behaviour_with_notary_duplicating_signer() {
        // Arrange
        let network = NetworkDefinition::simulator();
        let notary = Secp256k1PrivateKey::from_u64(1).unwrap();

        let babylon_validator = TransactionValidator::new_with_static_config(
            TransactionValidationConfig::babylon(),
            network.id,
        );
        let latest_validator = TransactionValidator::new_with_static_config(
            TransactionValidationConfig::latest(),
            network.id,
        );

        let transaction_v1 = TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: network.id,
                start_epoch_inclusive: Epoch::of(1),
                end_epoch_exclusive: Epoch::of(10),
                nonce: 0,
                notary_public_key: notary.public_key().into(),
                notary_is_signatory: true,
                tip_percentage: 0,
            })
            .manifest(ManifestBuilder::new().drop_auth_zone_proofs().build())
            .sign(&notary)
            .notarize(&notary)
            .build();

        let transaction_v2 = TransactionV2Builder::new_with_test_defaults()
            .notary_is_signatory(true)
            .notary_public_key(notary.public_key())
            .manifest(ManifestBuilder::new_v2().drop_auth_zone_proofs().build())
            .sign(&notary)
            .notarize(&notary)
            .build_minimal_no_validate();

        // Act & Assert - Transaction V1 permits using notary as signatory and also having it sign
        // It was deemed that we didn't want to start failing V1 transactions for this at Cuttlefish
        // as we didn't want existing integrations to break.
        assert!(transaction_v1
            .prepare_and_validate(&babylon_validator)
            .is_ok());
        assert!(transaction_v1
            .prepare_and_validate(&latest_validator)
            .is_ok());

        // Act & Assert - Transaction V2 does not permit duplicating a notary is signatory as a signatory
        assert_matches!(
            transaction_v2.prepare_and_validate(&babylon_validator),
            Err(TransactionValidationError::PrepareError(
                PrepareError::TransactionTypeNotSupported
            ))
        );
        assert_matches!(
            transaction_v2.prepare_and_validate(&latest_validator),
            Err(TransactionValidationError::SignatureValidationError(
                TransactionValidationErrorLocation::RootTransactionIntent(_),
                SignatureValidationError::NotaryIsSignatorySoShouldNotAlsoBeASigner
            )),
        );
    }

    struct FakeSigner<'a, S: Signer> {
        signer: &'a S,
    }

    impl<'a, S: Signer> FakeSigner<'a, S> {
        fn new(signer: &'a S) -> Self {
            Self { signer }
        }
    }

    impl<'a, S: Signer> Signer for FakeSigner<'a, S> {
        fn public_key(&self) -> PublicKey {
            self.signer.public_key().into()
        }

        fn sign_without_public_key(&self, message_hash: &impl IsHash) -> SignatureV1 {
            let mut signature = self.signer.sign_without_public_key(message_hash);
            match &mut signature {
                SignatureV1::Secp256k1(inner_signature) => {
                    inner_signature.0[5] += 1;
                }
                SignatureV1::Ed25519(inner_signature) => {
                    inner_signature.0[5] += 1;
                }
            }
            signature
        }

        fn sign_with_public_key(&self, message_hash: &impl IsHash) -> SignatureWithPublicKeyV1 {
            let mut signature = self.signer.sign_with_public_key(message_hash);
            match &mut signature {
                SignatureWithPublicKeyV1::Secp256k1 { signature } => {
                    signature.0[5] += 1;
                }
                SignatureWithPublicKeyV1::Ed25519 {
                    signature,
                    public_key: _,
                } => {
                    signature.0[5] += 1;
                }
            }
            signature
        }
    }

    #[test]
    fn test_invalid_signatures() {
        let network = NetworkDefinition::simulator();

        let validator = TransactionValidator::new_with_static_config(
            TransactionValidationConfig::latest(),
            network.id,
        );

        let versions_to_test = [TransactionVersion::V1, TransactionVersion::V2];

        fn validate_transaction(
            validator: &TransactionValidator,
            version: TransactionVersion,
            signer: &impl Signer,
            notary: &impl Signer,
        ) -> Result<IndexSet<PublicKey>, TransactionValidationError> {
            let signer_keys = match version {
                TransactionVersion::V1 => {
                    unsigned_v1_builder(notary.public_key().into())
                        .sign(signer)
                        .notarize(notary)
                        .build()
                        .prepare_and_validate(validator)?
                        .signer_keys
                }
                TransactionVersion::V2 => {
                    TransactionV2Builder::new_with_test_defaults()
                        .add_trivial_manifest()
                        .notary_public_key(notary.public_key())
                        .sign(signer)
                        .notarize(notary)
                        .build_minimal_no_validate()
                        .prepare_and_validate(validator)?
                        .transaction_intent_info
                        .signer_keys
                }
            };
            Ok(signer_keys)
        }

        {
            // Test Secp256k1
            let notary = Secp256k1PrivateKey::from_u64(1).unwrap();
            let signer = Secp256k1PrivateKey::from_u64(13).unwrap();
            for version in versions_to_test {
                assert_matches!(
                    validate_transaction(&validator, version, &signer, &notary),
                    Ok(signer_keys) => {
                        assert_eq!(signer_keys.len(), 1);
                        assert_eq!(signer_keys[0], signer.public_key().into());
                    }
                );
                match validate_transaction(&validator, version, &FakeSigner::new(&signer), &notary)
                {
                    // Coincidentally, between V1 and V2 we hit both cases below
                    Ok(signer_keys) => {
                        // NOTE: Because we recover our Secp256k1 public keys, by mutating the signature
                        // we might have stumbled on a valid signature for a different key - but that's okay.
                        // As long as we can't fake the signature of a particular key, that's okay.
                        assert_eq!(signer_keys.len(), 1);
                        assert_ne!(signer_keys[0], signer.public_key().into());
                    }
                    Err(TransactionValidationError::SignatureValidationError(
                        TransactionValidationErrorLocation::RootTransactionIntent(_),
                        SignatureValidationError::InvalidIntentSignature,
                    )) => {}
                    other_result => {
                        panic!("Unexpected result: {other_result:?}");
                    }
                }
                assert_matches!(
                    validate_transaction(&validator, version, &signer, &FakeSigner::new(&notary)),
                    Err(TransactionValidationError::SignatureValidationError(
                        TransactionValidationErrorLocation::RootTransactionIntent(_),
                        SignatureValidationError::InvalidNotarySignature
                    ))
                );
            }
        }

        {
            // Test Ed25519
            let notary = Ed25519PrivateKey::from_u64(1).unwrap();
            let signer = Ed25519PrivateKey::from_u64(13).unwrap();
            for version in versions_to_test {
                assert_matches!(
                    validate_transaction(&validator, version, &signer, &notary),
                    Ok(signer_keys) => {
                        assert_eq!(signer_keys.len(), 1);
                        assert_eq!(signer_keys[0], signer.public_key().into());
                    }
                );
                assert_matches!(
                    validate_transaction(&validator, version, &FakeSigner::new(&signer), &notary),
                    Err(TransactionValidationError::SignatureValidationError(
                        TransactionValidationErrorLocation::RootTransactionIntent(_),
                        SignatureValidationError::InvalidIntentSignature
                    ))
                );
                assert_matches!(
                    validate_transaction(&validator, version, &signer, &FakeSigner::new(&notary)),
                    Err(TransactionValidationError::SignatureValidationError(
                        TransactionValidationErrorLocation::RootTransactionIntent(_),
                        SignatureValidationError::InvalidNotarySignature
                    ))
                );
            }
        }
    }

    #[test]
    fn too_many_signatures_should_be_rejected() {
        fn validate_transaction(
            root_signature_count: usize,
            signature_counts: Vec<usize>,
        ) -> Result<ValidatedNotarizedTransactionV2, TransactionValidationError> {
            TransactionV2Builder::new_with_test_defaults()
                .add_children(
                    signature_counts
                        .iter()
                        .enumerate()
                        .map(|(i, signature_count)| {
                            create_leaf_partial_transaction(i as u64, *signature_count)
                        }),
                )
                .add_manifest_calling_each_child_once()
                .multi_sign(
                    (0..root_signature_count)
                        .into_iter()
                        .map(|i| Secp256k1PrivateKey::from_u64((100 + i) as u64).unwrap()),
                )
                .default_notarize_and_validate()
        }

        assert_matches!(validate_transaction(1, vec![10]), Ok(_));
        assert_matches!(
            validate_transaction(1, vec![10, 20]),
            Err(TransactionValidationError::SignatureValidationError(
                TransactionValidationErrorLocation::NonRootSubintent(SubintentIndex(1), _),
                SignatureValidationError::TooManySignatures {
                    total: 20,
                    limit: 16,
                },
            ))
        );
        assert_matches!(
            validate_transaction(17, vec![0, 3]),
            Err(TransactionValidationError::SignatureValidationError(
                TransactionValidationErrorLocation::RootTransactionIntent(_),
                SignatureValidationError::TooManySignatures {
                    total: 17,
                    limit: 16,
                },
            ))
        );
        assert_matches!(
            validate_transaction(1, vec![10, 10, 10, 10, 10, 10, 10]),
            Err(TransactionValidationError::SignatureValidationError(
                TransactionValidationErrorLocation::AcrossTransaction,
                SignatureValidationError::TooManySignatures {
                    total: 72, // 70 from subintent, 1 from transaction intent, 1 from notarization
                    limit: 64
                },
            ))
        );
    }

    #[test]
    fn test_incorrect_number_of_subintent_signature_batches() {
        // CASE 1: Too fee signatures
        let validator = TransactionValidator::new_for_latest_simulator();

        let mut transaction = TransactionV2Builder::new_with_test_defaults()
            .add_children(vec![PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .build()])
            .add_manifest_calling_each_child_once()
            .default_notarize()
            .build_minimal_no_validate();

        // Remove one signature batch
        let removed_signature_batch = transaction
            .signed_transaction_intent
            .non_root_subintent_signatures
            .by_subintent
            .pop()
            .unwrap();

        assert_matches!(
            transaction.prepare_and_validate(&validator),
            Err(TransactionValidationError::SignatureValidationError(
                TransactionValidationErrorLocation::AcrossTransaction,
                SignatureValidationError::IncorrectNumberOfSubintentSignatureBatches
            ))
        );

        // CASE 2: Too many signature batches
        let mut transaction = TransactionV2Builder::new_with_test_defaults()
            .add_trivial_manifest()
            .default_notarize()
            .build_minimal_no_validate();

        // Add an extra signature batch
        transaction
            .signed_transaction_intent
            .non_root_subintent_signatures
            .by_subintent
            .push(removed_signature_batch);

        assert_matches!(
            transaction.prepare_and_validate(&validator),
            Err(TransactionValidationError::SignatureValidationError(
                TransactionValidationErrorLocation::AcrossTransaction,
                SignatureValidationError::IncorrectNumberOfSubintentSignatureBatches
            ))
        );
    }
}
