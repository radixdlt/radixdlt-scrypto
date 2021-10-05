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
use crate::transaction::*;

/// Utility for building transaction.
pub struct TransactionBuilder<'a, A: AbiProvider> {
    abi_provider: &'a A,
    /// The address allocator for calculating reserved bucket id.
    allocator: IdAllocator,
    /// Bucket or BucketRef reservations
    reservations: Vec<Instruction>,
    /// Instructions generated.
    instructions: Vec<Instruction>,
    /// Collected Errors
    errors: Vec<BuildTransactionError>,
}

impl<'a, A: AbiProvider> TransactionBuilder<'a, A> {
    /// Starts a new transaction builder.
    pub fn new(abi_provider: &'a A) -> Self {
        Self {
            abi_provider,
            allocator: IdAllocator::new(),
            reservations: Vec::new(),
            instructions: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Reserves a bucket id.
    pub fn reserve_bucket_id(&mut self) -> Bid {
        let bid = self.allocator.new_bid();
        self.reservations.push(Instruction::ReserveBucketId);
        bid
    }

    /// Reserves a bucket ref id.
    pub fn reserve_bucket_ref_id(&mut self) -> Rid {
        let rid = self.allocator.new_rid();
        self.reservations.push(Instruction::ReserveBucketRefId);
        rid
    }

    /// Creates a bucket by withdrawing resource from context.
    pub fn create_bucket(&mut self, amount: Amount, resource_def: Address, bucket: Bid) {
        self.instruction(Instruction::CreateTempBucket {
            amount,
            resource_def,
            bucket,
        });
    }

    /// Creates a bucket ref by borrowing resource from context.
    pub fn create_bucket_ref(&mut self, amount: Amount, resource_def: Address, bucket_ref: Rid) {
        self.instruction(Instruction::CreateTempBucketRef {
            amount,
            resource_def,
            bucket_ref,
        });
    }

    /// Adds a raw instruction.
    pub fn instruction(&mut self, inst: Instruction) -> &mut Self {
        self.instructions.push(inst);
        self
    }

    /// Publishes a package.
    pub fn publish_package(&mut self, code: &[u8]) -> &mut Self {
        self.instruction(Instruction::CallFunction {
            package: SYSTEM_PACKAGE,
            name: "System".to_owned(),
            function: "publish_package".to_string(),
            args: vec![SmartValue(scrypto_encode(code))],
        })
    }

    /// Creates a resource with mutable supply.
    pub fn new_resource_mutable(&mut self, metadata: HashMap<String, String>) -> &mut Self {
        self.instruction(Instruction::CallFunction {
            package: SYSTEM_PACKAGE,
            name: "System".to_owned(),
            function: "new_resource_mutable".to_string(),
            args: vec![SmartValue::from(metadata)],
        })
    }

    /// Creates a resource with fixed supply.
    pub fn new_resource_fixed(
        &mut self,
        metadata: HashMap<String, String>,
        supply: Amount,
    ) -> &mut Self {
        self.instruction(Instruction::CallFunction {
            package: SYSTEM_PACKAGE,
            name: "System".to_owned(),
            function: "new_resource_fixed".to_string(),
            args: vec![SmartValue::from(metadata), SmartValue::from(supply)],
        })
    }

    /// Mints resource.
    pub fn mint_resource(&mut self, amount: Amount, resource_def: Address) -> &mut Self {
        self.instruction(Instruction::CallFunction {
            package: SYSTEM_PACKAGE,
            name: "System".to_owned(),
            function: "mint_resource".to_string(),
            args: vec![SmartValue::from(amount), SmartValue::from(resource_def)],
        })
    }

    /// Creates an account.
    pub fn new_account(&mut self) -> &mut Self {
        self.instruction(Instruction::CallFunction {
            package: ACCOUNT_PACKAGE,
            name: "Account".to_owned(),
            function: "new".to_string(),
            args: vec![],
        })
    }

    /// Creates an account with resource taken from context.
    ///
    /// Note: need to make sure the context contains the required resource.
    pub fn create_account_with_resource(
        &mut self,
        amount: Amount,
        resource_def: Address,
    ) -> &mut Self {
        let bid = self.reserve_bucket_id();
        self.create_bucket(amount, resource_def, bid);
        self.instruction(Instruction::CallFunction {
            package: ACCOUNT_PACKAGE,
            name: "Account".to_owned(),
            function: "with_bucket".to_string(),
            args: vec![SmartValue::from(scrypto::resource::Bucket::from(bid))],
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
            args: vec![SmartValue::from(amount), SmartValue::from(resource_def)],
        })
    }

    /// Deposits everything into an account.
    pub fn deposit_all(&mut self, account: Address) -> &mut Self {
        self.instruction(Instruction::DepositAll {
            component: account,
            method: "deposit_batch".to_string(),
        })
    }

    /// Calls a function.
    ///
    /// The implementation will automatically prepare the arguments based on the
    /// function ABI, including resource buckets and bucket refs.
    ///
    /// If an account address is provided, resources will be withdrawn from the given account;
    /// otherwise, they will be taken from transaction context.
    pub fn call_function(
        &mut self,
        package: Address,
        name: &str,
        function: &str,
        args: Vec<String>,
        account: Option<Address>,
    ) -> &mut Self {
        let result = self
            .abi_provider
            .export_abi(package, name, false)
            .map_err(|_| {
                BuildTransactionError::FailedToExportFunctionAbi(
                    package,
                    name.to_owned(),
                    function.to_owned(),
                )
            })
            .and_then(|abi| Self::find_function_abi(&abi, function))
            .and_then(|f| {
                self.prepare_args(&f.inputs, args, account)
                    .map_err(|e| BuildTransactionError::FailedToBuildArgs(e))
            });

        match result {
            Ok(args) => {
                self.instruction(Instruction::CallFunction {
                    package: package,
                    name: name.to_owned(),
                    function: function.to_owned(),
                    args,
                });
            }
            Err(e) => self.errors.push(e),
        }

        self
    }

    /// Calls a method.
    ///
    /// The implementation will automatically prepare the arguments based on the
    /// method ABI, including resource buckets and bucket refs.
    ///
    /// If an account address is provided, resources will be withdrawn from the given account;
    /// otherwise, they will be taken from transaction context.
    pub fn call_method(
        &mut self,
        component: Address,
        method: &str,
        args: Vec<String>,
        account: Option<Address>,
    ) -> &mut Self {
        let result = self
            .abi_provider
            .export_abi_component(component, false)
            .map_err(|_| {
                BuildTransactionError::FailedToExportMethodAbi(component, method.to_owned())
            })
            .and_then(|abi| Self::find_method_abi(&abi, method))
            .and_then(|m| {
                self.prepare_args(&m.inputs, args, account)
                    .map_err(|e| BuildTransactionError::FailedToBuildArgs(e))
            });

        match result {
            Ok(args) => {
                self.instruction(Instruction::CallMethod {
                    component: component,
                    method: method.to_owned(),
                    args,
                });
            }
            Err(e) => self.errors.push(e),
        }

        self
    }

    /// Builds a transaction.
    pub fn build(&mut self) -> Result<Transaction, BuildTransactionError> {
        if !self.errors.is_empty() {
            return Err(self.errors[0].clone());
        }

        let mut v = Vec::new();
        v.extend(self.reservations.clone());
        v.extend(self.instructions.clone());
        v.push(Instruction::End);

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

    fn prepare_args(
        &mut self,
        types: &[Type],
        args: Vec<String>,
        account: Option<Address>,
    ) -> Result<Vec<SmartValue>, BuildArgsError> {
        let mut encoded = Vec::new();

        for (i, t) in types.iter().enumerate() {
            let arg = args
                .get(i)
                .ok_or_else(|| BuildArgsError::MissingArgument(i, t.clone()))?;
            let res = match t {
                Type::Bool => self.prepare_basic_ty::<bool>(i, t, arg),
                Type::I8 => self.prepare_basic_ty::<i8>(i, t, arg),
                Type::I16 => self.prepare_basic_ty::<i16>(i, t, arg),
                Type::I32 => self.prepare_basic_ty::<i32>(i, t, arg),
                Type::I64 => self.prepare_basic_ty::<i64>(i, t, arg),
                Type::I128 => self.prepare_basic_ty::<i128>(i, t, arg),
                Type::U8 => self.prepare_basic_ty::<u8>(i, t, arg),
                Type::U16 => self.prepare_basic_ty::<u16>(i, t, arg),
                Type::U32 => self.prepare_basic_ty::<u32>(i, t, arg),
                Type::U64 => self.prepare_basic_ty::<u64>(i, t, arg),
                Type::U128 => self.prepare_basic_ty::<u128>(i, t, arg),
                Type::String => self.prepare_basic_ty::<String>(i, t, arg),
                Type::Custom { name } => self.prepare_custom_ty(i, t, arg, name, account),
                _ => Err(BuildArgsError::UnsupportedType(i, t.clone())),
            };
            encoded.push(res?);
        }

        Ok(encoded)
    }

    fn prepare_basic_ty<T>(
        &mut self,
        i: usize,
        ty: &Type,
        arg: &str,
    ) -> Result<SmartValue, BuildArgsError>
    where
        T: FromStr + Encode,
        T::Err: fmt::Debug,
    {
        let value = arg
            .parse::<T>()
            .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
        Ok(SmartValue::from(value))
    }

    fn prepare_custom_ty(
        &mut self,
        i: usize,
        ty: &Type,
        arg: &str,
        name: &str,
        account: Option<Address>,
    ) -> Result<SmartValue, BuildArgsError> {
        match name {
            SCRYPTO_NAME_AMOUNT => {
                let value = arg
                    .parse::<Amount>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(SmartValue::from(value))
            }
            SCRYPTO_NAME_ADDRESS => {
                let value = arg
                    .parse::<Address>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(SmartValue::from(value))
            }
            SCRYPTO_NAME_H256 => {
                let value = arg
                    .parse::<H256>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(SmartValue::from(value))
            }
            SCRYPTO_NAME_BID | SCRYPTO_NAME_BUCKET | SCRYPTO_NAME_RID | SCRYPTO_NAME_BUCKET_REF => {
                let mut split = arg.split(',');
                let amount = split.next().and_then(|v| v.trim().parse::<Amount>().ok());
                let resource_def = split.next().and_then(|v| v.trim().parse::<Address>().ok());
                match (amount, resource_def) {
                    (Some(a), Some(r)) => {
                        if let Some(account) = account {
                            self.withdraw(a, r, account);
                        }

                        match name {
                            SCRYPTO_NAME_BID => {
                                let bid = self.reserve_bucket_id();
                                self.create_bucket(a, r, bid);
                                Ok(SmartValue::from(bid))
                            }
                            SCRYPTO_NAME_BUCKET => {
                                let bid = self.reserve_bucket_id();
                                self.create_bucket(a, r, bid);
                                Ok(SmartValue::from(scrypto::resource::Bucket::from(bid)))
                            }
                            SCRYPTO_NAME_RID => {
                                let rid = self.reserve_bucket_ref_id();
                                self.create_bucket_ref(a, r, rid);
                                Ok(SmartValue::from(rid))
                            }
                            SCRYPTO_NAME_BUCKET_REF => {
                                let rid = self.reserve_bucket_ref_id();
                                self.create_bucket_ref(a, r, rid);
                                Ok(SmartValue::from(scrypto::resource::BucketRef::from(rid)))
                            }
                            _ => panic!("Unexpected"),
                        }
                    }
                    _ => Err(BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned())),
                }
            }
            _ => Err(BuildArgsError::UnsupportedType(i, ty.clone())),
        }
    }
}
