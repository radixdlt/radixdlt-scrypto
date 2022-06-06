use sbor::rust::vec;
use scrypto::buffer::scrypto_decode;
use scrypto::crypto::*;
use scrypto::values::*;

use crate::errors::{SignatureValidationError, *};
use crate::model::*;
use crate::validation::*;

pub struct TransactionValidator {}

impl TransactionValidator {
    pub fn validate_slice(
        transaction: &[u8],
        current_epoch: u64,
    ) -> Result<ValidatedTransaction, TransactionValidationError> {
        let transaction: NotarizedTransaction = scrypto_decode(transaction)
            .map_err(TransactionValidationError::DeserializationError)?;

        Self::validate(transaction, current_epoch)
    }

    pub fn validate(
        transaction: NotarizedTransaction,
        current_epoch: u64,
    ) -> Result<ValidatedTransaction, TransactionValidationError> {
        let mut instructions = vec![];

        // verify header and signature
        Self::validate_header(&transaction, current_epoch)
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
                    call_data,
                } => {
                    Self::validate_call_data(&call_data, &mut id_validator)
                        .map_err(TransactionValidationError::CallDataValidationError)?;
                    instructions.push(ExecutableInstruction::CallFunction {
                        package_address,
                        blueprint_name,
                        call_data,
                    });
                }
                Instruction::CallMethod {
                    component_address,
                    call_data,
                } => {
                    Self::validate_call_data(&call_data, &mut id_validator)
                        .map_err(TransactionValidationError::CallDataValidationError)?;
                    instructions.push(ExecutableInstruction::CallMethod {
                        component_address,
                        call_data,
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

        let signer_public_keys = transaction
            .signed_intent
            .intent_signatures
            .iter()
            .map(|e| e.0)
            .collect();

        Ok(ValidatedTransaction {
            transaction,
            transaction_hash,
            instructions,
            signer_public_keys,
        })
    }

    fn validate_header(
        transaction: &NotarizedTransaction,
        _current_epoch: u64,
    ) -> Result<(), HeaderValidationError> {
        let _header = &transaction.signed_intent.intent.header;

        // TODO: validate headers

        Ok(())
    }

    fn validate_signatures(
        transaction: &NotarizedTransaction,
    ) -> Result<(), SignatureValidationError> {
        // verify intent signature
        let intent_payload = transaction.signed_intent.intent.to_bytes();
        for sig in &transaction.signed_intent.intent_signatures {
            if !EcdsaVerifier::verify(&intent_payload, &sig.0, &sig.1) {
                return Err(SignatureValidationError::InvalidIntentSignature);
            }
        }

        // verify notary signature
        let signed_intent_payload = transaction.signed_intent.to_bytes();
        if !EcdsaVerifier::verify(
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
    ) -> Result<ScryptoValue, CallDataValidationError> {
        let value =
            ScryptoValue::from_slice(call_data).map_err(CallDataValidationError::DecodeError)?;
        id_validator
            .move_resources(&value)
            .map_err(CallDataValidationError::IdValidationError)?;
        if let Some(vault_id) = value.vault_ids.iter().nth(0) {
            return Err(CallDataValidationError::VaultNotAllowed(vault_id.clone()));
        }
        if let Some(lazy_map_id) = value.lazy_map_ids.iter().nth(0) {
            return Err(CallDataValidationError::LazyMapNotAllowed(
                lazy_map_id.clone(),
            ));
        }
        Ok(value)
    }
}
