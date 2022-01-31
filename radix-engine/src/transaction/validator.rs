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
    let mut buckets = HashMap::<Bid, usize>::new();
    let mut bucket_refs = HashMap::<Rid, Bid>::new();
    bucket_refs.insert(ECDSA_TOKEN_RID, ECDSA_TOKEN_BID);

    for (i, inst) in transaction.instructions.iter().enumerate() {
        match inst.clone() {
            Instruction::TakeFromContext {
                amount,
                resource_address,
            } => {
                buckets.insert(
                    id_allocator
                        .new_bid()
                        .map_err(TransactionValidationError::IdAllocatorError)?,
                    0,
                );
                instructions.push(ValidatedInstruction::TakeFromContext {
                    amount,
                    resource_address,
                });
            }
            Instruction::TakeAllFromContext { resource_address } => {
                buckets.insert(
                    id_allocator
                        .new_bid()
                        .map_err(TransactionValidationError::IdAllocatorError)?,
                    0,
                );
                instructions.push(ValidatedInstruction::TakeAllFromContext { resource_address });
            }
            Instruction::PutIntoContext { bid } => {
                if !buckets.contains_key(&bid) {
                    return Err(TransactionValidationError::BucketNotFound(bid));
                }
                if *buckets.get(&bid).unwrap() != 0 {
                    return Err(TransactionValidationError::BucketLocked(bid));
                }
                buckets.remove(&bid);
                instructions.push(ValidatedInstruction::PutIntoContext { bid });
            }
            Instruction::AssertContextContains {
                amount,
                resource_address,
            } => {
                instructions.push(ValidatedInstruction::AssertContextContains {
                    amount,
                    resource_address,
                });
            }
            Instruction::CreateBucketRef { bid } => {
                if !buckets.contains_key(&bid) {
                    return Err(TransactionValidationError::BucketNotFound(bid));
                }
                buckets.entry(bid).and_modify(|cnt| *cnt += 1);
                bucket_refs.insert(
                    id_allocator
                        .new_rid()
                        .map_err(TransactionValidationError::IdAllocatorError)?,
                    bid,
                );
                instructions.push(ValidatedInstruction::CreateBucketRef { bid });
            }
            Instruction::CloneBucketRef { rid } => {
                let bid = if let Some(b) = bucket_refs.get(&rid) {
                    *b
                } else {
                    return Err(TransactionValidationError::BucketRefNotFound(rid));
                };
                buckets.entry(bid).and_modify(|cnt| *cnt += 1);
                bucket_refs.insert(
                    id_allocator
                        .new_rid()
                        .map_err(TransactionValidationError::IdAllocatorError)?,
                    bid,
                );
                instructions.push(ValidatedInstruction::CloneBucketRef { rid });
            }
            Instruction::DropBucketRef { rid } => {
                let bid = if let Some(b) = bucket_refs.get(&rid) {
                    *b
                } else {
                    return Err(TransactionValidationError::BucketRefNotFound(rid));
                };
                buckets.entry(bid).and_modify(|cnt| *cnt -= 1);
                bucket_refs.remove(&rid);
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
                    args: validate_args(args, &mut buckets, &mut bucket_refs)?,
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
                    args: validate_args(args, &mut buckets, &mut bucket_refs)?,
                });
            }
            Instruction::CallMethodWithAllResources {
                component_address,
                method,
            } => {
                buckets.retain(|_, v| *v != 0);
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
    buckets: &mut HashMap<Bid, usize>,
    bucket_refs: &mut HashMap<Rid, Bid>,
) -> Result<Vec<ValidatedData>, TransactionValidationError> {
    let mut validated_args = vec![];
    for arg in args {
        let validated_arg =
            validate_data(&arg).map_err(TransactionValidationError::DataValidationError)?;
        for bid in &validated_arg.buckets {
            if !buckets.contains_key(bid) {
                return Err(TransactionValidationError::BucketNotFound(*bid));
            }
            if *buckets.get(bid).unwrap() != 0 {
                return Err(TransactionValidationError::BucketLocked(*bid));
            }
            buckets.remove(bid);
        }
        for rid in &validated_arg.bucket_refs {
            if !bucket_refs.contains_key(rid) {
                return Err(TransactionValidationError::BucketRefNotFound(*rid));
            }
            bucket_refs.remove(rid);
        }
        validated_args.push(validated_arg);
    }
    Ok(validated_args)
}
