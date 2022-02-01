use scrypto::rust::vec;
use scrypto::rust::vec::Vec;

use crate::engine::*;
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
                resource_address,
            } => {
                id_validator
                    .new_bucket()
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::TakeFromWorktop {
                    amount,
                    resource_address,
                });
            }
            Instruction::TakeAllFromWorktop { resource_address } => {
                id_validator
                    .new_bucket()
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::TakeAllFromWorktop { resource_address });
            }
            Instruction::ReturnToWorktop { bid } => {
                id_validator
                    .drop_bucket(bid)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::ReturnToWorktop { bid });
            }
            Instruction::AssertWorktopContains {
                amount,
                resource_address,
            } => {
                instructions.push(ValidatedInstruction::AssertWorktopContains {
                    amount,
                    resource_address,
                });
            }
            Instruction::CreateBucketRef { bid } => {
                id_validator
                    .new_bucket_ref(bid)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::CreateBucketRef { bid });
            }
            Instruction::CloneBucketRef { rid } => {
                id_validator
                    .clone_bucket_ref(rid)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::CloneBucketRef { rid });
            }
            Instruction::DropBucketRef { rid } => {
                id_validator
                    .drop_bucket_ref(rid)
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::DropBucketRef { rid });
            }
            Instruction::CallFunction {
                package_address,
                blueprint_name,
                function,
                args,
            } => {
                instructions.push(ValidatedInstruction::CallFunction {
                    package_address,
                    blueprint_name,
                    function,
                    args: validate_args(args, &mut id_validator)?,
                });
            }
            Instruction::CallMethod {
                component_address,
                method,
                args,
            } => {
                instructions.push(ValidatedInstruction::CallMethod {
                    component_address,
                    method,
                    args: validate_args(args, &mut id_validator)?,
                });
            }
            Instruction::CallMethodWithAllResources {
                component_address,
                method,
            } => {
                id_validator
                    .move_all_resources()
                    .map_err(TransactionValidationError::IdValidatorError)?;
                instructions.push(ValidatedInstruction::CallMethodWithAllResources {
                    component_address,
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
        let validated_arg =
            validate_data(&arg).map_err(TransactionValidationError::DataValidationError)?;
        id_validator.move_resources(&validated_arg)
        .map_err(TransactionValidationError::IdValidatorError)?;
        result.push(validated_arg);
    }
    Ok(result)
}
