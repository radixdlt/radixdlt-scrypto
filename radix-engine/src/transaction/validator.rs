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
            Instruction::TakeFromWorktop {
                amount,
                resource_def_ref,
            } => {
                id_validator
                    .new_bucket()
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::TakeFromWorktop {
                    amount,
                    resource_def_ref,
                });
            }
            Instruction::TakeAllFromWorktop { resource_def_ref } => {
                id_validator
                    .new_bucket()
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::TakeAllFromWorktop { resource_def_ref });
            }
            Instruction::TakeNonFungiblesFromWorktop {
                keys,
                resource_def_ref,
            } => {
                id_validator
                    .new_bucket()
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::TakeNonFungiblesFromWorktop {
                    keys,
                    resource_def_ref,
                });
            }
            Instruction::ReturnToWorktop { bucket_id } => {
                id_validator
                    .drop_bucket(bucket_id)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::ReturnToWorktop { bucket_id });
            }
            Instruction::AssertWorktopContains {
                amount,
                resource_def_ref,
            } => {
                instructions.push(ValidatedInstruction::AssertWorktopContains {
                    amount,
                    resource_def_ref,
                });
            }
            Instruction::CreateBucketRef { bucket_id } => {
                id_validator
                    .new_bucket_ref(bucket_id)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::CreateBucketRef { bucket_id });
            }
            Instruction::CloneBucketRef { bucket_ref_id } => {
                id_validator
                    .clone_bucket_ref(bucket_ref_id)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::CloneBucketRef { bucket_ref_id });
            }
            Instruction::DropBucketRef { bucket_ref_id } => {
                id_validator
                    .drop_bucket_ref(bucket_ref_id)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::DropBucketRef { bucket_ref_id });
            }
            Instruction::CallFunction {
                package_ref,
                blueprint_name,
                function,
                args,
            } => {
                instructions.push(ValidatedInstruction::CallFunction {
                    package_ref,
                    blueprint_name,
                    function,
                    args: validate_args(args, &mut id_validator)?,
                });
            }
            Instruction::CallMethod {
                component_ref,
                method,
                args,
            } => {
                instructions.push(ValidatedInstruction::CallMethod {
                    component_ref,
                    method,
                    args: validate_args(args, &mut id_validator)?,
                });
            }
            Instruction::CallMethodWithAllResources {
                component_ref,
                method,
            } => {
                id_validator
                    .move_all_resources()
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::CallMethodWithAllResources {
                    component_ref,
                    method,
                });
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
        result.push(validated_arg);
    }
    Ok(result)
}
