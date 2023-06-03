use radix_engine_common::native_addresses::PACKAGE_PACKAGE;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::node_modules::metadata::{
    MetadataSetInput, MetadataValue, METADATA_SET_IDENT,
};
use radix_engine_interface::api::node_modules::royalty::{
    ComponentClaimRoyaltyInput, ComponentSetRoyaltyConfigInput,
    COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT, COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::blueprints::access_controller::{
    RuleSet, ACCESS_CONTROLLER_BLUEPRINT, ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT,
};
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerCreateValidatorInput, CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT,
    VALIDATOR_CLAIM_XRD_IDENT, VALIDATOR_REGISTER_IDENT, VALIDATOR_STAKE_IDENT,
    VALIDATOR_UNREGISTER_IDENT, VALIDATOR_UNSTAKE_IDENT,
};
use radix_engine_interface::blueprints::identity::{
    IdentityCreateAdvancedInput, IdentityCreateInput, IDENTITY_BLUEPRINT,
    IDENTITY_CREATE_ADVANCED_IDENT, IDENTITY_CREATE_IDENT,
};
use radix_engine_interface::blueprints::package::{
    PackageClaimRoyaltyInput, PackageSetup, PackagePublishWasmAdvancedManifestInput,
    PackagePublishWasmManifestInput, PackageSetRoyaltyConfigInput, PACKAGE_BLUEPRINT,
    PACKAGE_CLAIM_ROYALTY_IDENT, PACKAGE_PUBLISH_WASM_ADVANCED_IDENT, PACKAGE_PUBLISH_WASM_IDENT,
    PACKAGE_SET_ROYALTY_CONFIG_IDENT,
};
use radix_engine_interface::blueprints::resource::ResourceMethodAuthKey::{Burn, Mint};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::{
    ACCESS_CONTROLLER_PACKAGE, ACCOUNT_PACKAGE, CONSENSUS_MANAGER, IDENTITY_PACKAGE,
    RESOURCE_PACKAGE,
};
use radix_engine_interface::crypto::{hash, EcdsaSecp256k1PublicKey, Hash};
#[cfg(feature = "dump_manifest_to_file")]
use radix_engine_interface::data::manifest::manifest_encode;
use radix_engine_interface::data::manifest::{
    model::*, to_manifest_value, ManifestEncode, ManifestValue,
};
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::math::*;
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
    instructions: Vec<InstructionV1>,
    /// Blobs
    blobs: BTreeMap<Hash, Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor)]
pub struct TransactionManifestV1 {
    pub instructions: Vec<InstructionV1>,
    pub blobs: BTreeMap<Hash, Vec<u8>>,
}

impl TransactionManifestV1 {
    pub fn for_intent(self) -> (InstructionsV1, BlobsV1) {
        (
            InstructionsV1(self.instructions),
            BlobsV1 {
                blobs: self
                    .blobs
                    .into_values()
                    .into_iter()
                    .map(|blob| BlobV1(blob))
                    .collect(),
            },
        )
    }
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
        inst: InstructionV1,
    ) -> (&mut Self, Option<ManifestBucket>, Option<ManifestProof>) {
        let mut new_bucket_id: Option<ManifestBucket> = None;
        let mut new_proof_id: Option<ManifestProof> = None;

        match &inst {
            InstructionV1::TakeAllFromWorktop { .. }
            | InstructionV1::TakeFromWorktop { .. }
            | InstructionV1::TakeNonFungiblesFromWorktop { .. } => {
                new_bucket_id = Some(self.id_allocator.new_bucket_id().unwrap());
            }
            InstructionV1::PopFromAuthZone { .. }
            | InstructionV1::CreateProofFromAuthZone { .. }
            | InstructionV1::CreateProofFromAuthZoneOfAmount { .. }
            | InstructionV1::CreateProofFromAuthZoneOfNonFungibles { .. }
            | InstructionV1::CreateProofFromAuthZoneOfAll { .. }
            | InstructionV1::CreateProofFromBucket { .. }
            | InstructionV1::CreateProofFromBucketOfAmount { .. }
            | InstructionV1::CreateProofFromBucketOfNonFungibles { .. }
            | InstructionV1::CreateProofFromBucketOfAll { .. }
            | InstructionV1::CloneProof { .. } => {
                new_proof_id = Some(self.id_allocator.new_proof_id().unwrap());
            }
            _ => {}
        }

        self.instructions.push(inst);

        (self, new_bucket_id, new_proof_id)
    }

    /// Takes resource from worktop.
    pub fn take_all_from_worktop<F>(
        &mut self,
        resource_address: ResourceAddress,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestBucket) -> &mut Self,
    {
        let (builder, bucket_id, _) =
            self.add_instruction(InstructionV1::TakeAllFromWorktop { resource_address });
        then(builder, bucket_id.unwrap())
    }

    /// Takes resource from worktop, by amount.
    pub fn take_from_worktop<F>(
        &mut self,
        resource_address: ResourceAddress,
        amount: Decimal,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestBucket) -> &mut Self,
    {
        let (builder, bucket_id, _) = self.add_instruction(InstructionV1::TakeFromWorktop {
            amount,
            resource_address,
        });
        then(builder, bucket_id.unwrap())
    }

    /// Takes resource from worktop, by non-fungible ids.
    pub fn take_non_fungibles_from_worktop<F>(
        &mut self,
        resource_address: ResourceAddress,
        ids: &BTreeSet<NonFungibleLocalId>,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestBucket) -> &mut Self,
    {
        let (builder, bucket_id, _) =
            self.add_instruction(InstructionV1::TakeNonFungiblesFromWorktop {
                ids: ids.clone().into_iter().collect(),
                resource_address,
            });
        then(builder, bucket_id.unwrap())
    }

    /// Adds a bucket of resource to worktop.
    pub fn return_to_worktop(&mut self, bucket_id: ManifestBucket) -> &mut Self {
        self.add_instruction(InstructionV1::ReturnToWorktop { bucket_id })
            .0
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains(
        &mut self,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::AssertWorktopContains {
            amount,
            resource_address,
        })
        .0
    }

    /// Asserts that worktop contains resource.
    pub fn assert_worktop_contains_non_fungibles(
        &mut self,
        resource_address: ResourceAddress,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::AssertWorktopContainsNonFungibles {
            ids: ids.clone().into_iter().collect(),
            resource_address,
        })
        .0
    }

    /// Pops the most recent proof from auth zone.
    pub fn pop_from_auth_zone<F>(&mut self, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) = self.add_instruction(InstructionV1::PopFromAuthZone {});
        then(builder, proof_id.unwrap())
    }

    /// Pushes a proof onto the auth zone
    pub fn push_to_auth_zone(&mut self, proof_id: ManifestProof) -> &mut Self {
        self.add_instruction(InstructionV1::PushToAuthZone { proof_id });
        self
    }

    /// Clears the auth zone.
    pub fn clear_auth_zone(&mut self) -> &mut Self {
        self.add_instruction(InstructionV1::ClearAuthZone).0
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
            self.add_instruction(InstructionV1::CreateProofFromAuthZone { resource_address });
        then(builder, proof_id.unwrap())
    }

    /// Creates proof from the auth zone by amount.
    pub fn create_proof_from_auth_zone_of_amount<F>(
        &mut self,
        resource_address: ResourceAddress,
        amount: Decimal,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(InstructionV1::CreateProofFromAuthZoneOfAmount {
                amount,
                resource_address,
            });
        then(builder, proof_id.unwrap())
    }

    /// Creates proof from the auth zone by non-fungible ids.
    pub fn create_proof_from_auth_zone_of_non_fungibles<F>(
        &mut self,
        resource_address: ResourceAddress,
        ids: &BTreeSet<NonFungibleLocalId>,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(InstructionV1::CreateProofFromAuthZoneOfNonFungibles {
                ids: ids.clone().into_iter().collect(),
                resource_address,
            });
        then(builder, proof_id.unwrap())
    }

    /// Creates proof from the auth zone
    pub fn create_proof_from_auth_zone_of_all<F>(
        &mut self,
        resource_address: ResourceAddress,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(InstructionV1::CreateProofFromAuthZoneOfAll { resource_address });
        then(builder, proof_id.unwrap())
    }

    /// Creates proof from a bucket.
    pub fn create_proof_from_bucket<F>(&mut self, bucket_id: &ManifestBucket, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) = self.add_instruction(InstructionV1::CreateProofFromBucket {
            bucket_id: bucket_id.clone(),
        });
        then(builder, proof_id.unwrap())
    }

    pub fn create_proof_from_bucket_of_amount<F>(
        &mut self,
        bucket_id: &ManifestBucket,
        amount: Decimal,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(InstructionV1::CreateProofFromBucketOfAmount {
                bucket_id: bucket_id.clone(),
                amount,
            });
        then(builder, proof_id.unwrap())
    }

    pub fn create_proof_from_bucket_of_non_fungibles<F>(
        &mut self,
        bucket_id: &ManifestBucket,
        ids: BTreeSet<NonFungibleLocalId>,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(InstructionV1::CreateProofFromBucketOfNonFungibles {
                bucket_id: bucket_id.clone(),
                ids: ids.into_iter().collect(),
            });
        then(builder, proof_id.unwrap())
    }

    pub fn create_proof_from_bucket_of_all<F>(
        &mut self,
        bucket_id: &ManifestBucket,
        then: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) =
            self.add_instruction(InstructionV1::CreateProofFromBucketOfAll {
                bucket_id: bucket_id.clone(),
            });
        then(builder, proof_id.unwrap())
    }

    /// Clones a proof.
    pub fn clone_proof<F>(&mut self, proof_id: &ManifestProof, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestProof) -> &mut Self,
    {
        let (builder, _, proof_id) = self.add_instruction(InstructionV1::CloneProof {
            proof_id: proof_id.clone(),
        });
        then(builder, proof_id.unwrap())
    }

    /// Drops a proof.
    pub fn drop_proof(&mut self, proof_id: ManifestProof) -> &mut Self {
        self.add_instruction(InstructionV1::DropProof { proof_id })
            .0
    }

    /// Drops all proofs.
    pub fn drop_all_proofs(&mut self) -> &mut Self {
        self.add_instruction(InstructionV1::DropAllProofs).0
    }

    /// Drops all virtual proofs.
    pub fn clear_signature_proofs(&mut self) -> &mut Self {
        self.add_instruction(InstructionV1::ClearSignatureProofs).0
    }

    /// Creates a fungible resource
    pub fn create_fungible_resource<R: Into<AccessRule>>(
        &mut self,
        divisibility: u8,
        metadata: BTreeMap<String, MetadataValue>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, R)>,
        initial_supply: Option<Decimal>,
    ) -> &mut Self {
        let access_rules = access_rules
            .into_iter()
            .map(|(k, v)| (k, (v.0, v.1.into())))
            .collect();
        if let Some(initial_supply) = initial_supply {
            self.add_instruction(InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE,
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
            self.add_instruction(InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE,
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
        metadata: BTreeMap<String, MetadataValue>,
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

            self.add_instruction(InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE,
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
            self.add_instruction(InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE,
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

    pub fn create_identity_advanced(&mut self, owner_rule: OwnerRole) -> &mut Self {
        self.add_instruction(InstructionV1::CallFunction {
            package_address: IDENTITY_PACKAGE,
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
            args: to_manifest_value(&IdentityCreateAdvancedInput { owner_rule }),
        });
        self
    }

    pub fn create_identity(&mut self) -> &mut Self {
        self.add_instruction(InstructionV1::CallFunction {
            package_address: IDENTITY_PACKAGE,
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_IDENT.to_string(),
            args: to_manifest_value(&IdentityCreateInput {}),
        });
        self
    }

    pub fn create_validator(&mut self, key: EcdsaSecp256k1PublicKey) -> &mut Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: CONSENSUS_MANAGER.into(),
            method_name: CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
            args: to_manifest_value(&ConsensusManagerCreateValidatorInput { key }),
        });
        self
    }

    pub fn register_validator(&mut self, validator_address: ComponentAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: validator_address.into(),
            method_name: VALIDATOR_REGISTER_IDENT.to_string(),
            args: manifest_args!(),
        });
        self
    }

    pub fn unregister_validator(&mut self, validator_address: ComponentAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: validator_address.into(),
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
        self.add_instruction(InstructionV1::CallMethod {
            address: validator_address.into(),
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
        self.add_instruction(InstructionV1::CallMethod {
            address: validator_address.into(),
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
        self.add_instruction(InstructionV1::CallMethod {
            address: validator_address.into(),
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
        self.add_instruction(InstructionV1::CallFunction {
            package_address,
            blueprint_name: blueprint_name.to_string(),
            function_name: function_name.to_string(),
            args: to_manifest_value(&args),
        });
        self
    }

    /// Calls a scrypto method where the arguments should be an array of encoded Scrypto value.
    pub fn call_method<A: Into<GlobalAddress>>(
        &mut self,
        address: A,
        method_name: &str,
        args: ManifestValue,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: address.into(),
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
        self.add_instruction(InstructionV1::CallMethod {
            address: package_address.into(),
            method_name: PACKAGE_SET_ROYALTY_CONFIG_IDENT.to_string(),
            args: to_manifest_value(&PackageSetRoyaltyConfigInput { royalty_config }),
        })
        .0
    }

    pub fn claim_package_royalty(&mut self, package_address: PackageAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: package_address.into(),
            method_name: PACKAGE_CLAIM_ROYALTY_IDENT.to_string(),
            args: to_manifest_value(&PackageClaimRoyaltyInput {}),
        })
        .0
    }

    pub fn set_component_royalty_config(
        &mut self,
        component_address: ComponentAddress,
        royalty_config: RoyaltyConfig,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallRoyaltyMethod {
            address: component_address.into(),
            method_name: COMPONENT_ROYALTY_SET_ROYALTY_CONFIG_IDENT.to_string(),
            args: to_manifest_value(&ComponentSetRoyaltyConfigInput { royalty_config }),
        })
        .0
    }

    pub fn claim_component_royalty(&mut self, component_address: ComponentAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallRoyaltyMethod {
            address: component_address.into(),
            method_name: COMPONENT_ROYALTY_CLAIM_ROYALTY_IDENT.to_string(),
            args: to_manifest_value(&ComponentClaimRoyaltyInput {}),
        })
        .0
    }

    pub fn update_role(
        &mut self,
        address: GlobalAddress,
        role_key: RoleKey,
        rule: AccessRule,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallAccessRulesMethod {
            address,
            method_name: ACCESS_RULES_UPDATE_ROLE_IDENT.to_string(),
            args: to_manifest_value(&AccessRulesUpdateRoleInput {
                role_key,
                rule: Some(rule),
                mutability: None,
            }),
        })
        .0
    }

    pub fn update_role_mutability(
        &mut self,
        address: GlobalAddress,
        role_key: RoleKey,
        mutability: (RoleList, bool),
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallAccessRulesMethod {
            address,
            method_name: ACCESS_RULES_UPDATE_ROLE_IDENT.to_string(),
            args: to_manifest_value(&AccessRulesUpdateRoleInput {
                role_key,
                rule: None,
                mutability: Some(mutability),
            }),
        })
        .0
    }

    pub fn set_metadata(
        &mut self,
        address: GlobalAddress,
        key: String,
        value: MetadataValue,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallMetadataMethod {
            address,
            method_name: METADATA_SET_IDENT.to_string(),
            args: to_manifest_value(&MetadataSetInput { key, value }),
        })
        .0
    }

    /// Publishes a package.
    pub fn publish_package_advanced(
        &mut self,
        code: Vec<u8>,
        definition: PackageSetup,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, MetadataValue>,
        owner_rule: OwnerRole,
    ) -> &mut Self {
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        self.add_instruction(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishWasmAdvancedManifestInput {
                code: ManifestBlobRef(code_hash.0),
                definition,
                royalty_config,
                metadata,
                package_address: None,
                owner_rule,
            }),
        });
        self
    }

    /// Publishes a package with an owner badge.
    pub fn publish_package(&mut self, code: Vec<u8>, definition: PackageSetup) -> &mut Self {
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        self.add_instruction(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishWasmManifestInput {
                code: ManifestBlobRef(code_hash.0),
                definition,
                royalty_config: BTreeMap::new(),
                metadata: BTreeMap::new(),
            }),
        });
        self
    }

    /// Publishes a package with an owner badge.
    pub fn publish_package_with_owner(
        &mut self,
        code: Vec<u8>,
        definition: PackageSetup,
        owner_badge: NonFungibleGlobalId,
    ) -> &mut Self {
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        self.add_instruction(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE,
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: to_manifest_value(&PackagePublishWasmAdvancedManifestInput {
                package_address: None,
                code: ManifestBlobRef(code_hash.0),
                definition,
                royalty_config: BTreeMap::new(),
                metadata: BTreeMap::new(),
                owner_rule: OwnerRole::Fixed(rule!(require(owner_badge.clone()))),
            }),
        });
        self
    }

    /// Builds a transaction manifest.
    pub fn build(&self) -> TransactionManifestV1 {
        let m = TransactionManifestV1 {
            instructions: self.instructions.clone(),
            blobs: self.blobs.clone(),
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
        metadata: BTreeMap<String, MetadataValue>,
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
        metadata: BTreeMap<String, MetadataValue>,
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
        metadata: BTreeMap<String, MetadataValue>,
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
        metadata: BTreeMap<String, MetadataValue>,
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
        self.take_from_worktop(resource_address, amount, |builder, bucket_id| {
            builder
                .add_instruction(InstructionV1::BurnResource { bucket_id })
                .0
        })
    }

    pub fn burn_all_from_worktop(&mut self, resource_address: ResourceAddress) -> &mut Self {
        self.take_all_from_worktop(resource_address, |builder, bucket_id| {
            builder
                .add_instruction(InstructionV1::BurnResource { bucket_id })
                .0
        })
    }

    pub fn mint_fungible(
        &mut self,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: resource_address.into(),
            method_name: FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
            args: to_manifest_value(&FungibleResourceManagerMintInput { amount }),
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

        self.add_instruction(InstructionV1::CallMethod {
            address: resource_address.into(),
            method_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
            args: to_manifest_value(&NonFungibleResourceManagerMintManifestInput { entries }),
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

        self.add_instruction(InstructionV1::CallMethod {
            address: resource_address.into(),
            method_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT.to_string(),
            args: to_manifest_value(&NonFungibleResourceManagerMintUuidManifestInput { entries }),
        });
        self
    }

    pub fn recall(&mut self, vault_id: InternalAddress, amount: Decimal) -> &mut Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            vault_id,
            method_name: VAULT_RECALL_IDENT.to_string(),
            args: to_manifest_value(&VaultRecallInput { amount }),
        });
        self
    }

    pub fn freeze(&mut self, vault_id: InternalAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            vault_id,
            method_name: VAULT_FREEZE_IDENT.to_string(),
            args: to_manifest_value(&VaultFreezeInput {}),
        });
        self
    }

    pub fn unfreeze(&mut self, vault_id: InternalAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            vault_id,
            method_name: VAULT_UNFREEZE_IDENT.to_string(),
            args: to_manifest_value(&VaultUnfreezeInput {}),
        });
        self
    }

    pub fn burn_non_fungible(&mut self, non_fungible_global_id: NonFungibleGlobalId) -> &mut Self {
        let mut ids = BTreeSet::new();
        ids.insert(non_fungible_global_id.local_id().clone());
        self.take_non_fungibles_from_worktop(
            non_fungible_global_id.resource_address().clone(),
            &ids,
            |builder, bucket_id| {
                builder
                    .add_instruction(InstructionV1::BurnResource { bucket_id })
                    .0
            },
        )
    }

    /// Creates an account.
    pub fn new_account_advanced(&mut self, owner_role: OwnerRole) -> &mut Self {
        self.add_instruction(InstructionV1::CallFunction {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
            function_name: ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
            args: to_manifest_value(&AccountCreateAdvancedInput { owner_role }),
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

        self.add_instruction(InstructionV1::CallMethod {
            address: account.into(),
            method_name: ACCOUNT_LOCK_FEE_AND_WITHDRAW_IDENT.to_string(),
            args,
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

        self.add_instruction(InstructionV1::CallMethod {
            address: account.into(),
            method_name: ACCOUNT_LOCK_FEE_AND_WITHDRAW_NON_FUNGIBLES_IDENT.to_string(),
            args,
        })
        .0
    }

    /// Locks a fee from the XRD vault of an account.
    pub fn lock_fee<A: Into<GlobalAddress>>(&mut self, account: A, amount: Decimal) -> &mut Self {
        let args = to_manifest_value(&AccountLockFeeInput { amount });

        self.add_instruction(InstructionV1::CallMethod {
            address: account.into(),
            method_name: ACCOUNT_LOCK_FEE_IDENT.to_string(),
            args,
        })
        .0
    }

    pub fn lock_contingent_fee(&mut self, account: ComponentAddress, amount: Decimal) -> &mut Self {
        let args = to_manifest_value(&AccountLockContingentFeeInput { amount });

        self.add_instruction(InstructionV1::CallMethod {
            address: account.into(),
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

        self.add_instruction(InstructionV1::CallMethod {
            address: account.into(),
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

        self.add_instruction(InstructionV1::CallMethod {
            address: account.into(),
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

        self.add_instruction(InstructionV1::CallMethod {
            address: account.into(),
            method_name: ACCOUNT_CREATE_PROOF_IDENT.to_string(),
            args,
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_of_amount(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> &mut Self {
        let args = to_manifest_value(&AccountCreateProofOfAmountInput {
            resource_address,
            amount,
        });

        self.add_instruction(InstructionV1::CallMethod {
            address: account.into(),
            method_name: ACCOUNT_CREATE_PROOF_OF_AMOUNT_IDENT.to_string(),
            args,
        })
        .0
    }

    /// Creates resource proof from an account.
    pub fn create_proof_from_account_of_non_fungibles(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
        ids: &BTreeSet<NonFungibleLocalId>,
    ) -> &mut Self {
        let args = to_manifest_value(&AccountCreateProofOfNonFungiblesInput {
            resource_address,
            ids: ids.clone(),
        });

        self.add_instruction(InstructionV1::CallMethod {
            address: account.into(),
            method_name: ACCOUNT_CREATE_PROOF_OF_NON_FUNGIBLES_IDENT.to_string(),
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
        self.add_instruction(InstructionV1::CallFunction {
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

    pub fn deposit_batch(&mut self, account_address: ComponentAddress) -> &mut Self {
        self.call_method(
            account_address,
            ACCOUNT_DEPOSIT_BATCH_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
    }

    pub fn try_deposit_batch_or_abort(&mut self, account_address: ComponentAddress) -> &mut Self {
        self.call_method(
            account_address,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_ABORT_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
    }

    pub fn try_deposit_batch_or_refund(&mut self, account_address: ComponentAddress) -> &mut Self {
        self.call_method(
            account_address,
            ACCOUNT_TRY_DEPOSIT_BATCH_OR_REFUND_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
    }

    pub fn borrow_mut<F, E>(&mut self, handler: F) -> Result<&mut Self, E>
    where
        F: FnOnce(&mut Self) -> Result<&mut Self, E>,
    {
        handler(self)
    }
}
