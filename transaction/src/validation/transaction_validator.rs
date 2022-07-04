use std::collections::HashSet;

use sbor::rust::vec;
use scrypto::buffer::scrypto_decode;
use scrypto::crypto::*;
use scrypto::values::*;

use crate::errors::{SignatureValidationError, *};
use crate::model::*;
use crate::validation::*;

pub struct TransactionValidator;

impl TransactionValidator {
    pub const MAX_PAYLOAD_SIZE: usize = 4 * 1024 * 1024;

    pub fn validate_from_slice<I: IntentHashManager, E: EpochManager>(
        transaction: &[u8],
        intent_hash_store: &I,
        epoch_manager: &E,
    ) -> Result<ValidatedTransaction, TransactionValidationError> {
        if transaction.len() > Self::MAX_PAYLOAD_SIZE {
            return Err(TransactionValidationError::TransactionTooLarge);
        }

        let transaction: NotarizedTransaction = scrypto_decode(transaction)
            .map_err(TransactionValidationError::DeserializationError)?;

        Self::validate(transaction, intent_hash_store, epoch_manager)
    }

    pub fn validate<I: IntentHashManager, E: EpochManager>(
        transaction: NotarizedTransaction,
        intent_hash_store: &I,
        epoch_manager: &E,
    ) -> Result<ValidatedTransaction, TransactionValidationError> {
        let mut instructions = vec![];

        // verify intent hash
        if !intent_hash_store.allows(&transaction.signed_intent.intent.hash()) {
            return Err(TransactionValidationError::IntentHashRejected);
        }

        // verify header and signature
        Self::validate_header(&transaction, epoch_manager.current_epoch())
            .map_err(TransactionValidationError::HeaderValidationError)?;
        Self::validate_signatures(&transaction)
            .map_err(TransactionValidationError::SignatureValidationError)?;

        // semantic analysis
        let mut id_validator = IdValidator::new();
        for inst in &transaction.signed_intent.intent.manifest.instructions {
            match inst.clone() {
                Instruction::TakeFromWorktop { resource_address } => {
                    id_validator
                        .new_bucket()
                        .map_err(TransactionValidationError::IdValidationError)?;
                    instructions.push(ExecutableInstruction::TakeFromWorktop { resource_address });
                }
                Instruction::TakeFromWorktopByAmount {
                    amount,
                    resource_address,
                } => {
                    id_validator
                        .new_bucket()
                        .map_err(TransactionValidationError::IdValidationError)?;
                    instructions.push(ExecutableInstruction::TakeFromWorktopByAmount {
                        amount,
                        resource_address,
                    });
                }
                Instruction::TakeFromWorktopByIds {
                    ids,
                    resource_address,
                } => {
                    id_validator
                        .new_bucket()
                        .map_err(TransactionValidationError::IdValidationError)?;
                    instructions.push(ExecutableInstruction::TakeFromWorktopByIds {
                        ids,
                        resource_address,
                    });
                }
                Instruction::ReturnToWorktop { bucket_id } => {
                    id_validator
                        .drop_bucket(bucket_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    instructions.push(ExecutableInstruction::ReturnToWorktop { bucket_id });
                }
                Instruction::AssertWorktopContains { resource_address } => {
                    instructions
                        .push(ExecutableInstruction::AssertWorktopContains { resource_address });
                }
                Instruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_address,
                } => {
                    instructions.push(ExecutableInstruction::AssertWorktopContainsByAmount {
                        amount,
                        resource_address,
                    });
                }
                Instruction::AssertWorktopContainsByIds {
                    ids,
                    resource_address,
                } => {
                    instructions.push(ExecutableInstruction::AssertWorktopContainsByIds {
                        ids,
                        resource_address,
                    });
                }
                Instruction::PopFromAuthZone => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    instructions.push(ExecutableInstruction::PopFromAuthZone);
                }
                Instruction::PushToAuthZone { proof_id } => {
                    id_validator
                        .drop_proof(proof_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    instructions.push(ExecutableInstruction::PushToAuthZone { proof_id });
                }
                Instruction::ClearAuthZone => {
                    instructions.push(ExecutableInstruction::ClearAuthZone);
                }
                Instruction::CreateProofFromAuthZone { resource_address } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    instructions
                        .push(ExecutableInstruction::CreateProofFromAuthZone { resource_address });
                }
                Instruction::CreateProofFromAuthZoneByAmount {
                    amount,
                    resource_address,
                } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    instructions.push(ExecutableInstruction::CreateProofFromAuthZoneByAmount {
                        amount,
                        resource_address,
                    });
                }
                Instruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_address,
                } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    instructions.push(ExecutableInstruction::CreateProofFromAuthZoneByIds {
                        ids,
                        resource_address,
                    });
                }
                Instruction::CreateProofFromBucket { bucket_id } => {
                    id_validator
                        .new_proof(ProofKind::BucketProof(bucket_id))
                        .map_err(TransactionValidationError::IdValidationError)?;
                    instructions.push(ExecutableInstruction::CreateProofFromBucket { bucket_id });
                }
                Instruction::CloneProof { proof_id } => {
                    id_validator
                        .clone_proof(proof_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    instructions.push(ExecutableInstruction::CloneProof { proof_id });
                }
                Instruction::DropProof { proof_id } => {
                    id_validator
                        .drop_proof(proof_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    instructions.push(ExecutableInstruction::DropProof { proof_id });
                }
                Instruction::CallFunction {
                    package_address,
                    blueprint_name,
                    method_name,
                    arg,
                } => {
                    // TODO: decode into Value
                    Self::validate_call_data(&arg, &mut id_validator)
                        .map_err(TransactionValidationError::CallDataValidationError)?;
                    instructions.push(ExecutableInstruction::CallFunction {
                        package_address,
                        blueprint_name,
                        method_name,
                        arg,
                    });
                }
                Instruction::CallMethod {
                    component_address,
                    method_name,
                    arg,
                } => {
                    // TODO: decode into Value
                    Self::validate_call_data(&arg, &mut id_validator)
                        .map_err(TransactionValidationError::CallDataValidationError)?;
                    instructions.push(ExecutableInstruction::CallMethod {
                        component_address,
                        method_name,
                        arg,
                    });
                }
                Instruction::CallMethodWithAllResources {
                    component_address,
                    method,
                } => {
                    id_validator
                        .move_all_resources()
                        .map_err(TransactionValidationError::IdValidationError)?;
                    instructions.push(ExecutableInstruction::CallMethodWithAllResources {
                        component_address,
                        method,
                    });
                }
                Instruction::PublishPackage { package } => {
                    instructions.push(ExecutableInstruction::PublishPackage { package });
                }
            }
        }

        // TODO: whether to use intent hash or transaction hash
        let transaction_hash = transaction.hash();

        let mut signer_public_keys: Vec<EcdsaPublicKey> = transaction
            .signed_intent
            .intent_signatures
            .iter()
            .map(|e| e.0)
            .collect();
        if transaction.signed_intent.intent.header.notary_as_signatory {
            signer_public_keys.push(transaction.signed_intent.intent.header.notary_public_key);
        }

        Ok(ValidatedTransaction {
            transaction,
            transaction_hash,
            instructions,
            signer_public_keys,
        })
    }

    fn validate_header(
        transaction: &NotarizedTransaction,
        current_epoch: u64,
    ) -> Result<(), HeaderValidationError> {
        let header = &transaction.signed_intent.intent.header;

        // version
        if header.version != TRANSACTION_VERSION_V1 {
            return Err(HeaderValidationError::UnknownVersion(header.version));
        }

        // epoch
        if header.end_epoch_exclusive <= header.start_epoch_inclusive {
            return Err(HeaderValidationError::InvalidEpochRange);
        }
        if header.end_epoch_exclusive - header.start_epoch_inclusive > MAX_EPOCH_DURATION {
            return Err(HeaderValidationError::EpochRangeTooLarge);
        }
        if current_epoch < header.start_epoch_inclusive
            || current_epoch >= header.end_epoch_exclusive
        {
            return Err(HeaderValidationError::OutOfEpochRange);
        }

        Ok(())
    }

    fn validate_signatures(
        transaction: &NotarizedTransaction,
    ) -> Result<(), SignatureValidationError> {
        // TODO: split into static validation part and runtime validation part to support more signatures
        if transaction.signed_intent.intent_signatures.len() > MAX_NUMBER_OF_INTENT_SIGNATURES {
            return Err(SignatureValidationError::TooManySignatures);
        }

        // verify intent signature
        let intent_payload = transaction.signed_intent.intent.to_bytes();
        let mut signers = HashSet::new();
        for sig in &transaction.signed_intent.intent_signatures {
            if !verify_ecdsa(&intent_payload, &sig.0, &sig.1) {
                return Err(SignatureValidationError::InvalidIntentSignature);
            }
            if !signers.insert(sig.0.to_vec()) {
                return Err(SignatureValidationError::DuplicateSigner);
            }
        }

        // verify notary signature
        let signed_intent_payload = transaction.signed_intent.to_bytes();
        if !verify_ecdsa(
            &signed_intent_payload,
            &transaction.signed_intent.intent.header.notary_public_key,
            &transaction.notary_signature,
        ) {
            return Err(SignatureValidationError::InvalidNotarySignature);
        }

        Ok(())
    }

    fn validate_call_data(
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
    use scrypto::core::Network;

    use super::*;
    use crate::{builder::ManifestBuilder, builder::TransactionBuilder, signing::EcdsaPrivateKey};

    macro_rules! assert_invalid_tx {
        ($result: expr, ($version: expr, $start_epoch: expr, $end_epoch: expr, $nonce: expr, $signers: expr, $notary: expr)) => {{
            let mut hash_mgr: TestIntentHashManager = TestIntentHashManager::new();
            let mut epoch_mgr: TestEpochManager = TestEpochManager::new(0);
            assert_eq!(
                Err($result),
                TransactionValidator::validate(
                    create_transaction(
                        $version,
                        $start_epoch,
                        $end_epoch,
                        $nonce,
                        $signers,
                        $notary
                    ),
                    &mut hash_mgr,
                    &mut epoch_mgr,
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

    fn create_transaction(
        version: u8,
        start_epoch: u64,
        end_epoch: u64,
        nonce: u64,
        signers: Vec<u64>,
        notary: u64,
    ) -> NotarizedTransaction {
        let sk_notary = EcdsaPrivateKey::from_u64(notary).unwrap();

        let mut builder = TransactionBuilder::new()
            .header(TransactionHeader {
                version,
                network: Network::LocalSimulator,
                start_epoch_inclusive: start_epoch,
                end_epoch_exclusive: end_epoch,
                nonce,
                notary_public_key: sk_notary.public_key(),
                notary_as_signatory: false,
            })
            .manifest(ManifestBuilder::new().clear_auth_zone().build());

        for signer in signers {
            builder = builder.sign(&EcdsaPrivateKey::from_u64(signer).unwrap());
        }

        builder = builder.notarize(&sk_notary);

        builder.build()
    }
}
