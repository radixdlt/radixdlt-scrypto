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
        ids: BTreeSet<u128>,
        resource_address: Address,
    },
}

/// Represents an error when parsing `ResourceAmount` from string.
#[derive(Debug, Clone)]
pub enum ParseResourceAmountError {
    InvalidAmount,
    InvalidNftId,
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
                let mut ids = BTreeSet::<u128>::new();
                for id in &tokens[..tokens.len() - 1] {
                    if id.starts_with('#') {
                        ids.insert(
                            id[1..]
                                .parse()
                                .map_err(|_| ParseResourceAmountError::InvalidNftId)?,
                        );
                    } else {
                        return Err(ParseResourceAmountError::InvalidNftId);
                    }
                }
                Ok(ResourceAmount::NonFungible {
                    ids,
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
            ResourceAmount::NonFungible { ids, .. } => ids.len().into(),
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
    abi_provider: &'a A,
    /// The address allocator for calculating temp object IDs.
    id_allocator: IdAllocator,
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
            id_allocator: IdAllocator::new(TRANSACTION_OBJECT_ID_RANGE),
            instructions: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Adds a raw instruction.
    pub fn add_instruction(&mut self, inst: Instruction) -> &mut Self {
        self.instructions.push(inst);
        self
    }

    /// Creates a temporary bucket.
    pub fn create_temp_bucket<F>(
        &mut self,
        amount: Decimal,
        resource_address: Address,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, Bid) -> &mut Self,
    {
        let bid = self.id_allocator.new_bid().unwrap();
        self.add_instruction(Instruction::CreateTempBucket {
            amount,
            resource_address,
        });
        then(self, bid)
    }

    /// Creates a temporary bucket ref.
    pub fn create_temp_bucket_ref<F>(&mut self, bid: Bid, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, Rid) -> &mut Self,
    {
        let rid = self.id_allocator.new_rid().unwrap();
        self.add_instruction(Instruction::CreateTempBucketRef { bid });
        then(self, rid)
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
    /// otherwise, they will be taken from transaction context.
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

    /// Calls a method with all the resources within the context.
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
    }

    /// Mints resource.
    pub fn mint(
        &mut self,
        amount: Decimal,
        resource_address: Address,
        mint_badge_address: Address,
    ) -> &mut Self {
        self.create_temp_bucket(1.into(), mint_badge_address, |builder, bid| {
            builder.create_temp_bucket_ref(bid, |builder, rid| {
                builder.add_instruction(Instruction::CallFunction {
                    package_address: SYSTEM_PACKAGE,
                    blueprint_name: "System".to_owned(),
                    function: "mint".to_owned(),
                    args: vec![
                        scrypto_encode(&amount),
                        scrypto_encode(&resource_address),
                        scrypto_encode(&rid),
                    ],
                })
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
    }

    /// Creates an account with resource taken from context.
    ///
    /// Note: need to make sure the context contains the required resource.
    pub fn new_account_with_resource(
        &mut self,
        key: Address,
        amount: Decimal,
        resource_address: Address,
    ) -> &mut Self {
        self.create_temp_bucket(amount, resource_address, |builder, bid| {
            builder.add_instruction(Instruction::CallFunction {
                package_address: ACCOUNT_PACKAGE,
                blueprint_name: "Account".to_owned(),
                function: "with_bucket".to_owned(),
                args: vec![scrypto_encode(&key), scrypto_encode(&bid)],
            })
        })
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account(
        &mut self,
        resource_spec: &ResourceAmount,
        account: Address,
    ) -> &mut Self {
        match resource_spec {
            ResourceAmount::Fungible {
                amount,
                resource_address,
            } => self.add_instruction(Instruction::CallMethod {
                component_address: account,
                method: "withdraw".to_owned(),
                args: vec![
                    scrypto_encode(amount),
                    scrypto_encode(resource_address),
                    scrypto_encode(&ECDSA_TOKEN_RID),
                ],
            }),
            ResourceAmount::NonFungible {
                ids,
                resource_address,
            } => self.add_instruction(Instruction::CallMethod {
                component_address: account,
                method: "withdraw_nfts".to_owned(),
                args: vec![
                    scrypto_encode(ids),
                    scrypto_encode(resource_address),
                    scrypto_encode(&ECDSA_TOKEN_RID),
                ],
            }),
        }
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
            SCRYPTO_NAME_BID | SCRYPTO_NAME_BUCKET => {
                let resource_spec = parse_resource_spec(i, ty, arg)?;

                if let Some(account) = account {
                    self.withdraw_from_account(&resource_spec, account);
                }
                let mut created_bid = None;
                self.create_temp_bucket(
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
                self.create_temp_bucket(
                    resource_spec.amount(),
                    resource_spec.resource_address(),
                    |builder, bid| {
                        builder.create_temp_bucket_ref(bid, |builder, rid| {
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
