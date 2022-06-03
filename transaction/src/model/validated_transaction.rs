use sbor::rust::vec;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_decode;
use scrypto::crypto::*;
use scrypto::values::*;

use crate::errors::*;
use crate::model::*;
use crate::validation::*;

/// Represents a validated transaction
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub struct ValidatedTransaction {
    pub transaction: Transaction,
    pub validated_instructions: Vec<ValidatedInstruction>,
    pub signers: Vec<EcdsaPublicKey>,
}

impl ValidatedTransaction {
    pub fn from_slice(
        transaction: &[u8],
        current_epoch: u64,
    ) -> Result<Self, TransactionValidationError> {
        let transaction: Transaction = scrypto_decode(transaction)
            .map_err(TransactionValidationError::DeserializationError)?;

        Self::from_struct(transaction, current_epoch)
    }

    pub fn from_struct(
        transaction: Transaction,
        current_epoch: u64,
    ) -> Result<Self, TransactionValidationError> {
        let mut validated_instructions = vec![];

        // verify header and signature
        transaction
            .validate_header(current_epoch)
            .map_err(TransactionValidationError::HeaderValidationError)?;
        transaction
            .validate_signatures()
            .map_err(TransactionValidationError::SignatureValidationError)?;

        // semantic analysis
        let mut id_validator = IdValidator::new();
        for inst in &transaction.signed_intent.intent.manifest.instructions {
            match inst.clone() {
                Instruction::TakeFromWorktop { resource_address } => {
                    id_validator
                        .new_bucket()
                        .map_err(TransactionValidationError::IdValidationError)?;
                    validated_instructions
                        .push(ValidatedInstruction::TakeFromWorktop { resource_address });
                }
                Instruction::TakeFromWorktopByAmount {
                    amount,
                    resource_address,
                } => {
                    id_validator
                        .new_bucket()
                        .map_err(TransactionValidationError::IdValidationError)?;
                    validated_instructions.push(ValidatedInstruction::TakeFromWorktopByAmount {
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
                    validated_instructions.push(ValidatedInstruction::TakeFromWorktopByIds {
                        ids,
                        resource_address,
                    });
                }
                Instruction::ReturnToWorktop { bucket_id } => {
                    id_validator
                        .drop_bucket(bucket_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    validated_instructions
                        .push(ValidatedInstruction::ReturnToWorktop { bucket_id });
                }
                Instruction::AssertWorktopContains { resource_address } => {
                    validated_instructions
                        .push(ValidatedInstruction::AssertWorktopContains { resource_address });
                }
                Instruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_address,
                } => {
                    validated_instructions.push(
                        ValidatedInstruction::AssertWorktopContainsByAmount {
                            amount,
                            resource_address,
                        },
                    );
                }
                Instruction::AssertWorktopContainsByIds {
                    ids,
                    resource_address,
                } => {
                    validated_instructions.push(ValidatedInstruction::AssertWorktopContainsByIds {
                        ids,
                        resource_address,
                    });
                }
                Instruction::PopFromAuthZone => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    validated_instructions.push(ValidatedInstruction::PopFromAuthZone);
                }
                Instruction::PushToAuthZone { proof_id } => {
                    id_validator
                        .drop_proof(proof_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    validated_instructions.push(ValidatedInstruction::PushToAuthZone { proof_id });
                }
                Instruction::ClearAuthZone => {
                    validated_instructions.push(ValidatedInstruction::ClearAuthZone);
                }
                Instruction::CreateProofFromAuthZone { resource_address } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    validated_instructions
                        .push(ValidatedInstruction::CreateProofFromAuthZone { resource_address });
                }
                Instruction::CreateProofFromAuthZoneByAmount {
                    amount,
                    resource_address,
                } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    validated_instructions.push(
                        ValidatedInstruction::CreateProofFromAuthZoneByAmount {
                            amount,
                            resource_address,
                        },
                    );
                }
                Instruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_address,
                } => {
                    id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    validated_instructions.push(
                        ValidatedInstruction::CreateProofFromAuthZoneByIds {
                            ids,
                            resource_address,
                        },
                    );
                }
                Instruction::CreateProofFromBucket { bucket_id } => {
                    id_validator
                        .new_proof(ProofKind::BucketProof(bucket_id))
                        .map_err(TransactionValidationError::IdValidationError)?;
                    validated_instructions
                        .push(ValidatedInstruction::CreateProofFromBucket { bucket_id });
                }
                Instruction::CloneProof { proof_id } => {
                    id_validator
                        .clone_proof(proof_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    validated_instructions.push(ValidatedInstruction::CloneProof { proof_id });
                }
                Instruction::DropProof { proof_id } => {
                    id_validator
                        .drop_proof(proof_id)
                        .map_err(TransactionValidationError::IdValidationError)?;
                    validated_instructions.push(ValidatedInstruction::DropProof { proof_id });
                }
                Instruction::CallFunction {
                    package_address,
                    blueprint_name,
                    call_data,
                } => {
                    validated_instructions.push(ValidatedInstruction::CallFunction {
                        package_address,
                        blueprint_name,
                        call_data: Self::validate_call_data(call_data, &mut id_validator)
                            .map_err(TransactionValidationError::CallDataValidationError)?,
                    });
                }
                Instruction::CallMethod {
                    component_address,
                    call_data,
                } => {
                    validated_instructions.push(ValidatedInstruction::CallMethod {
                        component_address,
                        call_data: Self::validate_call_data(call_data, &mut id_validator)
                            .map_err(TransactionValidationError::CallDataValidationError)?,
                    });
                }
                Instruction::CallMethodWithAllResources {
                    component_address,
                    method,
                } => {
                    id_validator
                        .move_all_resources()
                        .map_err(TransactionValidationError::IdValidationError)?;
                    validated_instructions.push(ValidatedInstruction::CallMethodWithAllResources {
                        component_address,
                        method,
                    });
                }
                Instruction::PublishPackage { package } => {
                    validated_instructions.push(ValidatedInstruction::PublishPackage { package });
                }
            }
        }

        let signers = transaction
            .signed_intent
            .intent_signatures
            .iter()
            .map(|e| e.0)
            .collect();

        Ok(Self {
            transaction,
            validated_instructions,
            signers,
        })
    }

    fn validate_call_data(
        call_data: Vec<u8>,
        id_validator: &mut IdValidator,
    ) -> Result<ScryptoValue, CallDataValidationError> {
        let value = ScryptoValue::from_slice(&call_data)
            .map_err(CallDataValidationError::InvalidScryptoValue)?;
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
