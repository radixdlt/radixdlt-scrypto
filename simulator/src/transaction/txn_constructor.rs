use radix_engine::execution::*;
use radix_engine::model::*;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::rust::collections::*;
use scrypto::types::*;

use crate::transaction::*;

fn withdraw(
    insts: &mut Vec<Instruction>,
    buckets: &HashMap<u8, Bucket>,
    account: Address,
    method: &str,
) {
    for (offset, bucket) in buckets {
        insts.push(Instruction::CallMethod {
            component: account,
            method: method.to_owned(),
            args: vec![
                scrypto_encode(&bucket.amount()),
                scrypto_encode(&bucket.resource()),
            ],
        });
        insts.push(Instruction::NewBucket {
            offset: *offset,
            amount: bucket.amount(),
            resource: bucket.resource(),
        });
    }
}

/// Construct a CALL_FUNCTION transaction.
pub fn construct_call_function_txn(
    account: Address,
    package: Address,
    blueprint: &str,
    function: &str,
    args: &Vec<&str>,
    trace: bool,
) -> Result<Transaction, TxnConstructionError> {
    let func = get_function_abi(package, blueprint, function, trace)?;
    let mut alloc = AddressAllocator::new();
    match parse_args(&func.inputs, args, &mut alloc) {
        Ok((new_args, tokens, badges)) => {
            let mut v = vec![];
            v.push(Instruction::ReserveBuckets {
                n: alloc.count() as u8,
            });
            withdraw(&mut v, &tokens, account, "withdraw_tokens");
            withdraw(&mut v, &badges, account, "withdraw_badges");
            v.push(Instruction::CallFunction {
                package,
                blueprint: blueprint.to_owned(),
                function: function.to_owned(),
                args: new_args,
            });
            v.push(Instruction::DepositAll {
                component: account,
                method: "deposit_tokens".to_owned(),
            });
            v.push(Instruction::Finalize);
            Ok(Transaction { instructions: v })
        }
        Err(e) => Err(TxnConstructionError::InvalidArguments(e)),
    }
}

/// Construct a CALL_METHOD transaction.
pub fn construct_call_method_txn(
    account: Address,
    component: Address,
    method: &str,
    args: &Vec<&str>,
    trace: bool,
) -> Result<Transaction, TxnConstructionError> {
    let meth = get_method_abi(component, method, trace)?;
    let mut alloc = AddressAllocator::new();
    match parse_args(&meth.inputs, args, &mut alloc) {
        Ok((new_args, tokens, badges)) => {
            let mut v = vec![];
            v.push(Instruction::ReserveBuckets {
                n: alloc.count() as u8,
            });
            withdraw(&mut v, &tokens, account, "withdraw_tokens");
            withdraw(&mut v, &badges, account, "withdraw_badges");
            v.push(Instruction::CallMethod {
                component,
                method: method.to_owned(),
                args: new_args,
            });
            v.push(Instruction::DepositAll {
                component: account,
                method: "deposit_tokens".to_owned(),
            });
            v.push(Instruction::Finalize);
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
