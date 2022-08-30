use sbor::describe::*;
use sbor::rust::borrow::ToOwned;
use sbor::rust::collections::*;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::abi::*;
use scrypto::address::Bech32Decoder;
use scrypto::buffer::*;
use scrypto::component::Package;
use scrypto::component::{ComponentAddress, PackageAddress};
use scrypto::constants::*;
use scrypto::core::Blob;
use scrypto::core::NetworkDefinition;
use scrypto::crypto::*;
use scrypto::engine::types::*;
use scrypto::math::*;
use scrypto::resource::MintParams;
use scrypto::resource::ResourceType;
use scrypto::resource::{require, LOCKED};
use scrypto::resource::{AccessRule, AccessRuleNode, Burn, Mint, Withdraw};
use scrypto::resource::{NonFungibleAddress, NonFungibleId, ResourceAddress};
use scrypto::values::*;
use scrypto::*;

use crate::errors::*;
use crate::model::*;
use crate::validation::*;

/// Utility for building transaction manifest.
pub struct ManifestBuilder {
    /// The decoder used by the manifest (mainly for the `call_*_with_abi)
    decoder: Bech32Decoder,
    /// ID validator for calculating transaction object id
    id_validator: IdValidator,
    /// Instructions generated.
    instructions: Vec<Instruction>,
    /// Blobs
    blobs: BTreeMap<Hash, Vec<u8>>,
}

impl ManifestBuilder {
    /// Starts a new transaction builder.
    pub fn new(network: &NetworkDefinition) -> Self {
        Self {
            decoder: Bech32Decoder::new(network),
            id_validator: IdValidator::new(),
            instructions: Vec::new(),
            blobs: BTreeMap::default(),
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
            Instruction::DropAllProofs => {
                self.id_validator.drop_all_proofs().unwrap();
            }
            Instruction::CallFunction { args, .. } | Instruction::CallMethod { args, .. } => {
                let scrypt_value = ScryptoValue::from_slice(&args).unwrap();
                self.id_validator.move_resources(&scrypt_value).unwrap();
            }
            Instruction::PublishPackage { .. } => {}
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

    /// Drops all proofs.
    pub fn drop_all_proofs(&mut self) -> &mut Self {
        self.add_instruction(Instruction::DropAllProofs).0
    }

    /// Calls a function where the arguments should be an array of encoded Scrypto value.
    pub fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        method_name: &str,
        args: Vec<u8>,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address,
            blueprint_name: blueprint_name.to_owned(),
            method_name: method_name.to_string(),
            args,
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
        blueprint_abi: &abi::BlueprintAbi,
    ) -> Result<&mut Self, BuildCallWithAbiError> {
        let abi = blueprint_abi
            .fns
            .iter()
            .find(|f| f.ident == function)
            .map(Clone::clone)
            .ok_or_else(|| BuildCallWithAbiError::FunctionNotFound(function.to_owned()))?;

        let arguments = self
            .parse_args(&abi.input, args, account)
            .map_err(|e| BuildCallWithAbiError::FailedToBuildArgs(e))?;

        let mut fields = Vec::new();
        for arg in arguments {
            fields.push(::sbor::decode_any(&arg).unwrap());
        }
        let input_struct = ::sbor::Value::Struct { fields };
        let bytes = ::sbor::encode_any(&input_struct);

        Ok(self
            .add_instruction(Instruction::CallFunction {
                package_address,
                blueprint_name: blueprint_name.to_owned(),
                method_name: function.to_string(),
                args: bytes,
            })
            .0)
    }

    /// Calls a method where the arguments should be an array of encoded Scrypto value.
    pub fn call_method(
        &mut self,
        component_address: ComponentAddress,
        method_name: &str,
        args: Vec<u8>,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address,
            method_name: method_name.to_owned(),
            args,
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
        blueprint_abi: &abi::BlueprintAbi,
    ) -> Result<&mut Self, BuildCallWithAbiError> {
        let abi = blueprint_abi
            .fns
            .iter()
            .find(|m| m.ident == method)
            .map(Clone::clone)
            .ok_or_else(|| BuildCallWithAbiError::MethodNotFound(method.to_owned()))?;

        let arguments = self
            .parse_args(&abi.input, args, account)
            .map_err(|e| BuildCallWithAbiError::FailedToBuildArgs(e))?;

        Ok(self
            .add_instruction(Instruction::CallMethod {
                component_address,
                method_name: method.to_owned(),
                args: args_from_bytes_vec!(arguments),
            })
            .0)
    }

    /// Publishes a package.
    pub fn publish_package(&mut self, package: Package) -> &mut Self {
        let package_blob = scrypto_encode(&package);
        let package_blob_hash = hash(&package_blob);
        self.blobs.insert(package_blob_hash, package_blob);
        self.add_instruction(Instruction::PublishPackage {
            package: Blob(package_blob_hash),
        })
        .0
    }

    /// Builds a transaction manifest.
    pub fn build(&self) -> TransactionManifest {
        TransactionManifest {
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
            package_address: SYS_UTILS_PACKAGE,
            blueprint_name: "SysUtils".to_owned(),
            method_name: "new_resource".to_string(),
            args: args!(
                ResourceType::Fungible { divisibility: 18 },
                metadata,
                resource_auth,
                mint_params
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
            package_address: SYS_UTILS_PACKAGE,
            blueprint_name: "SysUtils".to_owned(),
            method_name: "new_resource".to_string(),
            args: args!(
                ResourceType::Fungible { divisibility: 18 },
                metadata,
                resource_auth,
                Option::Some(MintParams::Fungible {
                    amount: initial_supply.into(),
                })
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
            package_address: SYS_UTILS_PACKAGE,
            blueprint_name: "SysUtils".to_owned(),
            method_name: "new_resource".to_string(),
            args: args!(
                ResourceType::Fungible { divisibility: 0 },
                metadata,
                resource_auth,
                mint_params
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
            package_address: SYS_UTILS_PACKAGE,
            blueprint_name: "SysUtils".to_owned(),
            method_name: "new_resource".to_string(),
            args: args!(
                ResourceType::Fungible { divisibility: 0 },
                metadata,
                resource_auth,
                Option::Some(MintParams::Fungible {
                    amount: initial_supply.into(),
                })
            ),
        })
        .0
    }

    /// Mints resource.
    pub fn mint(&mut self, amount: Decimal, resource_address: ResourceAddress) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address: SYS_UTILS_PACKAGE,
            blueprint_name: "SysUtils".to_owned(),
            method_name: "mint".to_string(),
            args: args!(amount, resource_address),
        });
        self
    }

    /// Burns a resource.
    pub fn burn(&mut self, amount: Decimal, resource_address: ResourceAddress) -> &mut Self {
        self.take_from_worktop_by_amount(amount, resource_address, |builder, bucket_id| {
            builder
                .add_instruction(Instruction::CallFunction {
                    package_address: SYS_UTILS_PACKAGE,
                    blueprint_name: "SysUtils".to_owned(),
                    method_name: "burn".to_string(),
                    args: args!(scrypto::resource::Bucket(bucket_id)),
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
                        package_address: SYS_UTILS_PACKAGE,
                        blueprint_name: "SysUtils".to_owned(),
                        method_name: "burn".to_string(),
                        args: args!(scrypto::resource::Bucket(bucket_id)),
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
            method_name: "new".to_string(),
            args: args!(withdraw_auth.clone()),
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
            method_name: "new_with_resource".to_string(),
            args: args!(withdraw_auth.clone(), scrypto::resource::Bucket(bucket_id)),
        })
        .0
    }

    /// Locks a fee from the XRD vault of an account.
    pub fn lock_fee(&mut self, amount: Decimal, account: ComponentAddress) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            method_name: "lock_fee".to_string(),
            args: args!(amount),
        })
        .0
    }

    pub fn lock_contingent_fee(&mut self, amount: Decimal, account: ComponentAddress) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            method_name: "lock_contingent_fee".to_string(),
            args: args!(amount),
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
            method_name: "withdraw".to_string(),
            args: args!(resource_address),
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
            method_name: "withdraw_by_amount".to_string(),
            args: args!(amount, resource_address),
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
            method_name: "withdraw_by_ids".to_string(),
            args: args!(ids.clone(), resource_address),
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
            method_name: "create_proof".to_string(),
            args: args!(resource_address),
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
            method_name: "create_proof_by_amount".to_string(),
            args: args!(amount, resource_address),
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
            method_name: "create_proof_by_ids".to_string(),
            args: args!(ids.clone(), resource_address),
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_by_resource_specifier(
        &mut self,
        resource_specifier: String,
        account: ComponentAddress,
    ) -> Result<&mut Self, BuildArgsError> {
        let resource_specifier = parse_resource_specifier(&resource_specifier, &self.decoder)
            .map_err(|_| BuildArgsError::InvalidResourceSpecifier(resource_specifier))?;
        let builder = match resource_specifier {
            ResourceSpecifier::Amount(amount, resource_address) => {
                self.create_proof_from_account_by_amount(amount, resource_address, account)
            }
            ResourceSpecifier::Ids(non_fungible_ids, resource_address) => {
                self.create_proof_from_account_by_ids(&non_fungible_ids, resource_address, account)
            }
        };
        Ok(builder)
    }

    //===============================
    // private methods below
    //===============================

    fn parse_args(
        &mut self,
        arg_type: &Type,
        args: Vec<String>,
        account: Option<ComponentAddress>,
    ) -> Result<Vec<Vec<u8>>, BuildArgsError> {
        let mut encoded = Vec::new();

        match arg_type {
            sbor::Type::Struct {
                name: _,
                fields: Fields::Named { named },
            } => {
                for (i, (_, t)) in named.iter().enumerate() {
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
                        Type::Custom { type_id, .. } => {
                            self.parse_custom_ty(i, t, arg, *type_id, account)
                        }
                        _ => Err(BuildArgsError::UnsupportedType(i, t.clone())),
                    };
                    encoded.push(res?);
                }
                Ok(())
            }
            _ => Err(BuildArgsError::UnsupportedRootType(arg_type.clone())),
        }?;

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
        type_id: u8,
        account: Option<ComponentAddress>,
    ) -> Result<Vec<u8>, BuildArgsError> {
        match ScryptoType::from_id(type_id).ok_or(BuildArgsError::UnsupportedType(i, ty.clone()))? {
            ScryptoType::Decimal => {
                let value = arg
                    .parse::<Decimal>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            ScryptoType::PreciseDecimal => {
                let value = arg
                    .parse::<PreciseDecimal>()
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            ScryptoType::PackageAddress => {
                let value = self
                    .decoder
                    .validate_and_decode_package_address(arg)
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            ScryptoType::ComponentAddress => {
                let value = self
                    .decoder
                    .validate_and_decode_component_address(arg)
                    .map_err(|_| BuildArgsError::FailedToParse(i, ty.clone(), arg.to_owned()))?;
                Ok(scrypto_encode(&value))
            }
            ScryptoType::ResourceAddress => {
                let value = self
                    .decoder
                    .validate_and_decode_resource_address(arg)
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
                let resource_specifier = parse_resource_specifier(arg, &self.decoder)
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
                let resource_specifier = parse_resource_specifier(arg, &self.decoder)
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

fn parse_resource_specifier(
    input: &str,
    decoder: &Bech32Decoder,
) -> Result<ResourceSpecifier, ParseResourceSpecifierError> {
    let tokens: Vec<&str> = input.trim().split(',').map(|s| s.trim()).collect();

    // check length
    if tokens.len() < 2 {
        return Err(ParseResourceSpecifierError::IncompleteResourceSpecifier);
    }

    // parse resource address
    let token = tokens[tokens.len() - 1];
    let resource_address = decoder
        .validate_and_decode_resource_address(token)
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
