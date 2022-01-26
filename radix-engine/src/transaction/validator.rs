use scrypto::rust::vec;

use crate::engine::*;
use crate::model::*;

pub fn validate_transaction(
    transaction: &Transaction,
) -> Result<ValidatedTransaction, TransactionValidationError> {
    // TODO should also consider semantic check, e.g. unused temp bucket/-ref.

    let mut instructions = vec![];
    let mut signers = vec![];
    for (i, inst) in transaction.instructions.iter().enumerate() {
        match inst.clone() {
            Instruction::DeclareTempBucket => {
                instructions.push(ValidatedInstruction::DeclareTempBucket);
            }
            Instruction::DeclareTempBucketRef => {
                instructions.push(ValidatedInstruction::DeclareTempBucketRef);
            }
            Instruction::TakeFromContext {
                amount,
                resource_address,
                to,
            } => {
                instructions.push(ValidatedInstruction::TakeFromContext {
                    amount,
                    resource_address,
                    to,
                });
            }
            Instruction::BorrowFromContext {
                amount,
                resource_address,
                to,
            } => {
                instructions.push(ValidatedInstruction::BorrowFromContext {
                    amount,
                    resource_address,
                    to,
                });
            }
            Instruction::CallFunction {
                package_address,
                blueprint_name,
                function,
                args,
            } => {
                let mut checked_args = vec![];
                for arg in args {
                    checked_args.push(
                        validate_data(&arg)
                            .map_err(TransactionValidationError::DataValidationError)?,
                    );
                }
                instructions.push(ValidatedInstruction::CallFunction {
                    package_address,
                    blueprint_name,
                    function,
                    args: checked_args,
                });
            }
            Instruction::CallMethod {
                component_address,
                method,
                args,
            } => {
                let mut checked_args = vec![];
                for arg in args {
                    checked_args.push(
                        validate_data(&arg)
                            .map_err(TransactionValidationError::DataValidationError)?,
                    );
                }
                instructions.push(ValidatedInstruction::CallMethod {
                    component_address,
                    method,
                    args: checked_args,
                });
            }
            Instruction::CallMethodWithAllResources {
                component_address,
                method,
            } => {
                instructions.push(ValidatedInstruction::CallMethodWithAllResources {
                    component_address,
                    method,
                });
            }
            Instruction::DropAllBucketRefs => {
                instructions.push(ValidatedInstruction::DropAllBucketRefs);
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
