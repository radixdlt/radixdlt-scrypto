use radix_engine_constants::*;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::blueprints::transaction_processor::RuntimeValidation;
use radix_engine_interface::constants::*;
use radix_engine_interface::crypto::{Hash, PublicKey};
use radix_engine_interface::network::NetworkDefinition;
use sbor::rust::collections::*;

use crate::errors::{SignatureValidationError, *};
use crate::model::*;
use crate::validation::*;
use radix_engine_interface::data::manifest::*;

pub trait TransactionValidator<T: ManifestDecode> {
    fn check_length_and_decode_from_slice(
        &self,
        transaction: &[u8],
    ) -> Result<T, TransactionValidationError> {
        if transaction.len() > MAX_TRANSACTION_SIZE {
            return Err(TransactionValidationError::TransactionTooLarge);
        }

        let transaction = manifest_decode(transaction)
            .map_err(TransactionValidationError::DeserializationError)?;

        Ok(transaction)
    }

    fn validate<'a, 't, I: IntentHashManager>(
        &'a self,
        transaction: &'t T,
        payload_size: usize,
        intent_hash_manager: &'a I,
    ) -> Result<Executable<'t>, TransactionValidationError>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ValidationConfig {
    pub network_id: u8,
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

impl TransactionValidator<NotarizedTransaction> for NotarizedTransactionValidator {
    fn validate<'a, 't, I: IntentHashManager>(
        &'a self,
        transaction: &'t NotarizedTransaction,
        payload_size: usize,
        intent_hash_manager: &'a I,
    ) -> Result<Executable<'t>, TransactionValidationError> {
        let intent = &transaction.signed_intent.intent;
        let intent_hash = intent.hash()?;

        self.validate_intent(&intent_hash, intent, intent_hash_manager)?;

        let signer_keys = self
            .validate_signatures(&transaction)
            .map_err(TransactionValidationError::SignatureValidationError)?;

        let transaction_hash = transaction.hash()?;

        let header = &intent.header;

        Ok(Executable::new(
            intent.manifest.instructions.clone(),
            &intent.manifest.blobs,
            ExecutionContext {
                transaction_hash,
                payload_size,
                auth_zone_params: AuthZoneParams {
                    initial_proofs: AuthAddresses::signer_set(&signer_keys),
                    virtual_resources: BTreeSet::new(),
                },
                fee_payment: FeePayment::User {
                    cost_unit_limit: header.cost_unit_limit,
                    tip_percentage: header.tip_percentage,
                },
                runtime_validations: vec![
                    RuntimeValidation::IntentHashUniqueness { intent_hash }.enforced(),
                    RuntimeValidation::WithinEpochRange {
                        start_epoch_inclusive: header.start_epoch_inclusive,
                        end_epoch_exclusive: header.end_epoch_exclusive,
                    }
                    .enforced(),
                ],
                pre_allocated_ids: BTreeSet::new(),
            },
        ))
    }
}

impl NotarizedTransactionValidator {
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    pub fn validate_preview_intent<'a, 't, I: IntentHashManager>(
        &'a self,
        preview_intent: &'t PreviewIntent,
        intent_hash_manager: &'a I,
    ) -> Result<Executable<'t>, TransactionValidationError> {
        let transaction_hash = preview_intent.hash()?;
        let intent = &preview_intent.intent;

        let flags = &preview_intent.flags;
        let intent_hash = intent.hash()?;
        self.validate_intent(&intent_hash, intent, intent_hash_manager)?;
        let initial_proofs = AuthAddresses::signer_set(&preview_intent.signer_public_keys);

        let mut virtual_resources = BTreeSet::new();
        if flags.assume_all_signature_proofs {
            virtual_resources.insert(ECDSA_SECP256K1_TOKEN);
            virtual_resources.insert(EDDSA_ED25519_TOKEN);
        }

        let header = &intent.header;
        let manifest = &intent.manifest;

        let fee_payment = if flags.unlimited_loan {
            FeePayment::NoFee
        } else {
            FeePayment::User {
                cost_unit_limit: header.cost_unit_limit,
                tip_percentage: header.tip_percentage,
            }
        };

        Ok(Executable::new(
            manifest.instructions.clone(),
            &manifest.blobs,
            ExecutionContext {
                transaction_hash,
                payload_size: 0,
                auth_zone_params: AuthZoneParams {
                    initial_proofs,
                    virtual_resources,
                },
                fee_payment,
                runtime_validations: vec![
                    RuntimeValidation::IntentHashUniqueness { intent_hash }
                        .with_skipped_assertion_if(flags.permit_duplicate_intent_hash),
                    RuntimeValidation::WithinEpochRange {
                        start_epoch_inclusive: header.start_epoch_inclusive,
                        end_epoch_exclusive: header.end_epoch_exclusive,
                    }
                    .with_skipped_assertion_if(flags.permit_invalid_header_epoch),
                ],
                pre_allocated_ids: BTreeSet::new(),
            },
        ))
    }

    pub fn validate_intent<I: IntentHashManager>(
        &self,
        intent_hash: &Hash,
        intent: &TransactionIntent,
        intent_hash_manager: &I,
    ) -> Result<(), TransactionValidationError> {
        // verify intent hash
        if !intent_hash_manager.allows(intent_hash) {
            return Err(TransactionValidationError::IntentHashRejected);
        }

        // verify intent header
        self.validate_header(&intent)
            .map_err(TransactionValidationError::HeaderValidationError)?;

        Self::validate_manifest(&intent.manifest)?;

        return Ok(());
    }

    pub fn validate_manifest(
        manifest: &TransactionManifest,
    ) -> Result<(), TransactionValidationError> {
        // semantic analysis
        let mut id_validator = ManifestValidator::new();
        for inst in &manifest.instructions {
            match inst {
                Instruction::TakeFromWorktop { .. } => {
                    id_validator
                        .new_bucket()
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                Instruction::TakeFromWorktopByAmount { .. } => {
                    id_validator
                        .new_bucket()
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                Instruction::TakeFromWorktopByIds { .. } => {
                    id_validator
                        .new_bucket()
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                Instruction::ReturnToWorktop { bucket_id } => {
                    id_validator
                        .drop_bucket(bucket_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                Instruction::AssertWorktopContains { .. } => {}
                Instruction::AssertWorktopContainsByAmount { .. } => {}
                Instruction::AssertWorktopContainsByIds { .. } => {}
                Instruction::PopFromAuthZone => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                Instruction::PushToAuthZone { proof_id } => {
                    id_validator
                        .drop_proof(proof_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                Instruction::ClearAuthZone => {}
                Instruction::CreateProofFromAuthZone { .. } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                Instruction::CreateProofFromAuthZoneByAmount { .. } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                Instruction::CreateProofFromAuthZoneByIds { .. } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                Instruction::CreateProofFromBucket { bucket_id } => {
                    id_validator
                        .new_proof(ProofKind::BucketProof(bucket_id.clone()))
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                Instruction::CloneProof { proof_id } => {
                    id_validator
                        .clone_proof(proof_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                Instruction::DropProof { proof_id } => {
                    id_validator
                        .drop_proof(proof_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                Instruction::DropAllProofs => {
                    id_validator
                        .drop_all_proofs()
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                Instruction::ClearSignatureProofs => {}
                Instruction::CallFunction { args, .. } | Instruction::CallMethod { args, .. } => {
                    // TODO: decode into Value
                    Self::validate_call_args(&args, &mut id_validator)
                        .map_err(TransactionValidationError::CallDataValidationError)?;
                }
                Instruction::BurnResource { bucket_id } => {
                    id_validator
                        .drop_bucket(bucket_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                }
                Instruction::PublishPackage { .. }
                | Instruction::PublishPackageAdvanced { .. }
                | Instruction::RecallResource { .. }
                | Instruction::SetMetadata { .. }
                | Instruction::RemoveMetadata { .. }
                | Instruction::SetPackageRoyaltyConfig { .. }
                | Instruction::SetComponentRoyaltyConfig { .. }
                | Instruction::ClaimPackageRoyalty { .. }
                | Instruction::ClaimComponentRoyalty { .. }
                | Instruction::SetMethodAccessRule { .. }
                | Instruction::MintFungible { .. }
                | Instruction::MintNonFungible { .. }
                | Instruction::MintUuidNonFungible { .. } => {}
            }
        }

        Ok(())
    }

    pub fn validate_header(&self, intent: &TransactionIntent) -> Result<(), HeaderValidationError> {
        let header = &intent.header;

        // version
        if header.version != TRANSACTION_VERSION_V1 {
            return Err(HeaderValidationError::UnknownVersion(header.version));
        }

        // network
        if header.network_id != self.config.network_id {
            return Err(HeaderValidationError::InvalidNetwork);
        }

        // epoch
        if header.end_epoch_exclusive <= header.start_epoch_inclusive {
            return Err(HeaderValidationError::InvalidEpochRange);
        }
        if header.end_epoch_exclusive - header.start_epoch_inclusive > self.config.max_epoch_range {
            return Err(HeaderValidationError::EpochRangeTooLarge);
        }

        // cost unit limit
        if header.cost_unit_limit < self.config.min_cost_unit_limit
            || header.cost_unit_limit > self.config.max_cost_unit_limit
        {
            return Err(HeaderValidationError::InvalidCostUnitLimit);
        }

        // tip percentage
        if header.tip_percentage < self.config.min_tip_percentage
            || header.tip_percentage > self.config.max_tip_percentage
        {
            return Err(HeaderValidationError::InvalidTipBps);
        }

        Ok(())
    }

    pub fn validate_signatures(
        &self,
        transaction: &NotarizedTransaction,
    ) -> Result<Vec<PublicKey>, SignatureValidationError> {
        // TODO: split into static validation part and runtime validation part to support more signatures
        if transaction.signed_intent.intent_signatures.len() > MAX_NUMBER_OF_INTENT_SIGNATURES {
            return Err(SignatureValidationError::TooManySignatures);
        }

        // verify intent signature
        let mut signers = index_set_new();
        let intent_payload = transaction.signed_intent.intent.to_bytes()?;
        for sig in &transaction.signed_intent.intent_signatures {
            let public_key = recover(&intent_payload, sig)
                .ok_or(SignatureValidationError::InvalidIntentSignature)?;

            if !verify(&intent_payload, &public_key, &sig.signature()) {
                return Err(SignatureValidationError::InvalidIntentSignature);
            }

            if !signers.insert(public_key) {
                return Err(SignatureValidationError::DuplicateSigner);
            }
        }

        if transaction.signed_intent.intent.header.notary_as_signatory {
            signers.insert(transaction.signed_intent.intent.header.notary_public_key);
        }

        // verify notary signature
        let signed_intent_payload = transaction.signed_intent.to_bytes()?;
        if !verify(
            &signed_intent_payload,
            &transaction.signed_intent.intent.header.notary_public_key,
            &transaction.notary_signature,
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
        ($result: expr, ($version: expr, $start_epoch: expr, $end_epoch: expr, $nonce: expr, $signers: expr, $notary: expr)) => {{
            let mut intent_hash_manager: TestIntentHashManager = TestIntentHashManager::new();
            let config: ValidationConfig = ValidationConfig::simulator();
            let validator = NotarizedTransactionValidator::new(config);
            assert_eq!(
                $result,
                validator
                    .validate(
                        &create_transaction(
                            $version,
                            $start_epoch,
                            $end_epoch,
                            $nonce,
                            $signers,
                            $notary
                        ),
                        0,
                        &mut intent_hash_manager,
                    )
                    .expect_err("Should be an error")
            );
        }};
    }

    #[test]
    fn test_invalid_header() {
        assert_invalid_tx!(
            TransactionValidationError::HeaderValidationError(
                HeaderValidationError::UnknownVersion(2)
            ),
            (2, 0, 100, 5, vec![1], 2)
        );
        assert_invalid_tx!(
            TransactionValidationError::HeaderValidationError(
                HeaderValidationError::InvalidEpochRange
            ),
            (1, 0, 0, 5, vec![1], 2)
        );
        assert_invalid_tx!(
            TransactionValidationError::HeaderValidationError(
                HeaderValidationError::EpochRangeTooLarge
            ),
            (1, 0, 1000, 5, vec![1], 2)
        );
    }

    #[test]
    fn test_invalid_signatures() {
        assert_invalid_tx!(
            TransactionValidationError::SignatureValidationError(
                SignatureValidationError::TooManySignatures
            ),
            (1, 0, 100, 5, (1..20).collect(), 2)
        );
        assert_invalid_tx!(
            TransactionValidationError::SignatureValidationError(
                SignatureValidationError::DuplicateSigner
            ),
            (1, 0, 100, 5, vec![1, 1], 2)
        );
    }

    #[test]
    fn test_valid_preview() {
        let mut intent_hash_manager: TestIntentHashManager = TestIntentHashManager::new();

        // Build the whole transaction but only really care about the intent
        let tx = create_transaction(1, 0, 100, 5, vec![1, 2], 2);

        let validator = NotarizedTransactionValidator::new(ValidationConfig::simulator());

        let preview_intent = PreviewIntent {
            intent: tx.signed_intent.intent,
            signer_public_keys: Vec::new(),
            flags: PreviewFlags {
                unlimited_loan: true,
                assume_all_signature_proofs: false,
                permit_invalid_header_epoch: false,
                permit_duplicate_intent_hash: false,
            },
        };

        let result = validator.validate_preview_intent(&preview_intent, &mut intent_hash_manager);

        assert!(result.is_ok());
    }

    fn create_transaction(
        version: u8,
        start_epoch: u64,
        end_epoch: u64,
        nonce: u64,
        signers: Vec<u64>,
        notary: u64,
    ) -> NotarizedTransaction {
        let sk_notary = EcdsaSecp256k1PrivateKey::from_u64(notary).unwrap();

        let mut builder = TransactionBuilder::new()
            .header(TransactionHeader {
                version,
                network_id: NetworkDefinition::simulator().id,
                start_epoch_inclusive: start_epoch,
                end_epoch_exclusive: end_epoch,
                nonce,
                notary_public_key: sk_notary.public_key().into(),
                notary_as_signatory: false,
                cost_unit_limit: 1_000_000,
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
