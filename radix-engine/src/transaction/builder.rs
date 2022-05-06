use sbor::describe::*;
use sbor::*;
use scrypto::buffer::*;
use scrypto::crypto::*;
use scrypto::engine::types::*;
use scrypto::prelude::{AccessRuleNode, Burn, AccessRule, Mint, Withdraw};
use scrypto::resource::{require, LOCKED};
use scrypto::rust::borrow::ToOwned;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::collections::*;
use scrypto::rust::fmt;
use scrypto::rust::str::FromStr;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::types::*;
use scrypto::values::*;
use scrypto::*;

use crate::engine::*;
use crate::model::*;
use crate::transaction::*;

/// Utility for building transaction.
pub struct TransactionBuilder {
    /// ID validator for calculating transaction object id
    id_validator: IdValidator,
    /// Instructions generated.
    instructions: Vec<Instruction>,
}

impl TransactionBuilder {
    /// Starts a new transaction builder.
    pub fn new() -> Self {
        Self {
            id_validator: IdValidator::new(),
            instructions: Vec::new(),
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
            Instruction::TakeFromWorktop { .. }
            | Instruction::TakeFromWorktopByAmount { .. }
            | Instruction::TakeFromWorktopByIds { .. } => {
                new_bucket_id = Some(self.id_validator.new_bucket().unwrap());
            }
            Instruction::ReturnToWorktop { bucket_id } => {
                self.id_validator.drop_bucket(bucket_id).unwrap();
            }
            Instruction::AssertWorktopContains { .. }
            | Instruction::AssertWorktopContainsByAmount { .. }
            | Instruction::AssertWorktopContainsByIds { .. } => {}
            Instruction::PopFromAuthZone { .. } => {
                new_proof_id = Some(
                    self.id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .unwrap(),
                );
            }
            Instruction::PushToAuthZone { proof_id } => {
                self.id_validator.drop_proof(proof_id).unwrap();
            }
            Instruction::ClearAuthZone => {}
            Instruction::CreateProofFromAuthZone { .. }
            | Instruction::CreateProofFromAuthZoneByAmount { .. }
            | Instruction::CreateProofFromAuthZoneByIds { .. } => {
                new_proof_id = Some(
                    self.id_validator
                        .new_proof(ProofKind::AuthZoneProof)
                        .unwrap(),
                );
            }
            Instruction::CreateProofFromBucket { bucket_id } => {
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
            Instruction::CallFunction { call_data , .. } | Instruction::CallMethod { call_data, .. } => {
                let scrypt_value = ScryptoValue::from_slice(&call_data).unwrap();
                self.id_validator.move_resources(&scrypt_value).unwrap();
            }
            Instruction::CallMethodWithAllResources { .. } => {
                self.id_validator.move_all_resources().unwrap();
            }
            Instruction::PublishPackage { .. } | Instruction::Nonce { .. } => {}
        }

        self.instructions.push(inst);

        (self, new_bucket_id, new_proof_id)
    }

    /// Takes resource from worktop.
    pub fn take_from_worktop<F>(&mut self, resource_address: ResourceAddress, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, BucketId) -> &mut Self,
    {
        let (builder, bucket_id, _) =
            self.add_instruction(Instruction::TakeFromWorktop { resource_address });
        then(builder, bucket_id.unwrap())
    }

    /// Takes resource from worktop, by amount.
    pub fn take_from_worktop_by_amount<F>(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, BucketId) -> &mut Self,
    {
        let (builder, bucket_id, _) = self.add_instruction(Instruction::TakeFromWorktopByAmount {
            amount,
            resource_address,
        });
        then(builder, bucket_id.unwrap())
    }

    /// Takes resource from worktop, by non-fungible ids.
    pub fn take_from_worktop_by_ids<F>(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, BucketId) -> &mut Self,
    {
        let (builder, bucket_id, _) = self.add_instruction(Instruction::TakeFromWorktopByIds {
            ids: ids.clone(),
            resource_address,
        });
        then(builder, bucket_id.unwrap())
    }

    /// Adds a bucket of resource to worktop.
    pub fn return_to_worktop(&mut self, bucket_id: BucketId) -> &mut Self {
        self.add_instruction(Instruction::ReturnToWorktop { bucket_id })
            .0
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains(&mut self, resource_address: ResourceAddress) -> &mut Self {
        self.add_instruction(Instruction::AssertWorktopContains { resource_address })
            .0
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains_by_amount(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(Instruction::AssertWorktopContainsByAmount {
            amount,
            resource_address,
        })
        .0
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(Instruction::AssertWorktopContainsByIds {
            ids: ids.clone(),
            resource_address,
        })
        .0
    }

    /// Pops the most recent proof from auth zone.
    pub fn pop_from_auth_zone<F>(&mut self, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ProofId) -> &mut Self,
    {
        let (builder, _, proof_id) = self.add_instruction(Instruction::PopFromAuthZone {});
        then(builder, proof_id.unwrap())
    }

    /// Pushes a proof onto the auth zone
    pub fn push_to_auth_zone(&mut self, proof_id: ProofId) -> &mut Self {
        self.add_instruction(Instruction::PushToAuthZone { proof_id });
        self
    }

    /// Clears the auth zone.
    pub fn clear_auth_zone(&mut self) -> &mut Self {
        self.add_instruction(Instruction::ClearAuthZone).0
    }

    /// Creates proof from the auth zone.
    pub fn create_proof_from_auth_zone<F>(
        &mut self,
        resource_address: ResourceAddress,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ProofId) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(Instruction::CreateProofFromAuthZone { resource_address });
        then(builder, proof_id.unwrap())
    }

    /// Creates proof from the auth zone by amount.
    pub fn create_proof_from_auth_zone_by_amount<F>(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ProofId) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(Instruction::CreateProofFromAuthZoneByAmount {
                amount,
                resource_address,
            });
        then(builder, proof_id.unwrap())
    }

    /// Creates proof from the auth zone by non-fungible ids.
    pub fn create_proof_from_auth_zone_by_ids<F>(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ProofId) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(Instruction::CreateProofFromAuthZoneByIds {
                ids: ids.clone(),
                resource_address,
            });
        then(builder, proof_id.unwrap())
    }

    /// Creates proof from a bucket.
    pub fn create_proof_from_bucket<F>(&mut self, bucket_id: BucketId, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ProofId) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(Instruction::CreateProofFromBucket { bucket_id });
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

    /// Calls a function where the arguments should be an array of encoded Scrypto value.
    pub fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        call_data: Vec<u8>,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address,
            blueprint_name: blueprint_name.to_owned(),
            call_data,
        });
        self
    }

    /// Calls a function.
    ///
    /// The implementation will automatically prepare the arguments based on the
    /// function ABI, including resource buckets and proofs.
    ///
    /// If an Account component address is provided, resources will be withdrawn from the given account;
    /// otherwise, they will be taken from transaction worktop.
    pub fn call_function_with_abi(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function: &str,
        args: Vec<String>,
        account: Option<ComponentAddress>,
        blueprint_abi: &abi::Blueprint,
    ) -> Result<&mut Self, CallWithAbiError> {
        let abi = blueprint_abi
            .functions
            .iter()
            .find(|f| f.name == function)
            .map(Clone::clone)
            .ok_or_else(|| CallWithAbiError::FunctionNotFound(function.to_owned()))?;

        let arguments = self
            .parse_args(&abi.inputs, args, account)
            .map_err(|e| CallWithAbiError::FailedToBuildArgs(e))?;

        let mut fields = Vec::new();
        for arg in arguments {
            fields.push(::sbor::decode_any(&arg).unwrap());
        }
        let variant = ::sbor::Value::Enum {
            name: function.to_owned(),
            fields
        };
        let mut bytes = Vec::new();
        let mut enc = ::sbor::Encoder::with_type(&mut bytes);
        ::sbor::encode_any(None, &variant, &mut enc);

        Ok(self
            .add_instruction(Instruction::CallFunction {
                package_address,
                blueprint_name: blueprint_name.to_owned(),
                call_data: bytes,
            })
            .0)
    }

    /// Calls a method where the arguments should be an array of encoded Scrypto value.
    pub fn call_method(
        &mut self,
        component_address: ComponentAddress,
        call_data: Vec<u8>,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address,
            call_data,
        });
        self
    }

    /// Calls a method.
    ///
    /// The implementation will automatically prepare the arguments based on the
    /// method ABI, including resource buckets and proofs.
    ///
    /// If an Account component address is provided, resources will be withdrawn from the given account;
    /// otherwise, they will be taken from transaction worktop.
    pub fn call_method_with_abi(
        &mut self,
        component_address: ComponentAddress,
        method: &str,
        args: Vec<String>,
        account: Option<ComponentAddress>,
        blueprint_abi: &abi::Blueprint,
    ) -> Result<&mut Self, CallWithAbiError> {
        let abi = blueprint_abi
            .methods
            .iter()
            .find(|m| m.name == method)
            .map(Clone::clone)
            .ok_or_else(|| CallWithAbiError::MethodNotFound(method.to_owned()))?;

        let arguments = self
            .parse_args(&abi.inputs, args, account)
            .map_err(|e| CallWithAbiError::FailedToBuildArgs(e))?;

        let call_data = call_data_bytes_args!(method.to_owned(), arguments);
        Ok(self
            .add_instruction(Instruction::CallMethod {
                component_address,
                call_data,
            })
            .0)
    }

    /// Calls a method with all the resources on worktop.
    ///
    /// The callee method must have only one parameter with type `Vec<Bucket>`; otherwise,
    /// a runtime failure is triggered.
    pub fn call_method_with_all_resources(
        &mut self,
        component_address: ComponentAddress,
        method: &str,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethodWithAllResources {
            component_address,
            method: method.into(),
        })
        .0
    }

    /// Publishes a package.
    pub fn publish_package(&mut self, code: &[u8]) -> &mut Self {
        self.add_instruction(Instruction::PublishPackage {
            code: code.to_vec(),
        })
        .0
    }

    /// Builds a transaction with the given nonce.
    pub fn build(&self, nonce: u64) -> Transaction {
        let mut instructions = self.instructions.clone();
        instructions.push(Instruction::Nonce { nonce });

        Transaction { instructions }
    }

    /// Builds a transaction with no nonce
    ///
    /// Nonce can be later filled by a third party or wallet.
    pub fn build_with_no_nonce(&self) -> Transaction {
        Transaction {
            instructions: self.instructions.clone(),
        }
    }

    /// Creates a token resource with mutable supply.
    pub fn new_token_mutable(
        &mut self,
        metadata: HashMap<String, String>,
        minter_resource_address: ResourceAddress,
    ) -> &mut Self {
        let mut resource_auth = HashMap::new();
        resource_auth.insert(Withdraw, (rule!(allow_all), LOCKED));
        resource_auth.insert(
            Mint,
            (rule!(require(minter_resource_address.clone())), LOCKED),
        );
        resource_auth.insert(
            Burn,
            (rule!(require(minter_resource_address.clone())), LOCKED),
        );

        let mint_params: Option<MintParams> = Option::None;

        self.add_instruction(Instruction::CallFunction {
            package_address: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            call_data: call_data!(
                new_resource(
                    ResourceType::Fungible { divisibility: 18 },
                    metadata,
                    resource_auth,
                    mint_params
                )
            ),
        })
        .0
    }

    /// Creates a token resource with fixed supply.
    pub fn new_token_fixed(
        &mut self,
        metadata: HashMap<String, String>,
        initial_supply: Decimal,
    ) -> &mut Self {
        let mut resource_auth = HashMap::new();
        resource_auth.insert(Withdraw, (rule!(allow_all), LOCKED));

        self.add_instruction(Instruction::CallFunction {
            package_address: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            call_data: call_data!(
                new_resource(
                    ResourceType::Fungible { divisibility: 18 },
                    metadata,
                    resource_auth,
                    Option::Some(MintParams::Fungible {
                        amount: initial_supply.into(),
                    })
                )
            ),
        })
        .0
    }

    /// Creates a badge resource with mutable supply.
    pub fn new_badge_mutable(
        &mut self,
        metadata: HashMap<String, String>,
        minter_resource_address: ResourceAddress,
    ) -> &mut Self {
        let mut resource_auth = HashMap::new();
        resource_auth.insert(Withdraw, (rule!(allow_all), LOCKED));
        resource_auth.insert(
            Mint,
            (rule!(require(minter_resource_address.clone())), LOCKED),
        );
        resource_auth.insert(
            Burn,
            (rule!(require(minter_resource_address.clone())), LOCKED),
        );

        let mint_params: Option<MintParams> = Option::None;

        self.add_instruction(Instruction::CallFunction {
            package_address: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            call_data: call_data!(
                new_resource(
                    ResourceType::Fungible { divisibility: 0 },
                    metadata,
                    resource_auth,
                    mint_params
                )
            ),
        })
        .0
    }

    /// Creates a badge resource with fixed supply.
    pub fn new_badge_fixed(
        &mut self,
        metadata: HashMap<String, String>,
        initial_supply: Decimal,
    ) -> &mut Self {
        let mut resource_auth = HashMap::new();
        resource_auth.insert(Withdraw, (rule!(allow_all), LOCKED));

        self.add_instruction(Instruction::CallFunction {
            package_address: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            call_data: call_data!(
                new_resource(
                    ResourceType::Fungible { divisibility: 0 },
                    metadata,
                    resource_auth,
                    Option::Some(MintParams::Fungible {
                        amount: initial_supply.into(),
                    })
                )
            ),
        })
        .0
    }

    /// Mints resource.
    pub fn mint(&mut self, amount: Decimal, resource_address: ResourceAddress) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address: SYSTEM_PACKAGE,
            blueprint_name: "System".to_owned(),
            call_data: call_data!(mint(amount, resource_address)),
        });
        self
    }

    /// Burns a resource.
    pub fn burn(&mut self, amount: Decimal, resource_address: ResourceAddress) -> &mut Self {
        self.take_from_worktop_by_amount(amount, resource_address, |builder, bucket_id| {
            builder
                .add_instruction(Instruction::CallFunction {
                    package_address: SYSTEM_PACKAGE,
                    blueprint_name: "System".to_owned(),
                    call_data: call_data!(burn(scrypto::resource::Bucket(bucket_id))),
                })
                .0
        })
    }

    pub fn burn_non_fungible(&mut self, non_fungible_address: NonFungibleAddress) -> &mut Self {
        let mut ids = BTreeSet::new();
        ids.insert(non_fungible_address.non_fungible_id());
        self.take_from_worktop_by_ids(
            &ids,
            non_fungible_address.resource_address(),
            |builder, bucket_id| {
                builder
                    .add_instruction(Instruction::CallFunction {
                        package_address: SYSTEM_PACKAGE,
                        blueprint_name: "System".to_owned(),
                        call_data: call_data!(burn(scrypto::resource::Bucket(bucket_id))),
                    })
                    .0
            },
        )
    }

    /// Creates an account.
    pub fn new_account(&mut self, withdraw_auth: &AccessRuleNode) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: "Account".to_owned(),
            call_data: call_data!(new(withdraw_auth.clone())),
        })
        .0
    }

    /// Creates an account with some initial resource.
    pub fn new_account_with_resource(
        &mut self,
        withdraw_auth: &AccessRule,
        bucket_id: BucketId,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: "Account".to_owned(),
            call_data: call_data!(new_with_resource(withdraw_auth.clone(), scrypto::resource::Bucket(bucket_id)))
        })
        .0
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account(
        &mut self,
        resource_address: ResourceAddress,
        account: ComponentAddress,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            call_data: call_data!(withdraw(resource_address)),
        })
        .0
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account_by_amount(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
        account: ComponentAddress,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            call_data: call_data!(withdraw_by_amount(amount, resource_address)),
        })
        .0
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
        account: ComponentAddress,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            call_data: call_data!(withdraw_by_ids(ids.clone(), resource_address)),
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account(
        &mut self,
        resource_address: ResourceAddress,
        account: ComponentAddress,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            call_data: call_data!(create_proof(resource_address)),
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_by_amount(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
        account: ComponentAddress,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            call_data: call_data!(create_proof_by_amount(amount, resource_address)),
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_by_ids(
        &mut self,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
        account: ComponentAddress,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            call_data: call_data!(create_proof_by_ids(ids.clone(), resource_address)),
        })
        .0
    }

    //===============================
    // private methods below
    //===============================

    fn parse_args(
        &mut self,
        types: &[Type],
        args: Vec<String>,
        account: Option<ComponentAddress>,
    ) -> Result<Vec<Vec<u8>>, BuildArgsError> {
        let mut encoded = Vec::new();

        for (i, t) in types.iter().enumerate() {
            let arg = args
                .get(i)
                .ok_or_else(|| BuildArgsError::MissingArgument(i, t.clone()))?;
            let res = match t {
                Type::Bool => self.parse_basic_ty::<bool>(i, t, arg),
                Type::I8 => self.parse_basic_ty::<i8>(i, t, arg),
                Type::I16 => self.parse_basic_ty::<i16>(i, t, arg),
                Type::I32 => self.parse_basic_ty::<i32>(i, t, arg),
                Type::I64 => self.parse_basic_ty::<i64>(i, t, arg),
                Type::I128 => self.parse_basic_ty::<i128>(i, t, arg),
                Type::U8 => self.parse_basic_ty::<u8>(i, t, arg),
                Type::U16 => self.parse_basic_ty::<u16>(i, t, arg),
                Type::U32 => self.parse_basic_ty::<u32>(i, t, arg),
                Type::U64 => self.parse_basic_ty::<u64>(i, t, arg),
                Type::U128 => self.parse_basic_ty::<u128>(i, t, arg),
                Type::String => self.parse_basic_ty::<String>(i, t, arg),
                Type::Custom { name, .. } => self.parse_custom_ty(i, t, arg, name, account),
                _ => Err(BuildArgsError::UnsupportedType(i, t.clone())),
            };
            encoded.push(res?);
        }

        Ok(encoded)
    }

    fn parse_basic_ty<T>(
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

    fn parse_custom_ty(
        &mut self,
        i: usize,
        ty: &Type,
        arg: &str,
        name: &str,
        account: Option<ComponentAddress>,
    ) -> Result<Vec<u8>, BuildArgsError> {
        match ScryptoType::from_name(name).ok_or(BuildArgsError::UnsupportedType(i, ty.clone()))? {
            ScryptoType::Decimal => {
                let value = arg
                    .parse::<Decimal>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            ScryptoType::PackageAddress => {
                let value = arg
                    .parse::<PackageAddress>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            ScryptoType::ComponentAddress => {
                let value = arg
                    .parse::<ComponentAddress>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            ScryptoType::ResourceAddress => {
                let value = arg
                    .parse::<ResourceAddress>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            ScryptoType::Hash => {
                let value = arg
                    .parse::<Hash>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            ScryptoType::NonFungibleId => {
                let value = arg
                    .parse::<NonFungibleId>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            ScryptoType::Bucket => {
                let resource_specifier = parse_resource_specifier(arg)
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                let bucket_id = match resource_specifier {
                    ResourceSpecifier::Amount(amount, resource_address) => {
                        if let Some(account) = account {
                            self.withdraw_from_account_by_amount(amount, resource_address, account);
                        }
                        self.add_instruction(Instruction::TakeFromWorktopByAmount {
                            amount,
                            resource_address,
                        })
                        .1
                        .unwrap()
                    }
                    ResourceSpecifier::Ids(ids, resource_address) => {
                        if let Some(account) = account {
                            self.withdraw_from_account_by_ids(&ids, resource_address, account);
                        }
                        self.add_instruction(Instruction::TakeFromWorktopByIds {
                            ids,
                            resource_address,
                        })
                        .1
                        .unwrap()
                    }
                };
                Ok(scrypto_encode(&scrypto::resource::Bucket(bucket_id)))
            }
            ScryptoType::Proof => {
                let resource_specifier = parse_resource_specifier(arg)
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                let proof_id = match resource_specifier {
                    ResourceSpecifier::Amount(amount, resource_address) => {
                        if let Some(account) = account {
                            self.create_proof_from_account_by_amount(
                                amount,
                                resource_address,
                                account,
                            );
                            self.add_instruction(Instruction::PopFromAuthZone)
                                .2
                                .unwrap()
                        } else {
                            todo!("Take from worktop and create proof")
                        }
                    }
                    ResourceSpecifier::Ids(ids, resource_address) => {
                        if let Some(account) = account {
                            self.create_proof_from_account_by_ids(&ids, resource_address, account);
                            self.add_instruction(Instruction::PopFromAuthZone)
                                .2
                                .unwrap()
                        } else {
                            todo!("Take from worktop and create proof")
                        }
                    }
                };
                Ok(scrypto_encode(&scrypto::resource::Proof(proof_id)))
            }
            _ => Err(BuildArgsError::UnsupportedType(i, ty.clone())),
        }
    }
}

enum ResourceSpecifier {
    Amount(Decimal, ResourceAddress),
    Ids(BTreeSet<NonFungibleId>, ResourceAddress),
}

enum ParseResourceSpecifierError {
    IncompleteResourceSpecifier,
    InvalidResourceAddress(String),
    InvalidAmount(String),
    InvalidNonFungibleId(String),
    MoreThanOneAmountSpecified,
}

fn parse_resource_specifier(input: &str) -> Result<ResourceSpecifier, ParseResourceSpecifierError> {
    let tokens: Vec<&str> = input.trim().split(',').map(|s| s.trim()).collect();

    // check length
    if tokens.len() < 2 {
        return Err(ParseResourceSpecifierError::IncompleteResourceSpecifier);
    }

    // parse resource address
    let token = tokens[tokens.len() - 1];
    let resource_address = token
        .parse::<ResourceAddress>()
        .map_err(|_| ParseResourceSpecifierError::InvalidResourceAddress(token.to_owned()))?;

    // parse non-fungible ids or amount
    if tokens[0].starts_with('#') {
        let mut ids = BTreeSet::<NonFungibleId>::new();
        for id in &tokens[..tokens.len() - 1] {
            ids.insert(
                id[1..].parse().map_err(|_| {
                    ParseResourceSpecifierError::InvalidNonFungibleId(id.to_string())
                })?,
            );
        }
        Ok(ResourceSpecifier::Ids(ids, resource_address))
    } else {
        if tokens.len() != 2 {
            return Err(ParseResourceSpecifierError::MoreThanOneAmountSpecified);
        }
        let amount: Decimal = tokens[0]
            .parse()
            .map_err(|_| ParseResourceSpecifierError::InvalidAmount(tokens[0].to_owned()))?;
        Ok(ResourceSpecifier::Amount(amount, resource_address))
    }
}
