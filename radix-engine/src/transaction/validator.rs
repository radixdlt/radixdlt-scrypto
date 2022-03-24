use scrypto::rust::vec;
use scrypto::rust::vec::Vec;

use crate::engine::*;
use crate::errors::*;
use crate::model::*;

pub fn validate_transaction(
    transaction: &Transaction,
) -> Result<ValidatedTransaction, TransactionValidationError> {
    let mut instructions = vec![];
    let mut signers = vec![];

    // semantic analysis
    let mut id_validator = IdValidator::new();
    for (i, inst) in transaction.instructions.iter().enumerate() {
        match inst.clone() {
            Instruction::TakeFromWorktop { resource_def_id } => {
                id_validator
                    .new_bucket()
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::TakeFromWorktop { resource_def_id });
            }
            Instruction::TakeFromWorktopByAmount {
                amount,
                resource_def_id,
            } => {
                id_validator
                    .new_bucket()
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::TakeFromWorktopByAmount {
                    amount,
                    resource_def_id,
                });
            }
            Instruction::TakeFromWorktopByIds {
                ids,
                resource_def_id,
            } => {
                id_validator
                    .new_bucket()
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::TakeFromWorktopByIds {
                    ids,
                    resource_def_id,
                });
            }
            Instruction::ReturnToWorktop { bucket_id } => {
                id_validator
                    .drop_bucket(bucket_id)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::ReturnToWorktop { bucket_id });
            }
            Instruction::AssertWorktopContains { resource_def_id } => {
                instructions.push(ValidatedInstruction::AssertWorktopContains { resource_def_id });
            }
            Instruction::AssertWorktopContainsByAmount {
                amount,
                resource_def_id,
            } => {
                instructions.push(ValidatedInstruction::AssertWorktopContainsByAmount {
                    amount,
                    resource_def_id,
                });
            }
            Instruction::AssertWorktopContainsByIds {
                ids,
                resource_def_id,
            } => {
                instructions.push(ValidatedInstruction::AssertWorktopContainsByIds {
                    ids,
                    resource_def_id,
                });
            }
            Instruction::TakeFromAuthZone => {
                id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::TakeFromAuthZone);
            }
            Instruction::ReturnToAuthZone { proof_id } => {
                id_validator
                    .drop_proof(proof_id)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::ReturnToAuthZone { proof_id });
            }
            Instruction::ClearAuthZone => {
                instructions.push(ValidatedInstruction::ClearAuthZone);
            }
            Instruction::CreateProofFromAuthZone { resource_def_id } => {
                id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions
                    .push(ValidatedInstruction::CreateProofFromAuthZone { resource_def_id });
            }
            Instruction::CreateProofFromAuthZoneByAmount {
                amount,
                resource_def_id,
            } => {
                id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::CreateProofFromAuthZoneByAmount {
                    amount,
                    resource_def_id,
                });
            }
            Instruction::CreateProofFromAuthZoneByIds {
                ids,
                resource_def_id,
            } => {
                id_validator
                    .new_proof(ProofKind::AuthZoneProof)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::CreateProofFromAuthZoneByIds {
                    ids,
                    resource_def_id,
                });
            }
            Instruction::CreateProofFromBucket { bucket_id } => {
                id_validator
                    .new_proof(ProofKind::BucketProof(bucket_id))
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::CreateProofFromBucket { bucket_id });
            }
            Instruction::CloneProof { proof_id } => {
                id_validator
                    .clone_proof(proof_id)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::CloneProof { proof_id });
            }
            Instruction::DropProof { proof_id } => {
                id_validator
                    .drop_proof(proof_id)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::DropProof { proof_id });
            }
            Instruction::CallFunction {
                package_id,
                blueprint_name,
                function,
                args,
            } => {
                instructions.push(ValidatedInstruction::CallFunction {
                    package_id,
                    blueprint_name,
                    function,
                    args: validate_args(args, &mut id_validator)?,
                });
            }
            Instruction::CallMethod {
                component_id,
                method,
                args,
            } => {
                instructions.push(ValidatedInstruction::CallMethod {
                    component_id,
                    method,
                    args: validate_args(args, &mut id_validator)?,
                });
            }
            Instruction::CallMethodWithAllResources {
                component_id,
                method,
            } => {
                id_validator
                    .move_all_resources()
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::CallMethodWithAllResources {
                    component_id,
                    method,
                });
            }
            Instruction::PublishPackage { code } => {
                instructions.push(ValidatedInstruction::PublishPackage { code });
            }
            Instruction::End { signatures } => {
                if i != transaction.instructions.len() - 1 {
                    return Err(TransactionValidationError::UnexpectedEnd);
                }
                signers.extend(signatures);
            }
        }
    }

    Ok(ValidatedTransaction {
        instructions,
        signers,
    })
}

fn validate_args(
    args: Vec<Vec<u8>>,
    id_validator: &mut IdValidator,
) -> Result<Vec<ValidatedData>, TransactionValidationError> {
    let mut result = vec![];
    for arg in args {
        let validated_arg = ValidatedData::from_slice(&arg)
            .map_err(TransactionValidationError::DataValidationError)?;
        id_validator
            .move_resources(&validated_arg)
            .map_err(TransactionValidationError::IdValidatorError)?;
        if let Some(vault_id) = validated_arg.vault_ids.first() {
            return Err(TransactionValidationError::VaultNotAllowed(
                vault_id.clone(),
            ));
        }
        if let Some(lazy_map_id) = validated_arg.lazy_map_ids.first() {
            return Err(TransactionValidationError::LazyMapNotAllowed(
                lazy_map_id.clone(),
            ));
        }
        result.push(validated_arg);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use scrypto::buffer::*;
    use scrypto::engine::types::*;
    use scrypto::rust::borrow::ToOwned;
    use scrypto::rust::marker::PhantomData;

    #[test]
    fn should_reject_transaction_passing_vault() {
        assert_eq!(
            validate_transaction(&Transaction {
                instructions: vec![Instruction::CallMethod {
                    component_id: ComponentId([1u8; 26]),
                    method: "test".to_owned(),
                    args: vec![scrypto_encode(&scrypto::resource::Vault((
                        Hash([2u8; 32]),
                        0,
                    )))],
                }],
            }),
            Err(TransactionValidationError::VaultNotAllowed((
                Hash([2u8; 32]),
                0,
            ))),
        );
    }

    #[test]
    fn should_reject_transaction_passing_lazy_map() {
        assert_eq!(
            validate_transaction(&Transaction {
                instructions: vec![Instruction::CallMethod {
                    component_id: ComponentId([1u8; 26]),
                    method: "test".to_owned(),
                    args: vec![scrypto_encode(&scrypto::component::LazyMap::<(), ()> {
                        id: (Hash([2u8; 32]), 0,),
                        key: PhantomData,
                        value: PhantomData,
                    })],
                }],
            }),
            Err(TransactionValidationError::LazyMapNotAllowed((
                Hash([2u8; 32]),
                0,
            ))),
        );
    }
}
