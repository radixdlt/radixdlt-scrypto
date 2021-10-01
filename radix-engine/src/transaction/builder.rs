use sbor::describe::*;
use sbor::*;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::*;
use scrypto::rust::fmt;
use scrypto::rust::str::FromStr;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
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
    reservations: Vec<(Address, Vec<Resource>)>,
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

    /// Publishes a package.
    pub fn publish_package(&mut self, code: &[u8]) -> &mut Self {
        self.instruction(Instruction::CallFunction {
            blueprint: (SYSTEM_PACKAGE, "System".to_owned()),
            function: "publish_package".to_string(),
            args: vec![scrypto_encode(code)],
        })
    }

    /// Creates an Account component.
    pub fn new_account(&mut self) -> &mut Self {
        self.instruction(Instruction::CallFunction {
            blueprint: (ACCOUNT_PACKAGE, "Account".to_owned()),
            function: "new".to_string(),
            args: vec![],
        })
    }

    /// Creates a resource with mutable supply.
    pub fn new_resource_mutable(&mut self, metadata: HashMap<String, String>) -> &mut Self {
        self.instruction(Instruction::CallFunction {
            blueprint: (SYSTEM_PACKAGE, "System".to_owned()),
            function: "new_resource_mutable".to_string(),
            args: vec![scrypto_encode(&metadata)],
        })
    }

    /// Creates a resource with fixed supply.
    pub fn new_resource_fixed(
        &mut self,
        metadata: HashMap<String, String>,
        supply: Amount,
    ) -> &mut Self {
        self.instruction(Instruction::CallFunction {
            blueprint: (SYSTEM_PACKAGE, "System".to_owned()),
            function: "new_resource_fixed".to_string(),
            args: vec![scrypto_encode(&metadata), scrypto_encode(&supply)],
        })
    }

    /// Mints resource.
    pub fn mint_resource(&mut self, amount: Amount, resource_def: Address) -> &mut Self {
        self.instruction(Instruction::CallFunction {
            blueprint: (SYSTEM_PACKAGE, "System".to_owned()),
            function: "mint_resource".to_string(),
            args: vec![scrypto_encode(&amount), scrypto_encode(&resource_def)],
        })
    }

    /// Withdraws resource from an account.
    pub fn withdraw(
        &mut self,
        amount: Amount,
        resource_def: Address,
        account: Address,
    ) -> &mut Self {
        self.instruction(Instruction::CallMethod {
            component: account,
            method: "withdraw".to_string(),
            args: vec![scrypto_encode(&amount), scrypto_encode(&resource_def)],
        })
    }

    /// Deposits everything to an account.
    pub fn deposit_all(&mut self, account: Address) -> &mut Self {
        self.instruction(Instruction::DepositAll {
            component: account,
            method: "deposit_batch".to_string(),
        })
    }

    /// Appends a raw instruction.
    pub fn instruction(&mut self, inst: Instruction) -> &mut Self {
        self.instructions.push(inst);
        self
    }

    /// Calls a function.
    pub fn call_function(
        &mut self,
        abi: &abi::Blueprint,
        function: &str,
        args: Vec<String>,
        account: Address,
    ) -> &mut Self {
        match Self::find_function_abi(abi, function.as_ref()) {
            Ok(f) => match prepare_args(&f.inputs, args, &mut self.allocator) {
                Ok(ParseArgsResult { encoded, resources }) => {
                    self.reservations.push((account, resources));
                    self.instructions.push(Instruction::CallFunction {
                        blueprint: (abi.package.parse().unwrap(), abi.name.clone()),
                        function: function.to_owned(),
                        args: encoded,
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

    /// Calls a method.
    pub fn call_method(
        &mut self,
        abi: &abi::Blueprint,
        component: Address,
        method: &str,
        args: Vec<String>,
        account: Address,
    ) -> &mut Self {
        match Self::find_method_abi(&abi, method.as_ref()) {
            Ok(m) => match prepare_args(&m.inputs, args, &mut self.allocator) {
                Ok(ParseArgsResult { encoded, resources }) => {
                    self.reservations.push((account, resources));
                    self.instructions.push(Instruction::CallMethod {
                        component,
                        method: method.to_owned(),
                        args: encoded,
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

    /// Builds the transaction.
    pub fn build(&mut self) -> Result<Transaction, BuildTransactionError> {
        if !self.errors.is_empty() {
            return Err(self.errors[0].clone());
        }
        let mut v = Vec::new();

        // Reserve buckets and references.
        for (_, resources) in &self.reservations {
            for r in resources {
                v.push(Instruction::CreateBucket {
                    resource_def: r.bucket.resource_def(),
                });
                if let Some(_) = r.rid {
                    v.push(Instruction::BorrowBucket { bucket: r.bid });
                }
            }
        }

        // Withdraw resources from account and move them to buckets.
        for (account, resources) in &self.reservations {
            for r in resources {
                v.push(Instruction::CallMethod {
                    component: account.clone(),
                    method: "withdraw".to_owned(),
                    args: vec![
                        scrypto_encode(&r.bucket.amount()),
                        scrypto_encode(&r.bucket.resource_def()),
                    ],
                });
                v.push(Instruction::MoveToBucket {
                    amount: r.bucket.amount(),
                    resource_def: r.bucket.resource_def(),
                    bucket: r.bid,
                });
            }
        }

        // Extend with instructions in the builder.
        v.extend(self.instructions.clone());

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

struct Resource {
    pub bucket: Bucket,
    pub bid: BID,
    pub rid: Option<RID>,
}

struct ParseArgsResult {
    pub encoded: Vec<Vec<u8>>,
    pub resources: Vec<Resource>,
}

fn prepare_args(
    types: &[Type],
    args: Vec<String>,
    allocator: &mut IdAllocator,
) -> Result<ParseArgsResult, BuildArgsError> {
    let mut encoded = Vec::new();
    let mut resources = Vec::new();

    for (i, t) in types.iter().enumerate() {
        let arg = args
            .get(i)
            .ok_or_else(|| BuildArgsError::MissingArgument(i, t.clone()))?;
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
            Type::Custom { name } => parse_custom_ty(i, t, arg, name, allocator, &mut resources),
            _ => Err(BuildArgsError::UnsupportedType(i, t.clone())),
        };
        encoded.push(res?);
    }

    Ok(ParseArgsResult { encoded, resources })
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
    reservations: &mut Vec<Resource>,
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
            let resource_def = split.next().and_then(|v| v.trim().parse::<Address>().ok());
            match (amount, resource_def) {
                (Some(a), Some(r)) => {
                    let bid = allocator.new_bid();
                    let bucket = Bucket::new(a, r);

                    match name {
                        SCRYPTO_NAME_BID => {
                            reservations.push(Resource {
                                bucket,
                                bid,
                                rid: None,
                            });
                            Ok(scrypto_encode(&bid))
                        }
                        SCRYPTO_NAME_BUCKET => {
                            reservations.push(Resource {
                                bucket,
                                bid,
                                rid: None,
                            });
                            Ok(scrypto_encode(&scrypto::resource::Bucket::from(bid)))
                        }
                        SCRYPTO_NAME_RID => {
                            let rid = allocator.new_rid();
                            reservations.push(Resource {
                                bucket,
                                bid,
                                rid: Some(rid),
                            });
                            Ok(scrypto_encode(&rid))
                        }
                        SCRYPTO_NAME_BUCKET_REF => {
                            let rid = allocator.new_rid();
                            reservations.push(Resource {
                                bucket,
                                bid,
                                rid: Some(rid),
                            });
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
