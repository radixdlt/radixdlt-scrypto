use radix_engine_interface::abi;
use radix_engine_interface::abi::*;
use radix_engine_interface::address::Bech32Decoder;
use radix_engine_interface::api::types::{BucketId, GlobalAddress, ProofId};
use radix_engine_interface::constants::*;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::crypto::{hash, Blob, Hash};
use radix_engine_interface::data::*;
use radix_engine_interface::math::{Decimal, PreciseDecimal};
use radix_engine_interface::model::*;
use radix_engine_interface::*;
use sbor::rust::borrow::ToOwned;
use sbor::rust::collections::*;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;

use crate::errors::*;
use crate::model::*;
use crate::validation::*;

#[macro_export]
macro_rules! args_from_bytes_vec {
    ($args: expr) => {{
        let mut fields = Vec::new();
        for arg in $args {
            fields.push(::radix_engine_interface::data::scrypto_decode(&arg).unwrap());
        }
        let input_struct = ::radix_engine_interface::data::ScryptoValue::Tuple { fields };
        ::radix_engine_interface::data::scrypto_encode(&input_struct).unwrap()
    }};
}

/// Utility for building transaction manifest.
pub struct ManifestBuilder {
    /// The decoder used by the manifest (mainly for the `call_*_with_abi)
    decoder: Bech32Decoder,
    /// ID validator for calculating transaction object id
    id_allocator: IdAllocator,
    /// Instructions generated.
    instructions: Vec<BasicInstruction>,
    /// Blobs
    blobs: BTreeMap<Hash, Vec<u8>>,
}

impl ManifestBuilder {
    /// Starts a new transaction builder.
    pub fn new(network: &NetworkDefinition) -> Self {
        Self {
            decoder: Bech32Decoder::new(network),
            id_allocator: IdAllocator::new(IdSpace::Transaction),
            instructions: Vec::new(),
            blobs: BTreeMap::default(),
        }
    }

    /// Adds a raw instruction.
    pub fn add_instruction(
        &mut self,
        inst: BasicInstruction,
    ) -> (&mut Self, Option<BucketId>, Option<ProofId>) {
        let mut new_bucket_id: Option<BucketId> = None;
        let mut new_proof_id: Option<ProofId> = None;

        match &inst {
            BasicInstruction::TakeFromWorktop { .. }
            | BasicInstruction::TakeFromWorktopByAmount { .. }
            | BasicInstruction::TakeFromWorktopByIds { .. } => {
                new_bucket_id = Some(self.id_allocator.new_bucket_id().unwrap());
            }
            BasicInstruction::PopFromAuthZone { .. }
            | BasicInstruction::CreateProofFromAuthZone { .. }
            | BasicInstruction::CreateProofFromAuthZoneByAmount { .. }
            | BasicInstruction::CreateProofFromAuthZoneByIds { .. }
            | BasicInstruction::CreateProofFromBucket { .. }
            | BasicInstruction::CloneProof { .. } => {
                new_proof_id = Some(self.id_allocator.new_proof_id().unwrap());
            }
            _ => {}
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
            self.add_instruction(BasicInstruction::TakeFromWorktop { resource_address });
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
        let (builder, bucket_id, _) =
            self.add_instruction(BasicInstruction::TakeFromWorktopByAmount {
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
        let (builder, bucket_id, _) =
            self.add_instruction(BasicInstruction::TakeFromWorktopByIds {
                ids: ids.clone(),
                resource_address,
            });
        then(builder, bucket_id.unwrap())
    }

    /// Adds a bucket of resource to worktop.
    pub fn return_to_worktop(&mut self, bucket_id: BucketId) -> &mut Self {
        self.add_instruction(BasicInstruction::ReturnToWorktop { bucket_id })
            .0
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains(&mut self, resource_address: ResourceAddress) -> &mut Self {
        self.add_instruction(BasicInstruction::AssertWorktopContains { resource_address })
            .0
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains_by_amount(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::AssertWorktopContainsByAmount {
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
        self.add_instruction(BasicInstruction::AssertWorktopContainsByIds {
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
        let (builder, _, proof_id) = self.add_instruction(BasicInstruction::PopFromAuthZone {});
        then(builder, proof_id.unwrap())
    }

    /// Pushes a proof onto the auth zone
    pub fn push_to_auth_zone(&mut self, proof_id: ProofId) -> &mut Self {
        self.add_instruction(BasicInstruction::PushToAuthZone { proof_id });
        self
    }

    /// Clears the auth zone.
    pub fn clear_auth_zone(&mut self) -> &mut Self {
        self.add_instruction(BasicInstruction::ClearAuthZone).0
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
            self.add_instruction(BasicInstruction::CreateProofFromAuthZone { resource_address });
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
            self.add_instruction(BasicInstruction::CreateProofFromAuthZoneByAmount {
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
            self.add_instruction(BasicInstruction::CreateProofFromAuthZoneByIds {
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
            self.add_instruction(BasicInstruction::CreateProofFromBucket { bucket_id });
        then(builder, proof_id.unwrap())
    }

    /// Clones a proof.
    pub fn clone_proof<F>(&mut self, proof_id: ProofId, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ProofId) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(BasicInstruction::CloneProof { proof_id });
        then(builder, proof_id.unwrap())
    }

    /// Drops a proof.
    pub fn drop_proof(&mut self, proof_id: ProofId) -> &mut Self {
        self.add_instruction(BasicInstruction::DropProof { proof_id })
            .0
    }

    /// Drops all proofs.
    pub fn drop_all_proofs(&mut self) -> &mut Self {
        self.add_instruction(BasicInstruction::DropAllProofs).0
    }

    pub fn create_resource(
        &mut self,
        resource_type: ResourceType,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
        mint_params: Option<MintParams>,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CreateResource {
            resource_type,
            metadata,
            access_rules,
            mint_params,
        });

        self
    }

    /// Calls a function where the arguments should be an array of encoded Scrypto value.
    pub fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallFunction {
            package_address,
            blueprint_name: blueprint_name.to_string(),
            function_name: function_name.to_string(),
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
            fields.push(scrypto_decode(&arg).unwrap());
        }
        let input_struct = ScryptoValue::Tuple { fields };
        let bytes = scrypto_encode(&input_struct).unwrap();

        Ok(self
            .add_instruction(BasicInstruction::CallFunction {
                package_address,
                blueprint_name: blueprint_name.to_string(),
                function_name: function.to_string(),
                args: bytes,
            })
            .0)
    }

    /// Calls a scrypto method where the arguments should be an array of encoded Scrypto value.
    pub fn call_method(
        &mut self,
        component_address: ComponentAddress,
        method_name: &str,
        args: Vec<u8>,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address,
            method_name: method_name.to_owned(),
            args,
        });
        self
    }

    pub fn set_package_royalty_config(
        &mut self,
        package_address: PackageAddress,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::SetPackageRoyaltyConfig {
            package_address,
            royalty_config,
        })
        .0
    }

    pub fn set_component_royalty_config(
        &mut self,
        component_address: ComponentAddress,
        royalty_config: RoyaltyConfig,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::SetComponentRoyaltyConfig {
            component_address,
            royalty_config,
        })
        .0
    }

    pub fn claim_package_royalty(&mut self, package_address: PackageAddress) -> &mut Self {
        self.add_instruction(BasicInstruction::ClaimPackageRoyalty { package_address })
            .0
    }

    pub fn claim_component_royalty(&mut self, component_address: ComponentAddress) -> &mut Self {
        self.add_instruction(BasicInstruction::ClaimComponentRoyalty { component_address })
            .0
    }

    pub fn set_metadata(
        &mut self,
        entity_address: GlobalAddress,
        key: String,
        value: String,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::SetMetadata {
            entity_address,
            key,
            value,
        })
        .0
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
        method_name: &str,
        args: Vec<String>,
        account: Option<ComponentAddress>,
        blueprint_abi: &abi::BlueprintAbi,
    ) -> Result<&mut Self, BuildCallWithAbiError> {
        let abi = blueprint_abi
            .fns
            .iter()
            .find(|m| m.ident == method_name)
            .map(Clone::clone)
            .ok_or_else(|| BuildCallWithAbiError::MethodNotFound(method_name.to_owned()))?;

        let arguments = self
            .parse_args(&abi.input, args, account)
            .map_err(|e| BuildCallWithAbiError::FailedToBuildArgs(e))?;

        Ok(self
            .add_instruction(BasicInstruction::CallMethod {
                component_address,
                method_name: method_name.to_owned(),
                args: args_from_bytes_vec!(arguments),
            })
            .0)
    }

    /// Publishes a package.
    pub fn publish_package(
        &mut self,
        code: Vec<u8>,
        abi: BTreeMap<String, BlueprintAbi>,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
        access_rules: AccessRules,
    ) -> &mut Self {
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        let abi = scrypto_encode(&abi).unwrap();
        let abi_hash = hash(&abi);
        self.blobs.insert(abi_hash, abi);

        self.add_instruction(BasicInstruction::PublishPackage {
            code: Blob(code_hash),
            abi: Blob(abi_hash),
            royalty_config,
            metadata,
            access_rules,
        })
        .0
    }

    /// Publishes a package.
    pub fn publish_package_with_owner(
        &mut self,
        code: Vec<u8>,
        abi: BTreeMap<String, BlueprintAbi>,
        owner_badge: NonFungibleAddress,
    ) -> &mut Self {
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        let abi = scrypto_encode(&abi).unwrap();
        let abi_hash = hash(&abi);
        self.blobs.insert(abi_hash, abi);

        self.add_instruction(BasicInstruction::PublishPackageWithOwner {
            code: Blob(code_hash),
            abi: Blob(abi_hash),
            owner_badge,
        })
        .0
    }

    /// Builds a transaction manifest.
    /// TODO: consider using self
    pub fn build(&self) -> TransactionManifest {
        TransactionManifest {
            instructions: self.instructions.clone(),
            blobs: self.blobs.values().cloned().collect(),
        }
    }

    /// Creates a token resource with mutable supply.
    pub fn new_token_mutable(
        &mut self,
        metadata: BTreeMap<String, String>,
        minter_resource_address: ResourceAddress,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(
            Mint,
            (rule!(require(minter_resource_address.clone())), LOCKED),
        );
        access_rules.insert(
            Burn,
            (rule!(require(minter_resource_address.clone())), LOCKED),
        );

        let mint_params: Option<MintParams> = Option::None;
        self.add_instruction(BasicInstruction::CreateResource {
            resource_type: ResourceType::Fungible { divisibility: 18 },
            metadata,
            access_rules,
            mint_params,
        })
        .0
    }

    /// Creates a token resource with fixed supply.
    pub fn new_token_fixed(
        &mut self,
        metadata: BTreeMap<String, String>,
        initial_supply: Decimal,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));

        self.add_instruction(BasicInstruction::CreateResource {
            resource_type: ResourceType::Fungible { divisibility: 18 },
            metadata,
            access_rules,
            mint_params: Option::Some(MintParams::Fungible {
                amount: initial_supply.into(),
            }),
        })
        .0
    }

    /// Creates a badge resource with mutable supply.
    pub fn new_badge_mutable(
        &mut self,
        metadata: BTreeMap<String, String>,
        minter_resource_address: ResourceAddress,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));
        access_rules.insert(
            Mint,
            (rule!(require(minter_resource_address.clone())), LOCKED),
        );
        access_rules.insert(
            Burn,
            (rule!(require(minter_resource_address.clone())), LOCKED),
        );

        let mint_params: Option<MintParams> = Option::None;

        self.add_instruction(BasicInstruction::CreateResource {
            resource_type: ResourceType::Fungible { divisibility: 0 },
            metadata,
            access_rules,
            mint_params,
        })
        .0
    }

    /// Creates a badge resource with fixed supply.
    pub fn new_badge_fixed(
        &mut self,
        metadata: BTreeMap<String, String>,
        initial_supply: Decimal,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(ResourceMethodAuthKey::Withdraw, (rule!(allow_all), LOCKED));

        self.add_instruction(BasicInstruction::CreateResource {
            resource_type: ResourceType::Fungible { divisibility: 0 },
            metadata,
            access_rules,
            mint_params: Option::Some(MintParams::Fungible {
                amount: initial_supply.into(),
            }),
        })
        .0
    }

    /// Mints resource.
    pub fn mint(&mut self, resource_address: ResourceAddress, amount: Decimal) -> &mut Self {
        self.add_instruction(BasicInstruction::MintFungible {
            resource_address,
            amount,
        });
        self
    }

    /// Burns a resource.
    pub fn burn(&mut self, resource_address: ResourceAddress, amount: Decimal) -> &mut Self {
        self.take_from_worktop_by_amount(amount, resource_address, |builder, bucket_id| {
            builder
                .add_instruction(BasicInstruction::BurnResource { bucket_id })
                .0
        })
    }

    pub fn burn_non_fungible(&mut self, non_fungible_address: NonFungibleAddress) -> &mut Self {
        let mut ids = BTreeSet::new();
        ids.insert(non_fungible_address.non_fungible_id().clone());
        self.take_from_worktop_by_ids(
            &ids,
            non_fungible_address.resource_address(),
            |builder, bucket_id| {
                builder
                    .add_instruction(BasicInstruction::BurnResource { bucket_id })
                    .0
            },
        )
    }

    /// Creates an account.
    pub fn new_account(&mut self, withdraw_auth: &AccessRuleNode) -> &mut Self {
        self.add_instruction(BasicInstruction::CallFunction {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: ACCOUNT_BLUEPRINT.to_owned(),
            function_name: "new".to_string(),
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
        self.add_instruction(BasicInstruction::CallFunction {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: ACCOUNT_BLUEPRINT.to_owned(),
            function_name: "new_with_resource".to_string(),
            args: args!(withdraw_auth.clone(), Bucket(bucket_id)),
        })
        .0
    }

    pub fn lock_fee_and_withdraw(
        &mut self,
        account: ComponentAddress,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "lock_fee_and_withdraw".to_string(),
            args: args!(amount, resource_address),
        })
        .0
    }

    pub fn lock_fee_and_withdraw_by_amount(
        &mut self,
        account: ComponentAddress,
        amount_to_lock: Decimal,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "lock_fee_and_withdraw_by_amount".to_string(),

            args: args!(amount_to_lock, amount, resource_address),
        })
        .0
    }

    pub fn lock_fee_and_withdraw_by_ids(
        &mut self,
        account: ComponentAddress,
        amount_to_lock: Decimal,
        ids: BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "lock_fee_and_withdraw_by_ids".to_string(),

            args: args!(amount_to_lock, ids, resource_address),
        })
        .0
    }

    /// Locks a fee from the XRD vault of an account.
    pub fn lock_fee(&mut self, account: ComponentAddress, amount: Decimal) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "lock_fee".to_string(),

            args: args!(amount),
        })
        .0
    }

    pub fn lock_contingent_fee(&mut self, account: ComponentAddress, amount: Decimal) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "lock_contingent_fee".to_string(),

            args: args!(amount),
        })
        .0
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "withdraw".to_string(),

            args: args!(resource_address),
        })
        .0
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account_by_amount(
        &mut self,
        account: ComponentAddress,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "withdraw_by_amount".to_string(),

            args: args!(amount, resource_address),
        })
        .0
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account_by_ids(
        &mut self,
        account: ComponentAddress,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "withdraw_by_ids".to_string(),

            args: args!(ids.clone(), resource_address),
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "create_proof".to_string(),

            args: args!(resource_address),
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_by_amount(
        &mut self,
        account: ComponentAddress,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "create_proof_by_amount".to_string(),

            args: args!(amount, resource_address),
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_by_ids(
        &mut self,
        account: ComponentAddress,
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.add_instruction(BasicInstruction::CallMethod {
            component_address: account,
            method_name: "create_proof_by_ids".to_string(),

            args: args!(ids.clone(), resource_address),
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_by_resource_specifier(
        &mut self,
        account: ComponentAddress,
        resource_specifier: String,
    ) -> Result<&mut Self, BuildArgsError> {
        let resource_specifier = parse_resource_specifier(&resource_specifier, &self.decoder)
            .map_err(|_| BuildArgsError::InvalidResourceSpecifier(resource_specifier))?;
        let builder = match resource_specifier {
            ResourceSpecifier::Amount(amount, resource_address) => {
                self.create_proof_from_account_by_amount(account, amount, resource_address)
            }
            ResourceSpecifier::Ids(non_fungible_ids, resource_address) => {
                self.create_proof_from_account_by_ids(account, &non_fungible_ids, resource_address)
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
            Type::Struct {
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
                        Type::Decimal => {
                            let value = arg.parse::<Decimal>().map_err(|_| {
                                BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                            })?;
                            Ok(scrypto_encode(&value).unwrap())
                        }
                        Type::PreciseDecimal => {
                            let value = arg.parse::<PreciseDecimal>().map_err(|_| {
                                BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                            })?;
                            Ok(scrypto_encode(&value).unwrap())
                        }
                        Type::PackageAddress => {
                            let value = self
                                .decoder
                                .validate_and_decode_package_address(arg)
                                .map_err(|_| {
                                    BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                                })?;
                            Ok(scrypto_encode(&value).unwrap())
                        }
                        Type::ComponentAddress => {
                            let value = self
                                .decoder
                                .validate_and_decode_component_address(arg)
                                .map_err(|_| {
                                    BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                                })?;
                            Ok(scrypto_encode(&value).unwrap())
                        }
                        Type::ResourceAddress => {
                            let value = self
                                .decoder
                                .validate_and_decode_resource_address(arg)
                                .map_err(|_| {
                                    BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                                })?;
                            Ok(scrypto_encode(&value).unwrap())
                        }
                        Type::Hash => {
                            let value = arg.parse::<Hash>().map_err(|_| {
                                BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                            })?;
                            Ok(scrypto_encode(&value).unwrap())
                        }
                        Type::NonFungibleId => {
                            let value = NonFungibleId::try_from_combined_simple_string(arg)
                                .map_err(|_| {
                                    BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                                })?;
                            Ok(scrypto_encode(&value).unwrap())
                        }
                        Type::NonFungibleAddress => {
                            let value = NonFungibleAddress::try_from_canonical_combined_string(
                                &self.decoder,
                                arg,
                            )
                            .map_err(|_| {
                                BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                            })?;
                            Ok(scrypto_encode(&value).unwrap())
                        }
                        Type::Bucket => {
                            let resource_specifier = parse_resource_specifier(arg, &self.decoder)
                                .map_err(|_| {
                                BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                            })?;
                            let bucket_id = match resource_specifier {
                                ResourceSpecifier::Amount(amount, resource_address) => {
                                    if let Some(account) = account {
                                        self.withdraw_from_account_by_amount(
                                            account,
                                            amount,
                                            resource_address,
                                        );
                                    }
                                    self.add_instruction(
                                        BasicInstruction::TakeFromWorktopByAmount {
                                            amount,
                                            resource_address,
                                        },
                                    )
                                    .1
                                    .unwrap()
                                }
                                ResourceSpecifier::Ids(ids, resource_address) => {
                                    if let Some(account) = account {
                                        self.withdraw_from_account_by_ids(
                                            account,
                                            &ids,
                                            resource_address,
                                        );
                                    }
                                    self.add_instruction(BasicInstruction::TakeFromWorktopByIds {
                                        ids,
                                        resource_address,
                                    })
                                    .1
                                    .unwrap()
                                }
                            };
                            Ok(scrypto_encode(&Bucket(bucket_id)).unwrap())
                        }
                        Type::Proof => {
                            let resource_specifier = parse_resource_specifier(arg, &self.decoder)
                                .map_err(|_| {
                                BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned())
                            })?;
                            let proof_id = match resource_specifier {
                                ResourceSpecifier::Amount(amount, resource_address) => {
                                    if let Some(account) = account {
                                        self.create_proof_from_account_by_amount(
                                            account,
                                            amount,
                                            resource_address,
                                        );
                                        self.add_instruction(BasicInstruction::PopFromAuthZone)
                                            .2
                                            .unwrap()
                                    } else {
                                        todo!("Take from worktop and create proof")
                                    }
                                }
                                ResourceSpecifier::Ids(ids, resource_address) => {
                                    if let Some(account) = account {
                                        self.create_proof_from_account_by_ids(
                                            account,
                                            &ids,
                                            resource_address,
                                        );
                                        self.add_instruction(BasicInstruction::PopFromAuthZone)
                                            .2
                                            .unwrap()
                                    } else {
                                        todo!("Take from worktop and create proof")
                                    }
                                }
                            };
                            Ok(scrypto_encode(&Proof(proof_id)).unwrap())
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
        t: &Type,
        arg: &str,
    ) -> Result<Vec<u8>, BuildArgsError>
    where
        T: FromStr + ScryptoEncode,
        T::Err: fmt::Debug,
    {
        let value = arg
            .parse::<T>()
            .map_err(|_| BuildArgsError::FailedToParse(i, t.clone(), arg.to_owned()))?;
        Ok(scrypto_encode(&value).unwrap())
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
    let resource_address_token = tokens[tokens.len() - 1];
    let resource_address = decoder
        .validate_and_decode_resource_address(resource_address_token)
        .map_err(|_| {
            ParseResourceSpecifierError::InvalidResourceAddress(resource_address_token.to_owned())
        })?;

    // parse non-fungible ids or amount
    if tokens[0].starts_with('#') {
        let mut ids = BTreeSet::<NonFungibleId>::new();
        for id in &tokens[..tokens.len() - 1] {
            if !id.starts_with('#') {
                return Err(ParseResourceSpecifierError::InvalidNonFungibleId(
                    id.to_string(),
                ));
            }
            ids.insert(
                NonFungibleId::try_from_combined_simple_string(&id[1..]).map_err(|_| {
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
