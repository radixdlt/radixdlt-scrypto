use crate::internal_prelude::*;

impl TransactionValidator {
    pub fn validate_notarized_v2(
        &self,
        prepared: PreparedNotarizedTransactionV2,
    ) -> Result<ValidatedNotarizedTransactionV2, TransactionValidationError> {
        let ValidatedTransactionTreeV2 {
            overall_validity_range,
            total_signature_validations,
            root_intent_info,
            root_yield_to_parent_count: _, // Checked to be 0 in the manifest validator.
            non_root_subintents_info,
        } = self.validate_transaction_tree_v2(
            &prepared,
            &prepared.signed_intent.transaction_intent.root_intent_core,
            &prepared
                .signed_intent
                .transaction_intent
                .non_root_subintents,
        )?;

        Ok(ValidatedNotarizedTransactionV2 {
            prepared,
            overall_validity_range,
            total_signature_validations,
            transaction_intent_info: root_intent_info,
            non_root_subintents_info,
        })
    }

    pub fn validate_preview_transaction_v2(
        &self,
        prepared: PreparedPreviewTransactionV2,
    ) -> Result<ValidatedPreviewTransactionV2, TransactionValidationError> {
        let ValidatedTransactionTreeV2 {
            overall_validity_range,
            total_signature_validations: total_expected_signature_validations,
            root_intent_info,
            root_yield_to_parent_count: _, // Checked to be 0 in the manifest validator.
            non_root_subintents_info,
        } = self.validate_transaction_tree_v2(
            &prepared,
            &prepared.transaction_intent.root_intent_core,
            &prepared.transaction_intent.non_root_subintents,
        )?;

        Ok(ValidatedPreviewTransactionV2 {
            prepared,
            overall_validity_range,
            total_expected_signature_validations,
            transaction_intent_info: root_intent_info,
            non_root_subintents_info,
        })
    }

    // This method is public so it can be used by the toolkit.
    pub fn validate_transaction_header_v2(
        &self,
        header: &TransactionHeaderV2,
    ) -> Result<(), HeaderValidationError> {
        if header.tip_basis_points < self.config.min_tip_basis_points
            || header.tip_basis_points > self.config.max_tip_basis_points
        {
            return Err(HeaderValidationError::InvalidTip);
        }

        Ok(())
    }

    pub fn validate_signed_partial_transaction_v2(
        &self,
        prepared: PreparedSignedPartialTransactionV2,
    ) -> Result<ValidatedSignedPartialTransactionV2, TransactionValidationError> {
        let ValidatedTransactionTreeV2 {
            overall_validity_range,
            root_intent_info,
            root_yield_to_parent_count,
            non_root_subintents_info,
            total_signature_validations,
        } = self.validate_transaction_tree_v2(
            &prepared,
            &prepared.partial_transaction.root_subintent.intent_core,
            &prepared.partial_transaction.non_root_subintents,
        )?;

        Ok(ValidatedSignedPartialTransactionV2 {
            prepared,
            total_signature_validations,
            overall_validity_range,
            root_subintent_info: root_intent_info,
            root_subintent_yield_to_parent_count: root_yield_to_parent_count,
            non_root_subintents_info,
        })
    }

    pub fn validate_transaction_tree_v2(
        &self,
        signed_intent_tree: &impl SignedIntentTreeStructure,
        root_intent_core: &PreparedIntentCoreV2,
        non_root_subintents: &PreparedNonRootSubintentsV2,
    ) -> Result<ValidatedTransactionTreeV2, TransactionValidationError> {
        if !self.config.v2_transactions_allowed {
            return Err(TransactionValidationError::TransactionVersionNotPermitted(
                2,
            ));
        }

        let signatures =
            signed_intent_tree.construct_pending_signature_validations(&self.config)?;

        let ValidatedIntentTreeInformation {
            intent_relationships,
            overall_validity_range,
            root_yield_summary,
        } = self.validate_intents_and_structure(signed_intent_tree.intent_tree())?;

        // We delay signature validation until after the other validations as it's more expensive.
        let SignatureValidationSummary {
            root_signer_keys,
            non_root_signer_keys,
            total_signature_validations,
        } = signatures.validate_all()?;

        let root_intent_info = ValidatedIntentInformationV2 {
            encoded_instructions: manifest_encode(&root_intent_core.instructions.inner.0)?.into(),
            children_subintent_indices: intent_relationships.root_intent.children,
            signer_keys: root_signer_keys,
        };
        let non_root_subintents_info = non_root_subintents
            .subintents
            .iter()
            .zip(non_root_signer_keys)
            .zip(intent_relationships.non_root_subintents.into_values())
            .map(
                |((subintent, signer_keys), info)| -> Result<_, TransactionValidationError> {
                    Ok(ValidatedIntentInformationV2 {
                        encoded_instructions: manifest_encode(
                            &subintent.intent_core.instructions.inner.0,
                        )?
                        .into(),
                        signer_keys,
                        children_subintent_indices: info.children,
                    })
                },
            )
            .collect::<Result<_, _>>()?;

        Ok(ValidatedTransactionTreeV2 {
            overall_validity_range,
            root_intent_info,
            root_yield_to_parent_count: root_yield_summary.parent_yields,
            non_root_subintents_info,
            total_signature_validations,
        })
    }

    // This method is public so it can be used by the toolkit.
    pub fn validate_v2_intent_core(
        &self,
        intent_core: &PreparedIntentCoreV2,
        aggregation: &mut AcrossIntentAggregation,
        is_subintent: bool,
    ) -> Result<ManifestYieldSummary, IntentValidationError> {
        self.validate_intent_header_v2(&intent_core.header.inner, aggregation)?;
        self.validate_message_v2(&intent_core.message.inner)?;
        aggregation
            .record_reference_count(intent_core.instructions.references.len(), &self.config)?;
        let yield_summary = self.validate_manifest_v2(
            &intent_core.instructions.inner.0,
            &intent_core.blobs.blobs_by_hash,
            &intent_core.children.children,
            is_subintent,
        )?;
        Ok(yield_summary)
    }

    // This method is public so it can be used by the toolkit.
    pub fn validate_intent_header_v2(
        &self,
        header: &IntentHeaderV2,
        aggregation: &mut AcrossIntentAggregation,
    ) -> Result<(), HeaderValidationError> {
        // Network
        if let Some(required_network_id) = self.required_network_id {
            if header.network_id != required_network_id {
                return Err(HeaderValidationError::InvalidNetwork);
            }
        }

        // Epoch
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

        match (
            header.min_proposer_timestamp_inclusive.as_ref(),
            header.max_proposer_timestamp_exclusive.as_ref(),
        ) {
            (Some(min_timestamp_inclusive), Some(max_timestamp_exclusive)) => {
                if min_timestamp_inclusive >= max_timestamp_exclusive {
                    return Err(HeaderValidationError::InvalidTimestampRange);
                }
            }
            _ => {}
        };

        aggregation.update_headers(
            header.start_epoch_inclusive,
            header.end_epoch_exclusive,
            header.min_proposer_timestamp_inclusive.as_ref(),
            header.max_proposer_timestamp_exclusive.as_ref(),
        )?;

        Ok(())
    }

    // This method is public so it can be used by the toolkit.
    pub fn validate_message_v2(&self, message: &MessageV2) -> Result<(), InvalidMessageError> {
        let validation = &self.config.message_validation;
        match message {
            MessageV2::None => {}
            MessageV2::Plaintext(plaintext_message) => {
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
            MessageV2::Encrypted(encrypted_message) => {
                let EncryptedMessageV2 {
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

    // This method is public so it can be used by the toolkit.
    /// The `is_subintent` property indicates whether it should be treated as a subintent.
    /// A subintent is able to `YIELD_TO_PARENT` and is required to end with a `YIELD_TO_PARENT`.
    pub fn validate_manifest_v2(
        &self,
        instructions: &[InstructionV2],
        blobs: &IndexMap<Hash, Vec<u8>>,
        children: &IndexSet<ChildSubintentSpecifier>,
        is_subintent: bool,
    ) -> Result<ManifestYieldSummary, ManifestValidationError> {
        if instructions.len() > self.config.max_instructions {
            return Err(ManifestValidationError::TooManyInstructions);
        }

        let mut yield_summary =
            ManifestYieldSummary::new_with_children(children.iter().map(|c| c.hash));
        StaticManifestInterpreter::new(
            ValidationRuleset::cuttlefish(),
            &EphemeralManifest::new(instructions, blobs, children, is_subintent),
        )
        .validate_and_apply_visitor(&mut yield_summary)?;

        Ok(yield_summary)
    }
}

impl IntentStructure for PreparedTransactionIntentV2 {
    fn intent_hash(&self) -> IntentHash {
        self.transaction_intent_hash().into()
    }

    fn children(&self) -> impl ExactSizeIterator<Item = SubintentHash> {
        self.root_intent_core
            .children
            .children
            .iter()
            .map(|child| child.hash)
    }

    fn validate_intent(
        &self,
        validator: &TransactionValidator,
        aggregation: &mut AcrossIntentAggregation,
    ) -> Result<ManifestYieldSummary, IntentValidationError> {
        validator
            .validate_transaction_header_v2(&self.transaction_header.inner)
            .map_err(IntentValidationError::HeaderValidationError)?;
        validator.validate_v2_intent_core(&self.root_intent_core, aggregation, false)
    }
}

impl IntentStructure for PreparedSubintentV2 {
    fn intent_hash(&self) -> IntentHash {
        self.subintent_hash().into()
    }

    fn children(&self) -> impl ExactSizeIterator<Item = SubintentHash> {
        self.intent_core
            .children
            .children
            .iter()
            .map(|child| child.hash)
    }

    fn validate_intent(
        &self,
        validator: &TransactionValidator,
        aggregation: &mut AcrossIntentAggregation,
    ) -> Result<ManifestYieldSummary, IntentValidationError> {
        validator.validate_v2_intent_core(&self.intent_core, aggregation, true)
    }
}

impl IntentTreeStructure for PreparedTransactionIntentV2 {
    type RootIntentStructure = Self;
    type SubintentStructure = PreparedSubintentV2;

    fn root(&self) -> &Self::RootIntentStructure {
        self
    }

    fn non_root_subintents<'a>(
        &'a self,
    ) -> impl ExactSizeIterator<Item = &'a Self::SubintentStructure> {
        self.non_root_subintents.subintents.iter()
    }
}

impl IntentTreeStructure for PreparedPartialTransactionV2 {
    type RootIntentStructure = PreparedSubintentV2;
    type SubintentStructure = PreparedSubintentV2;

    fn root(&self) -> &Self::RootIntentStructure {
        &self.root_subintent
    }

    fn non_root_subintents<'a>(
        &'a self,
    ) -> impl ExactSizeIterator<Item = &'a Self::SubintentStructure> {
        self.non_root_subintents.subintents.iter()
    }
}

impl SignedIntentTreeStructure for PreparedNotarizedTransactionV2 {
    type IntentTree = PreparedTransactionIntentV2;

    fn root_signatures(&self) -> PendingIntentSignatureValidations {
        let transaction_intent = &self.signed_intent.transaction_intent;
        PendingIntentSignatureValidations::TransactionIntent {
            notary_is_signatory: transaction_intent
                .transaction_header
                .inner
                .notary_is_signatory,
            notary_public_key: transaction_intent
                .transaction_header
                .inner
                .notary_public_key,
            notary_signature: self.notary_signature.inner.0,
            notarized_hash: self.signed_transaction_intent_hash(),
            intent_signatures: self
                .signed_intent
                .transaction_intent_signatures
                .inner
                .signatures
                .as_slice(),
            signed_hash: transaction_intent.transaction_intent_hash(),
        }
    }

    fn non_root_subintent_signatures(
        &self,
    ) -> impl ExactSizeIterator<Item = PendingSubintentSignatureValidations> {
        self.signed_intent
            .non_root_subintent_signatures
            .by_subintent
            .iter()
            .map(
                |signatures| PendingSubintentSignatureValidations::Subintent {
                    intent_signatures: signatures.inner.signatures.as_slice(),
                },
            )
    }

    fn intent_tree(&self) -> &Self::IntentTree {
        &self.signed_intent.transaction_intent
    }

    fn transaction_version(&self) -> TransactionVersion {
        TransactionVersion::V2
    }
}

impl SignedIntentTreeStructure for PreparedSignedPartialTransactionV2 {
    type IntentTree = PreparedPartialTransactionV2;

    fn root_signatures(&self) -> PendingIntentSignatureValidations {
        PendingIntentSignatureValidations::Subintent {
            intent_signatures: self.root_subintent_signatures.inner.signatures.as_slice(),
            signed_hash: self.intent_tree().subintent_hash(),
        }
    }

    fn non_root_subintent_signatures(
        &self,
    ) -> impl ExactSizeIterator<Item = PendingSubintentSignatureValidations> {
        self.non_root_subintent_signatures
            .by_subintent
            .iter()
            .map(
                |signatures| PendingSubintentSignatureValidations::Subintent {
                    intent_signatures: signatures.inner.signatures.as_slice(),
                },
            )
    }

    fn intent_tree(&self) -> &Self::IntentTree {
        &self.partial_transaction
    }

    fn transaction_version(&self) -> TransactionVersion {
        TransactionVersion::V2
    }
}

impl SignedIntentTreeStructure for PreparedPreviewTransactionV2 {
    type IntentTree = PreparedTransactionIntentV2;

    fn root_signatures(&self) -> PendingIntentSignatureValidations {
        let transaction_intent = &self.transaction_intent;
        PendingIntentSignatureValidations::PreviewTransactionIntent {
            notary_is_signatory: transaction_intent
                .transaction_header
                .inner
                .notary_is_signatory,
            notary_public_key: transaction_intent
                .transaction_header
                .inner
                .notary_public_key,
            intent_public_keys: self.root_subintent_signatures.inner.as_slice(),
        }
    }

    fn non_root_subintent_signatures(
        &self,
    ) -> impl ExactSizeIterator<Item = PendingSubintentSignatureValidations> {
        self.non_root_subintent_signatures
            .inner
            .iter()
            .map(
                |public_keys| PendingSubintentSignatureValidations::PreviewSubintent {
                    intent_public_keys: public_keys.as_slice(),
                },
            )
    }

    fn intent_tree(&self) -> &Self::IntentTree {
        &self.transaction_intent
    }

    fn transaction_version(&self) -> TransactionVersion {
        TransactionVersion::V2
    }
}

#[cfg(test)]
mod tests {
    use crate::internal_prelude::*;

    #[test]
    fn too_many_signatures_should_be_rejected() {
        fn create_partial_transaction(
            subintent_index: usize,
            num_signatures: usize,
        ) -> SignedPartialTransactionV2 {
            let mut builder = PartialTransactionV2Builder::new()
                .intent_header(IntentHeaderV2 {
                    network_id: NetworkDefinition::simulator().id,
                    start_epoch_inclusive: Epoch::of(0),
                    end_epoch_exclusive: Epoch::of(1),
                    min_proposer_timestamp_inclusive: None,
                    max_proposer_timestamp_exclusive: None,
                    intent_discriminator: subintent_index as u64,
                })
                .manifest_builder(|builder| builder.yield_to_parent(()));

            for i in 0..num_signatures {
                let signer =
                    Secp256k1PrivateKey::from_u64(((subintent_index + 1) * 1000 + i) as u64)
                        .unwrap();
                builder = builder.sign(&signer);
            }

            builder.build_minimal()
        }

        fn create_transaction(
            root_signature_count: usize,
            signature_counts: Vec<usize>,
        ) -> NotarizedTransactionV2 {
            let notary = Secp256k1PrivateKey::from_u64(2).unwrap();
            let mut builder = TransactionV2Builder::new();

            for (i, signature_count) in signature_counts.iter().enumerate() {
                builder = builder.add_signed_child(
                    format!("child{i}"),
                    create_partial_transaction(i, *signature_count),
                )
            }

            let mut builder = builder
                .intent_header(IntentHeaderV2 {
                    network_id: NetworkDefinition::simulator().id,
                    start_epoch_inclusive: Epoch::of(0),
                    end_epoch_exclusive: Epoch::of(1),
                    min_proposer_timestamp_inclusive: None,
                    max_proposer_timestamp_exclusive: None,
                    intent_discriminator: 0,
                })
                .manifest_builder(|mut builder| {
                    builder = builder.lock_fee_from_faucet();
                    for (i, _) in signature_counts.iter().enumerate() {
                        builder = builder.yield_to_child(format!("child{i}"), ());
                    }
                    builder
                })
                .transaction_header(TransactionHeaderV2 {
                    notary_public_key: notary.public_key().into(),
                    notary_is_signatory: false,
                    tip_basis_points: 0,
                });

            for i in 0..root_signature_count {
                let signer = Secp256k1PrivateKey::from_u64((100 + i) as u64).unwrap();
                builder = builder.sign(&signer);
            }

            builder.notarize(&notary).build_minimal_no_validate()
        }

        let validator = TransactionValidator::new_for_latest_simulator();
        assert_matches!(
            create_transaction(1, vec![10]).prepare_and_validate(&validator),
            Ok(_)
        );
        assert_matches!(
            create_transaction(1, vec![10, 20]).prepare_and_validate(&validator),
            Err(TransactionValidationError::SignatureValidationError(
                TransactionValidationErrorLocation::NonRootSubintent(SubintentIndex(1), _),
                SignatureValidationError::TooManySignatures {
                    total: 20,
                    limit: 16,
                },
            ))
        );
        assert_matches!(
            create_transaction(17, vec![0, 3]).prepare_and_validate(&validator),
            Err(TransactionValidationError::SignatureValidationError(
                TransactionValidationErrorLocation::RootTransactionIntent(_),
                SignatureValidationError::TooManySignatures {
                    total: 17,
                    limit: 16,
                },
            ))
        );
        assert_matches!(
            create_transaction(1, vec![10, 10, 10, 10, 10, 10, 10])
                .prepare_and_validate(&validator),
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
    fn too_many_references_should_be_rejected() {
        fn create_partial_transaction(
            subintent_index: usize,
            num_references: usize,
        ) -> SignedPartialTransactionV2 {
            PartialTransactionV2Builder::new()
                .intent_header(IntentHeaderV2 {
                    network_id: NetworkDefinition::simulator().id,
                    start_epoch_inclusive: Epoch::of(0),
                    end_epoch_exclusive: Epoch::of(1),
                    min_proposer_timestamp_inclusive: None,
                    max_proposer_timestamp_exclusive: None,
                    intent_discriminator: subintent_index as u64,
                })
                .manifest_builder(|mut builder| {
                    for i in 0..num_references {
                        let mut address =
                            [EntityType::GlobalPreallocatedSecp256k1Account as u8; NodeId::LENGTH];
                        address[1..9].copy_from_slice(
                            &(((subintent_index + 1) * 1000 + i) as u64).to_le_bytes(),
                        );
                        builder = builder.call_method(
                            ComponentAddress::new_or_panic(address),
                            "method_name",
                            (),
                        );
                    }

                    builder.yield_to_parent(())
                })
                .sign(&Secp256k1PrivateKey::from_u64(1000 + subintent_index as u64).unwrap())
                .build_minimal()
        }

        fn create_transaction(reference_counts: Vec<usize>) -> NotarizedTransactionV2 {
            let signer = Secp256k1PrivateKey::from_u64(1).unwrap();
            let notary = Secp256k1PrivateKey::from_u64(2).unwrap();
            let mut builder = TransactionV2Builder::new();

            for (i, reference_count) in reference_counts.iter().enumerate() {
                builder = builder.add_signed_child(
                    format!("child{i}"),
                    create_partial_transaction(i, *reference_count),
                )
            }

            builder
                .intent_header(IntentHeaderV2 {
                    network_id: NetworkDefinition::simulator().id,
                    start_epoch_inclusive: Epoch::of(0),
                    end_epoch_exclusive: Epoch::of(1),
                    min_proposer_timestamp_inclusive: None,
                    max_proposer_timestamp_exclusive: None,
                    intent_discriminator: 0,
                })
                .manifest_builder(|mut builder| {
                    builder = builder.lock_fee_from_faucet();
                    for (i, _) in reference_counts.iter().enumerate() {
                        builder = builder.yield_to_child(format!("child{i}"), ());
                    }
                    builder
                })
                .transaction_header(TransactionHeaderV2 {
                    notary_public_key: notary.public_key().into(),
                    notary_is_signatory: false,
                    tip_basis_points: 0,
                })
                .sign(&signer)
                .notarize(&notary)
                .build_minimal_no_validate()
        }

        let validator = TransactionValidator::new_for_latest_simulator();
        assert_matches!(
            create_transaction(vec![100]).prepare_and_validate(&validator),
            Ok(_)
        );
        assert_matches!(
            create_transaction(vec![100, 600]).prepare_and_validate(&validator),
            Err(TransactionValidationError::IntentValidationError(
                TransactionValidationErrorLocation::NonRootSubintent(SubintentIndex(1), _),
                IntentValidationError::TooManyReferences {
                    total: 600,
                    limit: 512,
                }
            ))
        );
        assert_matches!(
            create_transaction(vec![500, 500]).prepare_and_validate(&validator),
            Err(TransactionValidationError::IntentValidationError(
                TransactionValidationErrorLocation::AcrossTransaction,
                IntentValidationError::TooManyReferences {
                    total: 1001, // 1000 from subintent, 1 from transaction intent
                    limit: 512,
                }
            ))
        );
    }
}
