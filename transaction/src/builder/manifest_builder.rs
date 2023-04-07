use radix_engine_interface::api::node_modules::metadata::MetadataEntry;
use radix_engine_interface::blueprints::access_controller::{
    RuleSet, ACCESS_CONTROLLER_BLUEPRINT, ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT,
};
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::epoch_manager::{
    EpochManagerCreateValidatorInput, EPOCH_MANAGER_CREATE_VALIDATOR_IDENT,
    VALIDATOR_CLAIM_XRD_IDENT, VALIDATOR_REGISTER_IDENT, VALIDATOR_STAKE_IDENT,
    VALIDATOR_UNREGISTER_IDENT, VALIDATOR_UNSTAKE_IDENT,
};
use radix_engine_interface::blueprints::identity::{
    IdentityCreateAdvancedInput, IdentityCreateInput, IDENTITY_BLUEPRINT,
    IDENTITY_CREATE_ADVANCED_IDENT, IDENTITY_CREATE_IDENT,
};
use radix_engine_interface::blueprints::resource::ResourceMethodAuthKey::{Burn, Mint};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::{
    ACCESS_CONTROLLER_PACKAGE, ACCOUNT_PACKAGE, EPOCH_MANAGER, IDENTITY_PACKAGE,
    RESOURCE_MANAGER_PACKAGE,
};
use radix_engine_interface::crypto::{hash, EcdsaSecp256k1PublicKey, Hash};
#[cfg(feature = "dump_manifest_to_file")]
use radix_engine_interface::data::manifest::manifest_encode;
use radix_engine_interface::data::manifest::{
    model::*, to_manifest_value, ManifestEncode, ManifestValue,
};
use radix_engine_interface::data::scrypto::{model::*, scrypto_encode};
use radix_engine_interface::math::*;
use radix_engine_interface::schema::PackageSchema;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::borrow::ToOwned;
use sbor::rust::collections::*;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;

use crate::model::*;
use crate::validation::*;

/// Utility for building transaction manifest.
pub struct ManifestBuilder {
    /// ID validator for calculating transaction object id
    id_allocator: ManifestIdAllocator,
    /// Instructions generated.
    instructions: Vec<Instruction>,
    /// Blobs
    blobs: BTreeMap<Hash, Vec<u8>>,
}

impl ManifestBuilder {
    /// Starts a new transaction builder.
    pub fn new() -> Self {
        Self {
            id_allocator: ManifestIdAllocator::new(),
            instructions: Vec::new(),
            blobs: BTreeMap::default(),
        }
    }

    /// Adds a raw instruction.
    pub fn add_instruction(
        &mut self,
        inst: Instruction,
    ) -> (&mut Self, Option<ManifestBucket>, Option<ManifestProof>) {
        let mut new_bucket_id: Option<ManifestBucket> = None;
        let mut new_proof_id: Option<ManifestProof> = None;

        match &inst {
            Instruction::TakeFromWorktop { .. }
            | Instruction::TakeFromWorktopByAmount { .. }
            | Instruction::TakeFromWorktopByIds { .. } => {
                new_bucket_id = Some(self.id_allocator.new_bucket_id().unwrap());
            }
            Instruction::PopFromAuthZone { .. }
            | Instruction::CreateProofFromAuthZone { .. }
            | Instruction::CreateProofFromAuthZoneByAmount { .. }
            | Instruction::CreateProofFromAuthZoneByIds { .. }
            | Instruction::CreateProofFromBucket { .. }
            | Instruction::CloneProof { .. } => {
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
        F: FnOnce(&mut Self, ManifestBucket) -> &mut Self,
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
        F: FnOnce(&mut Self, ManifestBucket) -> &mut Self,
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
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestBucket) -> &mut Self,
    {
        let (builder, bucket_id, _) = self.add_instruction(Instruction::TakeFromWorktopByIds {
            ids: ids.clone(),
            resource_address,
        });
        then(builder, bucket_id.unwrap())
    }

    /// Adds a bucket of resource to worktop.
    pub fn return_to_worktop(&mut self, bucket_id: ManifestBucket) -> &mut Self {
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
        ids: &BTreeSet<NonFungibleLocalId>,
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
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) = self.add_instruction(Instruction::PopFromAuthZone {});
        then(builder, proof_id.unwrap())
    }

    /// Pushes a proof onto the auth zone
    pub fn push_to_auth_zone(&mut self, proof_id: ManifestProof) -> &mut Self {
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
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
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
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
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
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(Instruction::CreateProofFromAuthZoneByIds {
                ids: ids.clone(),
                resource_address,
            });
        then(builder, proof_id.unwrap())
    }

    /// Creates proof from a bucket.
    pub fn create_proof_from_bucket<F>(&mut self, bucket_id: &ManifestBucket, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) = self.add_instruction(Instruction::CreateProofFromBucket {
            bucket_id: bucket_id.clone(),
        });
        then(builder, proof_id.unwrap())
    }

    /// Clones a proof.
    pub fn clone_proof<F>(&mut self, proof_id: &ManifestProof, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) = self.add_instruction(Instruction::CloneProof {
            proof_id: proof_id.clone(),
        });
        then(builder, proof_id.unwrap())
    }

    /// Drops a proof.
    pub fn drop_proof(&mut self, proof_id: ManifestProof) -> &mut Self {
        self.add_instruction(Instruction::DropProof { proof_id }).0
    }

    /// Drops all proofs.
    pub fn drop_all_proofs(&mut self) -> &mut Self {
        self.add_instruction(Instruction::DropAllProofs).0
    }

    /// Drops all virtual proofs.
    pub fn clear_signature_proofs(&mut self) -> &mut Self {
        self.add_instruction(Instruction::ClearSignatureProofs).0
    }

    /// Creates a fungible resource
    pub fn create_fungible_resource<R: Into<AccessRule>>(
        &mut self,
        divisibility: u8,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, R)>,
        initial_supply: Option<Decimal>,
    ) -> &mut Self {
        let access_rules = access_rules
            .into_iter()
            .map(|(k, v)| (k, (v.0, v.1.into())))
            .collect();
        if let Some(initial_supply) = initial_supply {
            self.add_instruction(Instruction::CallFunction {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value(&FungibleResourceManagerCreateWithInitialSupplyInput {
                    divisibility,
                    metadata,
                    access_rules,
                    initial_supply,
                }),
            });
        } else {
            self.add_instruction(Instruction::CallFunction {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                args: to_manifest_value(&FungibleResourceManagerCreateInput {
                    divisibility,
                    metadata,
                    access_rules,
                }),
            });
        }

        self
    }

    /// Creates a new non-fungible resource
    pub fn create_non_fungible_resource<R, T, V>(
        &mut self,
        id_type: NonFungibleIdType,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, R)>,
        initial_supply: Option<T>,
    ) -> &mut Self
    where
        R: Into<AccessRule>,
        T: IntoIterator<Item = (NonFungibleLocalId, V)>,
        V: ManifestEncode + NonFungibleData,
    {
        let access_rules = access_rules
            .into_iter()
            .map(|(k, v)| (k, (v.0, v.1.into())))
            .collect();

        if let Some(initial_supply) = initial_supply {
            let entries = initial_supply
                .into_iter()
                .map(|(id, e)| (id, (to_manifest_value(&e),)))
                .collect();

            self.add_instruction(Instruction::CallFunction {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value(
                    &NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
                        id_type,
                        non_fungible_schema: NonFungibleDataSchema::new_schema::<V>(),
                        metadata,
                        access_rules,
                        entries,
                    },
                ),
            });
        } else {
            self.add_instruction(Instruction::CallFunction {
                package_address: RESOURCE_MANAGER_PACKAGE,
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                args: to_manifest_value(&NonFungibleResourceManagerCreateInput {
                    id_type,
                    non_fungible_schema: NonFungibleDataSchema::new_schema::<V>(),
                    metadata,
                    access_rules,
                }),
            });
        }

        self
    }

    pub fn create_identity_advanced(&mut self, config: AccessRulesConfig) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address: IDENTITY_PACKAGE,
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
            args: to_manifest_value(&IdentityCreateAdvancedInput { config }),
        });
        self
    }

    pub fn create_identity(&mut self) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address: IDENTITY_PACKAGE,
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_IDENT.to_string(),
            args: to_manifest_value(&IdentityCreateInput {}),
        });
        self
    }

    pub fn create_validator(&mut self, key: EcdsaSecp256k1PublicKey) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address: EPOCH_MANAGER,
            method_name: EPOCH_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
            args: to_manifest_value(&EpochManagerCreateValidatorInput { key }),
        });
        self
    }

    pub fn register_validator(&mut self, validator_address: ComponentAddress) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address: validator_address,
            method_name: VALIDATOR_REGISTER_IDENT.to_string(),
            args: manifest_args!(),
        });
        self
    }

    pub fn unregister_validator(&mut self, validator_address: ComponentAddress) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address: validator_address,
            method_name: VALIDATOR_UNREGISTER_IDENT.to_string(),
            args: manifest_args!(),
        });
        self
    }

    pub fn stake_validator(
        &mut self,
        validator_address: ComponentAddress,
        bucket: ManifestBucket,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address: validator_address,
            method_name: VALIDATOR_STAKE_IDENT.to_string(),
            args: manifest_args!(bucket),
        });
        self
    }

    pub fn unstake_validator(
        &mut self,
        validator_address: ComponentAddress,
        bucket: ManifestBucket,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address: validator_address,
            method_name: VALIDATOR_UNSTAKE_IDENT.to_string(),
            args: manifest_args!(bucket),
        });
        self
    }

    pub fn claim_xrd(
        &mut self,
        validator_address: ComponentAddress,
        bucket: ManifestBucket,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address: validator_address,
            method_name: VALIDATOR_CLAIM_XRD_IDENT.to_string(),
            args: manifest_args!(bucket),
        });
        self
    }

    /// Calls a function where the arguments should be an array of encoded Scrypto value.
    pub fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: ManifestValue,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address,
            blueprint_name: blueprint_name.to_string(),
            function_name: function_name.to_string(),
            args: to_manifest_value(&args),
        });
        self
    }

    /// Calls a scrypto method where the arguments should be an array of encoded Scrypto value.
    pub fn call_method(
        &mut self,
        component_address: ComponentAddress,
        method_name: &str,
        args: ManifestValue,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallMethod {
            component_address,
            method_name: method_name.to_owned(),
            args: args,
        });
        self
    }

    pub fn set_package_royalty_config(
        &mut self,
        package_address: PackageAddress,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
    ) -> &mut Self {
        self.add_instruction(Instruction::SetPackageRoyaltyConfig {
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
        self.add_instruction(Instruction::SetComponentRoyaltyConfig {
            component_address,
            royalty_config,
        })
        .0
    }

    pub fn claim_package_royalty(&mut self, package_address: PackageAddress) -> &mut Self {
        self.add_instruction(Instruction::ClaimPackageRoyalty { package_address })
            .0
    }

    pub fn claim_component_royalty(&mut self, component_address: ComponentAddress) -> &mut Self {
        self.add_instruction(Instruction::ClaimComponentRoyalty { component_address })
            .0
    }

    pub fn set_method_access_rule(
        &mut self,
        entity_address: GlobalAddress,
        key: MethodKey,
        rule: AccessRule,
    ) -> &mut Self {
        self.add_instruction(Instruction::SetMethodAccessRule {
            entity_address,
            key,
            rule,
        })
        .0
    }

    pub fn set_metadata(
        &mut self,
        entity_address: GlobalAddress,
        key: String,
        value: MetadataEntry,
    ) -> &mut Self {
        self.add_instruction(Instruction::SetMetadata {
            entity_address,
            key,
            value,
        })
        .0
    }

    /// Publishes a package.
    pub fn publish_package_advanced(
        &mut self,
        code: Vec<u8>,
        schema: PackageSchema,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
        access_rules: AccessRulesConfig,
    ) -> &mut Self {
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        let schema = scrypto_encode(&schema).unwrap();
        let schema_hash = hash(&schema);
        self.blobs.insert(schema_hash, schema);

        self.add_instruction(Instruction::PublishPackageAdvanced {
            code: ManifestBlobRef(code_hash.0),
            schema: ManifestBlobRef(schema_hash.0),
            royalty_config,
            metadata,
            access_rules,
        });
        self
    }

    /// Publishes a package with an owner badge.
    pub fn publish_package(&mut self, code: Vec<u8>, schema: PackageSchema) -> &mut Self {
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        let schema = scrypto_encode(&schema).unwrap();
        let schema_hash = hash(&schema);
        self.blobs.insert(schema_hash, schema);

        self.add_instruction(Instruction::PublishPackage {
            code: ManifestBlobRef(code_hash.0),
            schema: ManifestBlobRef(schema_hash.0),
            royalty_config: BTreeMap::new(),
            metadata: BTreeMap::new(),
        });
        self
    }

    /// Publishes a package with an owner badge.
    pub fn publish_package_with_owner(
        &mut self,
        code: Vec<u8>,
        schema: PackageSchema,
        owner_badge: NonFungibleGlobalId,
    ) -> &mut Self {
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        let schema = scrypto_encode(&schema).unwrap();
        let schema_hash = hash(&schema);
        self.blobs.insert(schema_hash, schema);

        self.add_instruction(Instruction::PublishPackageAdvanced {
            code: ManifestBlobRef(code_hash.0),
            schema: ManifestBlobRef(schema_hash.0),
            royalty_config: BTreeMap::new(),
            metadata: BTreeMap::new(),
            access_rules: package_access_rules_from_owner_badge(&owner_badge),
        });
        self
    }

    /// Builds a transaction manifest.
    /// TODO: consider using self
    pub fn build(&self) -> TransactionManifest {
        let m = TransactionManifest {
            instructions: self.instructions.clone(),
            blobs: self.blobs.values().cloned().collect(),
        };
        #[cfg(feature = "dump_manifest_to_file")]
        {
            let bytes = manifest_encode(&m).unwrap();
            let m_hash = hash(&bytes);
            let path = format!("manifest_{:?}.raw", m_hash);
            std::fs::write(&path, bytes).unwrap();
            println!("manifest dumped to file {}", &path);
        }
        m
    }

    /// Creates a token resource with mutable supply.
    pub fn new_token_mutable(
        &mut self,
        metadata: BTreeMap<String, String>,
        minter_rule: AccessRule,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceMethodAuthKey::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );
        access_rules.insert(Mint, (minter_rule.clone(), rule!(deny_all)));
        access_rules.insert(Burn, (minter_rule.clone(), rule!(deny_all)));

        let initial_supply = Option::None;
        self.create_fungible_resource(18, metadata, access_rules, initial_supply)
    }

    /// Creates a token resource with fixed supply.
    pub fn new_token_fixed(
        &mut self,
        metadata: BTreeMap<String, String>,
        initial_supply: Decimal,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceMethodAuthKey::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );

        self.create_fungible_resource(18, metadata, access_rules, Some(initial_supply))
    }

    /// Creates a badge resource with mutable supply.
    pub fn new_badge_mutable(
        &mut self,
        metadata: BTreeMap<String, String>,
        minter_rule: AccessRule,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceMethodAuthKey::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );
        access_rules.insert(Mint, (minter_rule.clone(), rule!(deny_all)));
        access_rules.insert(Burn, (minter_rule.clone(), rule!(deny_all)));

        let initial_supply = Option::None;
        self.create_fungible_resource(0, metadata, access_rules, initial_supply)
    }

    /// Creates a badge resource with fixed supply.
    pub fn new_badge_fixed(
        &mut self,
        metadata: BTreeMap<String, String>,
        initial_supply: Decimal,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceMethodAuthKey::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );

        self.create_fungible_resource(0, metadata, access_rules, Some(initial_supply))
    }

    pub fn burn_from_worktop(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        self.take_from_worktop_by_amount(amount, resource_address, |builder, bucket_id| {
            builder
                .add_instruction(Instruction::BurnResource { bucket_id })
                .0
        })
    }

    pub fn burn_all_from_worktop(&mut self, resource_address: ResourceAddress) -> &mut Self {
        self.take_from_worktop(resource_address, |builder, bucket_id| {
            builder
                .add_instruction(Instruction::BurnResource { bucket_id })
                .0
        })
    }

    pub fn mint_fungible(
        &mut self,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> &mut Self {
        self.add_instruction(Instruction::MintFungible {
            resource_address,
            amount,
        });
        self
    }

    pub fn mint_non_fungible<T, V>(
        &mut self,
        resource_address: ResourceAddress,
        entries: T,
    ) -> &mut Self
    where
        T: IntoIterator<Item = (NonFungibleLocalId, V)>,
        V: ManifestEncode,
    {
        let entries = entries
            .into_iter()
            .map(|(id, e)| (id, (to_manifest_value(&e),)))
            .collect();
        let input = NonFungibleResourceManagerMintManifestInput { entries };

        self.add_instruction(Instruction::MintNonFungible {
            resource_address,
            args: to_manifest_value(&input),
        });
        self
    }

    pub fn mint_uuid_non_fungible<T, V>(
        &mut self,
        resource_address: ResourceAddress,
        entries: T,
    ) -> &mut Self
    where
        T: IntoIterator<Item = V>,
        V: ManifestEncode,
    {
        let entries = entries
            .into_iter()
            .map(|e| (to_manifest_value(&e),))
            .collect();
        let input = NonFungibleResourceManagerMintUuidManifestInput { entries };

        self.add_instruction(Instruction::MintUuidNonFungible {
            resource_address,
            args: to_manifest_value(&input),
        });
        self
    }

    pub fn recall(&mut self, vault_id: LocalAddress, amount: Decimal) -> &mut Self {
        self.add_instruction(Instruction::RecallResource { vault_id, amount });
        self
    }

    pub fn burn_non_fungible(&mut self, non_fungible_global_id: NonFungibleGlobalId) -> &mut Self {
        let mut ids = BTreeSet::new();
        ids.insert(non_fungible_global_id.local_id().clone());
        self.take_from_worktop_by_ids(
            &ids,
            non_fungible_global_id.resource_address().clone(),
            |builder, bucket_id| {
                builder
                    .add_instruction(Instruction::BurnResource { bucket_id })
                    .0
            },
        )
    }

    /// Creates an account.
    pub fn new_account_advanced(&mut self, config: AccessRulesConfig) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
            function_name: ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
            args: to_manifest_value(&AccountCreateAdvancedInput { config }),
        })
        .0
    }

    pub fn lock_fee_and_withdraw(
        &mut self,
        account: ComponentAddress,
        amount_to_lock: Decimal,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> &mut Self {
        let args = to_manifest_value(&AccountLockFeeAndWithdrawInput {
            resource_address,
            amount,
            amount_to_lock,
        });

        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            method_name: ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT.to_string(),
            args: args,
        })
        .0
    }

    pub fn lock_fee_and_withdraw_non_fungibles(
        &mut self,
        account: ComponentAddress,
        amount_to_lock: Decimal,
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
    ) -> &mut Self {
        let args = to_manifest_value(&AccountLockFeeAndWithdrawNonFungiblesInput {
            amount_to_lock,
            resource_address,
            ids,
        });

        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            method_name: ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            args,
        })
        .0
    }

    /// Locks a fee from the XRD vault of an account.
    pub fn lock_fee(&mut self, account: ComponentAddress, amount: Decimal) -> &mut Self {
        let args = to_manifest_value(&AccountLockFeeInput { amount });

        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            method_name: ACCOUNT_LOCK_FEE_IDENT.to_string(),
            args,
        })
        .0
    }

    pub fn lock_contingent_fee(&mut self, account: ComponentAddress, amount: Decimal) -> &mut Self {
        let args = to_manifest_value(&AccountLockContingentFeeInput { amount });

        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            method_name: ACCOUNT_LOCK_CONTINGENT_FEE_IDENT.to_string(),
            args,
        })
        .0
    }

    /// Withdraws resource from an account.
    pub fn withdraw_from_account(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> &mut Self {
        let args = to_manifest_value(&AccountWithdrawInput {
            resource_address,
            amount,
        });

        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            method_name: ACCOUNT_WITHDRAW_IDENT.to_string(),
            args,
        })
        .0
    }

    /// Withdraws resource from an account.
    pub fn withdraw_non_fungibles_from_account(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> &mut Self {
        let args = to_manifest_value(&AccountWithdrawNonFungiblesInput {
            ids: ids.clone(),
            resource_address,
        });

        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            method_name: ACCOUNT_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            args,
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
    ) -> &mut Self {
        let args = to_manifest_value(&AccountCreateProofInput { resource_address });

        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            method_name: ACCOUNT_CREATE_PROOF_IDENT.to_string(),
            args,
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_by_amount(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> &mut Self {
        let args = to_manifest_value(&AccountCreateProofByAmountInput {
            resource_address,
            amount,
        });

        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            method_name: ACCOUNT_CREATE_PROOF_BY_AMOUNT_IDENT.to_string(),
            args,
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_by_ids(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> &mut Self {
        let args = to_manifest_value(&AccountCreateProofByIdsInput {
            resource_address,
            ids: ids.clone(),
        });

        self.add_instruction(Instruction::CallMethod {
            component_address: account,
            method_name: ACCOUNT_CREATE_PROOF_BY_IDS_IDENT.to_string(),
            args,
        })
        .0
    }

    pub fn create_access_controller(
        &mut self,
        controlled_asset: ManifestBucket,
        primary_role: AccessRule,
        recovery_role: AccessRule,
        confirmation_role: AccessRule,
        timed_recovery_delay_in_minutes: Option<u32>,
    ) -> &mut Self {
        self.add_instruction(Instruction::CallFunction {
            package_address: ACCESS_CONTROLLER_PACKAGE,
            blueprint_name: ACCESS_CONTROLLER_BLUEPRINT.to_string(),
            function_name: ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string(),
            args: manifest_args!(
                controlled_asset,
                RuleSet {
                    primary_role,
                    recovery_role,
                    confirmation_role,
                },
                timed_recovery_delay_in_minutes
            ),
        });
        self
    }

    pub fn borrow_mut<F, E>(&mut self, handler: F) -> Result<&mut Self, E>
    where
        F: FnOnce(&mut Self) -> Result<&mut Self, E>,
    {
        handler(self)
    }
}
