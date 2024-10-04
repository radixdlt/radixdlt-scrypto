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
    pub allow_notary_to_duplicate_signer: bool,
    pub preparation_settings: PreparationSettingsV1,
    pub manifest_validation: ManifestValidationRuleset,
    // V2 settings
    pub v2_transactions_allowed: bool,
    pub min_tip_basis_points: u32,
    pub max_tip_basis_points: u32,
    pub max_subintent_count: usize,
    /// A setting of N here allows a total depth of N + 1 if you
    /// include the root transaction intent.
    pub max_subintent_depth: usize,
    pub max_total_signer_signatures: usize,
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
            allow_notary_to_duplicate_signer: true,
            manifest_validation: ManifestValidationRuleset::BabylonBasicValidator,
            message_validation: MessageValidationConfig::babylon(),
            preparation_settings: PreparationSettings::babylon(),
            // V2-only settings
            v2_transactions_allowed: true,
            max_subintent_count: 0,
            max_subintent_depth: 0,
            min_tip_basis_points: 0,
            max_tip_basis_points: 0,
            max_total_signer_signatures: usize::MAX,
            max_total_references: usize::MAX,
        }
    }

    pub const fn cuttlefish() -> Self {
        Self {
            max_references_per_intent: 512,
            v2_transactions_allowed: true,
            max_subintent_count: 32,
            max_subintent_depth: 3,
            min_tip_basis_points: 0,
            max_instructions: 1000,
            allow_notary_to_duplicate_signer: false,
            manifest_validation: ManifestValidationRuleset::Interpreter(
                InterpreterValidationRulesetSpecifier::Cuttlefish,
            ),
            // Tip of 100 times the cost of a transaction
            max_tip_basis_points: 100 * 10000,
            preparation_settings: PreparationSettings::cuttlefish(),
            max_total_signer_signatures: 64,
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

    pub fn preparation_settings(&self) -> &PreparationSettings {
        &self.config.preparation_settings
    }

    #[allow(deprecated)]
    pub fn validate_notarized_v1(
        &self,
        transaction: PreparedNotarizedTransactionV1,
    ) -> Result<ValidatedNotarizedTransactionV1, TransactionValidationError> {
        self.validate_intent_v1(&transaction.signed_intent.intent)?;

        self.check_reference_limits(vec![
            &transaction.signed_intent.intent.instructions.references,
        ])?;

        self.check_signature_limits(
            &transaction.signed_intent.intent_signatures.inner.signatures,
            None,
        )?;

        let (signer_keys, num_of_signature_validations) = self
            .validate_signatures_v1(&transaction)
            .map_err(TransactionValidationError::SignatureValidationError)?;

        let encoded_instructions = Rc::new(manifest_encode(
            &transaction.signed_intent.intent.instructions.inner.0,
        )?);

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
        let intent = preview_intent.intent.prepare(self.preparation_settings())?;

        self.validate_intent_v1(&intent)?;

        let encoded_instructions = Rc::new(manifest_encode(&intent.instructions.inner.0)?);

        Ok(ValidatedPreviewIntent {
            intent,
            encoded_instructions,
            signer_public_keys: preview_intent.signer_public_keys,
            flags: preview_intent.flags,
        })
    }

    #[allow(deprecated)]
    pub fn validate_intent_v1(
        &self,
        intent: &PreparedIntentV1,
    ) -> Result<(), TransactionValidationError> {
        self.validate_header_v1(&intent.header.inner)?;
        self.validate_message_v1(&intent.message.inner)?;
        self.validate_instructions_v1(&intent.instructions.inner.0, &intent.blobs.blobs_by_hash)?;

        return Ok(());
    }

    pub fn validate_instructions_v1(
        &self,
        instructions: &[InstructionV1],
        blobs: &IndexMap<Hash, Vec<u8>>,
    ) -> Result<(), TransactionValidationError> {
        if instructions.len() > self.config.max_instructions {
            return Err(ManifestValidationError::TooManyInstructions.into());
        }
        impl<'a> ReadableManifest for (&'a [InstructionV1], &'a IndexMap<Hash, Vec<u8>>) {
            type Instruction = InstructionV1;

            fn is_subintent(&self) -> bool {
                false
            }

            fn get_instructions(&self) -> &[Self::Instruction] {
                self.0
            }

            fn get_blobs<'b>(&'b self) -> impl Iterator<Item = (&'b Hash, &'b Vec<u8>)> {
                self.1.iter()
            }

            fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef {
                ManifestObjectNamesRef::Unknown
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
                ManifestInstructionEffect::WorktopAssertion { .. } => {}
                ManifestInstructionEffect::Verification { .. } => {}
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

    #[allow(deprecated)]
    pub fn validate_signatures_v1(
        &self,
        transaction: &PreparedNotarizedTransactionV1,
    ) -> Result<(Vec<PublicKey>, usize), SignatureValidationError> {
        let intent_signatures = &transaction.signed_intent.intent_signatures.inner.signatures;
        let header = &transaction.signed_intent.intent.header.inner;

        self.validate_transaction_intent_and_notary_signatures_v1(
            transaction.transaction_intent_hash(),
            transaction.signed_transaction_intent_hash(),
            intent_signatures,
            header.notary_is_signatory,
            &header.notary_public_key,
            &transaction.notary_signature.inner.0,
        )
    }

    pub fn validate_transaction_intent_and_notary_signatures_v1(
        &self,
        transaction_intent_hash: TransactionIntentHash,
        signed_transaction_intent_hash: SignedTransactionIntentHash,
        transaction_intent_signatures: &[IntentSignatureV1],
        notary_is_signatory: bool,
        notary_public_key: &PublicKey,
        notary_signature: &SignatureV1,
    ) -> Result<(Vec<PublicKey>, usize), SignatureValidationError> {
        let mut signers = index_set_with_capacity(transaction_intent_signatures.len() + 1);
        for intent_signature in transaction_intent_signatures.iter() {
            let public_key = recover(&transaction_intent_hash.0, &intent_signature.0)
                .ok_or(SignatureValidationError::InvalidIntentSignature)?;

            if !verify(
                &transaction_intent_hash.0,
                &public_key,
                &intent_signature.0.signature(),
            ) {
                return Err(SignatureValidationError::InvalidIntentSignature);
            }

            if !signers.insert(public_key) {
                return Err(SignatureValidationError::DuplicateSigner);
            }
        }

        if notary_is_signatory {
            if !signers.insert(notary_public_key.clone()) {
                if !self.config.allow_notary_to_duplicate_signer {
                    return Err(SignatureValidationError::DuplicateSigner);
                }
            }
        }

        if !verify(
            &signed_transaction_intent_hash.0,
            notary_public_key,
            notary_signature,
        ) {
            return Err(SignatureValidationError::InvalidNotarySignature);
        }

        let num_validations = transaction_intent_signatures.len() + 1; // + 1 for the notary signature

        Ok((signers.into_iter().collect(), num_validations))
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
        let root_subintent = &prepared.signed_intent.transaction_intent;
        let root_subintent_signatures = &prepared.signed_intent.transaction_intent_signatures;
        let non_root_subintents = &transaction_intent.non_root_subintents;
        let non_root_subintent_signatures = &prepared.signed_intent.non_root_subintent_signatures;

        self.validate_transaction_header_v2(&transaction_intent.transaction_header.inner)?;

        self.check_reference_limits(
            [&root_subintent.root_intent_core.instructions.references]
                .into_iter()
                .chain(
                    non_root_subintents
                        .subintents
                        .iter()
                        .map(|x| &x.intent_core.instructions.references),
                )
                .collect(),
        )?;

        self.check_signature_limits(
            &root_subintent_signatures.inner.signatures,
            Some(non_root_subintent_signatures),
        )?;

        let ValidatedPartialTransactionTreeV2 {
            overall_validity_range,
            root_intent_info,
            root_yield_to_parent_count: _, // Checked to be 0 in the manifest validator.
            non_root_subintents_info,
        } = self.validate_transaction_subtree_v2(
            &transaction_intent.root_intent_core,
            transaction_intent.transaction_intent_hash().into(),
            self.validate_transaction_intent_and_notary_signatures_v2(&prepared)?,
            non_root_subintents,
            non_root_subintent_signatures,
        )?;

        Ok(ValidatedNotarizedTransactionV2 {
            prepared,
            overall_validity_range,
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
        let root_subintent_signatures = &prepared.root_subintent_signatures;
        let non_root_subintents = &prepared.partial_transaction.non_root_subintents;
        let non_root_subintent_signatures = &prepared.non_root_subintent_signatures;

        self.check_reference_limits(
            [&root_subintent.intent_core.instructions.references]
                .into_iter()
                .chain(
                    non_root_subintents
                        .subintents
                        .iter()
                        .map(|x| &x.intent_core.instructions.references),
                )
                .collect(),
        )?;

        self.check_signature_limits(
            &root_subintent_signatures.inner.signatures,
            Some(non_root_subintent_signatures),
        )?;

        let ValidatedPartialTransactionTreeV2 {
            overall_validity_range,
            root_intent_info,
            root_yield_to_parent_count,
            non_root_subintents_info,
        } = self.validate_transaction_subtree_v2(
            &root_subintent.intent_core,
            root_subintent.subintent_hash().into(),
            self.validate_subintent_signatures_v2(root_subintent, root_subintent_signatures)?,
            non_root_subintents,
            non_root_subintent_signatures,
        )?;

        Ok(ValidatedSignedPartialTransactionV2 {
            prepared,
            overall_validity_range,
            root_subintent_info: root_intent_info,
            root_subintent_yield_to_parent_count: root_yield_to_parent_count,
            non_root_subintents_info,
        })
    }

    pub fn check_reference_limits(
        &self,
        subintent_references: Vec<&IndexSet<Reference>>,
    ) -> Result<(), TransactionValidationError> {
        let mut total = 0;
        for refs in subintent_references {
            if refs.len() > self.config.max_references_per_intent {
                return Err(TransactionValidationError::TooManyReferencesForIntent);
            }
            total += refs.len();
        }

        if total > self.config.max_total_references {
            return Err(TransactionValidationError::TooManyReferences {
                total,
                limit: self.config.max_total_references,
            });
        }

        Ok(())
    }

    pub fn check_signature_limits(
        &self,
        root_subintent_signatures: &[IntentSignatureV1],
        non_root_subintent_signatures: Option<&PreparedNonRootSubintentSignaturesV2>,
    ) -> Result<(), TransactionValidationError> {
        if root_subintent_signatures.len() > self.config.max_signer_signatures_per_intent {
            return Err(TransactionValidationError::TooManySignaturesForIntent);
        }
        let mut total = root_subintent_signatures.len();
        if let Some(sigs) = non_root_subintent_signatures {
            for intent_sigs in &sigs.by_subintent {
                if intent_sigs.inner.signatures.len() > self.config.max_signer_signatures_per_intent
                {
                    return Err(TransactionValidationError::TooManySignaturesForIntent);
                }
                total += intent_sigs.inner.signatures.len();
            }
        }

        if total > self.config.max_total_signer_signatures {
            return Err(TransactionValidationError::TooManySignatures {
                total,
                limit: self.config.max_total_signer_signatures,
            });
        }

        Ok(())
    }

    pub fn validate_transaction_subtree_v2(
        &self,
        root_intent_core: &PreparedIntentCoreV2,
        root_intent_hash: IntentHash,
        root_signature_validations: SignatureValidations,
        non_root_subintents: &PreparedNonRootSubintentsV2,
        non_root_subintent_signatures: &PreparedNonRootSubintentSignaturesV2,
    ) -> Result<ValidatedPartialTransactionTreeV2, TransactionValidationError> {
        let non_root_subintents = non_root_subintents.subintents.as_slice();
        let non_root_subintent_signatures = non_root_subintent_signatures.by_subintent.as_slice();
        if non_root_subintents.len() != non_root_subintent_signatures.len() {
            return Err(
                SignatureValidationError::IncorrectNumberOfSubintentSignatureBatches.into(),
            );
        }

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
        let root_intent_info = ValidatedIntentInformationV2 {
            children_subintent_indices: intent_relationships.root_intent.children,
            encoded_instructions: manifest_encode(&root_intent_core.instructions.inner.0)?,
            signature_validations: root_signature_validations,
        };
        let non_root_subintents_info = non_root_subintents
            .iter()
            .zip(non_root_subintent_signatures)
            .zip(intent_relationships.non_root_subintents.into_values())
            .map(
                |((subintent, signatures), info)| -> Result<_, TransactionValidationError> {
                    Ok(ValidatedIntentInformationV2 {
                        encoded_instructions: manifest_encode(
                            &subintent.intent_core.instructions.inner.0,
                        )?,
                        signature_validations: self
                            .validate_subintent_signatures_v2(subintent, signatures)?,
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
        non_root_subintent_details: &IndexMap<SubintentHash, SubIntentRelationshipDetails>,
    ) -> Result<(OverallValidityRangeV2, ManifestYieldSummary), TransactionValidationError> {
        let mut header_aggregation = HeaderAggregation::start();
        let mut yield_summaries: IndexMap<IntentHash, ManifestYieldSummary> =
            index_map_with_capacity(non_root_subintents.len() + 1);
        let root_yield_summary = {
            let yield_summary = self.validate_v2_intent_core(
                root_intent_core,
                &mut header_aggregation,
                root_intent_hash.is_for_subintent(),
            )?;
            yield_summaries.insert(root_intent_hash, yield_summary.clone());
            yield_summary
        };
        for subintent in non_root_subintents.iter() {
            let yield_summary = self.validate_v2_intent_core(
                &subintent.intent_core,
                &mut header_aggregation,
                true,
            )?;
            yield_summaries.insert(subintent.subintent_hash().into(), yield_summary);
        }

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
                return Err(SubintentValidationError::MismatchingYieldChildAndYieldParentCountsForSubintent(*child_hash).into());
            }
        }

        Ok((header_aggregation.into(), root_yield_summary))
    }

    fn validate_v2_intent_core(
        &self,
        intent_core: &PreparedIntentCoreV2,
        header_aggregation: &mut HeaderAggregation,
        is_subintent: bool,
    ) -> Result<ManifestYieldSummary, TransactionValidationError> {
        self.validate_intent_header_v2(&intent_core.header.inner, header_aggregation)?;
        self.validate_message_v2(&intent_core.message.inner)?;
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
        aggregation: &mut HeaderAggregation,
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

        aggregation.update(
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
        children: &[ChildSubintent],
        is_subintent: bool,
    ) -> Result<ManifestYieldSummary, ManifestValidationError> {
        if instructions.len() > self.config.max_instructions {
            return Err(ManifestValidationError::TooManyInstructions);
        }
        impl<'a> ReadableManifest
            for (
                &'a [InstructionV2],
                &'a IndexMap<Hash, Vec<u8>>,
                &'a [ChildSubintent],
                bool,
            )
        {
            type Instruction = InstructionV2;

            fn is_subintent(&self) -> bool {
                self.3
            }

            fn get_instructions(&self) -> &[Self::Instruction] {
                self.0
            }

            fn get_blobs<'b>(&'b self) -> impl Iterator<Item = (&'b Hash, &'b Vec<u8>)> {
                self.1.iter()
            }

            fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef {
                ManifestObjectNamesRef::Unknown
            }

            fn get_child_subintents(&self) -> &[ChildSubintent] {
                &self.2
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

    fn validate_transaction_intent_and_notary_signatures_v2(
        &self,
        prepared: &PreparedNotarizedTransactionV2,
    ) -> Result<SignatureValidations, SignatureValidationError> {
        let transaction_intent_signatures = &prepared
            .signed_intent
            .transaction_intent_signatures
            .inner
            .signatures;
        let transaction_header = &prepared
            .signed_intent
            .transaction_intent
            .transaction_header
            .inner;

        let (signer_keys, num_validations) = self
            .validate_transaction_intent_and_notary_signatures_v1(
                prepared.transaction_intent_hash(),
                prepared.signed_transaction_intent_hash(),
                transaction_intent_signatures,
                transaction_header.notary_is_signatory,
                &transaction_header.notary_public_key,
                &prepared.notary_signature.inner.0,
            )?;

        Ok(SignatureValidations {
            num_validations,
            signer_keys,
        })
    }

    fn validate_subintent_signatures_v2(
        &self,
        prepared: &PreparedSubintentV2,
        signatures: &PreparedIntentSignaturesV2,
    ) -> Result<SignatureValidations, SignatureValidationError> {
        let intent_signatures = &signatures.inner.signatures;
        let intent_hash = prepared.subintent_hash();
        let mut signers = index_set_with_capacity(intent_signatures.len());
        for intent_signature in intent_signatures.iter() {
            let public_key = recover(&intent_hash.0, &intent_signature.0)
                .ok_or(SignatureValidationError::InvalidIntentSignature)?;

            if !verify(&intent_hash.0, &public_key, &intent_signature.0.signature()) {
                return Err(SignatureValidationError::InvalidIntentSignature);
            }

            if !signers.insert(public_key) {
                return Err(SignatureValidationError::DuplicateSigner);
            }
        }
        Ok(SignatureValidations {
            num_validations: intent_signatures.len(),
            signer_keys: signers.into_iter().collect(),
        })
    }

    /// The root intent can be either:
    /// * If validating a full transaction: a transaction intent
    /// * If validating a partial transaction: a root subintent
    fn validate_intent_relationships_v2(
        &self,
        root_intent_hash: IntentHash,
        root_intent_core: &PreparedIntentCoreV2,
        subintents: &[PreparedSubintentV2],
    ) -> Result<IntentRelationships, SubintentValidationError> {
        if subintents.len() > self.config.max_subintent_count {
            return Err(SubintentValidationError::TooManySubintents {
                limit: self.config.max_subintent_count,
                actual: subintents.len(),
            });
        }

        let mut root_intent_details = RootIntentRelationshipDetails::default();
        let mut all_subintent_details =
            IndexMap::<SubintentHash, SubIntentRelationshipDetails>::default();

        // STEP 1
        // ------
        // * We establish that the subintents are unique
        // * We create an index from the SubintentHash to SubintentIndex
        for subintent in subintents.iter() {
            let subintent_hash = subintent.subintent_hash();
            let details = SubIntentRelationshipDetails::default();
            if let Some(_) = all_subintent_details.insert(subintent_hash, details) {
                return Err(SubintentValidationError::DuplicateSubintent(subintent_hash));
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
            for child_subintent in root_intent_core.children.children.iter() {
                let child_hash = child_subintent.hash;
                let (subintent_index, _, subintent_details) = all_subintent_details
                    .get_full_mut(&child_hash)
                    .ok_or_else(|| {
                        SubintentValidationError::ChildSubintentNotIncludedInTransaction(child_hash)
                    })?;
                if subintent_details.parent == PLACEHOLDER_PARENT {
                    subintent_details.parent = parent_hash;
                } else {
                    return Err(SubintentValidationError::SubintentHasMultipleParents(
                        child_hash,
                    ));
                }
                intent_details
                    .children
                    .push(SubintentIndex(subintent_index));
            }
        }

        // STEP 2B - Handle the children of each subintent
        for subintent in subintents.iter() {
            let subintent_hash = subintent.subintent_hash();
            let parent_hash: IntentHash = subintent_hash.into();
            let children = &subintent.intent_core.children.children;
            let mut children_details = Vec::with_capacity(children.len());
            for child_subintent in children.iter() {
                let child_hash = child_subintent.hash;
                let (subintent_index, _, subintent_details) = all_subintent_details
                    .get_full_mut(&child_hash)
                    .ok_or_else(|| {
                        SubintentValidationError::ChildSubintentNotIncludedInTransaction(child_hash)
                    })?;
                if subintent_details.parent == PLACEHOLDER_PARENT {
                    subintent_details.parent = parent_hash;
                } else {
                    return Err(SubintentValidationError::SubintentHasMultipleParents(
                        child_hash,
                    ));
                }
                children_details.push(SubintentIndex(subintent_index));
            }
            all_subintent_details
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
                let (hash, _) = all_subintent_details.get_index(index.0).unwrap();
                return Err(SubintentValidationError::SubintentExceedsMaxDepth(*hash));
            }
            let (_, subintent_details) = all_subintent_details.get_index_mut(index.0).unwrap();
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
        for (hash, details) in all_subintent_details.iter() {
            if details.depth == 0 {
                return Err(
                    SubintentValidationError::SubintentIsNotReachableFromTheTransactionIntent(
                        *hash,
                    ),
                );
            }
        }

        Ok(IntentRelationships {
            root_intent: root_intent_details,
            non_root_subintents: all_subintent_details,
        })
    }
}

struct HeaderAggregation {
    overall_start_epoch_inclusive: Epoch,
    overall_end_epoch_exclusive: Epoch,
    overall_start_timestamp_inclusive: Option<Instant>,
    overall_end_timestamp_exclusive: Option<Instant>,
}

impl From<HeaderAggregation> for OverallValidityRangeV2 {
    fn from(value: HeaderAggregation) -> Self {
        Self {
            epoch_range: EpochRange {
                start_epoch_inclusive: value.overall_start_epoch_inclusive,
                end_epoch_exclusive: value.overall_end_epoch_exclusive,
            },
            proposer_timestamp_range: ProposerTimestampRange {
                start_timestamp_inclusive: value.overall_start_timestamp_inclusive,
                end_timestamp_exclusive: value.overall_end_timestamp_exclusive,
            },
        }
    }
}

impl HeaderAggregation {
    fn start() -> Self {
        Self {
            overall_start_epoch_inclusive: Epoch::zero(),
            overall_end_epoch_exclusive: Epoch::of(u64::MAX),
            overall_start_timestamp_inclusive: None,
            overall_end_timestamp_exclusive: None,
        }
    }

    fn update(
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
    pub non_root_subintents: IndexMap<SubintentHash, SubIntentRelationshipDetails>,
}
#[derive(Default)]
pub struct RootIntentRelationshipDetails {
    children: Vec<SubintentIndex>,
}

pub struct SubIntentRelationshipDetails {
    parent: IntentHash,
    depth: usize,
    children: Vec<SubintentIndex>,
}

impl Default for SubIntentRelationshipDetails {
    fn default() -> Self {
        Self {
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
    use radix_common::network::NetworkDefinition;

    use super::*;
    use crate::{builder::ManifestBuilder, builder::TransactionBuilder};

    macro_rules! assert_invalid_tx {
        ($result: expr, ($start_epoch: expr, $end_epoch: expr, $nonce: expr, $signers: expr, $notary: expr)) => {{
            let validator = TransactionValidator::new_for_latest_simulator();
            assert_eq!(
                $result,
                create_transaction($start_epoch, $end_epoch, $nonce, $signers, $notary)
                    .prepare_and_validate(&validator)
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
                HeaderValidationError::InvalidEpochRange
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
            TransactionValidationError::HeaderValidationError(
                HeaderValidationError::InvalidEpochRange
            ),
            (Epoch::of(u64::MAX - 5), Epoch::of(u64::MAX), 5, vec![1], 2)
        );
    }

    #[test]
    fn test_invalid_signatures() {
        assert_invalid_tx!(
            TransactionValidationError::TooManySignaturesForIntent,
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
        assert!(matches!(
            transaction.prepare_and_validate(&validator),
            Err(TransactionValidationError::ManifestValidationError(
                ManifestValidationError::BucketConsumedWhilstLockedByProof(ManifestBucket(0), _,)
            ))
        ));
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
        assert!(matches!(
            transaction.prepare_and_validate(&validator),
            Err(TransactionValidationError::ManifestValidationError(
                ManifestValidationError::ProofAlreadyUsed(ManifestProof(0), _,)
            ))
        ));
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
        assert!(matches!(
            transaction.prepare_and_validate(&validator),
            Err(TransactionValidationError::ManifestValidationError(
                ManifestValidationError::BucketAlreadyUsed(ManifestBucket(0), _,)
            ))
        ));
    }
}
