use sbor::describe::*;
use sbor::*;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::kernel::*;
use scrypto::resource::resource_flags::*;
use scrypto::resource::resource_permissions::*;
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::*;
use scrypto::rust::fmt;
use scrypto::rust::str::FromStr;
use scrypto::rust::string::String;
use scrypto::rust::vec;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

use crate::engine::*;
use crate::model::*;
use crate::transaction::*;

/// Represents some amount of resource.
pub enum ResourceAmount {
    Fungible {
        amount: Decimal,
        resource_address: Address,
    },
    NonFungible {
        keys: BTreeSet<NftKey>,
        resource_address: Address,
    },
}

/// Represents an error when parsing `ResourceAmount` from string.
#[derive(Debug, Clone)]
pub enum ParseResourceAmountError {
    InvalidAmount,
    InvalidNftKey,
    InvalidResourceAddress,
    MissingResourceAddress,
}

impl FromStr for ResourceAmount {
    type Err = ParseResourceAmountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<&str> = s.trim().split(',').collect();

        if tokens.len() >= 2 {
            let resource_address = tokens
                .last()
                .unwrap()
                .parse::<Address>()
                .map_err(|_| ParseResourceAmountError::InvalidResourceAddress)?;
            if tokens[0].starts_with('#') {
                let mut keys = BTreeSet::<NftKey>::new();
                for key in &tokens[..tokens.len() - 1] {
                    if key.starts_with('#') {
                        keys.insert(
                            key[1..]
                                .parse()
                                .map_err(|_| ParseResourceAmountError::InvalidNftKey)?,
                        );
                    } else {
                        return Err(ParseResourceAmountError::InvalidNftKey);
                    }
                }
                Ok(ResourceAmount::NonFungible {
                    keys,
                    resource_address,
                })
            } else {
                if tokens.len() == 2 {
                    Ok(ResourceAmount::Fungible {
                        amount: tokens[0]
                            .parse()
                            .map_err(|_| ParseResourceAmountError::InvalidAmount)?,
                        resource_address,
                    })
                } else {
                    Err(ParseResourceAmountError::InvalidAmount)
                }
            }
        } else {
            Err(ParseResourceAmountError::MissingResourceAddress)
        }
    }
}

impl ResourceAmount {
    pub fn amount(&self) -> Decimal {
        match self {
            ResourceAmount::Fungible { amount, .. } => *amount,
            ResourceAmount::NonFungible { keys, .. } => keys.len().into(),
        }
    }
    pub fn resource_address(&self) -> Address {
        match self {
            ResourceAmount::Fungible {
                resource_address, ..
            }
            | ResourceAmount::NonFungible {
                resource_address, ..
            } => *resource_address,
        }
    }
}

/// Utility for building transaction.
pub struct TransactionBuilder<'a, A: AbiProvider> {
    /// ABI provider for constructing arguments
    abi_provider: &'a A,
    /// ID validator for calculating transaction object id
    id_validator: IdValidator,
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
            id_validator: IdValidator::new(),
            instructions: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Adds a raw instruction.
    pub fn add_instruction(&mut self, inst: Instruction) -> (&mut Self, Option<Bid>, Option<Rid>) {
        let mut new_bid: Option<Bid> = None;
        let mut new_rid: Option<Rid> = None;

        match inst.clone() {
            Instruction::TakeFromWorktop { .. } => {
                new_bid = Some(self.id_validator.new_bucket().unwrap());
            }
            Instruction::TakeAllFromWorktop { .. } => {
                new_bid = Some(self.id_validator.new_bucket().unwrap());
            }
            Instruction::ReturnToWorktop { bid } => {
                self.id_validator.drop_bucket(bid).unwrap();
            }
            Instruction::AssertWorktopContains { .. } => {}
            Instruction::CreateBucketRef { bid } => {
                new_rid = Some(self.id_validator.new_bucket_ref(bid).unwrap());
            }
            Instruction::CloneBucketRef { rid } => {
                new_rid = Some(self.id_validator.clone_bucket_ref(rid).unwrap());
            }
            Instruction::DropBucketRef { rid } => {
                self.id_validator.drop_bucket_ref(rid).unwrap();
            }
            Instruction::CallFunction { args, .. } | Instruction::CallMethod { args, .. } => {
                for arg in &args {
                    let validated_arg = validate_data(arg).unwrap();
                    self.id_validator.move_resources(&validated_arg).unwrap();
                }
            }
            Instruction::CallMethodWithAllResources { .. } => {
                self.id_validator.move_all_resources().unwrap();
            }
            Instruction::End { .. } => {}
        }

        self.instructions.push(inst);

        (self, new_bid, new_rid)
    }

    /// Takes resources from worktop.
    pub fn take_from_worktop<F>(
        &mut self,
        amount: Decimal,
        resource_address: Address,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, Bid) -> &mut Self,
    {
        let (builder, bid, _) = self.add_instruction(Instruction::TakeFromWorktop {
            amount,
            resource_address,
        });
        then(builder, bid.unwrap())
    }

    /// Asserts that worktop contains at least this amount of resource.
    pub fn assert_worktop_contains(
        &mut self,
        amount: Decimal,
        resource_address: Address,
    ) -> &mut Self {
        self.add_instruction(Instruction::AssertWorktopContains {
            amount,
            resource_address,
        })
        .0
    }

    /// Creates a bucket ref.
    pub fn create_bucket_ref<F>(&mut self, bid: Bid, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, Rid) -> &mut Self,
    {
        let (builder, _, rid) = self.add_instruction(Instruction::CreateBucketRef { bid });
        then(builder, rid.unwrap())
    }

    /// Clones a bucket ref.
    pub fn clone_bucket_ref<F>(&mut self, rid: Rid, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, Rid) -> &mut Self,
    {
        let (builder, _, rid) = self.add_instruction(Instruction::CloneBucketRef { rid });
        then(builder, rid.unwrap())
    }

    /// Drops a bucket ref.
    pub fn drop_bucket_ref(&mut self, rid: Rid) -> &mut Self {
        self.add_instruction(Instruction::DropBucketRef { rid }).0
    }

    /// Calls a function.
    ///
    /// The implementation will automatically prepare the arguments based on the
    /// function ABI, including resource buckets and bucket refs.
    ///
    /// If an account address is provided, resources will be withdrawn from the given account;
    /// otherwise, they will be taken from transaction worktop.
    pub fn call_function(
        &mut self,
        package_address: Address,
        blueprint_name: &str,
        function: &str,
        args: Vec<String>,
        account: Option<Address>,
    ) -> &mut Self {
        let result = self
            .abi_provider
            .export_abi(package_address, blueprint_name)
            .map_err(|_| {
                BuildTransactionError::FailedToExportFunctionAbi(
                    package_address,
                    blueprint_name.to_owned(),
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
                self.add_instruction(Instruction::CallFunction {
                    package_address,
                    blueprint_name: blueprint_name.to_owned(),
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
    /// otherwise, they will be taken from transaction worktop.
    pub fn call_method(
        &mut self,
        component_address: Address,
        method: &str,
        args: Vec<String>,
        account: Option<Address>,
    ) -> &mut Self {
        let result = self
            .abi_provider
            .export_abi_component(component_address)
            .map_err(|_| {
                BuildTransactionError::FailedToExportMethodAbi(component_address, method.to_owned())
            })
            .and_then(|abi| Self::find_method_abi(&abi, method))
            .and_then(|m| {
                self.prepare_args(&m.inputs, args, account)
                    .map_err(|e| BuildTransactionError::FailedToBuildArgs(e))
            });

        match result {
            Ok(args) => {
                self.add_instruction(Instruction::CallMethod {
                    component_address,
                    method: method.to_owned(),
                    args,
                });
            }
            Err(e) => self.errors.push(e),
        }

        self
    }

    /// Calls a method with all the resources on worktop.
    ///
    /// The callee method must have only one parameter with type `Vec<Bucket>`; otherwise,
    /// a runtime failure is triggered.
    pub fn call_method_with_all_resources(
        &mut self,
        component_address: Address,
        method: &str,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethodWithAllResources {
            component_address,
            method: method.into(),
        })
        .0
    }

    /// Builds a transaction.
    pub fn build(&mut self, signers: Vec<Address>) -> Result<Transaction, BuildTransactionError> {
        if !self.errors.is_empty() {
            return Err(self.errors[0].clone());
        }

        let mut v = Vec::new();
        v.extend(self.instructions.clone());
        v.push(Instruction::End {
            signatures: signers, // TODO sign
        });

        Ok(Transaction { instructions: v })
    }

    //===============================
    // complex instruction below
    //===============================

    /// Publishes a package.
    pub fn publish_package(&mut self, code: &[u8]) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            function: "publish_package".to_owned(),
            args: vec![scrypto_encode(&code.to_vec())],
        })
        .0
    }

    fn single_authority(badge: Address, permission: u16) -> HashMap<Address, u16> {
        let mut map = HashMap::new();
        map.insert(badge, permission);
        map
    }

    /// Creates a token resource with mutable supply.
    pub fn new_token_mutable(
        &mut self,
        metadata: HashMap<String, String>,
        mint_badge_address: Address,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            function: "new_resource".to_owned(),
            args: vec![
                scrypto_encode(&ResourceType::Fungible { divisibility: 18 }),
                scrypto_encode(&metadata),
                scrypto_encode(&(MINTABLE | BURNABLE)),
                scrypto_encode(&0u16),
                scrypto_encode(&Self::single_authority(
                    mint_badge_address,
                    MAY_MINT | MAY_BURN,
                )),
                scrypto_encode::<Option<NewSupply>>(&None),
            ],
        })
        .0
    }

    /// Creates a token resource with fixed supply.
    pub fn new_token_fixed(
        &mut self,
        metadata: HashMap<String, String>,
        initial_supply: Decimal,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            function: "new_resource".to_owned(),
            args: vec![
                scrypto_encode(&ResourceType::Fungible { divisibility: 18 }),
                scrypto_encode(&metadata),
                scrypto_encode(&0u16),
                scrypto_encode(&0u16),
                scrypto_encode(&HashMap::<Address, u16>::new()),
                scrypto_encode(&Some(NewSupply::Fungible {
                    amount: initial_supply.into(),
                })),
            ],
        })
        .0
    }

    /// Creates a badge resource with mutable supply.
    pub fn new_badge_mutable(
        &mut self,
        metadata: HashMap<String, String>,
        mint_badge_address: Address,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            function: "new_resource".to_owned(),
            args: vec![
                scrypto_encode(&ResourceType::Fungible { divisibility: 0 }),
                scrypto_encode(&metadata),
                scrypto_encode(&(MINTABLE | BURNABLE)),
                scrypto_encode(&0u16),
                scrypto_encode(&Self::single_authority(
                    mint_badge_address,
                    MAY_MINT | MAY_BURN,
                )),
                scrypto_encode::<Option<NewSupply>>(&None),
            ],
        })
        .0
    }

    /// Creates a badge resource with fixed supply.
    pub fn new_badge_fixed(
        &mut self,
        metadata: HashMap<String, String>,
        initial_supply: Decimal,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            function: "new_resource".to_owned(),
            args: vec![
                scrypto_encode(&ResourceType::Fungible { divisibility: 0 }),
                scrypto_encode(&metadata),
                scrypto_encode(&0u16),
                scrypto_encode(&0u16),
                scrypto_encode(&HashMap::<Address, u16>::new()),
                scrypto_encode(&Some(NewSupply::Fungible {
                    amount: initial_supply.into(),
                })),
            ],
        })
        .0
    }

    /// Mints resource.
    pub fn mint(
        &mut self,
        amount: Decimal,
        resource_address: Address,
        mint_badge_address: Address,
    ) -> &mut Self {
        self.take_from_worktop(1.into(), mint_badge_address, |builder, bid| {
            builder.create_bucket_ref(bid, |builder, rid| {
                builder
                    .add_instruction(Instruction::CallFunction {
                        package_address: SYSTEM_PACKAGE,
                        blueprint_name: "System".to_owned(),
                        function: "mint".to_owned(),
                        args: vec![
                            scrypto_encode(&amount),
                            scrypto_encode(&resource_address),
                            scrypto_encode(&rid),
                        ],
                    })
                    .0
            })
        })
    }

    /// Creates an account.
    pub fn new_account(&mut self, key: Address) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: "Account".to_owned(),
            function: "new".to_owned(),
            args: vec![scrypto_encode(&key)],
        })
        .0
    }

    /// Creates an account with resource taken from transaction worktop.
    ///
    /// Note: you need to make sure the worktop contains the required resource to avoid runtime error.
    pub fn new_account_with_resource(
        &mut self,
        key: Address,
        amount: Decimal,
        resource_address: Address,
    ) -> &mut Self {
        self.take_from_worktop(amount, resource_address, |builder, bid| {
            builder
                .add_instruction(Instruction::CallFunction {
                    package_address: ACCOUNT_PACKAGE,
                    blueprint_name: "Account".to_owned(),
                    function: "with_bucket".to_owned(),
                    args: vec![scrypto_encode(&key), scrypto_encode(&bid)],
                })
                .0
        })
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account(
        &mut self,
        resource_spec: &ResourceAmount,
        account: Address,
    ) -> &mut Self {
        self.clone_bucket_ref(ECDSA_TOKEN_RID, |builder, rid| match resource_spec {
            ResourceAmount::Fungible {
                amount,
                resource_address,
            } => {
                builder
                    .add_instruction(Instruction::CallMethod {
                        component_address: account,
                        method: "withdraw".to_owned(),
                        args: vec![
                            scrypto_encode(amount),
                            scrypto_encode(resource_address),
                            scrypto_encode(&rid),
                        ],
                    })
                    .0
            }
            ResourceAmount::NonFungible {
                keys,
                resource_address,
            } => {
                builder
                    .add_instruction(Instruction::CallMethod {
                        component_address: account,
                        method: "withdraw_nfts".to_owned(),
                        args: vec![
                            scrypto_encode(keys),
                            scrypto_encode(resource_address),
                            scrypto_encode(&rid),
                        ],
                    })
                    .0
            }
        })
    }

    //===============================
    // private methods below
    //===============================

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
    ) -> Result<Vec<Vec<u8>>, BuildArgsError> {
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
                Type::Custom { name, .. } => self.prepare_custom_ty(i, t, arg, name, account),
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
    ) -> Result<Vec<u8>, BuildArgsError>
    where
        T: FromStr + Encode,
        T::Err: fmt::Debug,
    {
        let value = arg
            .parse::<T>()
            .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
        Ok(scrypto_encode(&value))
    }

    fn prepare_custom_ty(
        &mut self,
        i: usize,
        ty: &Type,
        arg: &str,
        name: &str,
        account: Option<Address>,
    ) -> Result<Vec<u8>, BuildArgsError> {
        match name {
            SCRYPTO_NAME_DECIMAL => {
                let value = arg
                    .parse::<Decimal>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            SCRYPTO_NAME_BIG_DECIMAL => {
                let value = arg
                    .parse::<BigDecimal>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            SCRYPTO_NAME_ADDRESS => {
                let value = arg
                    .parse::<Address>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            SCRYPTO_NAME_H256 => {
                let value = arg
                    .parse::<H256>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            SCRYPTO_NAME_NFT_KEY => {
                let value = arg
                    .parse::<NftKey>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            SCRYPTO_NAME_BID | SCRYPTO_NAME_BUCKET => {
                let resource_spec = parse_resource_spec(i, ty, arg)?;

                if let Some(account) = account {
                    self.withdraw_from_account(&resource_spec, account);
                }
                let mut created_bid = None;
                self.take_from_worktop(
                    resource_spec.amount(),
                    resource_spec.resource_address(),
                    |builder, bid| {
                        created_bid = Some(bid);
                        builder
                    },
                );
                Ok(scrypto_encode(&created_bid.unwrap()))
            }
            SCRYPTO_NAME_RID | SCRYPTO_NAME_BUCKET_REF => {
                let resource_spec = parse_resource_spec(i, ty, arg)?;
                if let Some(account) = account {
                    self.withdraw_from_account(&resource_spec, account);
                }
                let mut created_rid = None;
                self.take_from_worktop(
                    resource_spec.amount(),
                    resource_spec.resource_address(),
                    |builder, bid| {
                        builder.create_bucket_ref(bid, |builder, rid| {
                            created_rid = Some(rid);
                            builder
                        });
                        builder
                    },
                );
                Ok(scrypto_encode(&created_rid.unwrap()))
            }
            _ => Err(BuildArgsError::UnsupportedType(i, ty.clone())),
        }
    }
}

fn parse_resource_spec(i: usize, ty: &Type, arg: &str) -> Result<ResourceAmount, BuildArgsError> {
    ResourceAmount::from_str(arg)
        .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))
}
