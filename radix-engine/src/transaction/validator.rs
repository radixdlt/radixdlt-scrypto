use scrypto::rust::collections::*;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::engine::*;
use crate::model::*;

pub fn validate_transaction(
    transaction: &Transaction,
) -> Result<ValidatedTransaction, TransactionValidationError> {
    let mut instructions = vec![];
    let mut signers = vec![];

    // semantic analysis
    let mut id_allocator = IdAllocator::new(TRANSACTION_OBJECT_ID_RANGE);
    let mut temp_buckets = HashMap::<Bid, usize>::new();
    let mut temp_bucket_refs = HashMap::<Rid, Bid>::new();
    temp_bucket_refs.insert(ECDSA_TOKEN_RID, ECDSA_TOKEN_BID);

    for (i, inst) in transaction.instructions.iter().enumerate() {
        match inst.clone() {
            Instruction::CreateTempBucket {
                amount,
                resource_address,
            } => {
                temp_buckets.insert(
                    id_allocator
                        .new_bid()
                        .map_err(TransactionValidationError::IdAllocatorError)?,
                    0,
                );
                instructions.push(ValidatedInstruction::CreateTempBucket {
                    amount,
                    resource_address,
                });
            }
            Instruction::CreateTempBucketRef { bid } => {
                if !temp_buckets.contains_key(&bid) {
                    return Err(TransactionValidationError::TempBucketNotFound(bid));
                }
                temp_buckets.entry(bid).and_modify(|cnt| *cnt += 1);
                temp_bucket_refs.insert(
                    id_allocator
                        .new_rid()
                        .map_err(TransactionValidationError::IdAllocatorError)?,
                    bid,
                );
                instructions.push(ValidatedInstruction::CreateTempBucketRef { bid });
            }
            Instruction::CloneTempBucketRef { rid } => {
                let bid = if let Some(b) = temp_bucket_refs.get(&rid) {
                    *b
                } else {
                    return Err(TransactionValidationError::TempBucketRefNotFound(rid));
                };
                temp_buckets.entry(bid).and_modify(|cnt| *cnt += 1);
                temp_bucket_refs.insert(
                    id_allocator
                        .new_rid()
                        .map_err(TransactionValidationError::IdAllocatorError)?,
                    bid,
                );
                instructions.push(ValidatedInstruction::CloneTempBucketRef { rid });
            }
            Instruction::DropTempBucketRef { rid } => {
                let bid = if let Some(b) = temp_bucket_refs.get(&rid) {
                    *b
                } else {
                    return Err(TransactionValidationError::TempBucketRefNotFound(rid));
                };
                temp_buckets.entry(bid).and_modify(|cnt| *cnt -= 1);
                temp_bucket_refs.remove(&rid);
                instructions.push(ValidatedInstruction::DropTempBucketRef { rid });
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
                    args: validate_args(args, &mut temp_buckets, &mut temp_bucket_refs)?,
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
                    args: validate_args(args, &mut temp_buckets, &mut temp_bucket_refs)?,
                });
            }
            Instruction::CallMethodWithAllResources {
                component_address,
                method,
            } => {
                temp_buckets.retain(|_, v| *v != 0);
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
    temp_buckets: &mut HashMap<Bid, usize>,
    temp_bucket_refs: &mut HashMap<Rid, Bid>,
) -> Result<Vec<ValidatedData>, TransactionValidationError> {
    let mut validated_args = vec![];
    for arg in args {
        let validated_arg =
            validate_data(&arg).map_err(TransactionValidationError::DataValidationError)?;
        for bid in &validated_arg.buckets {
            if !temp_buckets.contains_key(bid) {
                return Err(TransactionValidationError::TempBucketNotFound(*bid));
            }
            if *temp_buckets.get(bid).unwrap() != 0 {
                return Err(TransactionValidationError::TempBucketLocked(*bid));
            }
            temp_buckets.remove(bid);
        }
        for rid in &validated_arg.bucket_refs {
            if !temp_bucket_refs.contains_key(rid) {
                return Err(TransactionValidationError::TempBucketRefNotFound(*rid));
            }
            temp_bucket_refs.remove(rid);
        }
        validated_args.push(validated_arg);
    }
    Ok(validated_args)
}
