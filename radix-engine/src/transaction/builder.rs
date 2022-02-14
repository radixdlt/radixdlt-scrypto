use sbor::describe::*;
use sbor::*;
use scrypto::abi;
use scrypto::buffer::*;
use scrypto::engine::types::*;
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
#[derive(Debug, Clone)]
pub enum ResourceSpecification {
    Fungible {
        amount: Decimal,
        resource_def_ref: ResourceDefRef,
    },
    NonFungible {
        keys: BTreeSet<NonFungibleKey>,
        resource_def_ref: ResourceDefRef,
    },
    All {
        resource_def_ref: ResourceDefRef,
    },
}

/// Represents an error when parsing `Resource` from string.
#[derive(Debug, Clone)]
pub enum ParseResourceSpecificationError {
    MissingResourceDefRef,
    InvalidAmount,
    InvalidNftId,
    InvalidResourceDefRef,
}

impl fmt::Display for ParseResourceSpecificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseResourceSpecificationError {}

impl FromStr for ResourceSpecification {
    type Err = ParseResourceSpecificationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tokens: Vec<&str> = s.trim().split(',').collect();

        if tokens.len() >= 2 {
            let resource_def_ref = tokens
                .last()
                .unwrap()
                .parse::<ResourceDefRef>()
                .map_err(|_| ParseResourceSpecificationError::InvalidResourceDefRef)?;
            if tokens[0].starts_with('#') {
                let mut keys = BTreeSet::<NonFungibleKey>::new();
                for key in &tokens[..tokens.len() - 1] {
                    if key.starts_with('#') {
                        keys.insert(
                            key[1..]
                                .parse()
                                .map_err(|_| ParseResourceSpecificationError::InvalidNftId)?,
                        );
                    } else {
                        return Err(ParseResourceSpecificationError::InvalidNftId);
                    }
                }
                Ok(ResourceSpecification::NonFungible {
                    keys,
                    resource_def_ref,
                })
            } else {
                if tokens.len() == 2 {
                    Ok(ResourceSpecification::Fungible {
                        amount: tokens[0]
                            .parse()
                            .map_err(|_| ParseResourceSpecificationError::InvalidAmount)?,
                        resource_def_ref,
                    })
                } else {
                    Err(ParseResourceSpecificationError::InvalidAmount)
                }
            }
        } else {
            Err(ParseResourceSpecificationError::MissingResourceDefRef)
        }
    }
}

impl ResourceSpecification {
    pub fn amount(&self) -> Option<Decimal> {
        match self {
            ResourceSpecification::Fungible { amount, .. } => Some(*amount),
            ResourceSpecification::NonFungible { keys, .. } => Some(keys.len().into()),
            ResourceSpecification::All { .. } => None,
        }
    }

    pub fn resource_def_ref(&self) -> ResourceDefRef {
        match self {
            ResourceSpecification::Fungible {
                resource_def_ref, ..
            }
            | ResourceSpecification::NonFungible {
                resource_def_ref, ..
            }
            | ResourceSpecification::All { resource_def_ref } => *resource_def_ref,
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
    pub fn add_instruction(
        &mut self,
        inst: Instruction,
    ) -> (&mut Self, Option<BucketId>, Option<BucketRefId>) {
        let mut new_bucket_id: Option<BucketId> = None;
        let mut new_bucket_ref_id: Option<BucketRefId> = None;

        match inst.clone() {
            Instruction::TakeFromWorktop { .. } => {
                new_bucket_id = Some(self.id_validator.new_bucket().unwrap());
            }
            Instruction::TakeAllFromWorktop { .. } => {
                new_bucket_id = Some(self.id_validator.new_bucket().unwrap());
            }
            Instruction::TakeNonFungiblesFromWorktop { .. } => {
                new_bucket_id = Some(self.id_validator.new_bucket().unwrap());
            }
            Instruction::ReturnToWorktop { bucket_id } => {
                self.id_validator.drop_bucket(bucket_id).unwrap();
            }
            Instruction::AssertWorktopContains { .. } => {}
            Instruction::CreateBucketRef { bucket_id } => {
                new_bucket_ref_id = Some(self.id_validator.new_bucket_ref(bucket_id).unwrap());
            }
            Instruction::CloneBucketRef { bucket_ref_id } => {
                new_bucket_ref_id =
                    Some(self.id_validator.clone_bucket_ref(bucket_ref_id).unwrap());
            }
            Instruction::DropBucketRef { bucket_ref_id } => {
                self.id_validator.drop_bucket_ref(bucket_ref_id).unwrap();
            }
            Instruction::CallFunction { args, .. } | Instruction::CallMethod { args, .. } => {
                for arg in &args {
                    let validated_arg = ValidatedData::from_slice(arg).unwrap();
                    self.id_validator.move_resources(&validated_arg).unwrap();
                }
            }
            Instruction::CallMethodWithAllResources { .. } => {
                self.id_validator.move_all_resources().unwrap();
            }
            Instruction::End { .. } => {}
        }

        self.instructions.push(inst);

        (self, new_bucket_id, new_bucket_ref_id)
    }

    /// Takes resources from worktop.
    pub fn take_from_worktop<F>(
        &mut self,
        resource_spec: &ResourceSpecification,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, BucketId) -> &mut Self,
    {
        let (builder, bucket_id, _) = match resource_spec.clone() {
            ResourceSpecification::Fungible {
                amount,
                resource_def_ref,
            } => self.add_instruction(Instruction::TakeFromWorktop {
                amount,
                resource_def_ref,
            }),
            ResourceSpecification::NonFungible {
                keys,
                resource_def_ref,
            } => self.add_instruction(Instruction::TakeNonFungiblesFromWorktop {
                keys,
                resource_def_ref,
            }),
            ResourceSpecification::All { resource_def_ref } => {
                self.add_instruction(Instruction::TakeAllFromWorktop { resource_def_ref })
            }
        };
        then(builder, bucket_id.unwrap())
    }

    /// Asserts that worktop contains at least this amount of resource.
    pub fn assert_worktop_contains(
        &mut self,
        amount: Decimal,
        resource_def_ref: ResourceDefRef,
    ) -> &mut Self {
        self.add_instruction(Instruction::AssertWorktopContains {
            amount,
            resource_def_ref,
        })
        .0
    }

    /// Creates a bucket ref.
    pub fn create_bucket_ref<F>(&mut self, bucket_id: BucketId, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, BucketRefId) -> &mut Self,
    {
        let (builder, _, bucket_ref_id) =
            self.add_instruction(Instruction::CreateBucketRef { bucket_id });
        then(builder, bucket_ref_id.unwrap())
    }

    /// Clones a bucket ref.
    pub fn clone_bucket_ref<F>(&mut self, bucket_ref_id: BucketRefId, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, BucketRefId) -> &mut Self,
    {
        let (builder, _, bucket_ref_id) =
            self.add_instruction(Instruction::CloneBucketRef { bucket_ref_id });
        then(builder, bucket_ref_id.unwrap())
    }

    /// Drops a bucket ref.
    pub fn drop_bucket_ref(&mut self, bucket_ref_id: BucketRefId) -> &mut Self {
        self.add_instruction(Instruction::DropBucketRef { bucket_ref_id })
            .0
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
        package_ref: PackageRef,
        blueprint_name: &str,
        function: &str,
        args: Vec<String>,
        account: Option<ComponentRef>,
    ) -> &mut Self {
        let result = self
            .abi_provider
            .export_abi(package_ref, blueprint_name)
            .map_err(|_| {
                BuildTransactionError::FailedToExportFunctionAbi(
                    package_ref,
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
                    package_ref,
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
        component_ref: ComponentRef,
        method: &str,
        args: Vec<String>,
        account: Option<ComponentRef>,
    ) -> &mut Self {
        let result = self
            .abi_provider
            .export_abi_component(component_ref)
            .map_err(|_| {
                BuildTransactionError::FailedToExportMethodAbi(component_ref, method.to_owned())
            })
            .and_then(|abi| Self::find_method_abi(&abi, method))
            .and_then(|m| {
                self.prepare_args(&m.inputs, args, account)
                    .map_err(|e| BuildTransactionError::FailedToBuildArgs(e))
            });

        match result {
            Ok(args) => {
                self.add_instruction(Instruction::CallMethod {
                    component_ref,
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
        component_ref: ComponentRef,
        method: &str,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethodWithAllResources {
            component_ref,
            method: method.into(),
        })
        .0
    }

    /// Builds a transaction.
    pub fn build(
        &mut self,
        signers: Vec<EcdsaPublicKey>,
    ) -> Result<Transaction, BuildTransactionError> {
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
            package_ref: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            function: "publish_package".to_owned(),
            args: vec![scrypto_encode(&code.to_vec())],
        })
        .0
    }

    fn single_authority(badge: ResourceDefRef, permission: u64) -> HashMap<ResourceDefRef, u64> {
        let mut map = HashMap::new();
        map.insert(badge, permission);
        map
    }

    /// Creates a token resource with mutable supply.
    pub fn new_token_mutable(
        &mut self,
        metadata: HashMap<String, String>,
        badge: ResourceDefRef,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_ref: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            function: "new_resource".to_owned(),
            args: vec![
                scrypto_encode(&ResourceType::Fungible { divisibility: 18 }),
                scrypto_encode(&metadata),
                scrypto_encode(&(MINTABLE | BURNABLE)),
                scrypto_encode(&0u64),
                scrypto_encode(&Self::single_authority(badge, MAY_MINT | MAY_BURN)),
                scrypto_encode::<Option<Supply>>(&None),
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
            package_ref: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            function: "new_resource".to_owned(),
            args: vec![
                scrypto_encode(&ResourceType::Fungible { divisibility: 18 }),
                scrypto_encode(&metadata),
                scrypto_encode(&0u64),
                scrypto_encode(&0u64),
                scrypto_encode(&HashMap::<ResourceDefRef, u64>::new()),
                scrypto_encode(&Some(Supply::Fungible {
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
        badge: ResourceDefRef,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_ref: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            function: "new_resource".to_owned(),
            args: vec![
                scrypto_encode(&ResourceType::Fungible { divisibility: 0 }),
                scrypto_encode(&metadata),
                scrypto_encode(&(MINTABLE | BURNABLE)),
                scrypto_encode(&0u64),
                scrypto_encode(&Self::single_authority(badge, MAY_MINT | MAY_BURN)),
                scrypto_encode::<Option<Supply>>(&None),
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
            package_ref: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            function: "new_resource".to_owned(),
            args: vec![
                scrypto_encode(&ResourceType::Fungible { divisibility: 0 }),
                scrypto_encode(&metadata),
                scrypto_encode(&0u64),
                scrypto_encode(&0u64),
                scrypto_encode(&HashMap::<ResourceDefRef, u64>::new()),
                scrypto_encode(&Some(Supply::Fungible {
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
        resource_def_ref: ResourceDefRef,
        badge: ResourceDefRef,
    ) -> &mut Self {
        self.take_from_worktop(
            &ResourceSpecification::Fungible {
                amount: 1.into(),
                resource_def_ref: badge,
            },
            |builder, bucket_id| {
                builder.create_bucket_ref(bucket_id, |builder, bucket_ref_id| {
                    builder
                        .add_instruction(Instruction::CallFunction {
                            package_ref: SYSTEM_PACKAGE,
                            blueprint_name: "System".to_owned(),
                            function: "mint".to_owned(),
                            args: vec![
                                scrypto_encode(&amount),
                                scrypto_encode(&resource_def_ref),
                                scrypto_encode(&scrypto::resource::BucketRef(bucket_ref_id)),
                            ],
                        })
                        .0
                })
            },
        )
    }

    /// Creates an account.
    pub fn new_account(&mut self, public_key: EcdsaPublicKey) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_ref: ACCOUNT_PACKAGE,
            blueprint_name: "Account".to_owned(),
            function: "new".to_owned(),
            args: vec![scrypto_encode(&public_key)],
        })
        .0
    }

    /// Creates an account with resource taken from transaction worktop.
    ///
    /// Note: you need to make sure the worktop contains the required resource to avoid runtime error.
    pub fn new_account_with_resource(
        &mut self,
        key: EcdsaPublicKey,
        resource_spec: &ResourceSpecification,
    ) -> &mut Self {
        self.take_from_worktop(resource_spec, |builder, bucket_id| {
            builder
                .add_instruction(Instruction::CallFunction {
                    package_ref: ACCOUNT_PACKAGE,
                    blueprint_name: "Account".to_owned(),
                    function: "with_bucket".to_owned(),
                    args: vec![
                        scrypto_encode(&key),
                        scrypto_encode(&scrypto::resource::Bucket(bucket_id)),
                    ],
                })
                .0
        })
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account(
        &mut self,
        resource_spec: &ResourceSpecification,
        account: ComponentRef,
    ) -> &mut Self {
        self.clone_bucket_ref(ECDSA_TOKEN_BUCKET_REF_ID, |builder, bucket_ref_id| {
            match resource_spec {
                ResourceSpecification::Fungible {
                    amount,
                    resource_def_ref,
                } => {
                    builder
                        .add_instruction(Instruction::CallMethod {
                            component_ref: account,
                            method: "withdraw".to_owned(),
                            args: vec![
                                scrypto_encode(amount),
                                scrypto_encode(resource_def_ref),
                                scrypto_encode(&scrypto::resource::BucketRef(bucket_ref_id)),
                            ],
                        })
                        .0
                }
                ResourceSpecification::NonFungible {
                    keys,
                    resource_def_ref,
                } => {
                    builder
                        .add_instruction(Instruction::CallMethod {
                            component_ref: account,
                            method: "withdraw_non_fungibles".to_owned(),
                            args: vec![
                                scrypto_encode(keys),
                                scrypto_encode(resource_def_ref),
                                scrypto_encode(&scrypto::resource::BucketRef(bucket_ref_id)),
                            ],
                        })
                        .0
                }
                ResourceSpecification::All { .. } => {
                    panic!("Withdrawing all from account is not supported!");
                }
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
        account: Option<ComponentRef>,
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
        account: Option<ComponentRef>,
    ) -> Result<Vec<u8>, BuildArgsError> {
        match CustomType::from_name(name).ok_or(BuildArgsError::UnsupportedType(i, ty.clone()))? {
            CustomType::Decimal => {
                let value = arg
                    .parse::<Decimal>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            CustomType::BigDecimal => {
                let value = arg
                    .parse::<BigDecimal>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            CustomType::PackageRef => {
                let value = arg
                    .parse::<PackageRef>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            CustomType::ComponentRef => {
                let value = arg
                    .parse::<ComponentRef>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            CustomType::ResourceDefRef => {
                let value = arg
                    .parse::<ResourceDefRef>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            CustomType::Hash => {
                let value = arg
                    .parse::<Hash>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            CustomType::NonFungibleKey => {
                let value = arg
                    .parse::<NonFungibleKey>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            CustomType::Bucket => {
                let resource_spec = parse_resource_specification(i, ty, arg)?;

                if let Some(account) = account {
                    self.withdraw_from_account(&resource_spec, account);
                }
                let mut created_bucket_id = None;
                self.take_from_worktop(&resource_spec, |builder, bucket_id| {
                    created_bucket_id = Some(bucket_id);
                    builder
                });
                Ok(scrypto_encode(&created_bucket_id.unwrap()))
            }
            CustomType::BucketRef => {
                let resource_spec = parse_resource_specification(i, ty, arg)?;
                if let Some(account) = account {
                    self.withdraw_from_account(&resource_spec, account);
                }
                let mut created_bucket_ref_id = None;
                self.take_from_worktop(&resource_spec, |builder, bucket_id| {
                    builder.create_bucket_ref(bucket_id, |builder, bucket_ref_id| {
                        created_bucket_ref_id = Some(bucket_ref_id);
                        builder
                    });
                    builder
                });
                Ok(scrypto_encode(&created_bucket_ref_id.unwrap()))
            }
            _ => Err(BuildArgsError::UnsupportedType(i, ty.clone())),
        }
    }
}

fn parse_resource_specification(
    i: usize,
    ty: &Type,
    arg: &str,
) -> Result<ResourceSpecification, BuildArgsError> {
    ResourceSpecification::from_str(arg)
        .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))
}
