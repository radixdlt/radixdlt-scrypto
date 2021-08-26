use scrypto::abi;
use scrypto::types::*;

use crate::transaction::*;

/// Construct a CALL_FUNCTION transaction.
pub fn construct_call_function_txn(
    package: Address,
    blueprint: &str,
    function: &str,
    args: &Vec<&str>,
    trace: bool,
) -> Result<Transaction, TxnConstructionError> {
    let func = get_function_abi(package, blueprint, function, trace)?;
    let mut allocator = BidAllocator::new();
    match parse_args(&func.inputs, args, &mut allocator) {
        Ok((new_args, buckets)) => {
            let mut v = vec![];
            v.push(Instruction::ReserveBuckets { n: allocator.len() });
            for (bid, bucket) in buckets {
                v.push(Instruction::PrepareBucket {
                    offset: allocator.offset(bid).unwrap(),
                    amount: bucket.amount(),
                    resource: bucket.resource(),
                });
            }
            v.push(Instruction::CallFunction {
                package,
                blueprint: blueprint.to_owned(),
                function: function.to_owned(),
                args: new_args,
            });
            Ok(Transaction { instructions: v })
        }
        Err(e) => Err(TxnConstructionError::InvalidArguments(e)),
    }
}

/// Construct a CALL_METHOD transaction.
pub fn construct_call_method_txn(
    component: Address,
    method: &str,
    args: &Vec<&str>,
    trace: bool,
) -> Result<Transaction, TxnConstructionError> {
    let meth = get_method_abi(component, method, trace)?;
    let mut allocator = BidAllocator::new();
    match parse_args(&meth.inputs, args, &mut allocator) {
        Ok((new_args, buckets)) => {
            let mut v = vec![];
            v.push(Instruction::ReserveBuckets { n: allocator.len() });
            for (bid, bucket) in buckets {
                v.push(Instruction::PrepareBucket {
                    offset: allocator.offset(bid).unwrap(),
                    amount: bucket.amount(),
                    resource: bucket.resource(),
                });
            }
            v.push(Instruction::CallMethod {
                component,
                method: method.to_owned(),
                args: new_args,
            });
            Ok(Transaction { instructions: v })
        }
        Err(e) => Err(TxnConstructionError::InvalidArguments(e)),
    }
}

/// Returns the ABI of a function.
pub fn get_function_abi(
    package: Address,
    blueprint: &str,
    function: &str,
    trace: bool,
) -> Result<abi::Function, TxnConstructionError> {
    export_abi(package, blueprint, trace)
        .map_err(|e| TxnConstructionError::FailedToExportAbi(e))?
        .functions
        .iter()
        .filter(|f| f.name == function)
        .nth(0)
        .map(Clone::clone)
        .ok_or(TxnConstructionError::FunctionNotFound(function.to_owned()))
}

/// Returns the ABI of a method.
pub fn get_method_abi(
    component: Address,
    method: &str,
    trace: bool,
) -> Result<abi::Method, TxnConstructionError> {
    export_abi_by_component(component, trace)
        .map_err(|e| TxnConstructionError::FailedToExportAbi(e))?
        .methods
        .iter()
        .filter(|m| m.name == method)
        .nth(0)
        .map(Clone::clone)
        .ok_or(TxnConstructionError::MethodNotFound(method.to_owned()))
}
