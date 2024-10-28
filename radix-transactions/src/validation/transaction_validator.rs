use core::ops::ControlFlow;

use radix_substate_store_interface::interface::{SubstateDatabase, SubstateDatabaseExtensions};

use crate::internal_prelude::*;

pub trait TransactionPreparer {
    fn preparation_settings(&self) -> &PreparationSettings;
}

define_single_versioned! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
    pub TransactionValidationConfigurationSubstate(TransactionValidationConfigurationVersions) => TransactionValidationConfig = TransactionValidationConfigV1,
    outer_attributes: [
        #[derive(ScryptoSborAssertion)]
        #[sbor_assert(backwards_compatible(
            cuttlefish = "FILE:transaction_validation_configuration_substate_cuttlefish_schema.bin",
        ))]
    ]
}

impl TransactionValidationConfig {
    pub fn load(database: &impl SubstateDatabase) -> Self {
        database
            .get_substate::<TransactionValidationConfigurationSubstate>(
                TRANSACTION_TRACKER,
                BOOT_LOADER_PARTITION,
                BootLoaderField::TransactionValidationConfiguration,
            )
            .map(|s| s.fully_update_and_into_latest_version())
            .unwrap_or_else(|| Self::babylon())
    }

    fn allow_notary_to_duplicate_signer(&self, version: TransactionVersion) -> bool {
        match version {
            TransactionVersion::V1 => self.v1_transactions_allow_notary_to_duplicate_signer,
            TransactionVersion::V2 => false,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
pub struct TransactionValidationConfigV1 {
    /// Signer signatures only, not including notary signature
    pub max_signer_signatures_per_intent: usize,
    pub max_references_per_intent: usize,
    pub min_tip_percentage: u16,
    pub max_tip_percentage: u16,
    pub max_epoch_range: u64,
    pub max_instructions: usize,
    pub message_validation: MessageValidationConfig,
    pub v1_transactions_allow_notary_to_duplicate_signer: bool,
    pub preparation_settings: PreparationSettingsV1,
    pub manifest_validation: ManifestValidationRuleset,
    // V2 settings
    pub v2_transactions_allowed: bool,
    pub min_tip_basis_points: u32,
    pub max_tip_basis_points: u32,
    /// A setting of N here allows a total depth of N + 1 if you
    /// include the root transaction intent.
    pub max_subintent_depth: usize,
    pub max_total_signature_validations: usize,
    pub max_total_references: usize,
}

impl TransactionValidationConfig {
    pub const fn latest() -> Self {
        Self::cuttlefish()
    }

    pub const fn babylon() -> Self {
        Self {
            max_signer_signatures_per_intent: 16,
            max_references_per_intent: usize::MAX,
            min_tip_percentage: 0,
            max_tip_percentage: u16::MAX,
            max_instructions: usize::MAX,
            // ~30 days given 5 minute epochs
            max_epoch_range: 12 * 24 * 30,
            v1_transactions_allow_notary_to_duplicate_signer: true,
            manifest_validation: ManifestValidationRuleset::BabylonBasicValidator,
            message_validation: MessageValidationConfig::babylon(),
            preparation_settings: PreparationSettings::babylon(),
            // V2-only settings
            v2_transactions_allowed: true,
            max_subintent_depth: 0,
            min_tip_basis_points: 0,
            max_tip_basis_points: 0,
            max_total_signature_validations: usize::MAX,
            max_total_references: usize::MAX,
        }
    }

    pub const fn cuttlefish() -> Self {
        Self {
            max_references_per_intent: 512,
            v2_transactions_allowed: true,
            max_subintent_depth: 3,
            min_tip_basis_points: 0,
            max_instructions: 1000,
            manifest_validation: ManifestValidationRuleset::Interpreter(
                InterpreterValidationRulesetSpecifier::Cuttlefish,
            ),
            // Tip of 100 times the cost of a transaction
            max_tip_basis_points: 100 * 10000,
            preparation_settings: PreparationSettings::cuttlefish(),
            max_total_signature_validations: 64,
            max_total_references: 512,
            ..Self::babylon()
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
pub enum ManifestValidationRuleset {
    BabylonBasicValidator,
    Interpreter(InterpreterValidationRulesetSpecifier),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Sbor)]
pub struct MessageValidationConfig {
    pub max_plaintext_message_length: usize,
    pub max_encrypted_message_length: usize,
    pub max_mime_type_length: usize,
    pub max_decryptors: usize,
}

impl MessageValidationConfig {
    pub const fn latest() -> Self {
        Self::babylon()
    }

    pub const fn babylon() -> Self {
        Self {
            max_plaintext_message_length: 2048,
            max_mime_type_length: 128,
            max_encrypted_message_length: 2048 + 12 + 16, // Account for IV and MAC - see AesGcmPayload
            max_decryptors: 20,
        }
    }
}

impl Default for MessageValidationConfig {
    fn default() -> Self {
        Self::latest()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TransactionValidator {
    config: TransactionValidationConfig,
    required_network_id: Option<u8>,
}

impl TransactionValidator {
    /// This is the best constructor to use, as it reads the configuration dynamically
    /// Note that the validator needs recreating every time a protocol update runs,
    /// as the config can get updated then.
    pub fn new(database: &impl SubstateDatabase, network_definition: &NetworkDefinition) -> Self {
        Self::new_with_static_config(
            TransactionValidationConfig::load(database),
            network_definition.id,
        )
    }

    pub fn new_for_latest_simulator() -> Self {
        Self::new_with_static_config(
            TransactionValidationConfig::latest(),
            NetworkDefinition::simulator().id,
        )
    }

    pub fn new_with_latest_config(network_definition: &NetworkDefinition) -> Self {
        Self::new_with_static_config(TransactionValidationConfig::latest(), network_definition.id)
    }

    pub fn new_with_static_config(config: TransactionValidationConfig, network_id: u8) -> Self {
        Self {
            config,
            required_network_id: Some(network_id),
        }
    }

    pub fn new_with_latest_config_network_agnostic() -> Self {
        Self::new_with_static_config_network_agnostic(TransactionValidationConfig::latest())
    }

    pub fn new_with_static_config_network_agnostic(config: TransactionValidationConfig) -> Self {
        Self {
            config,
            required_network_id: None,
        }
    }

    /// Will typically be [`Some`], but [`None`] if the validator is network-independent.
    pub fn network_id(&self) -> Option<u8> {
        self.required_network_id
    }

    pub fn config(&self) -> &TransactionValidationConfig {
        &self.config
    }

    pub fn preparation_settings(&self) -> &PreparationSettings {
        &self.config.preparation_settings
    }

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
                    .clone(),
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

    #[allow(deprecated)]
    fn validate_intent_v1(
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
        impl<'a> ReadableManifestBase for (&'a [InstructionV1], &'a IndexMap<Hash, Vec<u8>>) {
            fn is_subintent(&self) -> bool {
                false
            }

            fn get_blobs<'b>(&'b self) -> impl Iterator<Item = (&'b Hash, &'b Vec<u8>)> {
                self.1.iter()
            }

            fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef {
                ManifestObjectNamesRef::Unknown
            }
        }
        impl<'a> TypedReadableManifest for (&'a [InstructionV1], &'a IndexMap<Hash, Vec<u8>>) {
            type Instruction = InstructionV1;

            fn get_typed_instructions(&self) -> &[Self::Instruction] {
                self.0
            }
        }

        match self.config.manifest_validation {
            ManifestValidationRuleset::BabylonBasicValidator => self
                .validate_instructions_basic_v1(instructions)
                .map_err(|err| err.into()),
            ManifestValidationRuleset::Interpreter(specifier) => StaticManifestInterpreter::new(
                ValidationRuleset::for_specifier(specifier),
                &(instructions, blobs),
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

    pub fn validate_notarized_v2(
        &self,
        prepared: PreparedNotarizedTransactionV2,
    ) -> Result<ValidatedNotarizedTransactionV2, TransactionValidationError> {
        if !self.config.v2_transactions_allowed {
            return Err(TransactionValidationError::TransactionVersionNotPermitted(
                2,
            ));
        }

        let transaction_intent = &prepared.signed_intent.transaction_intent;
        let non_root_subintents = &transaction_intent.non_root_subintents;

        self.validate_transaction_header_v2(&transaction_intent.transaction_header.inner)
            .map_err(|err| {
                TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::RootTransactionIntent(
                        transaction_intent.transaction_intent_hash(),
                    ),
                    IntentValidationError::HeaderValidationError(err),
                )
            })?;

        let mut signatures = AllPendingSignatureValidations::new_with_root(
            TransactionVersion::V2,
            &self.config,
            transaction_intent.transaction_intent_hash().into(),
            PendingIntentSignatureValidations::TransactionIntent {
                notary_is_signatory: transaction_intent
                    .transaction_header
                    .inner
                    .notary_is_signatory,
                notary_public_key: transaction_intent
                    .transaction_header
                    .inner
                    .notary_public_key,
                notary_signature: prepared.notary_signature.inner.0,
                notarized_hash: prepared.signed_transaction_intent_hash(),
                intent_signatures: prepared
                    .signed_intent
                    .transaction_intent_signatures
                    .inner
                    .signatures
                    .clone(),
                signed_hash: transaction_intent.transaction_intent_hash(),
            },
        )?;
        signatures.add_non_root_subintents_v2(
            non_root_subintents,
            &prepared.signed_intent.non_root_subintent_signatures,
        )?;

        let ValidatedPartialTransactionTreeV2 {
            overall_validity_range,
            total_signature_validations,
            root_intent_info,
            root_yield_to_parent_count: _, // Checked to be 0 in the manifest validator.
            non_root_subintents_info,
        } = self.validate_transaction_subtree_v2(
            &transaction_intent.root_intent_core,
            transaction_intent.transaction_intent_hash().into(),
            non_root_subintents,
            signatures,
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
        if !self.config.v2_transactions_allowed {
            return Err(TransactionValidationError::TransactionVersionNotPermitted(
                2,
            ));
        }

        let transaction_intent = &prepared.transaction_intent;
        let non_root_subintents = &transaction_intent.non_root_subintents;

        self.validate_transaction_header_v2(&transaction_intent.transaction_header.inner)
            .map_err(|err| {
                TransactionValidationError::IntentValidationError(
                    TransactionValidationErrorLocation::RootTransactionIntent(
                        transaction_intent.transaction_intent_hash(),
                    ),
                    IntentValidationError::HeaderValidationError(err),
                )
            })?;

        let mut signatures = AllPendingSignatureValidations::new_with_root(
            TransactionVersion::V2,
            &self.config,
            transaction_intent.transaction_intent_hash().into(),
            PendingIntentSignatureValidations::PreviewTransactionIntent {
                notary_is_signatory: transaction_intent
                    .transaction_header
                    .inner
                    .notary_is_signatory,
                notary_public_key: transaction_intent
                    .transaction_header
                    .inner
                    .notary_public_key,
                intent_public_keys: prepared.root_subintent_signatures.inner.clone(),
            },
        )?;
        signatures.add_non_root_preview_subintents_v2(
            non_root_subintents,
            &prepared.non_root_subintent_signatures.inner,
        )?;

        let ValidatedPartialTransactionTreeV2 {
            overall_validity_range,
            total_signature_validations: total_expected_signature_validations,
            root_intent_info,
            root_yield_to_parent_count: _, // Checked to be 0 in the manifest validator.
            non_root_subintents_info,
        } = self.validate_transaction_subtree_v2(
            &transaction_intent.root_intent_core,
            transaction_intent.transaction_intent_hash().into(),
            non_root_subintents,
            signatures,
        )?;

        Ok(ValidatedPreviewTransactionV2 {
            prepared,
            overall_validity_range,
            total_expected_signature_validations,
            transaction_intent_info: root_intent_info,
            non_root_subintents_info,
        })
    }

    fn validate_transaction_header_v2(
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
        if !self.config.v2_transactions_allowed {
            return Err(TransactionValidationError::TransactionVersionNotPermitted(
                2,
            ));
        }

        let root_subintent = &prepared.partial_transaction.root_subintent;
        let root_intent_hash: IntentHash = root_subintent.subintent_hash().into();
        let non_root_subintents = &prepared.partial_transaction.non_root_subintents;

        let mut signatures = AllPendingSignatureValidations::new_with_root(
            TransactionVersion::V2,
            &self.config,
            root_intent_hash,
            PendingIntentSignatureValidations::Subintent {
                intent_signatures: prepared.root_subintent_signatures.inner.signatures.clone(),
                signed_hash: root_subintent.subintent_hash(),
            },
        )?;
        signatures.add_non_root_subintents_v2(
            non_root_subintents,
            &prepared.non_root_subintent_signatures,
        )?;

        let ValidatedPartialTransactionTreeV2 {
            overall_validity_range,
            root_intent_info,
            root_yield_to_parent_count,
            non_root_subintents_info,
            total_signature_validations,
        } = self.validate_transaction_subtree_v2(
            &root_subintent.intent_core,
            root_subintent.subintent_hash().into(),
            non_root_subintents,
            signatures,
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

    pub fn validate_transaction_subtree_v2(
        &self,
        root_intent_core: &PreparedIntentCoreV2,
        root_intent_hash: IntentHash,
        non_root_subintents: &PreparedNonRootSubintentsV2,
        signatures: AllPendingSignatureValidations,
    ) -> Result<ValidatedPartialTransactionTreeV2, TransactionValidationError> {
        let non_root_subintents = non_root_subintents.subintents.as_slice();

        let intent_relationships = self.validate_intent_relationships_v2(
            root_intent_hash,
            root_intent_core,
            non_root_subintents,
        )?;
        let (overall_validity_range, root_yield_summary) = self
            .validate_v2_intent_cores_and_subintent_connection_counts(
                root_intent_hash,
                root_intent_core,
                non_root_subintents,
                &intent_relationships.non_root_subintents,
            )?;

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

        Ok(ValidatedPartialTransactionTreeV2 {
            overall_validity_range,
            root_intent_info,
            root_yield_to_parent_count: root_yield_summary.parent_yields,
            non_root_subintents_info,
            total_signature_validations,
        })
    }

    /// Can be used for both partial and complete trees.
    ///
    /// This should be run after `validate_intent_relationships_v2`, which creates
    /// the subintent details map for you which describes the relationship between
    /// different intents.
    fn validate_v2_intent_cores_and_subintent_connection_counts(
        &self,
        root_intent_hash: IntentHash,
        root_intent_core: &PreparedIntentCoreV2,
        non_root_subintents: &[PreparedSubintentV2],
        non_root_subintent_details: &IndexMap<SubintentHash, SubintentRelationshipDetails>,
    ) -> Result<(OverallValidityRangeV2, ManifestYieldSummary), TransactionValidationError> {
        let mut aggregation = AcrossIntentAggregation::start();
        let mut yield_summaries: IndexMap<IntentHash, ManifestYieldSummary> =
            index_map_with_capacity(non_root_subintents.len() + 1);
        let root_yield_summary = {
            let yield_summary = self
                .validate_v2_intent_core(
                    root_intent_core,
                    &mut aggregation,
                    root_intent_hash.is_for_subintent(),
                )
                .map_err(|err| {
                    TransactionValidationError::IntentValidationError(
                        TransactionValidationErrorLocation::for_root(root_intent_hash),
                        err,
                    )
                })?;
            yield_summaries.insert(root_intent_hash, yield_summary.clone());
            yield_summary
        };
        for (index, subintent) in non_root_subintents.iter().enumerate() {
            let subintent_hash = subintent.subintent_hash();
            let yield_summary = self
                .validate_v2_intent_core(&subintent.intent_core, &mut aggregation, true)
                .map_err(|err| {
                    TransactionValidationError::IntentValidationError(
                        TransactionValidationErrorLocation::NonRootSubintent(
                            SubintentIndex(index),
                            subintent_hash,
                        ),
                        err,
                    )
                })?;
            yield_summaries.insert(subintent_hash.into(), yield_summary);
        }

        let overall_validity_range = aggregation.finalize(&self.config)?;

        for (child_hash, child_details) in non_root_subintent_details {
            let child_intent_hash = IntentHash::Subintent(*child_hash);
            // This checks that the YIELD_TO_PARENTs in a subintent match the YIELD_TO_CHILDS in the parent.
            // The instruction validation has already checked that the subintents end with a YIELD_TO_PARENT.
            let parent_yield_summary = yield_summaries.get(&child_details.parent).unwrap();
            let parent_yield_child_calls =
                *parent_yield_summary.child_yields.get(child_hash).unwrap();
            let child_yield_summary = yield_summaries.get(&child_intent_hash).unwrap();
            let child_yield_parent_calls = child_yield_summary.parent_yields;
            if parent_yield_child_calls != child_yield_parent_calls {
                return Err(
                    SubintentStructureError::MismatchingYieldChildAndYieldParentCountsForSubintent
                        .for_subintent(child_details.index, *child_hash),
                );
            }
        }

        Ok((overall_validity_range, root_yield_summary))
    }

    fn validate_v2_intent_core(
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

    fn validate_intent_header_v2(
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

    fn validate_message_v2(&self, message: &MessageV2) -> Result<(), InvalidMessageError> {
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

    /// The `is_subintent` property indicates whether it should be treated as a subintent.
    /// A subintent is able to `YIELD_TO_PARENT` and is required to end with a `YIELD_TO_PARENT`.
    fn validate_manifest_v2(
        &self,
        instructions: &[InstructionV2],
        blobs: &IndexMap<Hash, Vec<u8>>,
        children: &IndexSet<ChildSubintentSpecifier>,
        is_subintent: bool,
    ) -> Result<ManifestYieldSummary, ManifestValidationError> {
        if instructions.len() > self.config.max_instructions {
            return Err(ManifestValidationError::TooManyInstructions);
        }
        impl<'a> ReadableManifestBase
            for (
                &'a [InstructionV2],
                &'a IndexMap<Hash, Vec<u8>>,
                &'a IndexSet<ChildSubintentSpecifier>,
                bool,
            )
        {
            fn is_subintent(&self) -> bool {
                self.3
            }

            fn get_blobs<'b>(&'b self) -> impl Iterator<Item = (&'b Hash, &'b Vec<u8>)> {
                self.1.iter()
            }

            fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef {
                ManifestObjectNamesRef::Unknown
            }

            fn get_child_subintent_hashes<'b>(
                &'b self,
            ) -> impl ExactSizeIterator<Item = &'b ChildSubintentSpecifier> {
                self.2.iter()
            }
        }
        impl<'a> TypedReadableManifest
            for (
                &'a [InstructionV2],
                &'a IndexMap<Hash, Vec<u8>>,
                &'a IndexSet<ChildSubintentSpecifier>,
                bool,
            )
        {
            type Instruction = InstructionV2;

            fn get_typed_instructions(&self) -> &[Self::Instruction] {
                self.0
            }
        }
        let mut yield_summary = ManifestYieldSummary {
            parent_yields: 0,
            child_yields: children.iter().map(|child| (child.hash, 0)).collect(),
        };
        StaticManifestInterpreter::new(
            ValidationRuleset::cuttlefish(),
            &(instructions, blobs, children, is_subintent),
        )
        .validate_and_apply_visitor(&mut yield_summary)?;
        Ok(yield_summary)
    }

    /// The root intent can be either:
    /// * If validating a full transaction: a transaction intent
    /// * If validating a partial transaction: a root subintent
    fn validate_intent_relationships_v2(
        &self,
        root_intent_hash: IntentHash,
        root_intent_core: &PreparedIntentCoreV2,
        non_root_subintents: &[PreparedSubintentV2],
    ) -> Result<IntentRelationships, TransactionValidationError> {
        let mut root_intent_details = RootIntentRelationshipDetails::default();
        let mut non_root_subintent_details =
            IndexMap::<SubintentHash, SubintentRelationshipDetails>::default();

        // STEP 1
        // ------
        // * We establish that the subintents are unique
        // * We create an index from the SubintentHash to SubintentIndex
        for (index, subintent) in non_root_subintents.iter().enumerate() {
            let subintent_hash = subintent.subintent_hash();
            let index = SubintentIndex(index);
            let details = SubintentRelationshipDetails::default_for(index);
            if let Some(_) = non_root_subintent_details.insert(subintent_hash, details) {
                return Err(SubintentStructureError::DuplicateSubintent
                    .for_subintent(index, subintent_hash));
            }
        }

        // STEP 2
        // ------
        // We establish, for each parent intent, that each of its children:
        // * Exist as subintents in the transaction
        // * Only is the child of that parent intent and no other
        //
        // We also:
        // * Save the unique parent on each subintent which is a child
        // * Save the children of an intent into its intent details

        // STEP 2A - Handle children of the transaction intent
        {
            let parent_hash = root_intent_hash;
            let intent_details = &mut root_intent_details;
            for child_subintent_hash in root_intent_core.children.children.iter() {
                let child_hash = child_subintent_hash.hash;
                let child_subintent_details = non_root_subintent_details
                    .get_mut(&child_hash)
                    .ok_or_else(|| {
                        SubintentStructureError::ChildSubintentNotIncludedInTransaction(child_hash)
                            .for_unindexed()
                    })?;
                if child_subintent_details.parent == PLACEHOLDER_PARENT {
                    child_subintent_details.parent = parent_hash;
                } else {
                    return Err(SubintentStructureError::SubintentHasMultipleParents
                        .for_subintent(child_subintent_details.index, child_hash));
                }
                intent_details.children.push(child_subintent_details.index);
            }
        }

        // STEP 2B - Handle the children of each subintent
        for subintent in non_root_subintents.iter() {
            let subintent_hash = subintent.subintent_hash();
            let parent_hash: IntentHash = subintent_hash.into();
            let children = &subintent.intent_core.children.children;
            let mut children_details = Vec::with_capacity(children.len());
            for child_subintent in children.iter() {
                let child_hash = child_subintent.hash;
                let child_subintent_details = non_root_subintent_details
                    .get_mut(&child_hash)
                    .ok_or_else(|| {
                        SubintentStructureError::ChildSubintentNotIncludedInTransaction(child_hash)
                            .for_unindexed()
                    })?;
                if child_subintent_details.parent == PLACEHOLDER_PARENT {
                    child_subintent_details.parent = parent_hash;
                } else {
                    return Err(SubintentStructureError::SubintentHasMultipleParents
                        .for_subintent(child_subintent_details.index, child_hash));
                }
                children_details.push(child_subintent_details.index);
            }
            non_root_subintent_details
                .get_mut(&subintent_hash)
                .unwrap()
                .children = children_details;
        }

        // STEP 3
        // ------
        // We traverse the child relationships from the root, and mark a depth.
        // We error if any exceed the maximum depth.
        //
        // As each child has at most one parent, we can guarantee the work is bounded
        // by the total number of subintents.
        let mut work_list = vec![];
        for index in root_intent_details.children.iter() {
            work_list.push((*index, 1));
        }

        let max_depth = if root_intent_hash.is_for_subintent() {
            self.config.max_subintent_depth - 1
        } else {
            self.config.max_subintent_depth
        };

        loop {
            let Some((index, depth)) = work_list.pop() else {
                break;
            };
            if depth > max_depth {
                let (hash, _) = non_root_subintent_details.get_index(index.0).unwrap();
                return Err(
                    SubintentStructureError::SubintentExceedsMaxDepth.for_subintent(index, *hash)
                );
            }
            let (_, subintent_details) = non_root_subintent_details.get_index_mut(index.0).unwrap();
            subintent_details.depth = depth;
            for index in subintent_details.children.iter() {
                work_list.push((*index, depth + 1));
            }
        }

        // STEP 4
        // ------
        // We check that every subintent has a marked "depth from root".
        //
        // Combined with step 2 and step 3, we now have that:
        // * Every subintent has a unique parent.
        // * Every subintent is reachable from the root.
        //
        // Therefore there is a unique path from every subintent to the root
        // So we have confirmed the subintents form a tree.
        for (hash, details) in non_root_subintent_details.iter() {
            if details.depth == 0 {
                return Err(
                    SubintentStructureError::SubintentIsNotReachableFromTheTransactionIntent
                        .for_subintent(details.index, *hash),
                );
            }
        }

        Ok(IntentRelationships {
            root_intent: root_intent_details,
            non_root_subintents: non_root_subintent_details,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransactionVersion {
    V1,
    V2,
}

#[must_use]
struct AcrossIntentAggregation {
    total_reference_count: usize,
    overall_start_epoch_inclusive: Epoch,
    overall_end_epoch_exclusive: Epoch,
    overall_start_timestamp_inclusive: Option<Instant>,
    overall_end_timestamp_exclusive: Option<Instant>,
}

impl AcrossIntentAggregation {
    fn start() -> Self {
        Self {
            total_reference_count: 0,
            overall_start_epoch_inclusive: Epoch::zero(),
            overall_end_epoch_exclusive: Epoch::of(u64::MAX),
            overall_start_timestamp_inclusive: None,
            overall_end_timestamp_exclusive: None,
        }
    }

    fn finalize(
        self,
        config: &TransactionValidationConfig,
    ) -> Result<OverallValidityRangeV2, TransactionValidationError> {
        if self.total_reference_count > config.max_total_references {
            return Err(TransactionValidationError::IntentValidationError(
                TransactionValidationErrorLocation::AcrossTransaction,
                IntentValidationError::TooManyReferences {
                    total: self.total_reference_count,
                    limit: config.max_total_references,
                },
            ));
        }
        Ok(OverallValidityRangeV2 {
            epoch_range: EpochRange {
                start_epoch_inclusive: self.overall_start_epoch_inclusive,
                end_epoch_exclusive: self.overall_end_epoch_exclusive,
            },
            proposer_timestamp_range: ProposerTimestampRange {
                start_timestamp_inclusive: self.overall_start_timestamp_inclusive,
                end_timestamp_exclusive: self.overall_end_timestamp_exclusive,
            },
        })
    }

    fn record_reference_count(
        &mut self,
        count: usize,
        config: &TransactionValidationConfig,
    ) -> Result<(), IntentValidationError> {
        if count > config.max_references_per_intent {
            return Err(IntentValidationError::TooManyReferences {
                total: count,
                limit: config.max_references_per_intent,
            });
        }
        self.total_reference_count = self.total_reference_count.saturating_add(count);
        Ok(())
    }

    fn update_headers(
        &mut self,
        start_epoch_inclusive: Epoch,
        end_epoch_exclusive: Epoch,
        start_timestamp_inclusive: Option<&Instant>,
        end_timestamp_exclusive: Option<&Instant>,
    ) -> Result<(), HeaderValidationError> {
        if start_epoch_inclusive > self.overall_start_epoch_inclusive {
            self.overall_start_epoch_inclusive = start_epoch_inclusive;
        }
        if end_epoch_exclusive < self.overall_end_epoch_exclusive {
            self.overall_end_epoch_exclusive = end_epoch_exclusive;
        }
        if self.overall_start_epoch_inclusive >= self.overall_end_epoch_exclusive {
            return Err(HeaderValidationError::NoValidEpochRangeAcrossAllIntents);
        }
        if let Some(start_timestamp_inclusive) = start_timestamp_inclusive {
            if self.overall_start_timestamp_inclusive.is_none()
                || self
                    .overall_start_timestamp_inclusive
                    .as_ref()
                    .is_some_and(|t| start_timestamp_inclusive > t)
            {
                self.overall_start_timestamp_inclusive = Some(*start_timestamp_inclusive);
            }
        }
        if let Some(end_timestamp_exclusive) = end_timestamp_exclusive {
            if self.overall_end_timestamp_exclusive.is_none()
                || self
                    .overall_end_timestamp_exclusive
                    .as_ref()
                    .is_some_and(|t| end_timestamp_exclusive < t)
            {
                self.overall_end_timestamp_exclusive = Some(*end_timestamp_exclusive);
            }
        }
        match (
            self.overall_start_timestamp_inclusive.as_ref(),
            self.overall_end_timestamp_exclusive.as_ref(),
        ) {
            (Some(start_inclusive), Some(end_exclusive)) => {
                if start_inclusive >= end_exclusive {
                    return Err(HeaderValidationError::NoValidTimestampRangeAcrossAllIntents);
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub struct AllPendingSignatureValidations<'a> {
    transaction_version: TransactionVersion,
    config: &'a TransactionValidationConfig,
    root: (
        PendingIntentSignatureValidations,
        TransactionValidationErrorLocation,
    ),
    non_roots: Vec<(
        PendingIntentSignatureValidations,
        TransactionValidationErrorLocation,
    )>,
    total_signature_validations: usize,
}

pub struct SignatureValidationSummary {
    root_signer_keys: IndexSet<PublicKey>,
    non_root_signer_keys: Vec<IndexSet<PublicKey>>,
    total_signature_validations: usize,
}

impl<'a> AllPendingSignatureValidations<'a> {
    fn new_with_root(
        transaction_version: TransactionVersion,
        config: &'a TransactionValidationConfig,
        root_intent_hash: IntentHash,
        signatures: PendingIntentSignatureValidations,
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

    pub fn add_non_root_subintents_v2(
        &mut self,
        non_root_subintents: &PreparedNonRootSubintentsV2,
        signatures: &PreparedNonRootSubintentSignaturesV2,
    ) -> Result<(), TransactionValidationError> {
        let non_root_subintents = &non_root_subintents.subintents;
        let non_root_subintent_signatures = &signatures.by_subintent;
        if non_root_subintents.len() != non_root_subintent_signatures.len() {
            return Err(
                SignatureValidationError::IncorrectNumberOfSubintentSignatureBatches
                    .located(TransactionValidationErrorLocation::AcrossTransaction),
            );
        }
        for (index, (subintent, signatures)) in non_root_subintents
            .iter()
            .zip(non_root_subintent_signatures)
            .enumerate()
        {
            self.add_non_root(
                SubintentIndex(index),
                subintent.subintent_hash(),
                PendingIntentSignatureValidations::Subintent {
                    intent_signatures: signatures.inner.signatures.clone(),
                    signed_hash: subintent.subintent_hash(),
                },
            )?;
        }
        Ok(())
    }

    pub fn add_non_root_preview_subintents_v2(
        &mut self,
        non_root_subintents: &PreparedNonRootSubintentsV2,
        non_root_subintent_signers: &Vec<Vec<PublicKey>>,
    ) -> Result<(), TransactionValidationError> {
        let non_root_subintents = &non_root_subintents.subintents;
        if non_root_subintents.len() != non_root_subintent_signers.len() {
            return Err(
                SignatureValidationError::IncorrectNumberOfSubintentSignatureBatches
                    .located(TransactionValidationErrorLocation::AcrossTransaction),
            );
        }
        for (index, (subintent, signatures)) in non_root_subintents
            .iter()
            .zip(non_root_subintent_signers)
            .enumerate()
        {
            self.add_non_root(
                SubintentIndex(index),
                subintent.subintent_hash(),
                PendingIntentSignatureValidations::PreviewSubintent {
                    intent_public_keys: signatures.clone(),
                },
            )?;
        }
        Ok(())
    }

    fn add_non_root(
        &mut self,
        subintent_index: SubintentIndex,
        subintent_hash: SubintentHash,
        signatures: PendingIntentSignatureValidations,
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

    pub fn validate_all(self) -> Result<SignatureValidationSummary, TransactionValidationError> {
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
                    if !checked_intent_public_keys.insert(key) {
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
                    if !checked_intent_public_keys.insert(key) {
                        return Err(SignatureValidationError::DuplicateSigner);
                    }
                }
                checked_intent_public_keys
            }
        };

        Ok(public_keys)
    }
}

/// This can assume that the signature counts are within checked limits,
/// so calculations cannot overflow.
enum PendingIntentSignatureValidations {
    TransactionIntent {
        notary_is_signatory: bool,
        notary_public_key: PublicKey,
        notary_signature: SignatureV1,
        notarized_hash: SignedTransactionIntentHash,
        intent_signatures: Vec<IntentSignatureV1>,
        signed_hash: TransactionIntentHash,
    },
    PreviewTransactionIntent {
        notary_is_signatory: bool,
        notary_public_key: PublicKey,
        intent_public_keys: Vec<PublicKey>,
    },
    Subintent {
        intent_signatures: Vec<IntentSignatureV1>,
        signed_hash: SubintentHash,
    },
    PreviewSubintent {
        intent_public_keys: Vec<PublicKey>,
    },
}

impl PendingIntentSignatureValidations {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ManifestYieldSummary {
    parent_yields: usize,
    child_yields: IndexMap<SubintentHash, usize>,
}

impl ManifestInterpretationVisitor for ManifestYieldSummary {
    type Output = ManifestValidationError;

    fn on_end_instruction(&mut self, details: OnEndInstruction) -> ControlFlow<Self::Output> {
        // Safe from overflow due to checking max instruction count
        match details.effect {
            ManifestInstructionEffect::Invocation {
                kind: InvocationKind::YieldToParent,
                ..
            } => {
                self.parent_yields += 1;
            }
            ManifestInstructionEffect::Invocation {
                kind:
                    InvocationKind::YieldToChild {
                        child_index: ManifestNamedIntent(index),
                    },
                ..
            } => {
                let index = index as usize;

                // This should exist because we are handling this after the instruction,
                // so the interpreter should have errored with ChildIntentNotRegistered
                // if the child yield was invalid.
                let (_, count) = self.child_yields.get_index_mut(index).unwrap();
                *count += 1;
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}

struct IntentRelationships {
    pub root_intent: RootIntentRelationshipDetails,
    pub non_root_subintents: IndexMap<SubintentHash, SubintentRelationshipDetails>,
}
#[derive(Default)]
pub struct RootIntentRelationshipDetails {
    children: Vec<SubintentIndex>,
}

pub struct SubintentRelationshipDetails {
    index: SubintentIndex,
    parent: IntentHash,
    depth: usize,
    children: Vec<SubintentIndex>,
}

impl SubintentRelationshipDetails {
    fn default_for(index: SubintentIndex) -> Self {
        Self {
            index,
            parent: PLACEHOLDER_PARENT,
            depth: Default::default(),
            children: Default::default(),
        }
    }
}

const PLACEHOLDER_PARENT: IntentHash =
    IntentHash::Transaction(TransactionIntentHash(Hash([0u8; Hash::LENGTH])));

#[cfg(test)]
mod tests {
    use std::ops::AddAssign;

    use radix_common::network::NetworkDefinition;

    use super::*;
    use crate::{builder::ManifestBuilder, builder::TransactionBuilder};

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

        let transaction_v2 = TransactionV2Builder::new()
            .intent_header(IntentHeaderV2 {
                network_id: network.id,
                start_epoch_inclusive: Epoch::of(1),
                end_epoch_exclusive: Epoch::of(10),
                min_proposer_timestamp_inclusive: None,
                max_proposer_timestamp_exclusive: None,
                intent_discriminator: 0,
            })
            .transaction_header(TransactionHeaderV2 {
                notary_public_key: notary.public_key().into(),
                notary_is_signatory: true,
                tip_basis_points: 0,
            })
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
                SignatureV1::Secp256k1(inner_signature) => inner_signature.0[5].add_assign(1),
                SignatureV1::Ed25519(inner_signature) => inner_signature.0[5].add_assign(1),
            }
            signature
        }

        fn sign_with_public_key(&self, message_hash: &impl IsHash) -> SignatureWithPublicKeyV1 {
            let mut signature = self.signer.sign_with_public_key(message_hash);
            match &mut signature {
                SignatureWithPublicKeyV1::Secp256k1 { signature } => signature.0[5].add_assign(1),
                SignatureWithPublicKeyV1::Ed25519 {
                    signature,
                    public_key: _,
                } => signature.0[5].add_assign(1),
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
                    unsigned_v2_builder(notary.public_key().into())
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

    fn unsigned_v1_builder(notary_public_key: PublicKey) -> TransactionV1Builder {
        TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: Epoch::of(1),
                end_epoch_exclusive: Epoch::of(10),
                nonce: 0,
                notary_public_key,
                notary_is_signatory: false,
                tip_percentage: 5,
            })
            .manifest(ManifestBuilder::new().drop_auth_zone_proofs().build())
    }

    fn unsigned_v2_builder(notary_public_key: PublicKey) -> TransactionV2Builder {
        TransactionBuilder::new_v2()
            .transaction_header(TransactionHeaderV2 {
                notary_public_key,
                notary_is_signatory: false,
                tip_basis_points: 5,
            })
            .intent_header(IntentHeaderV2 {
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: Epoch::of(1),
                end_epoch_exclusive: Epoch::of(10),
                min_proposer_timestamp_inclusive: None,
                max_proposer_timestamp_exclusive: None,
                intent_discriminator: 0,
            })
            .manifest(ManifestBuilder::new_v2().drop_auth_zone_proofs().build())
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
            #[allow(deprecated)]
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

        fn create_transaction(signature_counts: Vec<usize>) -> NotarizedTransactionV2 {
            let signer = Secp256k1PrivateKey::from_u64(1).unwrap();
            let notary = Secp256k1PrivateKey::from_u64(2).unwrap();
            let mut builder = TransactionV2Builder::new();

            for (i, signature_count) in signature_counts.iter().enumerate() {
                builder = builder.add_signed_child(
                    format!("child{i}"),
                    create_partial_transaction(i, *signature_count),
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
                    for (i, _) in signature_counts.iter().enumerate() {
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
            create_transaction(vec![10]).prepare_and_validate(&validator),
            Ok(_)
        );
        assert_matches!(
            create_transaction(vec![10, 20]).prepare_and_validate(&validator),
            Err(TransactionValidationError::SignatureValidationError(
                TransactionValidationErrorLocation::NonRootSubintent(SubintentIndex(1), _),
                SignatureValidationError::TooManySignatures {
                    total: 20,
                    limit: 16,
                },
            ))
        );
        assert_matches!(
            create_transaction(vec![10, 10, 10, 10, 10, 10, 10]).prepare_and_validate(&validator),
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
