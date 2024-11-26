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

    fn mutate_subintents(
        transaction: &mut NotarizedTransactionV2,
        subintents_mutate: impl FnOnce(&mut Vec<SubintentV2>),
        subintent_signatures_mutate: impl FnOnce(&mut Vec<IntentSignaturesV2>),
    ) {
        subintents_mutate(
            &mut transaction
                .signed_transaction_intent
                .transaction_intent
                .non_root_subintents
                .0,
        );
        subintent_signatures_mutate(
            &mut transaction
                .signed_transaction_intent
                .non_root_subintent_signatures
                .by_subintent,
        );
    }

    #[test]
    fn test_subintent_structure_errors() {
        let validator = TransactionValidator::new_for_latest_simulator();

        // SubintentStructureError::DuplicateSubintent
        {
            let duplicated_subintent = create_leaf_partial_transaction(0, 0);
            let duplicated_subintent_hash = duplicated_subintent.root_subintent_hash;
            let mut transaction = TransactionV2Builder::new_with_test_defaults()
                .add_children([duplicated_subintent])
                .add_manifest_calling_each_child_once()
                .default_notarize()
                .build_minimal_no_validate();

            mutate_subintents(
                &mut transaction,
                |subintents| {
                    subintents.push(subintents[0].clone());
                },
                |subintent_signatures| {
                    subintent_signatures.push(subintent_signatures[0].clone());
                },
            );

            assert_matches!(
                transaction.prepare_and_validate(&validator),
                Err(TransactionValidationError::SubintentStructureError(
                    TransactionValidationErrorLocation::NonRootSubintent(SubintentIndex(1), subintent_hash),
                    SubintentStructureError::DuplicateSubintent,
                )) => {
                    assert_eq!(subintent_hash, duplicated_subintent_hash);
                }
            );
        }

        // SubintentStructureError::SubintentHasMultipleParents
        // ====================================================
        // CASE 1 - Two duplicates as children in the same intent
        // =======> This isn't possible because `ChildSubintentSpecifiersV2` wraps an `IndexSet<ChildSubintentSpecifier>`
        // Case 2 - Both duplicates across different intents
        // =======> This is tested below
        {
            let duplicated_subintent = create_leaf_partial_transaction(1, 0);

            let parent_subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_children([duplicated_subintent.clone()])
                .add_manifest_calling_each_child_once()
                .build();
            let mut transaction = TransactionV2Builder::new_with_test_defaults()
                .add_children([parent_subintent, duplicated_subintent.clone()])
                .add_manifest_calling_each_child_once()
                .default_notarize()
                .build_minimal_no_validate();

            mutate_subintents(
                &mut transaction,
                |subintents| {
                    subintents.remove(1);
                },
                |subintent_signatures| {
                    subintent_signatures.remove(1);
                },
            );

            assert_matches!(
                transaction.prepare_and_validate(&validator),
                Err(TransactionValidationError::SubintentStructureError(
                    TransactionValidationErrorLocation::NonRootSubintent(SubintentIndex(1), subintent_hash),
                    SubintentStructureError::SubintentHasMultipleParents,
                )) => {
                    assert_eq!(subintent_hash, duplicated_subintent.root_subintent_hash);
                }
            );
        }

        // SubintentStructureError::ChildSubintentNotIncludedInTransaction(SubintentHash)
        {
            let missing_subintent = create_leaf_partial_transaction(0, 0);
            let missing_subintent_hash = missing_subintent.root_subintent_hash;
            let mut transaction = TransactionV2Builder::new_with_test_defaults()
                .add_children([missing_subintent])
                .add_manifest_calling_each_child_once()
                .default_notarize()
                .build_minimal_no_validate();

            mutate_subintents(
                &mut transaction,
                |subintents| {
                    subintents.pop();
                },
                |subintent_signatures| {
                    subintent_signatures.pop();
                },
            );

            assert_matches!(
                transaction.prepare_and_validate(&validator),
                Err(TransactionValidationError::SubintentStructureError(
                    TransactionValidationErrorLocation::Unlocatable,
                    SubintentStructureError::ChildSubintentNotIncludedInTransaction(subintent_hash),
                )) => {
                    assert_eq!(subintent_hash, missing_subintent_hash);
                }
            );
        }

        // SubintentStructureError::SubintentExceedsMaxDepth
        {
            let depth_4 = create_leaf_partial_transaction(0, 0);
            let depth_4_hash = depth_4.root_subintent_hash;
            let depth_3 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_children([depth_4])
                .add_manifest_calling_each_child_once()
                .build();
            let depth_2 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_children([depth_3])
                .add_manifest_calling_each_child_once()
                .build();
            let depth_1 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_children([depth_2])
                .add_manifest_calling_each_child_once()
                .build();
            let transaction = TransactionV2Builder::new_with_test_defaults()
                .add_children([depth_1])
                .add_manifest_calling_each_child_once()
                .default_notarize()
                .build_minimal_no_validate();

            assert_matches!(
                transaction.prepare_and_validate(&validator),
                Err(TransactionValidationError::SubintentStructureError(
                    TransactionValidationErrorLocation::NonRootSubintent(SubintentIndex(_), subintent_hash),
                    SubintentStructureError::SubintentExceedsMaxDepth,
                )) => {
                    assert_eq!(subintent_hash, depth_4_hash);
                }
            );
        }

        // SubintentStructureError::SubintentIsNotReachableFromTheTransactionIntent
        // ========================================================================
        // CASE 1 - The subintent is superfluous / has no parent
        // This is tested below
        //
        // CASE 2 - Without a "no parent" short-circuit.
        // To hit this error (but none of the previous errors) requires that we have
        // a cycle in the subintent graph.
        //
        // But, because parents include a subintent hash of their direct children,
        // which is itself part of their hash, a cycle would require a hash collision!
        //
        // But we can hack around this by explicitly overwriting the prepared subintent
        // hashes.
        {
            // CASE 1 - The subintent has no parent
            let no_parent_subintent = create_leaf_partial_transaction(0, 0);
            let no_parent_subintent_hash = no_parent_subintent.root_subintent_hash;

            let mut transaction = TransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .default_notarize()
                .build_minimal_no_validate();

            mutate_subintents(
                &mut transaction,
                |subintents| {
                    subintents.push(
                        no_parent_subintent
                            .partial_transaction
                            .partial_transaction
                            .root_subintent,
                    );
                },
                |subintent_signatures| {
                    subintent_signatures.push(IntentSignaturesV2::none());
                },
            );

            assert_matches!(
                transaction.prepare_and_validate(&validator),
                Err(TransactionValidationError::SubintentStructureError(
                    TransactionValidationErrorLocation::NonRootSubintent(SubintentIndex(0), subintent_hash),
                    SubintentStructureError::SubintentIsNotReachableFromTheTransactionIntent,
                )) => {
                    assert_eq!(subintent_hash, no_parent_subintent_hash);
                }
            );

            // CASE 2 - Without a potential "no parent" short-circuit
            let faked_hash = SubintentHash::from_bytes([1; 32]);

            let self_parent_subintent = SubintentV2 {
                intent_core: IntentCoreV2 {
                    header: IntentHeaderV2 {
                        network_id: NetworkDefinition::simulator().id,
                        start_epoch_inclusive: Epoch::of(0),
                        end_epoch_exclusive: Epoch::of(1),
                        min_proposer_timestamp_inclusive: None,
                        max_proposer_timestamp_exclusive: None,
                        intent_discriminator: 0,
                    },
                    message: MessageV2::None,
                    instructions: InstructionsV2(vec![InstructionV2::YieldToParent(
                        YieldToParent::empty(),
                    )]),
                    blobs: BlobsV1::none(),
                    children: ChildSubintentSpecifiersV2 {
                        children: indexset![faked_hash.into()],
                    },
                },
            };

            let mut transaction = TransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .default_notarize()
                .build_minimal_no_validate();

            mutate_subintents(
                &mut transaction,
                |subintents| {
                    subintents.push(self_parent_subintent);
                },
                |subintent_signatures| {
                    subintent_signatures.push(IntentSignaturesV2::none());
                },
            );

            let mut prepared = transaction
                .prepare(validator.preparation_settings())
                .unwrap();

            // We overwrite the subintent hash to the faked hash
            prepared
                .signed_intent
                .transaction_intent
                .non_root_subintents
                .subintents[0]
                .summary
                .hash = faked_hash.0;

            assert_matches!(
                prepared.validate(&validator),
                Err(TransactionValidationError::SubintentStructureError(
                    TransactionValidationErrorLocation::NonRootSubintent(SubintentIndex(0), subintent_hash),
                    SubintentStructureError::SubintentIsNotReachableFromTheTransactionIntent,
                )) => {
                    assert_eq!(subintent_hash, faked_hash);
                }
            );
        }

        // SubintentStructureError::MismatchingYieldChildAndYieldParentCountsForSubintent
        {
            let single_yield_subintent = create_leaf_partial_transaction(0, 0);
            let single_yield_subintent_hash = single_yield_subintent.root_subintent_hash;

            // CASE 1: We yield twice to it, but it yields to us only once
            let transaction = TransactionV2Builder::new_with_test_defaults()
                .add_signed_child("child", single_yield_subintent.clone())
                .manifest_builder(|builder| {
                    builder
                        .yield_to_child("child", ())
                        .yield_to_child("child", ())
                })
                .default_notarize()
                .build_minimal_no_validate();

            assert_matches!(
                transaction.prepare_and_validate(&validator),
                Err(TransactionValidationError::SubintentStructureError(
                    TransactionValidationErrorLocation::NonRootSubintent(SubintentIndex(0), subintent_hash),
                    SubintentStructureError::MismatchingYieldChildAndYieldParentCountsForSubintent,
                )) => {
                    assert_eq!(subintent_hash, single_yield_subintent_hash);
                }
            );

            // CASE 2: We yield zero times to it
            let transaction = TransactionV2Builder::new_with_test_defaults()
                .add_signed_child("child", single_yield_subintent)
                .manifest_builder(|builder| builder)
                .default_notarize()
                .build_minimal_no_validate();

            assert_matches!(
                transaction.prepare_and_validate(&validator),
                Err(TransactionValidationError::SubintentStructureError(
                    TransactionValidationErrorLocation::NonRootSubintent(SubintentIndex(0), subintent_hash),
                    SubintentStructureError::MismatchingYieldChildAndYieldParentCountsForSubintent,
                )) => {
                    assert_eq!(subintent_hash, single_yield_subintent_hash);
                }
            );

            // CASE 3: More complex example, between two subintents, with 2 parent and 3 child yields:
            let two_parent_yield_subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .manifest_builder(|builder| builder.yield_to_parent(()).yield_to_parent(()))
                .build();
            let two_parent_yield_subintent_hash = two_parent_yield_subintent.root_subintent_hash;

            let three_child_yield_parent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_signed_child("child", two_parent_yield_subintent)
                .manifest_builder(|builder| {
                    builder
                        .yield_to_child("child", ())
                        .yield_to_child("child", ())
                        .yield_to_child("child", ())
                        .yield_to_parent(())
                })
                .build();

            let transaction = TransactionV2Builder::new_with_test_defaults()
                .add_children([three_child_yield_parent])
                .add_manifest_calling_each_child_once()
                .default_notarize()
                .build_minimal_no_validate();

            assert_matches!(
                transaction.prepare_and_validate(&validator),
                Err(TransactionValidationError::SubintentStructureError(
                    TransactionValidationErrorLocation::NonRootSubintent(SubintentIndex(_), subintent_hash),
                    SubintentStructureError::MismatchingYieldChildAndYieldParentCountsForSubintent,
                )) => {
                    assert_eq!(subintent_hash, two_parent_yield_subintent_hash);
                }
            );
        }
    }

    // NOTE: This is very similar to the V1 tests, just adjusted to the V2 models
    #[test]
    fn test_valid_messages() {
        // None
        {
            let message = MessageV2::None;
            assert_matches!(validate_transaction_with_message(message), Ok(_),);
        }
        // Plaintext
        {
            let message = MessageV2::Plaintext(PlaintextMessageV1 {
                mime_type: "text/plain".to_owned(),
                message: MessageContentsV1::String("Hello world!".to_string()),
            });
            assert_matches!(validate_transaction_with_message(message), Ok(_),);
        }
        // Encrypted
        {
            // Note - this isn't actually a validly encrypted message,
            // this just shows that a sufficiently valid encrypted message can pass validation
            let message = MessageV2::Encrypted(EncryptedMessageV2 {
                encrypted: AesGcmPayload(vec![]),
                decryptors_by_curve: indexmap!(
                    CurveType::Ed25519 => DecryptorsByCurveV2::Ed25519 {
                        dh_ephemeral_public_key: Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]),
                        decryptors: indexmap!(
                            PublicKeyFingerprint([0; PublicKeyFingerprint::LENGTH]) => AesWrapped256BitKey([0; AesWrapped256BitKey::LENGTH]),
                        ),
                    },
                    CurveType::Secp256k1 => DecryptorsByCurveV2::Secp256k1 {
                        dh_ephemeral_public_key: Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH]),
                        decryptors: indexmap!(
                            PublicKeyFingerprint([0; PublicKeyFingerprint::LENGTH]) => AesWrapped256BitKey([0; AesWrapped256BitKey::LENGTH]),
                            PublicKeyFingerprint([1; PublicKeyFingerprint::LENGTH]) => AesWrapped256BitKey([0; AesWrapped256BitKey::LENGTH]),
                        ),
                    },
                ),
            });
            assert_matches!(validate_transaction_with_message(message), Ok(_),);
        }
    }

    // NOTE: This is very similar to the V1 tests, just adjusted to the V2 models
    #[test]
    fn test_invalid_message_errors() {
        // MimeTypeTooLong
        {
            let message = MessageV2::Plaintext(PlaintextMessageV1 {
                mime_type: "very long mimetype, very long mimetype, very long mimetype, very long mimetype, very long mimetype, very long mimetype, very long mimetype, very long mimetype, ".to_owned(),
                message: MessageContentsV1::String("Hello".to_string()),
            });
            assert_matches!(
                validate_transaction_with_message(message),
                Err(InvalidMessageError::MimeTypeTooLong { .. }),
            );
        }

        // PlaintextMessageTooLong
        {
            let mut long_message: String = "".to_owned();
            while long_message.len() <= 2048 {
                long_message.push_str("more text please!");
            }
            let message = MessageV2::Plaintext(PlaintextMessageV1 {
                mime_type: "text/plain".to_owned(),
                message: MessageContentsV1::String(long_message),
            });
            assert_matches!(
                validate_transaction_with_message(message),
                Err(InvalidMessageError::PlaintextMessageTooLong { .. }),
            );
        }

        // EncryptedMessageTooLong
        {
            let mut message_which_is_too_long: String = "".to_owned();
            while message_which_is_too_long.len() <= 2048 + 50 {
                // Some more bytes for the AES padding
                message_which_is_too_long.push_str("more text please!");
            }
            let message = MessageV2::Encrypted(EncryptedMessageV2 {
                encrypted: AesGcmPayload(message_which_is_too_long.as_bytes().to_vec()),
                decryptors_by_curve: indexmap!(
                    CurveType::Ed25519 => DecryptorsByCurveV2::Ed25519 {
                        dh_ephemeral_public_key: Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]),
                        decryptors: indexmap!(
                            PublicKeyFingerprint([0; PublicKeyFingerprint::LENGTH]) => AesWrapped256BitKey([0; AesWrapped256BitKey::LENGTH]),
                        ),
                    }
                ),
            });
            assert_matches!(
                validate_transaction_with_message(message),
                Err(InvalidMessageError::EncryptedMessageTooLong { .. }),
            );
        }

        // NoDecryptors
        {
            let message = MessageV2::Encrypted(EncryptedMessageV2 {
                encrypted: AesGcmPayload(vec![]),
                decryptors_by_curve: indexmap!(),
            });
            assert_matches!(
                validate_transaction_with_message(message),
                Err(InvalidMessageError::NoDecryptors),
            );
        }

        // NoDecryptorsForCurveType
        {
            let message = MessageV2::Encrypted(EncryptedMessageV2 {
                encrypted: AesGcmPayload(vec![]),
                decryptors_by_curve: indexmap!(
                    CurveType::Ed25519 => DecryptorsByCurveV2::Ed25519 {
                        dh_ephemeral_public_key: Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]),
                        decryptors: indexmap!(),
                    }
                ),
            });
            assert_matches!(
                validate_transaction_with_message(message),
                Err(InvalidMessageError::NoDecryptorsForCurveType {
                    curve_type: CurveType::Ed25519
                }),
            );
        }

        // MismatchingDecryptorCurves
        {
            let message = MessageV2::Encrypted(EncryptedMessageV2 {
                encrypted: AesGcmPayload(vec![]),
                decryptors_by_curve: indexmap!(
                    CurveType::Ed25519 => DecryptorsByCurveV2::Secp256k1 {
                        dh_ephemeral_public_key: Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH]),
                        decryptors: indexmap!(
                            PublicKeyFingerprint([0; PublicKeyFingerprint::LENGTH]) => AesWrapped256BitKey([0; AesWrapped256BitKey::LENGTH]),
                        ),
                    }
                ),
            });
            assert_matches!(
                validate_transaction_with_message(message),
                Err(InvalidMessageError::MismatchingDecryptorCurves {
                    actual: CurveType::Secp256k1,
                    expected: CurveType::Ed25519
                }),
            );
        }

        // TooManyDecryptors
        {
            let mut decryptors = IndexMap::<PublicKeyFingerprint, AesWrapped256BitKey>::default();
            for i in 0..30 {
                decryptors.insert(
                    PublicKeyFingerprint([0, 0, 0, 0, 0, 0, 0, i as u8]),
                    AesWrapped256BitKey([0; AesWrapped256BitKey::LENGTH]),
                );
            }
            let message = MessageV2::Encrypted(EncryptedMessageV2 {
                encrypted: AesGcmPayload(vec![]),
                decryptors_by_curve: indexmap!(
                    CurveType::Ed25519 => DecryptorsByCurveV2::Ed25519 {
                        dh_ephemeral_public_key: Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]),
                        decryptors,
                    }
                ),
            });
            assert_matches!(
                validate_transaction_with_message(message),
                Err(InvalidMessageError::TooManyDecryptors {
                    actual: 30,
                    permitted: 20
                }),
            );
        }
    }

    fn validate_transaction_with_message(
        message: MessageV2,
    ) -> Result<ValidatedNotarizedTransactionV2, InvalidMessageError> {
        TransactionV2Builder::new_with_test_defaults()
            .add_trivial_manifest()
            .message(message)
            .default_notarize_and_validate()
            .map_err(|e| match e {
                TransactionValidationError::IntentValidationError(
                    _,
                    IntentValidationError::InvalidMessage(e),
                ) => e,
                _ => panic!("Expected InvalidMessageError, but got: {:?}", e),
            })
    }

    #[test]
    fn too_many_references_should_be_rejected() {
        fn create_partial_transaction(
            subintent_index: usize,
            num_references: usize,
        ) -> SignedPartialTransactionV2 {
            PartialTransactionV2Builder::new_with_test_defaults()
                .intent_discriminator(subintent_index as u64)
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

        fn validate_transaction(
            reference_counts: Vec<usize>,
        ) -> Result<ValidatedNotarizedTransactionV2, TransactionValidationError> {
            TransactionV2Builder::new_with_test_defaults()
                .add_children(
                    reference_counts
                        .iter()
                        .enumerate()
                        .map(|(i, reference_count)| {
                            create_partial_transaction(i, *reference_count)
                        }),
                )
                .add_manifest_calling_each_child_once()
                .sign(&Secp256k1PrivateKey::from_u64(1).unwrap())
                .default_notarize_and_validate()
        }

        assert_matches!(validate_transaction(vec![100]), Ok(_));
        assert_matches!(
            validate_transaction(vec![100, 600]),
            Err(TransactionValidationError::IntentValidationError(
                TransactionValidationErrorLocation::NonRootSubintent(SubintentIndex(1), _),
                IntentValidationError::TooManyReferences {
                    total: 600,
                    limit: 512,
                }
            ))
        );
        assert_matches!(
            validate_transaction(vec![500, 500]),
            Err(TransactionValidationError::IntentValidationError(
                TransactionValidationErrorLocation::AcrossTransaction,
                IntentValidationError::TooManyReferences {
                    total: 1001, // 1000 from subintent, 1 from transaction intent
                    limit: 512,
                }
            ))
        );
    }

    #[test]
    fn test_header_validations() {
        let simulator_validator = TransactionValidator::new_for_latest_simulator();
        let network_agnostic_validator =
            TransactionValidator::new_with_latest_config_network_agnostic();
        let config = simulator_validator.config().clone();

        // InvalidEpochRange
        {
            // CASE 1 - Negative range
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .start_epoch_inclusive(Epoch::of(100))
                .end_epoch_exclusive(Epoch::of(98))
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::RootTransactionIntent(_),
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::InvalidEpochRange
                    ),
                )),
            );

            let subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .start_epoch_inclusive(Epoch::of(100))
                .end_epoch_exclusive(Epoch::of(98))
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent])
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::NonRootSubintent { .. },
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::InvalidEpochRange
                    ),
                )),
            );

            // CASE 2 - Equal range
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .start_epoch_inclusive(Epoch::of(100))
                .end_epoch_exclusive(Epoch::of(100))
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::RootTransactionIntent(_),
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::InvalidEpochRange
                    ),
                )),
            );

            let subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .start_epoch_inclusive(Epoch::of(100))
                .end_epoch_exclusive(Epoch::of(100))
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent])
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::NonRootSubintent { .. },
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::InvalidEpochRange
                    ),
                )),
            );

            // CASE 3 - Range too large
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .start_epoch_inclusive(Epoch::of(100))
                .end_epoch_exclusive(Epoch::of(100 + config.max_epoch_range + 1))
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::RootTransactionIntent(_),
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::InvalidEpochRange
                    ),
                )),
            );

            let subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .start_epoch_inclusive(Epoch::of(100))
                .end_epoch_exclusive(Epoch::of(100 + config.max_epoch_range + 1))
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent])
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::NonRootSubintent { .. },
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::InvalidEpochRange
                    ),
                )),
            );
        }

        // InvalidTimestampRange
        {
            // CASE 1 - Negative range
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .min_proposer_timestamp_inclusive(Some(Instant::new(5000)))
                .max_proposer_timestamp_exclusive(Some(Instant::new(4999)))
                .end_epoch_exclusive(Epoch::of(98))
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::RootTransactionIntent(_),
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::InvalidTimestampRange
                    ),
                )),
            );

            let subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .min_proposer_timestamp_inclusive(Some(Instant::new(5000)))
                .max_proposer_timestamp_exclusive(Some(Instant::new(4999)))
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent])
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::NonRootSubintent { .. },
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::InvalidTimestampRange
                    ),
                )),
            );

            // CASE 2 - Equal range
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .min_proposer_timestamp_inclusive(Some(Instant::new(5000)))
                .max_proposer_timestamp_exclusive(Some(Instant::new(5000)))
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::RootTransactionIntent(_),
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::InvalidTimestampRange
                    ),
                )),
            );

            let subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .min_proposer_timestamp_inclusive(Some(Instant::new(5000)))
                .max_proposer_timestamp_exclusive(Some(Instant::new(5000)))
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent])
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::NonRootSubintent { .. },
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::InvalidTimestampRange
                    ),
                )),
            );

            // And for good measure, let's test some valid ranges:
            let subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .min_proposer_timestamp_inclusive(Some(Instant::new(5000)))
                .max_proposer_timestamp_exclusive(Some(Instant::new(5001)))
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent])
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(result, Ok(_),);
            let subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .min_proposer_timestamp_inclusive(Some(Instant::new(5000)))
                .max_proposer_timestamp_exclusive(None)
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent])
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(result, Ok(_),);
            let subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .min_proposer_timestamp_inclusive(None)
                .max_proposer_timestamp_exclusive(Some(Instant::new(5000)))
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent])
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(result, Ok(_),);
            let subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .min_proposer_timestamp_inclusive(None)
                .max_proposer_timestamp_exclusive(None)
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent])
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(result, Ok(_),);
        }

        // InvalidNetwork
        {
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .network_id(NetworkDefinition::mainnet().id)
                .default_notarize()
                .build_minimal_no_validate()
                .prepare_and_validate(&simulator_validator);
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::RootTransactionIntent(_),
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::InvalidNetwork
                    ),
                )),
            );

            let subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .network_id(NetworkDefinition::mainnet().id)
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent])
                .add_manifest_calling_each_child_once()
                .default_notarize()
                .build_minimal_no_validate()
                .prepare_and_validate(&simulator_validator);
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::NonRootSubintent { .. },
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::InvalidNetwork
                    ),
                )),
            );

            // And for good measure, demonstrate that the network agnostic validator is okay with this:
            // (even with different intents being for different networks(!) - which is a bit weird, but
            // it's only intended for testing)
            let subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .network_id(NetworkDefinition::mainnet().id)
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent])
                .add_manifest_calling_each_child_once()
                .default_notarize()
                .build_minimal_no_validate()
                .prepare_and_validate(&network_agnostic_validator);
            assert_matches!(result, Ok(_),);
        }

        // InvalidTip
        {
            // Note - min tip is 0, so we can't hit that error
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .tip_basis_points(config.max_tip_basis_points + 1)
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::RootTransactionIntent(_),
                    IntentValidationError::HeaderValidationError(HeaderValidationError::InvalidTip),
                )),
            );
        }

        // NoValidEpochRangeAcrossAllIntents
        {
            // Subintent doesn't overlap with TransactionIntent
            let subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .start_epoch_inclusive(Epoch::of(100))
                .end_epoch_exclusive(Epoch::of(102))
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent])
                .start_epoch_inclusive(Epoch::of(102))
                .end_epoch_exclusive(Epoch::of(103))
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::NonRootSubintent { .. },
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::NoValidEpochRangeAcrossAllIntents
                    ),
                )),
            );

            // Only one pair of subintents don't overlap
            let subintent_1 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .start_epoch_inclusive(Epoch::of(100))
                .end_epoch_exclusive(Epoch::of(102))
                .build();
            let subintent_2 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .start_epoch_inclusive(Epoch::of(101))
                .end_epoch_exclusive(Epoch::of(103))
                .build();
            let subintent_3 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .start_epoch_inclusive(Epoch::of(102))
                .end_epoch_exclusive(Epoch::of(104))
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent_1, subintent_2, subintent_3])
                .start_epoch_inclusive(Epoch::of(100))
                .end_epoch_exclusive(Epoch::of(105))
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::NonRootSubintent { .. },
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::NoValidEpochRangeAcrossAllIntents
                    ),
                )),
            );

            // There is an overlap
            let subintent_1 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .start_epoch_inclusive(Epoch::of(100))
                .end_epoch_exclusive(Epoch::of(102))
                .build();
            let subintent_2 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .start_epoch_inclusive(Epoch::of(101))
                .end_epoch_exclusive(Epoch::of(103))
                .build();
            let subintent_3 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .start_epoch_inclusive(Epoch::of(101))
                .end_epoch_exclusive(Epoch::of(104))
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent_1, subintent_2, subintent_3])
                .start_epoch_inclusive(Epoch::of(100))
                .end_epoch_exclusive(Epoch::of(105))
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Ok(validated) => {
                    assert_eq!(validated.overall_validity_range.epoch_range.start_epoch_inclusive, Epoch::of(101));
                    assert_eq!(validated.overall_validity_range.epoch_range.end_epoch_exclusive, Epoch::of(102));
                },
            );
        }

        // NoValidTimestampRangeAcrossAllIntents
        {
            // Subintent doesn't overlap with TransactionIntent
            let subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .min_proposer_timestamp_inclusive(Some(Instant::new(5000)))
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent])
                .max_proposer_timestamp_exclusive(Some(Instant::new(5000)))
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::NonRootSubintent { .. },
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::NoValidTimestampRangeAcrossAllIntents
                    ),
                )),
            );

            // Only one pair of subintents don't overlap
            let subintent_1 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .max_proposer_timestamp_exclusive(Some(Instant::new(4000)))
                .build();
            let subintent_2 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .max_proposer_timestamp_exclusive(Some(Instant::new(4003)))
                .build();
            let subintent_3 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .min_proposer_timestamp_inclusive(Some(Instant::new(4001)))
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent_1, subintent_2, subintent_3])
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Err(TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::NonRootSubintent { .. },
                    IntentValidationError::HeaderValidationError(
                        HeaderValidationError::NoValidTimestampRangeAcrossAllIntents
                    ),
                )),
            );

            // There is an overlap
            let subintent_1 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .max_proposer_timestamp_exclusive(Some(Instant::new(4003)))
                .build();
            let subintent_2 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .max_proposer_timestamp_exclusive(Some(Instant::new(4005)))
                .build();
            let subintent_3 = PartialTransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .min_proposer_timestamp_inclusive(Some(Instant::new(3998)))
                .max_proposer_timestamp_exclusive(Some(Instant::new(4001)))
                .build();
            let result = TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent_1, subintent_2, subintent_3])
                .min_proposer_timestamp_inclusive(Some(Instant::new(3999)))
                .max_proposer_timestamp_exclusive(Some(Instant::new(5999)))
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate();
            assert_matches!(
                result,
                Ok(validated) => {
                    assert_eq!(validated.overall_validity_range.proposer_timestamp_range.start_timestamp_inclusive, Some(Instant::new(3999)));
                    assert_eq!(validated.overall_validity_range.proposer_timestamp_range.end_timestamp_exclusive, Some(Instant::new(4001)));
                },
            );
        }
    }

    trait ManifestBuilderExtensions {
        fn add_test_method_call_with(self, value: impl ManifestEncode) -> Self;
    }

    impl<M: BuildableManifest> ManifestBuilderExtensions for ManifestBuilder<M>
    where
        CallMethod: Into<M::Instruction>,
    {
        fn add_test_method_call_with(self, value: impl ManifestEncode) -> Self {
            self.add_raw_instruction_ignoring_all_side_effects(CallMethod {
                address: XRD.into(),
                method_name: "method".into(),
                args: manifest_decode(&manifest_encode(&(value,)).unwrap()).unwrap(),
            })
        }
    }

    #[test]
    fn test_manifest_validations() {
        let account_address = ComponentAddress::preallocated_account_from_public_key(
            &Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]),
        );

        fn validate_transaction_manifest(
            manifest: TransactionManifestV2,
        ) -> Result<ValidatedNotarizedTransactionV2, ManifestValidationError> {
            let builder = TransactionV2Builder::new_with_test_defaults().manifest(manifest);
            validate_transaction_builder_manifest(builder)
        }

        fn validate_transaction_builder_manifest(
            builder: TransactionV2Builder,
        ) -> Result<ValidatedNotarizedTransactionV2, ManifestValidationError> {
            builder
                .default_notarize_and_validate()
                .map_err(|err| match err {
                    TransactionValidationError::IntentValidationError(
                        _,
                        IntentValidationError::ManifestValidationError(err),
                    ) => err,
                    _ => panic!("Expected ManifestValidationError, but got: {:?}", err),
                })
        }

        fn validate_subintent_manifest(
            subintent_manifest: SubintentManifestV2,
        ) -> Result<ValidatedNotarizedTransactionV2, ManifestValidationError> {
            let subintent = PartialTransactionV2Builder::new_with_test_defaults()
                .manifest(subintent_manifest)
                .build();
            TransactionV2Builder::new_with_test_defaults()
                .add_children([subintent])
                .add_manifest_calling_each_child_once()
                .default_notarize_and_validate()
                .map_err(|err| match err {
                    TransactionValidationError::IntentValidationError(
                        _,
                        IntentValidationError::ManifestValidationError(err),
                    ) => err,
                    _ => panic!("Expected ManifestValidationError, but got: {:?}", err),
                })
        }

        // DuplicateBlob(ManifestBlobRef)
        {
            // This is not actually possible to get in TransactionV2, because the manifest stores an IndexMap<Hash, Bytes>.
            // Currently we remove duplicates at the `PreparedBlobsV1` layer.
        }

        // BlobNotRegistered(ManifestBlobRef)
        {
            let transaction_manifest = ManifestBuilder::new_v2()
                .add_test_method_call_with(ManifestBlobRef([2; 32]))
                .build_no_validate();

            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::BlobNotRegistered(ManifestBlobRef(blob_ref))) => {
                    assert_eq!(blob_ref, [2; 32]);
                }
            );

            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .add_test_method_call_with(ManifestBlobRef([3; 32]))
                .yield_to_parent(())
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::BlobNotRegistered(ManifestBlobRef(blob_ref))) => {
                    assert_eq!(blob_ref, [3; 32]);
                }
            );
        }

        // BucketNotYetCreated(ManifestBucket)
        {
            let transaction_manifest = ManifestBuilder::new_v2()
                .add_test_method_call_with(ManifestBucket(2))
                .build_no_validate();

            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::BucketNotYetCreated(bucket)) => {
                    assert_eq!(bucket, ManifestBucket(2));
                },
            );

            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .add_test_method_call_with(ManifestBucket(3))
                .yield_to_parent(())
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::BucketNotYetCreated(bucket)) => {
                    assert_eq!(bucket, ManifestBucket(3));
                }
            );
        }

        // BucketAlreadyUsed(ManifestBucket, String)
        {
            let transaction_manifest = ManifestBuilder::new_v2()
                .take_all_from_worktop(XRD, "reused_bucket")
                .add_test_method_call_with(ManifestBucket(0))
                .add_test_method_call_with(ManifestBucket(0))
                .build_no_validate();

            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::BucketAlreadyUsed(bucket, _)) => {
                    assert_eq!(bucket, ManifestBucket(0));
                },
            );

            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .take_all_from_worktop(XRD, "reused_bucket")
                .add_test_method_call_with(ManifestBucket(0))
                .add_test_method_call_with(ManifestBucket(0))
                .yield_to_parent(())
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::BucketAlreadyUsed(bucket, _)) => {
                    assert_eq!(bucket, ManifestBucket(0));
                },
            );
        }

        // BucketConsumedWhilstLockedByProof(ManifestBucket, String)
        {
            let transaction_manifest = ManifestBuilder::new_v2()
                .take_all_from_worktop(XRD, "my_bucket")
                .create_proof_from_bucket_of_all("my_bucket", "my_proof")
                .deposit(account_address, "my_bucket")
                .build_no_validate();

            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::BucketConsumedWhilstLockedByProof(bucket, _)) => {
                    assert_eq!(bucket, ManifestBucket(0));
                },
            );

            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .take_all_from_worktop(XRD, "my_bucket")
                .create_proof_from_bucket_of_all("my_bucket", "my_proof")
                .deposit(account_address, "my_bucket")
                .yield_to_parent(())
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::BucketConsumedWhilstLockedByProof(bucket, _)) => {
                    assert_eq!(bucket, ManifestBucket(0));
                },
            );
        }

        // ProofNotYetCreated(ManifestProof)
        {
            let transaction_manifest = ManifestBuilder::new_v2()
                .add_test_method_call_with(ManifestProof(2))
                .build_no_validate();

            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::ProofNotYetCreated(proof)) => {
                    assert_eq!(proof, ManifestProof(2));
                },
            );

            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .add_test_method_call_with(ManifestProof(2))
                .yield_to_parent(())
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::ProofNotYetCreated(proof)) => {
                    assert_eq!(proof, ManifestProof(2));
                },
            );
        }

        // ProofAlreadyUsed(ManifestProof, String)
        {
            let transaction_manifest = ManifestBuilder::new_v2()
                .create_proof_from_auth_zone_of_all(XRD, "proof")
                .add_test_method_call_with(ManifestProof(0))
                .add_test_method_call_with(ManifestProof(0))
                .build_no_validate();

            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::ProofAlreadyUsed(proof, _)) => {
                    assert_eq!(proof, ManifestProof(0));
                },
            );

            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .create_proof_from_auth_zone_of_all(XRD, "proof")
                .add_test_method_call_with(ManifestProof(0))
                .add_test_method_call_with(ManifestProof(0))
                .yield_to_parent(())
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::ProofAlreadyUsed(proof, _)) => {
                    assert_eq!(proof, ManifestProof(0));
                },
            );
        }

        // AddressReservationNotYetCreated(ManifestAddressReservation)
        {
            let transaction_manifest = ManifestBuilder::new_v2()
                .add_test_method_call_with(ManifestAddressReservation(2))
                .build_no_validate();

            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::AddressReservationNotYetCreated(reservation)) => {
                    assert_eq!(reservation, ManifestAddressReservation(2));
                },
            );

            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .add_test_method_call_with(ManifestAddressReservation(2))
                .yield_to_parent(())
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::AddressReservationNotYetCreated(reservation)) => {
                    assert_eq!(reservation, ManifestAddressReservation(2));
                },
            );
        }

        // AddressReservationAlreadyUsed(ManifestAddressReservation, String)
        {
            let transaction_manifest = ManifestBuilder::new_v2()
                .allocate_global_address(
                    RESOURCE_PACKAGE,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    "my_address_reservation",
                    "my_address",
                )
                .add_test_method_call_with(ManifestAddressReservation(0))
                .add_test_method_call_with(ManifestAddressReservation(0))
                .build_no_validate();

            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::AddressReservationAlreadyUsed(reservation, _)) => {
                    assert_eq!(reservation, ManifestAddressReservation(0));
                },
            );

            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .allocate_global_address(
                    RESOURCE_PACKAGE,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    "my_address_reservation",
                    "my_address",
                )
                .add_test_method_call_with(ManifestAddressReservation(0))
                .add_test_method_call_with(ManifestAddressReservation(0))
                .yield_to_parent(())
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::AddressReservationAlreadyUsed(reservation, _)) => {
                    assert_eq!(reservation, ManifestAddressReservation(0));
                },
            );
        }

        // NamedAddressNotYetCreated(ManifestNamedAddress)
        {
            let transaction_manifest = ManifestBuilder::new_v2()
                .add_test_method_call_with(ManifestAddress::Named(ManifestNamedAddress(2)))
                .build_no_validate();

            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::NamedAddressNotYetCreated(named_address)) => {
                    assert_eq!(named_address, ManifestNamedAddress(2));
                },
            );

            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .add_test_method_call_with(ManifestAddress::Named(ManifestNamedAddress(2)))
                .yield_to_parent(())
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::NamedAddressNotYetCreated(named_address)) => {
                    assert_eq!(named_address, ManifestNamedAddress(2));
                },
            );
        }

        // ChildIntentNotRegistered(ManifestNamedIntent)
        {
            let transaction_manifest = ManifestBuilder::new_v2()
                .add_raw_instruction_ignoring_all_side_effects(YieldToChild {
                    child_index: ManifestNamedIntentIndex(2),
                    args: ManifestValue::unit(),
                })
                .build_no_validate();

            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::ChildIntentNotRegistered(named_intent)) => {
                    assert_eq!(named_intent, ManifestNamedIntent(2));
                },
            );

            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .add_raw_instruction_ignoring_all_side_effects(YieldToChild {
                    child_index: ManifestNamedIntentIndex(3),
                    args: ManifestValue::unit(),
                })
                .yield_to_parent(())
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::ChildIntentNotRegistered(named_intent)) => {
                    assert_eq!(named_intent, ManifestNamedIntent(3));
                },
            );
        }

        // DanglingBucket(ManifestBucket, String)
        {
            let transaction_manifest = ManifestBuilder::new_v2()
                .take_all_from_worktop(XRD, "my_bucket")
                .build_no_validate();

            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::DanglingBucket(bucket, _)) => {
                    assert_eq!(bucket, ManifestBucket(0));
                },
            );

            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .take_all_from_worktop(XRD, "my_bucket")
                .yield_to_parent(())
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::DanglingBucket(bucket, _)) => {
                    assert_eq!(bucket, ManifestBucket(0));
                },
            );
        }

        // DanglingAddressReservation(ManifestAddressReservation, String)
        {
            let transaction_manifest = ManifestBuilder::new_v2()
                .allocate_global_address(
                    RESOURCE_PACKAGE,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    "my_address_reservation",
                    "my_address",
                )
                .build_no_validate();

            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::DanglingAddressReservation(reservation, _)) => {
                    assert_eq!(reservation, ManifestAddressReservation(0));
                },
            );

            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .allocate_global_address(
                    RESOURCE_PACKAGE,
                    FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    "my_address_reservation",
                    "my_address",
                )
                .yield_to_parent(())
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::DanglingAddressReservation(reservation, _)) => {
                    assert_eq!(reservation, ManifestAddressReservation(0));
                },
            );
        }

        // ArgsEncodeError(EncodeError)
        {
            // Hard to create when coming from a prepared transaction, because the values
            // come from being decoded
        }

        // ArgsDecodeError(DecodeError)
        {
            // Hard to create when coming from a prepared transaction, because the values
            // come from being decoded
        }

        // InstructionNotSupportedInTransactionIntent
        {
            // YIELD_TO_PARENT
            let transaction_manifest = ManifestBuilder::new_v2()
                .add_raw_instruction_ignoring_all_side_effects(YieldToParent {
                    args: ManifestValue::unit(),
                })
                .build_no_validate();

            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::InstructionNotSupportedInTransactionIntent),
            );

            // VERIFY_PARENT
            let transaction_manifest = ManifestBuilder::new_v2()
                .add_raw_instruction_ignoring_all_side_effects(VerifyParent {
                    access_rule: rule!(allow_all),
                })
                .build_no_validate();
            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::InstructionNotSupportedInTransactionIntent),
            );
        }

        // SubintentDoesNotEndWithYieldToParent
        {
            // CASE 1: At least 1 instruction
            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .add_test_method_call_with(())
                .build_no_validate();

            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::SubintentDoesNotEndWithYieldToParent),
            );

            // CASE 2: No instructions
            let subintent_manifest = ManifestBuilder::new_subintent_v2().build_no_validate();

            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::SubintentDoesNotEndWithYieldToParent),
            );
        }

        // ProofCannotBePassedToAnotherIntent
        {
            let subintent = create_leaf_partial_transaction(0, 0);
            let builder = ManifestBuilder::new_v2();
            let lookup = builder.name_lookup();
            let transaction_manifest = builder
                .use_child("child_1", subintent.root_subintent_hash)
                .create_proof_from_auth_zone_of_all(XRD, "my_proof")
                .yield_to_child("child_1", (lookup.proof("my_proof"),))
                .build_no_validate();
            let builder = TransactionV2Builder::new_with_test_defaults()
                .add_signed_child("child_1", subintent)
                .manifest(transaction_manifest);

            assert_matches!(
                validate_transaction_builder_manifest(builder),
                Err(ManifestValidationError::ProofCannotBePassedToAnotherIntent),
            );

            let builder = ManifestBuilder::new_subintent_v2();
            let lookup = builder.name_lookup();
            let subintent_manifest = builder
                .create_proof_from_auth_zone_of_all(XRD, "my_proof")
                .yield_to_parent((lookup.proof("my_proof"),))
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::ProofCannotBePassedToAnotherIntent),
            );
        }

        // TooManyInstructions
        {
            let mut builder = ManifestBuilder::new_v2();
            for _ in 0..1001 {
                builder = builder.drop_all_proofs();
            }
            let transaction_manifest = builder.build_no_validate();
            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::TooManyInstructions),
            );
            // And test that one less is fine
            let mut builder = ManifestBuilder::new_v2();
            for _ in 0..1000 {
                builder = builder.drop_all_proofs();
            }
            let transaction_manifest = builder.build_no_validate();
            assert_matches!(validate_transaction_manifest(transaction_manifest), Ok(_),);

            let mut builder = ManifestBuilder::new_subintent_v2();
            for _ in 0..1000 {
                // Only 1000 because we're adding a yield_to_parent below
                builder = builder.drop_all_proofs();
            }
            let subintent_manifest = builder.yield_to_parent(()).build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::TooManyInstructions),
            );
            // And test that one less is fine
            let mut builder = ManifestBuilder::new_subintent_v2();
            for _ in 0..999 {
                builder = builder.drop_all_proofs();
            }
            let subintent_manifest = builder.yield_to_parent(()).build_no_validate();
            assert_matches!(validate_subintent_manifest(subintent_manifest), Ok(_),);
        }

        // InvalidResourceConstraint
        {
            // Invalid because there's no overlap between `required_ids` and `allowed_ids`
            let invalid_constraints = ManifestResourceConstraints::new().with_unchecked(
                XRD,
                ManifestResourceConstraint::General(GeneralResourceConstraint {
                    required_ids: indexset!(NonFungibleLocalId::integer(3)),
                    lower_bound: LowerBound::NonZero,
                    upper_bound: UpperBound::Unbounded,
                    allowed_ids: AllowedIds::Allowlist(indexset!(
                        NonFungibleLocalId::integer(4),
                        NonFungibleLocalId::integer(5),
                    )),
                }),
            );

            let transaction_manifest = ManifestBuilder::new_v2()
                .add_raw_instruction_ignoring_all_side_effects(AssertWorktopResourcesOnly {
                    constraints: invalid_constraints.clone(),
                })
                .build_no_validate();

            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::InvalidResourceConstraint),
            );

            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .assert_worktop_resources_only(invalid_constraints.clone())
                .yield_to_parent(())
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::InvalidResourceConstraint),
            );
        }

        // InstructionFollowingNextCallAssertionWasNotInvocation
        {
            let transaction_manifest = ManifestBuilder::new_v2()
                .assert_next_call_returns_include(ManifestResourceConstraints::new())
                .drop_all_proofs() // This is not an invocation
                .add_test_method_call_with(())
                .build_no_validate();
            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::InstructionFollowingNextCallAssertionWasNotInvocation),
            );

            let subintent_manifest = ManifestBuilder::new_subintent_v2()
                .assert_next_call_returns_only(ManifestResourceConstraints::new())
                .drop_auth_zone_signature_proofs()
                .yield_to_parent(())
                .build_no_validate();
            assert_matches!(
                validate_subintent_manifest(subintent_manifest),
                Err(ManifestValidationError::InstructionFollowingNextCallAssertionWasNotInvocation),
            );
        }

        // ManifestEndedWhilstExpectingNextCallAssertion
        {
            let transaction_manifest = ManifestBuilder::new_v2()
                .assert_next_call_returns_include(ManifestResourceConstraints::new())
                .build_no_validate();
            assert_matches!(
                validate_transaction_manifest(transaction_manifest),
                Err(ManifestValidationError::ManifestEndedWhilstExpectingNextCallAssertion),
            );
        }
    }

    #[test]
    fn test_prepare_errors() {
        let babylon_validator = TransactionValidator::new_with_static_config(
            TransactionValidationConfig::babylon(),
            NetworkDefinition::simulator().id,
        );
        let latest_validator = TransactionValidator::new_for_latest_simulator();

        fn create_unvalidated_notarized_transaction_from_manifest(
            manifest: TransactionManifestV2,
        ) -> NotarizedTransactionV2 {
            NotarizedTransactionV2 {
                signed_transaction_intent: SignedTransactionIntentV2 {
                    transaction_intent: TransactionIntentV2 {
                        transaction_header:
                            TransactionV2Builder::testing_default_transaction_header(),
                        root_intent_core: manifest.to_intent_core(
                            TransactionV2Builder::testing_default_intent_header(),
                            MessageV2::None,
                        ),
                        non_root_subintents: NonRootSubintentsV2(vec![]),
                    },
                    transaction_intent_signatures: IntentSignaturesV2::none(),
                    non_root_subintent_signatures: NonRootSubintentSignaturesV2 {
                        by_subintent: vec![],
                    },
                },
                notary_signature: NotarySignatureV2(SignatureV1::Ed25519(Ed25519Signature(
                    [0; Ed25519Signature::LENGTH],
                ))),
            }
        }

        fn create_unvalidated_raw_notarized_transaction_from_manifest(
            manifest: TransactionManifestV2,
        ) -> RawNotarizedTransaction {
            let transaction = create_unvalidated_notarized_transaction_from_manifest(manifest);
            let manually_encoded_transaction = manifest_encode_with_depth_limit(
                &AnyTransaction::NotarizedTransactionV2(transaction),
                100,
            )
            .unwrap();
            RawNotarizedTransaction::from_vec(manually_encoded_transaction)
        }

        // TransactionTypeNotSupported
        {
            let transaction_v2 = TransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .default_notarize()
                .build_minimal_no_validate();

            assert_matches!(
                transaction_v2.prepare_and_validate(&babylon_validator),
                Err(TransactionValidationError::PrepareError(
                    PrepareError::TransactionTypeNotSupported
                )),
            );
        }

        // TransactionTooLarge
        {
            let mut manifest_builder = ManifestBuilder::new_v2();
            manifest_builder.add_blob(vec![0; 1_100_000]);
            let manifest = manifest_builder.build_no_validate();
            let transaction = TransactionV2Builder::new_with_test_defaults()
                .manifest(manifest)
                .default_notarize()
                .build_minimal_no_validate();

            assert_matches!(
                transaction.prepare_and_validate(&latest_validator),
                Err(TransactionValidationError::PrepareError(
                    PrepareError::TransactionTooLarge
                )),
            );
        }

        // EncodeError(EncodeError) and DecodeError(DecodeError)
        // Note that EncodeError doesn't happen as part of preparation in the node, only when preparing from
        // an encodable model. But we can test it here anyway...
        {
            let mut nested_value = ManifestValue::unit();
            for _ in 0..50 {
                nested_value = ManifestValue::tuple([nested_value]);
            }
            let manifest = ManifestBuilder::new_v2()
                .add_raw_instruction_ignoring_all_side_effects(CallMethod {
                    address: XRD.into(),
                    method_name: "method".into(),
                    args: nested_value,
                })
                .build_no_validate();

            let transaction =
                create_unvalidated_notarized_transaction_from_manifest(manifest.clone());

            // We get an EncodeError when preparing directly from the model
            assert_matches!(
                transaction.prepare_and_validate(&latest_validator),
                Err(TransactionValidationError::PrepareError(
                    PrepareError::EncodeError(EncodeError::MaxDepthExceeded(24))
                )),
            );

            // We get a DecodeError when preparing directly from the raw transaction
            let raw_transaction =
                create_unvalidated_raw_notarized_transaction_from_manifest(manifest);
            assert_matches!(
                raw_transaction.validate(&latest_validator),
                Err(TransactionValidationError::PrepareError(
                    PrepareError::DecodeError(DecodeError::MaxDepthExceeded(24))
                )),
            );
        }

        // TooManyValues { value_type: ValueType, actual: usize, max: usize, }
        {
            // Blob
            {
                let mut manifest_builder = ManifestBuilder::new_v2();
                for i in 0..65 {
                    manifest_builder.add_blob(vec![0; i as usize]);
                }
                let transaction = create_unvalidated_notarized_transaction_from_manifest(
                    manifest_builder.build_no_validate(),
                );

                assert_matches!(
                    transaction.prepare_and_validate(&latest_validator),
                    Err(TransactionValidationError::PrepareError(
                        PrepareError::TooManyValues {
                            value_type: ValueType::Blob,
                            actual: 65,
                            max: 64,
                        }
                    )),
                );
            }

            // Subintent
            {
                let mut transaction = TransactionV2Builder::new_with_test_defaults()
                    .add_trivial_manifest()
                    .default_notarize()
                    .build_minimal_no_validate();
                let subintents = (0..33)
                    .map(|i| {
                        create_leaf_partial_transaction(i, 0)
                            .partial_transaction
                            .partial_transaction
                            .root_subintent
                    })
                    .collect::<Vec<_>>();
                transaction
                    .signed_transaction_intent
                    .transaction_intent
                    .non_root_subintents = NonRootSubintentsV2(subintents);
                assert_matches!(
                    transaction.prepare_and_validate(&latest_validator),
                    Err(TransactionValidationError::PrepareError(
                        PrepareError::TooManyValues {
                            value_type: ValueType::Subintent,
                            actual: 33,
                            max: 32,
                        }
                    )),
                );
            }

            // ChildSubintentSpecifier
            {
                let mut transaction = TransactionV2Builder::new_with_test_defaults()
                    .add_trivial_manifest()
                    .default_notarize()
                    .build_minimal_no_validate();
                let child_specifiers = (0..33)
                    .map(|i| ChildSubintentSpecifier {
                        hash: SubintentHash::from_bytes([i as u8; Hash::LENGTH]),
                    })
                    .collect();
                transaction
                    .signed_transaction_intent
                    .transaction_intent
                    .root_intent_core
                    .children = ChildSubintentSpecifiersV2 {
                    children: child_specifiers,
                };
                assert_matches!(
                    transaction.prepare_and_validate(&latest_validator),
                    Err(TransactionValidationError::PrepareError(
                        PrepareError::TooManyValues {
                            value_type: ValueType::ChildSubintentSpecifier,
                            actual: 33,
                            max: 32,
                        }
                    )),
                );
            }

            // SubintentSignatureBatches
            {
                let mut transaction = TransactionV2Builder::new_with_test_defaults()
                    .add_trivial_manifest()
                    .default_notarize()
                    .build_minimal_no_validate();
                let subintent_signature_batches = (0..33)
                    .map(|_| IntentSignaturesV2::none())
                    .collect::<Vec<_>>();
                transaction
                    .signed_transaction_intent
                    .non_root_subintent_signatures = NonRootSubintentSignaturesV2 {
                    by_subintent: subintent_signature_batches,
                };
                assert_matches!(
                    transaction.prepare_and_validate(&latest_validator),
                    Err(TransactionValidationError::PrepareError(
                        PrepareError::TooManyValues {
                            value_type: ValueType::SubintentSignatureBatches,
                            actual: 33,
                            max: 32,
                        }
                    )),
                );
            }
        }

        // LengthOverflow
        // -> Rather hard to test, we can leave this.

        // UnexpectedTransactionDiscriminator
        {
            let raw_transaction = TransactionV2Builder::new_with_test_defaults()
                .add_trivial_manifest()
                .default_notarize()
                .build_minimal_no_validate()
                .to_raw()
                .unwrap();

            let mut amended_payload = raw_transaction.to_vec();
            amended_payload[2] = 4;
            let amended_raw = RawNotarizedTransaction::from_vec(amended_payload);
            assert_eq!(
                amended_raw.validate(&latest_validator),
                Err(TransactionValidationError::PrepareError(
                    PrepareError::UnexpectedTransactionDiscriminator { actual: Some(4) }
                ))
            )
        }
    }
}
