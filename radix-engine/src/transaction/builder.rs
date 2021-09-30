use sbor::describe::*;
use sbor::*;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::fmt;
use scrypto::rust::str::FromStr;
use scrypto::rust::string::String;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::engine::*;
use crate::model::*;
use crate::transaction::*;

/// A utility for building transactions.
pub struct TransactionBuilder {
    /// The address allocator for calculating reserved bucket id.
    allocator: IdAllocator,
    /// Bucket or BucketRef reservations.
    reservations: Vec<(BID, bool, Bucket)>,
    /// Instructions generated.
    instructions: Vec<Instruction>,
    /// Collected Errors
    errors: Vec<BuildTransactionError>,
}

impl TransactionBuilder {
    /// Starts a new transaction builder.
    pub fn new() -> Self {
        Self {
            allocator: IdAllocator::new(),
            reservations: Vec::new(),
            instructions: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Appends an instruction.
    pub fn instruction(&mut self, inst: Instruction) -> &mut Self {
        self.instructions.push(inst);
        self
    }

    /// Adds instructions for calling a function.
    pub fn call_function(
        &mut self,
        abi: &abi::Blueprint,
        function: &str,
        args: Vec<&str>,
    ) -> &mut Self {
        match Self::find_function_abi(abi, function) {
            Ok(f) => match prepare_args(&f.inputs, args, &mut self.allocator) {
                Ok(ParseArgsResult {
                    result,
                    reservations,
                }) => {
                    self.reservations.extend(reservations);
                    self.instructions.push(Instruction::CallFunction {
                        blueprint: (abi.package.parse().unwrap(), abi.name.clone()),
                        function: function.to_owned(),
                        args: result,
                    });
                }
                Err(e) => {
                    self.errors
                        .push(BuildTransactionError::FailedToBuildArgs(e));
                }
            },
            Err(e) => {
                self.errors.push(e);
            }
        }
        self
    }

    /// Adds instructions for calling a method.
    pub fn call_method(
        &mut self,
        abi: &abi::Blueprint,
        component: Address,
        method: &str,
        args: Vec<&str>,
    ) -> &mut Self {
        match Self::find_method_abi(&abi, method) {
            Ok(m) => match prepare_args(&m.inputs, args, &mut self.allocator) {
                Ok(ParseArgsResult {
                    result,
                    reservations,
                }) => {
                    self.reservations.extend(reservations);
                    self.instructions.push(Instruction::CallMethod {
                        component,
                        method: method.to_owned(),
                        args: result,
                    });
                }
                Err(e) => {
                    self.errors
                        .push(BuildTransactionError::FailedToBuildArgs(e));
                }
            },
            Err(e) => {
                self.errors.push(e);
            }
        }
        self
    }

    /// Builds transaction with an account.
    pub fn build_with(
        &mut self,
        account: Option<Address>,
    ) -> Result<Transaction, BuildTransactionError> {
        let mut v = Vec::new();
        if !self.errors.is_empty() {
            return Err(self.errors[0].clone());
        }
        // Allocate bucket and reference IDs.
        for reservation in &self.reservations {
            v.push(Instruction::ReserveBucket);
            if reservation.1 {
                v.push(Instruction::BorrowBucket { bid: reservation.0 });
            }
        }

        // Withdraw resources from accounts to buckets
        for reservation in &self.reservations {
            v.push(Instruction::CallMethod {
                component: account.ok_or(BuildTransactionError::AccountNotProvided)?,
                method: "withdraw".to_owned(),
                args: vec![
                    scrypto_encode(&reservation.2.amount()),
                    scrypto_encode(&reservation.2.resource_address()),
                ],
            });
            v.push(Instruction::MoveToBucket {
                amount: reservation.2.amount(),
                resource_address: reservation.2.resource_address(),
                bid: reservation.0,
            });
        }

        // Call instructions
        v.extend(self.instructions.clone());

        // Transfer all resource to signer
        if let Some(acc) = account {
            v.push(Instruction::DepositAll {
                component: acc,
                method: "deposit_batch".to_owned(),
            });
        }

        // Finalize
        v.push(Instruction::Finalize);

        Ok(Transaction { instructions: v })
    }

    fn find_function_abi(
        abi: &abi::Blueprint,
        function: &str,
    ) -> Result<abi::Function, BuildTransactionError> {
        abi.functions
            .iter()
            .find(|f| f.name == function)
            .map(Clone::clone)
            .ok_or_else(|| BuildTransactionError::FunctionNotFound(function.to_owned()))
    }

    fn find_method_abi(
        abi: &abi::Blueprint,
        method: &str,
    ) -> Result<abi::Method, BuildTransactionError> {
        abi.methods
            .iter()
            .find(|m| m.name == method)
            .map(Clone::clone)
            .ok_or_else(|| BuildTransactionError::MethodNotFound(method.to_owned()))
    }
}

struct ParseArgsResult {
    result: Vec<Vec<u8>>,
    reservations: Vec<(BID, bool, Bucket)>,
}

fn prepare_args(
    types: &[Type],
    args: Vec<&str>,
    allocator: &mut IdAllocator,
) -> Result<ParseArgsResult, BuildArgsError> {
    let mut result = Vec::new();
    let mut reservations = Vec::new();

    for (i, t) in types.iter().enumerate() {
        let arg = *(args
            .get(i)
            .ok_or_else(|| BuildArgsError::MissingArgument(i, t.clone()))?);
        let res = match t {
            Type::Bool => parse_basic_type::<bool>(i, t, arg),
            Type::I8 => parse_basic_type::<i8>(i, t, arg),
            Type::I16 => parse_basic_type::<i16>(i, t, arg),
            Type::I32 => parse_basic_type::<i32>(i, t, arg),
            Type::I64 => parse_basic_type::<i64>(i, t, arg),
            Type::I128 => parse_basic_type::<i128>(i, t, arg),
            Type::U8 => parse_basic_type::<u8>(i, t, arg),
            Type::U16 => parse_basic_type::<u16>(i, t, arg),
            Type::U32 => parse_basic_type::<u32>(i, t, arg),
            Type::U64 => parse_basic_type::<u64>(i, t, arg),
            Type::U128 => parse_basic_type::<u128>(i, t, arg),
            Type::String => parse_basic_type::<String>(i, t, arg),
            Type::Custom { name } => parse_custom_ty(i, t, arg, name, allocator, &mut reservations),
            _ => Err(BuildArgsError::UnsupportedType(i, t.clone())),
        };
        result.push(res?);
    }

    Ok(ParseArgsResult {
        result,
        reservations,
    })
}

fn parse_basic_type<T>(i: usize, ty: &Type, arg: &str) -> Result<Vec<u8>, BuildArgsError>
where
    T: FromStr + Encode,
    T::Err: fmt::Debug,
{
    let value = arg
        .parse::<T>()
        .map_err(|_| BuildArgsError::UnableToParse(i, ty.clone(), arg.to_owned()))?;
    Ok(scrypto_encode(&value))
}

fn parse_custom_ty(
    i: usize,
    ty: &Type,
    arg: &str,
    name: &str,
    allocator: &mut IdAllocator,
    reservations: &mut Vec<(BID, bool, Bucket)>,
) -> Result<Vec<u8>, BuildArgsError> {
    match name {
        SCRYPTO_NAME_AMOUNT => {
            let value = arg
                .parse::<Amount>()
                .map_err(|_| BuildArgsError::UnableToParse(i, ty.clone(), arg.to_owned()))?;
            Ok(scrypto_encode(&value))
        }
        SCRYPTO_NAME_ADDRESS => {
            let value = arg
                .parse::<Address>()
                .map_err(|_| BuildArgsError::UnableToParse(i, ty.clone(), arg.to_owned()))?;
            Ok(scrypto_encode(&value))
        }
        SCRYPTO_NAME_H256 => {
            let value = arg
                .parse::<Address>()
                .map_err(|_| BuildArgsError::UnableToParse(i, ty.clone(), arg.to_owned()))?;
            Ok(scrypto_encode(&value))
        }
        SCRYPTO_NAME_BID | SCRYPTO_NAME_BUCKET | SCRYPTO_NAME_RID | SCRYPTO_NAME_BUCKET_REF => {
            let mut split = arg.split(',');
            let amount = split.next().and_then(|v| v.trim().parse::<Amount>().ok());
            let resource_address = split.next().and_then(|v| v.trim().parse::<Address>().ok());
            match (amount, resource_address) {
                (Some(a), Some(r)) => {
                    let bid = allocator.new_bid();
                    let bucket = Bucket::new(a, r);

                    match name {
                        SCRYPTO_NAME_BID => {
                            reservations.push((bid, false, bucket));
                            Ok(scrypto_encode(&bid))
                        }
                        SCRYPTO_NAME_BUCKET => {
                            reservations.push((bid, false, bucket));
                            Ok(scrypto_encode(&scrypto::resource::Bucket::from(bid)))
                        }
                        SCRYPTO_NAME_RID => {
                            let rid = allocator.new_rid();
                            reservations.push((bid, true, bucket));
                            Ok(scrypto_encode(&rid))
                        }
                        SCRYPTO_NAME_BUCKET_REF => {
                            let rid = allocator.new_rid();
                            reservations.push((bid, true, bucket));
                            Ok(scrypto_encode(&scrypto::resource::BucketRef::from(rid)))
                        }
                        _ => panic!("Unexpected"),
                    }
                }
                _ => Err(BuildArgsError::UnableToParse(i, ty.clone(), arg.to_owned())),
            }
        }
        _ => Err(BuildArgsError::UnsupportedType(i, ty.clone())),
    }
}
