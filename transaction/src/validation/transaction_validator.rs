use sbor::Decode;
use std::collections::HashSet;

use scrypto::buffer::scrypto_decode;
use scrypto::crypto::PublicKey;
use scrypto::values::*;

use crate::errors::{SignatureValidationError, *};
use crate::model::*;
use crate::validation::*;

pub const MAX_PAYLOAD_SIZE: usize = 4 * 1024 * 1024;

pub trait TransactionValidator<T: Decode> {
    fn validate_from_slice<I: IntentHashManager>(
        &self,
        transaction: &[u8],
        intent_hash_manager: &I,
    ) -> Result<Validated<T>, TransactionValidationError> {
        if transaction.len() > MAX_PAYLOAD_SIZE {
            return Err(TransactionValidationError::TransactionTooLarge);
        }

        let transaction: T = scrypto_decode(transaction)
            .map_err(TransactionValidationError::DeserializationError)?;

        self.validate(transaction, intent_hash_manager)
    }

    fn validate<I: IntentHashManager>(
        &self,
        transaction: T,
        intent_hash_manager: &I,
    ) -> Result<Validated<T>, TransactionValidationError>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ValidationConfig {
    pub network_id: u8,
    pub current_epoch: u64,
    pub max_cost_unit_limit: u32,
    pub min_tip_percentage: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NotarizedTransactionValidator {
    config: ValidationConfig,
}

impl TransactionValidator<NotarizedTransaction> for NotarizedTransactionValidator {
    fn validate<I: IntentHashManager>(
        &self,
        transaction: NotarizedTransaction,
        intent_hash_manager: &I,
    ) -> Result<Validated<NotarizedTransaction>, TransactionValidationError> {
        // verify the intent
        let instructions =
            self.validate_intent(&transaction.signed_intent.intent, intent_hash_manager)?;

        // verify signatures
        let keys = self
            .validate_signatures(&transaction)
            .map_err(TransactionValidationError::SignatureValidationError)?;

        let transaction_hash = transaction.hash();

        let cost_unit_limit = transaction.signed_intent.intent.header.cost_unit_limit;
        let tip_percentage = transaction.signed_intent.intent.header.tip_percentage;
        let blobs = transaction.signed_intent.intent.manifest.blobs.clone();

        Ok(Validated::new(
            transaction,
            transaction_hash,
            instructions,
            AuthModule::pk_non_fungibles(&keys),
            cost_unit_limit,
            tip_percentage,
            blobs,
        ))
    }
}

impl NotarizedTransactionValidator {
    pub fn new(config: ValidationConfig) -> Self {
        Self { config }
    }

    pub fn validate_preview_intent<I: IntentHashManager>(
        &self,
        preview_intent: PreviewIntent,
        intent_hash_manager: &I,
    ) -> Result<ValidatedPreviewTransaction, TransactionValidationError> {
        let intent = &preview_intent.intent;

        let transaction_hash = preview_intent.hash();

        let instructions = self.validate_intent(&intent, intent_hash_manager)?;

        Ok(ValidatedPreviewTransaction {
            preview_intent,
            transaction_hash,
            instructions,
        })
    }

    pub fn validate_intent<I: IntentHashManager>(
        &self,
        intent: &TransactionIntent,
        intent_hash_manager: &I,
    ) -> Result<Vec<Instruction>, TransactionValidationError> {
        // verify intent hash
        if !intent_hash_manager.allows(&intent.hash()) {
            return Err(TransactionValidationError::IntentHashRejected);
        }

        // verify intent header
        self.validate_header(&intent)
            .map_err(TransactionValidationError::HeaderValidationError)?;

        let instructions = Self::validate_manifest(&intent.manifest)?;

        return Ok(instructions);
    }

    pub fn validate_manifest(
        manifest: &TransactionManifest,
    ) -> Result<Vec<Instruction>, TransactionValidationError> {
        // semantic analysis
        let mut id_validator = IdValidator::new();
        for inst in &manifest.instructions {
            match inst.clone() {
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
                        .new_proof(ProofKind::BucketProof(bucket_id))
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
                Instruction::CallFunction { args, .. } => {
                    // TODO: decode into Value
                    Self::validate_call_data(&args, &mut id_validator)
                        .map_err(TransactionValidationError::CallDataValidationError)?;
                }
                Instruction::CallMethod { args, .. } => {
                    // TODO: decode into Value
                    Self::validate_call_data(&args, &mut id_validator)
                        .map_err(TransactionValidationError::CallDataValidationError)?;
                }
                Instruction::PublishPackage { .. } => {}
            }
        }

        Ok(manifest.instructions.clone())
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
        if header.end_epoch_exclusive - header.start_epoch_inclusive > MAX_EPOCH_DURATION {
            return Err(HeaderValidationError::EpochRangeTooLarge);
        }
        if self.config.current_epoch < header.start_epoch_inclusive
            || self.config.current_epoch >= header.end_epoch_exclusive
        {
            return Err(HeaderValidationError::OutOfEpochRange);
        }

        // cost unit limit and tip
        if header.cost_unit_limit > self.config.max_cost_unit_limit {
            return Err(HeaderValidationError::InvalidCostUnitLimit);
        }
        if header.tip_percentage < self.config.min_tip_percentage {
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
        let mut signers = HashSet::new();
        let intent_payload = transaction.signed_intent.intent.to_bytes();
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
        let signed_intent_payload = transaction.signed_intent.to_bytes();
        if !verify(
            &signed_intent_payload,
            &transaction.signed_intent.intent.header.notary_public_key,
            &transaction.notary_signature,
        ) {
            return Err(SignatureValidationError::InvalidNotarySignature);
        }

        Ok(signers.into_iter().collect())
    }

    pub fn validate_call_data(
        call_data: &[u8],
        id_validator: &mut IdValidator,
    ) -> Result<(), CallDataValidationError> {
        let value =
            ScryptoValue::from_slice(call_data).map_err(CallDataValidationError::DecodeError)?;
        id_validator
            .move_resources(&value)
            .map_err(CallDataValidationError::IdValidationError)?;
        if let Some(vault_id) = value.vault_ids.iter().nth(0) {
            return Err(CallDataValidationError::VaultNotAllowed(vault_id.clone()));
        }
        if let Some(kv_store_id) = value.kv_store_ids.iter().nth(0) {
            return Err(CallDataValidationError::KeyValueStoreNotAllowed(
                kv_store_id.clone(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use scrypto::core::NetworkDefinition;

    use super::*;
    use crate::{
        builder::ManifestBuilder, builder::TransactionBuilder, signing::EcdsaSecp256k1PrivateKey,
    };

    macro_rules! assert_invalid_tx {
        ($result: expr, ($version: expr, $start_epoch: expr, $end_epoch: expr, $nonce: expr, $signers: expr, $notary: expr)) => {{
            let mut intent_hash_manager: TestIntentHashManager = TestIntentHashManager::new();
            let config: ValidationConfig = ValidationConfig {
                network_id: NetworkDefinition::simulator().id,
                current_epoch: 1,
                max_cost_unit_limit: 10_000_000,
                min_tip_percentage: 0,
            };
            let validator = NotarizedTransactionValidator::new(config);
            assert_eq!(
                Err($result),
                validator.validate(
                    create_transaction(
                        $version,
                        $start_epoch,
                        $end_epoch,
                        $nonce,
                        $signers,
                        $notary
                    ),
                    &mut intent_hash_manager,
                )
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
        assert_invalid_tx!(
            TransactionValidationError::HeaderValidationError(
                HeaderValidationError::OutOfEpochRange
            ),
            (1, 100, 101, 5, vec![1], 2)
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

        let validator = NotarizedTransactionValidator::new(ValidationConfig {
            network_id: NetworkDefinition::simulator().id,
            current_epoch: 1,
            max_cost_unit_limit: 10_000_000,
            min_tip_percentage: 0,
        });

        let result = validator.validate_preview_intent(
            PreviewIntent {
                intent: tx.signed_intent.intent,
                signer_public_keys: Vec::new(),
                flags: PreviewFlags {
                    unlimited_loan: true,
                },
            },
            &mut intent_hash_manager,
        );

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
            .manifest(
                ManifestBuilder::new(&NetworkDefinition::simulator())
                    .clear_auth_zone()
                    .build(),
            );

        for signer in signers {
            builder = builder.sign(&EcdsaSecp256k1PrivateKey::from_u64(signer).unwrap());
        }
        builder = builder.notarize(&sk_notary);

        builder.build()
    }
}
