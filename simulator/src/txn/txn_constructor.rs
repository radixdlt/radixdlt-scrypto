use radix_engine::execution::*;
use radix_engine::ledger::*;
use radix_engine::model::*;
use sbor::describe::*;
use sbor::*;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::constants::*;
use scrypto::rust::collections::*;
use scrypto::rust::fmt;
use scrypto::rust::str::FromStr;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::abi::*;
use crate::txn::*;

/// Construct a CALL_FUNCTION transaction.
pub fn build_call_function<T: Ledger>(
    ledger: &mut T,
    account: Address,
    package: Address,
    blueprint: &str,
    function: &str,
    args: &Vec<&str>,
    trace: bool,
) -> Result<Transaction, BuildTxnError> {
    let func = get_function_abi(ledger, package, blueprint, function, trace)?;
    let mut alloc = AddressAllocator::new();
    match prepare_args(&func.inputs, args, &mut alloc) {
        Ok((new_args, tokens, badges)) => {
            let mut v = vec![];
            v.push(Instruction::ReserveBuckets {
                n: alloc.count() as u8,
            });
            prepare_buckets(&mut v, &tokens, account, "withdraw_tokens");
            prepare_buckets(&mut v, &badges, account, "withdraw_badges");
            v.push(Instruction::CallFunction {
                package,
                blueprint: blueprint.to_owned(),
                function: function.to_owned(),
                args: new_args,
            });
            v.push(Instruction::DepositAll {
                component: account,
                method: "deposit_buckets".to_owned(),
            });
            v.push(Instruction::Finalize);
            Ok(Transaction { instructions: v })
        }
        Err(e) => Err(BuildTxnError::ArgConstructionError(e)),
    }
}

/// Construct a CALL_METHOD transaction.
pub fn build_call_method<T: Ledger>(
    ledger: &mut T,
    account: Address,
    component: Address,
    method: &str,
    args: &Vec<&str>,
    trace: bool,
) -> Result<Transaction, BuildTxnError> {
    let meth = get_method_abi(ledger, component, method, trace)?;
    let mut alloc = AddressAllocator::new();
    match prepare_args(&meth.inputs, args, &mut alloc) {
        Ok((new_args, tokens, badges)) => {
            let mut v = vec![];
            v.push(Instruction::ReserveBuckets {
                n: alloc.count() as u8,
            });
            prepare_buckets(&mut v, &tokens, account, "withdraw_tokens");
            prepare_buckets(&mut v, &badges, account, "withdraw_badges");
            v.push(Instruction::CallMethod {
                component,
                method: method.to_owned(),
                args: new_args,
            });
            v.push(Instruction::DepositAll {
                component: account,
                method: "deposit_buckets".to_owned(),
            });
            v.push(Instruction::Finalize);
            Ok(Transaction { instructions: v })
        }
        Err(e) => Err(BuildTxnError::ArgConstructionError(e)),
    }
}

fn get_function_abi<T: Ledger>(
    ledger: &mut T,
    package: Address,
    blueprint: &str,
    function: &str,
    trace: bool,
) -> Result<abi::Function, BuildTxnError> {
    export_abi(ledger, package, blueprint, trace)
        .map_err(|e| BuildTxnError::FailedToExportAbi(e))?
        .functions
        .iter()
        .filter(|f| f.name == function)
        .nth(0)
        .map(Clone::clone)
        .ok_or(BuildTxnError::FunctionNotFound(function.to_owned()))
}

fn get_method_abi<T: Ledger>(
    ledger: &mut T,
    component: Address,
    method: &str,
    trace: bool,
) -> Result<abi::Method, BuildTxnError> {
    export_abi_by_component(ledger, component, trace)
        .map_err(|e| BuildTxnError::FailedToExportAbi(e))?
        .methods
        .iter()
        .filter(|m| m.name == method)
        .nth(0)
        .map(Clone::clone)
        .ok_or(BuildTxnError::MethodNotFound(method.to_owned()))
}

fn prepare_args(
    types: &Vec<Type>,
    args: &Vec<&str>,
    alloc: &mut AddressAllocator,
) -> Result<(Vec<Vec<u8>>, HashMap<u8, Bucket>, HashMap<u8, Bucket>), BuildArgError> {
    let mut result = Vec::new();
    let mut tokens = HashMap::new();
    let mut badges = HashMap::new();

    for (i, t) in types.iter().enumerate() {
        let arg = args
            .get(i)
            .ok_or(BuildArgError::MissingArgument(i, t.clone()))?
            .clone();
        let res = match t {
            Type::Bool => handle_basic_ty::<bool>(i, t, arg),
            Type::I8 => handle_basic_ty::<i8>(i, t, arg),
            Type::I16 => handle_basic_ty::<i16>(i, t, arg),
            Type::I32 => handle_basic_ty::<i32>(i, t, arg),
            Type::I64 => handle_basic_ty::<i64>(i, t, arg),
            Type::I128 => handle_basic_ty::<i128>(i, t, arg),
            Type::U8 => handle_basic_ty::<u8>(i, t, arg),
            Type::U16 => handle_basic_ty::<u16>(i, t, arg),
            Type::U32 => handle_basic_ty::<u32>(i, t, arg),
            Type::U64 => handle_basic_ty::<u64>(i, t, arg),
            Type::U128 => handle_basic_ty::<u128>(i, t, arg),
            Type::String => handle_basic_ty::<String>(i, t, arg),
            Type::Custom { name } => {
                handle_custom_ty(i, t, arg, name, alloc, &mut tokens, &mut badges)
            }
            _ => Err(BuildArgError::UnsupportedType(i, t.clone())),
        };
        result.push(res?);
    }

    Ok((result, tokens, badges))
}

fn handle_basic_ty<T>(i: usize, ty: &Type, arg: &str) -> Result<Vec<u8>, BuildArgError>
where
    T: FromStr + Encode,
    T::Err: fmt::Debug,
{
    let value = arg
        .parse::<T>()
        .map_err(|_| BuildArgError::UnableToParse(i, ty.clone(), arg.to_owned()))?;
    Ok(scrypto_encode(&value))
}

fn handle_custom_ty(
    i: usize,
    ty: &Type,
    arg: &str,
    name: &str,
    alloc: &mut AddressAllocator,
    tokens: &mut HashMap<u8, Bucket>,
    badges: &mut HashMap<u8, Bucket>,
) -> Result<Vec<u8>, BuildArgError> {
    match name {
        SCRYPTO_NAME_U256 => handle_basic_ty::<U256>(i, ty, arg),
        SCRYPTO_NAME_ADDRESS => {
            let value = arg
                .parse::<Address>()
                .map_err(|_| BuildArgError::UnableToParse(i, ty.clone(), arg.to_owned()))?;
            Ok(scrypto_encode(&value))
        }
        SCRYPTO_NAME_H256 => {
            let value = arg
                .parse::<Address>()
                .map_err(|_| BuildArgError::UnableToParse(i, ty.clone(), arg.to_owned()))?;
            Ok(scrypto_encode(&value))
        }
        SCRYPTO_NAME_TOKENS | SCRYPTO_NAME_BADGES => {
            let mut split = arg.split(",");
            let amount = split.next().and_then(|v| U256::from_dec_str(v.trim()).ok());
            let resource = split.next().and_then(|v| v.trim().parse::<Address>().ok());
            match (amount, resource) {
                (Some(a), Some(r)) => {
                    let n = alloc.count();
                    if n >= 255 {
                        return Err(BuildArgError::BucketLimitReached);
                    }

                    let bid = alloc.new_transient_bid();
                    if name == SCRYPTO_NAME_TOKENS {
                        tokens.insert(n as u8, Bucket::new(a, r));
                        Ok(scrypto_encode(&scrypto::resource::Tokens::from(bid)))
                    } else {
                        badges.insert(n as u8, Bucket::new(a, r));
                        Ok(scrypto_encode(&scrypto::resource::Badges::from(bid)))
                    }
                }
                _ => Err(BuildArgError::UnableToParse(i, ty.clone(), arg.to_owned())),
            }
        }
        _ => Err(BuildArgError::UnsupportedType(i, ty.clone())),
    }
}

fn prepare_buckets(
    instructions: &mut Vec<Instruction>,
    buckets: &HashMap<u8, Bucket>,
    account: Address,
    method: &str,
) {
    for (offset, bucket) in buckets {
        instructions.push(Instruction::CallMethod {
            component: account,
            method: method.to_owned(),
            args: vec![
                scrypto_encode(&bucket.amount()),
                scrypto_encode(&bucket.resource()),
            ],
        });
        instructions.push(Instruction::NewBucket {
            offset: *offset,
            amount: bucket.amount(),
            resource: bucket.resource(),
        });
    }
}
