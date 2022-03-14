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
        resource_def_id: ResourceDefId,
    },
    NonFungible {
        ids: BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
    },
    All {
        resource_def_id: ResourceDefId,
    },
}

/// Represents an error when parsing `Resource` from string.
#[derive(Debug, Clone)]
pub enum ParseResourceSpecificationError {
    MissingResourceDefId,
    InvalidAmount,
    InvalidNftId,
    InvalidResourceDefId,
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
            let resource_def_id = tokens
                .last()
                .unwrap()
                .parse::<ResourceDefId>()
                .map_err(|_| ParseResourceSpecificationError::InvalidResourceDefId)?;
            if tokens[0].starts_with('#') {
                let mut ids = BTreeSet::<NonFungibleId>::new();
                for key in &tokens[..tokens.len() - 1] {
                    if key.starts_with('#') {
                        ids.insert(
                            key[1..]
                                .parse()
                                .map_err(|_| ParseResourceSpecificationError::InvalidNftId)?,
                        );
                    } else {
                        return Err(ParseResourceSpecificationError::InvalidNftId);
                    }
                }
                Ok(ResourceSpecification::NonFungible {
                    ids,
                    resource_def_id,
                })
            } else {
                if tokens.len() == 2 {
                    Ok(ResourceSpecification::Fungible {
                        amount: tokens[0]
                            .parse()
                            .map_err(|_| ParseResourceSpecificationError::InvalidAmount)?,
                        resource_def_id,
                    })
                } else {
                    Err(ParseResourceSpecificationError::InvalidAmount)
                }
            }
        } else {
            Err(ParseResourceSpecificationError::MissingResourceDefId)
        }
    }
}

impl ResourceSpecification {
    pub fn amount(&self) -> Option<Decimal> {
        match self {
            ResourceSpecification::Fungible { amount, .. } => Some(*amount),
            ResourceSpecification::NonFungible { ids, .. } => Some(ids.len().into()),
            ResourceSpecification::All { .. } => None,
        }
    }

    pub fn resource_def_id(&self) -> ResourceDefId {
        match self {
            ResourceSpecification::Fungible {
                resource_def_id, ..
            }
            | ResourceSpecification::NonFungible {
                resource_def_id, ..
            }
            | ResourceSpecification::All { resource_def_id } => *resource_def_id,
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
    ) -> (&mut Self, Option<BucketId>, Option<ProofId>) {
        let mut new_bucket_id: Option<BucketId> = None;
        let mut new_proof_id: Option<ProofId> = None;

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
            Instruction::TakeFromAuthWorktop { .. } => {
                new_proof_id = Some(
                    self.id_validator
                        .new_proof(ProofKind::RuntimeProof)
                        .unwrap(),
                );
            }
            Instruction::PutOnAuthWorktop { proof_id } => {
                self.id_validator.drop_proof(proof_id).unwrap();
            }
            Instruction::CreateBucketProof { bucket_id } => {
                new_proof_id = Some(
                    self.id_validator
                        .new_proof(ProofKind::BucketProof(bucket_id))
                        .unwrap(),
                );
            }
            Instruction::CloneProof { proof_id } => {
                new_proof_id = Some(self.id_validator.clone_proof(proof_id).unwrap());
            }
            Instruction::DropProof { proof_id } => {
                self.id_validator.drop_proof(proof_id).unwrap();
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
            Instruction::PublishPackage { .. } | Instruction::End { .. } => {}
        }

        self.instructions.push(inst);

        (self, new_bucket_id, new_proof_id)
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
                resource_def_id,
            } => self.add_instruction(Instruction::TakeFromWorktop {
                amount,
                resource_def_id,
            }),
            ResourceSpecification::NonFungible {
                ids,
                resource_def_id,
            } => self.add_instruction(Instruction::TakeNonFungiblesFromWorktop {
                ids,
                resource_def_id,
            }),
            ResourceSpecification::All { resource_def_id } => {
                self.add_instruction(Instruction::TakeAllFromWorktop { resource_def_id })
            }
        };
        then(builder, bucket_id.unwrap())
    }

    /// Asserts that worktop contains at least this amount of resource.
    pub fn assert_worktop_contains(
        &mut self,
        amount: Decimal,
        resource_def_id: ResourceDefId,
    ) -> &mut Self {
        self.add_instruction(Instruction::AssertWorktopContains {
            amount,
            resource_def_id,
        })
        .0
    }

    /// Creates a proof.
    pub fn create_bucket_proof<F>(&mut self, bucket_id: BucketId, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ProofId) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(Instruction::CreateBucketProof { bucket_id });
        then(builder, proof_id.unwrap())
    }

    /// Clones a proof.
    pub fn clone_proof<F>(&mut self, proof_id: ProofId, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ProofId) -> &mut Self,
    {
        let (builder, _, proof_id) = self.add_instruction(Instruction::CloneProof { proof_id });
        then(builder, proof_id.unwrap())
    }

    /// Drops a proof.
    pub fn drop_proof(&mut self, proof_id: ProofId) -> &mut Self {
        self.add_instruction(Instruction::DropProof { proof_id }).0
    }

    /// Calls a function.
    ///
    /// The implementation will automatically prepare the arguments based on the
    /// function ABI, including resource buckets and proofs.
    ///
    /// If an Account component ID is provided, resources will be withdrawn from the given account;
    /// otherwise, they will be taken from transaction worktop.
    pub fn call_function(
        &mut self,
        package_id: PackageId,
        blueprint_name: &str,
        function: &str,
        args: Vec<String>,
        account: Option<ComponentId>,
    ) -> &mut Self {
        let result = self
            .abi_provider
            .export_abi(package_id, blueprint_name)
            .map_err(|_| {
                BuildTransactionError::FailedToExportFunctionAbi(
                    package_id,
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
                    package_id,
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
    /// method ABI, including resource buckets and proofs.
    ///
    /// If an Account component ID is provided, resources will be withdrawn from the given account;
    /// otherwise, they will be taken from transaction worktop.
    pub fn call_method(
        &mut self,
        component_id: ComponentId,
        method: &str,
        args: Vec<String>,
        account: Option<ComponentId>,
    ) -> &mut Self {
        let result = self
            .abi_provider
            .export_abi_component(component_id)
            .map_err(|_| {
                BuildTransactionError::FailedToExportMethodAbi(component_id, method.to_owned())
            })
            .and_then(|abi| Self::find_method_abi(&abi, method))
            .and_then(|m| {
                self.prepare_args(&m.inputs, args, account)
                    .map_err(|e| BuildTransactionError::FailedToBuildArgs(e))
            });

        match result {
            Ok(args) => {
                self.add_instruction(Instruction::CallMethod {
                    component_id,
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
        component_id: ComponentId,
        method: &str,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethodWithAllResources {
            component_id,
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
        self.add_instruction(Instruction::PublishPackage {
            code: code.to_vec(),
        })
        .0
    }

    fn single_authority(
        resource_def_id: ResourceDefId,
        permission: u64,
    ) -> HashMap<ResourceDefId, u64> {
        let mut map = HashMap::new();
        map.insert(resource_def_id, permission);
        map
    }

    /// Creates a token resource with mutable supply.
    pub fn new_token_mutable(
        &mut self,
        metadata: HashMap<String, String>,
        minter_resource_def_id: ResourceDefId,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_id: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            function: "new_resource".to_owned(),
            args: vec![
                scrypto_encode(&ResourceType::Fungible { divisibility: 18 }),
                scrypto_encode(&metadata),
                scrypto_encode(&(MINTABLE | BURNABLE)),
                scrypto_encode(&0u64),
                scrypto_encode(&Self::single_authority(
                    minter_resource_def_id,
                    MAY_MINT | MAY_BURN,
                )),
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
            package_id: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            function: "new_resource".to_owned(),
            args: vec![
                scrypto_encode(&ResourceType::Fungible { divisibility: 18 }),
                scrypto_encode(&metadata),
                scrypto_encode(&0u64),
                scrypto_encode(&0u64),
                scrypto_encode(&HashMap::<ResourceDefId, u64>::new()),
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
        minter_resource_def_id: ResourceDefId,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_id: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            function: "new_resource".to_owned(),
            args: vec![
                scrypto_encode(&ResourceType::Fungible { divisibility: 0 }),
                scrypto_encode(&metadata),
                scrypto_encode(&(MINTABLE | BURNABLE)),
                scrypto_encode(&0u64),
                scrypto_encode(&Self::single_authority(
                    minter_resource_def_id,
                    MAY_MINT | MAY_BURN,
                )),
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
            package_id: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            function: "new_resource".to_owned(),
            args: vec![
                scrypto_encode(&ResourceType::Fungible { divisibility: 0 }),
                scrypto_encode(&metadata),
                scrypto_encode(&0u64),
                scrypto_encode(&0u64),
                scrypto_encode(&HashMap::<ResourceDefId, u64>::new()),
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
        resource_def_id: ResourceDefId,
        minter_resource_def_id: ResourceDefId,
    ) -> &mut Self {
        self.take_from_worktop(
            &ResourceSpecification::Fungible {
                amount: 1.into(),
                resource_def_id: minter_resource_def_id,
            },
            |builder, bucket_id| {
                builder.create_bucket_proof(bucket_id, |builder, proof_id| {
                    builder
                        .add_instruction(Instruction::CallFunction {
                            package_id: SYSTEM_PACKAGE,
                            blueprint_name: "System".to_owned(),
                            function: "mint".to_owned(),
                            args: vec![
                                scrypto_encode(&amount),
                                scrypto_encode(&resource_def_id),
                                scrypto_encode(&scrypto::resource::Proof(proof_id)),
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
            package_id: ACCOUNT_PACKAGE,
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
                    package_id: ACCOUNT_PACKAGE,
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
        account: ComponentId,
    ) -> &mut Self {
        self.clone_proof(
            ECDSA_TOKEN_PROOF_ID,
            |builder, proof_id| match resource_spec {
                ResourceSpecification::Fungible {
                    amount,
                    resource_def_id,
                } => {
                    builder
                        .add_instruction(Instruction::CallMethod {
                            component_id: account,
                            method: "withdraw".to_owned(),
                            args: vec![
                                scrypto_encode(amount),
                                scrypto_encode(resource_def_id),
                                scrypto_encode(&scrypto::resource::Proof(proof_id)),
                            ],
                        })
                        .0
                }
                ResourceSpecification::NonFungible {
                    ids,
                    resource_def_id,
                } => {
                    builder
                        .add_instruction(Instruction::CallMethod {
                            component_id: account,
                            method: "withdraw_non_fungibles".to_owned(),
                            args: vec![
                                scrypto_encode(ids),
                                scrypto_encode(resource_def_id),
                                scrypto_encode(&scrypto::resource::Proof(proof_id)),
                            ],
                        })
                        .0
                }
                ResourceSpecification::All { .. } => {
                    panic!("Withdrawing all from account is not supported!");
                }
            },
        )
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
        account: Option<ComponentId>,
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
        account: Option<ComponentId>,
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
            CustomType::PackageId => {
                let value = arg
                    .parse::<PackageId>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            CustomType::ComponentId => {
                let value = arg
                    .parse::<ComponentId>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            CustomType::ResourceDefId => {
                let value = arg
                    .parse::<ResourceDefId>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            CustomType::Hash => {
                let value = arg
                    .parse::<Hash>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            CustomType::NonFungibleId => {
                let value = arg
                    .parse::<NonFungibleId>()
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
                Ok(scrypto_encode(&scrypto::resource::Bucket(
                    created_bucket_id.unwrap(),
                )))
            }
            CustomType::Proof => {
                let resource_spec = parse_resource_specification(i, ty, arg)?;
                if let Some(account) = account {
                    self.withdraw_from_account(&resource_spec, account);
                }
                let mut created_proof_id = None;
                self.take_from_worktop(&resource_spec, |builder, bucket_id| {
                    builder.create_bucket_proof(bucket_id, |builder, proof_id| {
                        created_proof_id = Some(proof_id);
                        builder
                    });
                    builder
                });
                Ok(scrypto_encode(&scrypto::resource::Proof(
                    created_proof_id.unwrap(),
                )))
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
