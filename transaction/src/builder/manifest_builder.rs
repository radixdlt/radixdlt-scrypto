use radix_engine_common::native_addresses::PACKAGE_PACKAGE;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::node_modules::metadata::{
    MetadataInit, MetadataLockInput, MetadataSetInput, MetadataValue, METADATA_LOCK_IDENT,
    METADATA_SET_IDENT,
};
use radix_engine_interface::api::node_modules::royalty::{
    ComponentClaimRoyaltiesInput, ComponentLockRoyaltyInput, ComponentSetRoyaltyInput,
    COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT, COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT,
    COMPONENT_ROYALTY_SET_ROYALTY_IDENT,
};
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::access_controller::{
    RuleSet, ACCESS_CONTROLLER_BLUEPRINT, ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT,
};
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::consensus_manager::{
    CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT, VALIDATOR_CLAIM_XRD_IDENT, VALIDATOR_REGISTER_IDENT,
    VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS, VALIDATOR_STAKE_AS_OWNER_IDENT,
    VALIDATOR_STAKE_IDENT, VALIDATOR_UNREGISTER_IDENT, VALIDATOR_UNSTAKE_IDENT,
};
use radix_engine_interface::blueprints::identity::{
    IdentityCreateAdvancedInput, IdentityCreateInput, IDENTITY_BLUEPRINT,
    IDENTITY_CREATE_ADVANCED_IDENT, IDENTITY_CREATE_IDENT,
};
use radix_engine_interface::blueprints::package::{
    PackageClaimRoyaltiesInput, PackageDefinition, PackagePublishWasmAdvancedManifestInput,
    PackagePublishWasmManifestInput, PACKAGE_BLUEPRINT, PACKAGE_CLAIM_ROYALTIES_IDENT,
    PACKAGE_PUBLISH_WASM_ADVANCED_IDENT, PACKAGE_PUBLISH_WASM_IDENT,
};
use radix_engine_interface::blueprints::resource::ResourceAction::{Burn, Mint};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::{
    ACCESS_CONTROLLER_PACKAGE, ACCOUNT_PACKAGE, CONSENSUS_MANAGER, IDENTITY_PACKAGE,
    RESOURCE_PACKAGE,
};
use radix_engine_interface::crypto::{hash, Hash, Secp256k1PublicKey};
#[cfg(feature = "dump_manifest_to_file")]
use radix_engine_interface::data::manifest::manifest_encode;
use radix_engine_interface::data::manifest::{model::*, ManifestEncode, ManifestValue};
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
    pub fn from_intent(intent: &IntentV1) -> Self {
        Self {
            instructions: intent.instructions.0.clone(),
            blobs: intent
                .blobs
                .blobs
                .iter()
                .map(|blob| (hash(&blob.0), blob.0.clone()))
                .collect(),
        }
    }

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

pub struct Symbols {
    pub new_buckets: Vec<ManifestBucket>,
    pub new_proofs: Vec<ManifestProof>,
    pub new_address_reservations: Vec<ManifestAddressReservation>,
    pub new_addresses: Vec<u32>,
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

    pub fn add_blob(&mut self, blob: Vec<u8>) -> &mut Self {
        let hash = hash(&blob);
        self.blobs.insert(hash, blob);
        self
    }

    /// Adds a raw instruction.
    pub fn add_instruction(
        &mut self,
        inst: InstructionV1,
    ) -> (&mut Self, Option<ManifestBucket>, Option<ManifestProof>) {
        let (builder, mut symbols) = self.add_instruction_advanced(inst);

        (builder, symbols.new_buckets.pop(), symbols.new_proofs.pop())
    }

    pub fn add_instruction_advanced(&mut self, inst: InstructionV1) -> (&mut Self, Symbols) {
        let mut new_buckets: Vec<ManifestBucket> = Vec::new();
        let mut new_proofs: Vec<ManifestProof> = Vec::new();
        let mut new_address_reservations: Vec<ManifestAddressReservation> = Vec::new();
        let mut new_addresses: Vec<u32> = Vec::new();

        match &inst {
            InstructionV1::TakeAllFromWorktop { .. }
            | InstructionV1::TakeFromWorktop { .. }
            | InstructionV1::TakeNonFungiblesFromWorktop { .. } => {
                new_buckets.push(self.id_allocator.new_bucket_id());
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
                new_proofs.push(self.id_allocator.new_proof_id());
            }
            InstructionV1::AllocateGlobalAddress { .. } => {
                new_address_reservations.push(self.id_allocator.new_address_reservation_id());
                new_addresses.push(self.id_allocator.new_address_id());
            }
            _ => {}
        }

        self.instructions.push(inst);

        (
            self,
            Symbols {
                new_buckets,
                new_proofs,
                new_address_reservations,
                new_addresses,
            },
        )
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
    pub fn assert_worktop_contains_any(&mut self, resource_address: ResourceAddress) -> &mut Self {
        self.add_instruction(InstructionV1::AssertWorktopContainsAny { resource_address })
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

    pub fn allocate_global_address<F>(&mut self, blueprint_id: BlueprintId, then: F) -> &mut Self
    where
        F: FnOnce(&mut Self, ManifestAddressReservation, u32) -> &mut Self,
    {
        let (builder, mut symbols) =
            self.add_instruction_advanced(InstructionV1::AllocateGlobalAddress {
                package_address: blueprint_id.package_address,
                blueprint_name: blueprint_id.blueprint_name,
            });
        then(
            builder,
            symbols.new_address_reservations.pop().unwrap(),
            symbols.new_addresses.pop().unwrap(),
        )
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
        owner_role: OwnerRole,
        track_total_supply: bool,
        divisibility: u8,
        metadata: ModuleConfig<MetadataInit>,
        access_rules: BTreeMap<ResourceAction, (AccessRule, R)>,
        initial_supply: Option<Decimal>,
    ) -> &mut Self {
        let access_rules = access_rules
            .into_iter()
            .map(|(k, v)| (k, (v.0, v.1.into())))
            .collect();
        if let Some(initial_supply) = initial_supply {
            self.add_instruction(InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value_and_unwrap!(
                    &FungibleResourceManagerCreateWithInitialSupplyManifestInput {
                        owner_role,
                        divisibility,
                        track_total_supply,
                        metadata,
                        access_rules,
                        initial_supply,
                        address_reservation: None,
                    }
                ),
            });
        } else {
            self.add_instruction(InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(&FungibleResourceManagerCreateManifestInput {
                    owner_role,
                    divisibility,
                    track_total_supply,
                    metadata,
                    access_rules,
                    address_reservation: None,
                }),
            });
        }

        self
    }

    /// Creates a new non-fungible resource
    pub fn create_non_fungible_resource<R, T, V>(
        &mut self,
        owner_role: OwnerRole,
        id_type: NonFungibleIdType,
        track_total_supply: bool,
        metadata: ModuleConfig<MetadataInit>,
        access_rules: BTreeMap<ResourceAction, (AccessRule, R)>,
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
                .map(|(id, e)| (id, (to_manifest_value_and_unwrap!(&e),)))
                .collect();

            self.add_instruction(InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value_and_unwrap!(
                    &NonFungibleResourceManagerCreateWithInitialSupplyManifestInput {
                        owner_role,
                        id_type,
                        track_total_supply,
                        non_fungible_schema: NonFungibleDataSchema::new_schema::<V>(),
                        metadata,
                        access_rules,
                        entries,
                        address_reservation: None,
                    }
                ),
            });
        } else {
            self.add_instruction(InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT.to_string(),
                args: to_manifest_value_and_unwrap!(
                    &NonFungibleResourceManagerCreateManifestInput {
                        owner_role,
                        id_type,
                        track_total_supply,
                        non_fungible_schema: NonFungibleDataSchema::new_schema::<V>(),
                        access_rules,
                        metadata,
                        address_reservation: None,
                    }
                ),
            });
        }

        self
    }

    pub fn create_identity_advanced(&mut self, owner_rule: OwnerRole) -> &mut Self {
        self.add_instruction(InstructionV1::CallFunction {
            package_address: IDENTITY_PACKAGE.into(),
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&IdentityCreateAdvancedInput { owner_rule }),
        });
        self
    }

    pub fn create_identity(&mut self) -> &mut Self {
        self.add_instruction(InstructionV1::CallFunction {
            package_address: IDENTITY_PACKAGE.into(),
            blueprint_name: IDENTITY_BLUEPRINT.to_string(),
            function_name: IDENTITY_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&IdentityCreateInput {}),
        });
        self
    }

    pub fn create_validator(
        &mut self,
        key: Secp256k1PublicKey,
        fee_factor: Decimal,
        xrd_payment: ManifestBucket,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: CONSENSUS_MANAGER.into(),
            method_name: CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
            args: manifest_args!(key, fee_factor, xrd_payment),
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

    pub fn signal_protocol_update_readiness(
        &mut self,
        validator_address: ComponentAddress,
        protocol_version_name: &str,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: validator_address.into(),
            method_name: VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS.to_string(),
            args: manifest_args!(protocol_version_name.to_string()),
        });
        self
    }

    pub fn stake_validator_as_owner(
        &mut self,
        validator_address: ComponentAddress,
        bucket: ManifestBucket,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: validator_address.into(),
            method_name: VALIDATOR_STAKE_AS_OWNER_IDENT.to_string(),
            args: manifest_args!(bucket),
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
    pub fn call_function<P>(
        &mut self,
        package_address: P,
        blueprint_name: &str,
        function_name: &str,
        args: ManifestValue,
    ) -> &mut Self
    where
        P: Into<DynamicPackageAddress>,
    {
        self.add_instruction(InstructionV1::CallFunction {
            package_address: package_address.into(),
            blueprint_name: blueprint_name.to_string(),
            function_name: function_name.to_string(),
            args: to_manifest_value_and_unwrap!(&args),
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
            address: address.into().into(),
            method_name: method_name.to_owned(),
            args: args,
        });
        self
    }

    pub fn claim_package_royalties(&mut self, package_address: PackageAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallMethod {
            address: package_address.into(),
            method_name: PACKAGE_CLAIM_ROYALTIES_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackageClaimRoyaltiesInput {}),
        })
        .0
    }

    pub fn set_component_royalty<S: ToString>(
        &mut self,
        component_address: ComponentAddress,
        method: S,
        amount: RoyaltyAmount,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallRoyaltyMethod {
            address: component_address.into(),
            method_name: COMPONENT_ROYALTY_SET_ROYALTY_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ComponentSetRoyaltyInput {
                method: method.to_string(),
                amount,
            }),
        })
        .0
    }

    pub fn lock_component_royalty<S: ToString>(
        &mut self,
        component_address: ComponentAddress,
        method: S,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallRoyaltyMethod {
            address: component_address.into(),
            method_name: COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ComponentLockRoyaltyInput {
                method: method.to_string(),
            }),
        })
        .0
    }

    pub fn claim_component_royalties(&mut self, component_address: ComponentAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallRoyaltyMethod {
            address: component_address.into(),
            method_name: COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&ComponentClaimRoyaltiesInput {}),
        })
        .0
    }

    pub fn set_owner_role(&mut self, address: GlobalAddress, rule: AccessRule) -> &mut Self {
        self.add_instruction(InstructionV1::CallAccessRulesMethod {
            address: address.into(),
            method_name: ACCESS_RULES_SET_OWNER_ROLE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccessRulesSetOwnerRoleInput { rule }),
        })
        .0
    }

    pub fn update_role(
        &mut self,
        address: GlobalAddress,
        module: ObjectModuleId,
        role_key: RoleKey,
        rule: AccessRule,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallAccessRulesMethod {
            address: address.into(),
            method_name: ACCESS_RULES_SET_ROLE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccessRulesSetRoleInput {
                module,
                role_key,
                rule,
            }),
        })
        .0
    }

    pub fn lock_role(
        &mut self,
        address: GlobalAddress,
        module: ObjectModuleId,
        role_key: RoleKey,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallAccessRulesMethod {
            address: address.into(),
            method_name: ACCESS_RULES_LOCK_ROLE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccessRulesLockRoleInput { module, role_key }),
        })
        .0
    }

    pub fn lock_owner_role(&mut self, address: GlobalAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallAccessRulesMethod {
            address: address.into(),
            method_name: ACCESS_RULES_LOCK_OWNER_ROLE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccessRulesLockOwnerRoleInput {}),
        })
        .0
    }

    pub fn get_role(
        &mut self,
        address: GlobalAddress,
        module: ObjectModuleId,
        role_key: RoleKey,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallAccessRulesMethod {
            address: address.into(),
            method_name: ACCESS_RULES_GET_ROLE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccessRulesGetRoleInput { module, role_key }),
        })
        .0
    }

    pub fn set_metadata<A: Into<DynamicGlobalAddress>, S: ToString>(
        &mut self,
        address: A,
        key: S,
        value: MetadataValue,
    ) -> &mut Self {
        self.add_instruction(InstructionV1::CallMetadataMethod {
            address: address.into(),
            method_name: METADATA_SET_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&MetadataSetInput {
                key: key.to_string(),
                value
            }),
        })
        .0
    }

    pub fn freeze_metadata(&mut self, address: GlobalAddress, key: String) -> &mut Self {
        self.add_instruction(InstructionV1::CallMetadataMethod {
            address: address.into(),
            method_name: METADATA_LOCK_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&MetadataLockInput { key }),
        })
        .0
    }

    /// Publishes a package.
    pub fn publish_package_advanced<M: Into<MetadataInit>>(
        &mut self,
        address: Option<ManifestAddressReservation>,
        code: Vec<u8>,
        definition: PackageDefinition,
        metadata: M,
        owner_role: OwnerRole,
    ) -> &mut Self {
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        self.add_instruction(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishWasmAdvancedManifestInput {
                code: ManifestBlobRef(code_hash.0),
                setup: definition,
                metadata: metadata.into(),
                package_address: address,
                owner_role,
            }),
        });
        self
    }

    /// Publishes a package with an owner badge.
    pub fn publish_package(&mut self, code: Vec<u8>, definition: PackageDefinition) -> &mut Self {
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        self.add_instruction(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishWasmManifestInput {
                code: ManifestBlobRef(code_hash.0),
                setup: definition,
                metadata: metadata_init!(),
            }),
        });
        self
    }

    /// Publishes a package with an owner badge.
    pub fn publish_package_with_owner(
        &mut self,
        code: Vec<u8>,
        definition: PackageDefinition,
        owner_badge: NonFungibleGlobalId,
    ) -> &mut Self {
        let code_hash = hash(&code);
        self.blobs.insert(code_hash, code);

        self.add_instruction(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishWasmAdvancedManifestInput {
                package_address: None,
                code: ManifestBlobRef(code_hash.0),
                setup: definition,
                metadata: metadata_init!(),
                owner_role: OwnerRole::Fixed(rule!(require(owner_badge.clone()))),
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
        owner_role: OwnerRole,
        metadata: ModuleConfig<MetadataInit>,
        minter_rule: AccessRule,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceAction::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );
        access_rules.insert(Mint, (minter_rule.clone(), rule!(deny_all)));
        access_rules.insert(Burn, (minter_rule.clone(), rule!(deny_all)));

        let initial_supply = Option::None;
        self.create_fungible_resource(owner_role, true, 18, metadata, access_rules, initial_supply)
    }

    /// Creates a token resource with fixed supply.
    pub fn new_token_fixed(
        &mut self,
        owner_role: OwnerRole,
        metadata: ModuleConfig<MetadataInit>,
        initial_supply: Decimal,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceAction::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );

        self.create_fungible_resource(
            owner_role,
            true,
            18,
            metadata,
            access_rules,
            Some(initial_supply),
        )
    }

    /// Creates a badge resource with mutable supply.
    pub fn new_badge_mutable(
        &mut self,
        owner_role: OwnerRole,
        metadata: ModuleConfig<MetadataInit>,
        minter_rule: AccessRule,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceAction::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );
        access_rules.insert(Mint, (minter_rule.clone(), rule!(deny_all)));
        access_rules.insert(Burn, (minter_rule.clone(), rule!(deny_all)));

        let initial_supply = Option::None;
        self.create_fungible_resource(owner_role, false, 0, metadata, access_rules, initial_supply)
    }

    /// Creates a badge resource with fixed supply.
    pub fn new_badge_fixed(
        &mut self,
        owner_role: OwnerRole,
        metadata: ModuleConfig<MetadataInit>,
        initial_supply: Decimal,
    ) -> &mut Self {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            ResourceAction::Withdraw,
            (rule!(allow_all), rule!(deny_all)),
        );

        self.create_fungible_resource(
            owner_role,
            false,
            0,
            metadata,
            access_rules,
            Some(initial_supply),
        )
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
            args: to_manifest_value_and_unwrap!(&FungibleResourceManagerMintInput { amount }),
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
            .map(|(id, e)| (id, (to_manifest_value_and_unwrap!(&e),)))
            .collect();

        self.add_instruction(InstructionV1::CallMethod {
            address: resource_address.into(),
            method_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&NonFungibleResourceManagerMintManifestInput {
                entries
            }),
        });
        self
    }

    pub fn mint_ruid_non_fungible<T, V>(
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
            .map(|e| (to_manifest_value_and_unwrap!(&e),))
            .collect();

        self.add_instruction(InstructionV1::CallMethod {
            address: resource_address.into(),
            method_name: NON_FUNGIBLE_RESOURCE_MANAGER_MINT_RUID_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&NonFungibleResourceManagerMintRuidManifestInput {
                entries
            }),
        });
        self
    }

    pub fn recall(&mut self, vault_id: InternalAddress, amount: Decimal) -> &mut Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_RECALL_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultRecallInput { amount }),
        });
        self
    }

    pub fn recall_non_fungibles(
        &mut self,
        vault_id: InternalAddress,
        non_fungible_local_ids: BTreeSet<NonFungibleLocalId>,
    ) -> &mut Self {
        let args = to_manifest_value_and_unwrap!(&NonFungibleVaultRecallNonFungiblesInput {
            non_fungible_local_ids,
        });

        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT.to_string(),
            args,
        });
        self
    }

    pub fn freeze_withdraw(&mut self, vault_id: InternalAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_FREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultFreezeInput {
                to_freeze: VaultFreezeFlags::WITHDRAW,
            }),
        });
        self
    }

    pub fn unfreeze_withdraw(&mut self, vault_id: InternalAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_UNFREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultUnfreezeInput {
                to_unfreeze: VaultFreezeFlags::WITHDRAW,
            }),
        });
        self
    }

    pub fn freeze_deposit(&mut self, vault_id: InternalAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_FREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultFreezeInput {
                to_freeze: VaultFreezeFlags::DEPOSIT,
            }),
        });
        self
    }

    pub fn unfreeze_deposit(&mut self, vault_id: InternalAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_UNFREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultUnfreezeInput {
                to_unfreeze: VaultFreezeFlags::DEPOSIT,
            }),
        });
        self
    }

    pub fn freeze_burn(&mut self, vault_id: InternalAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_FREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultFreezeInput {
                to_freeze: VaultFreezeFlags::BURN,
            }),
        });
        self
    }

    pub fn unfreeze_burn(&mut self, vault_id: InternalAddress) -> &mut Self {
        self.add_instruction(InstructionV1::CallDirectVaultMethod {
            address: vault_id,
            method_name: VAULT_UNFREEZE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&VaultUnfreezeInput {
                to_unfreeze: VaultFreezeFlags::BURN,
            }),
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
            package_address: ACCOUNT_PACKAGE.into(),
            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
            function_name: ACCOUNT_CREATE_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccountCreateAdvancedInput { owner_role }),
        })
        .0
    }

    pub fn new_account(&mut self) -> &mut Self {
        self.add_instruction(InstructionV1::CallFunction {
            package_address: ACCOUNT_PACKAGE.into(),
            blueprint_name: ACCOUNT_BLUEPRINT.to_string(),
            function_name: ACCOUNT_CREATE_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&AccountCreateInput {}),
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
        let args = to_manifest_value_and_unwrap!(&AccountLockFeeAndWithdrawInput {
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
        let args = to_manifest_value_and_unwrap!(&AccountLockFeeAndWithdrawNonFungiblesInput {
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
        let args = to_manifest_value_and_unwrap!(&AccountLockFeeInput { amount });

        self.add_instruction(InstructionV1::CallMethod {
            address: account.into().into(),
            method_name: ACCOUNT_LOCK_FEE_IDENT.to_string(),
            args,
        })
        .0
    }

    pub fn lock_contingent_fee(&mut self, account: ComponentAddress, amount: Decimal) -> &mut Self {
        let args = to_manifest_value_and_unwrap!(&AccountLockContingentFeeInput { amount });

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
        let args = to_manifest_value_and_unwrap!(&AccountWithdrawInput {
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
        let args = to_manifest_value_and_unwrap!(&AccountWithdrawNonFungiblesInput {
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

    /// Withdraws resource from an account.
    pub fn burn_in_account(
        &mut self,
        account: ComponentAddress,
        resource_address: ResourceAddress,
        amount: Decimal,
    ) -> &mut Self {
        let args = to_manifest_value_and_unwrap!(&AccountBurnInput {
            resource_address,
            amount
        });

        self.add_instruction(InstructionV1::CallMethod {
            address: account.into(),
            method_name: ACCOUNT_BURN_IDENT.to_string(),
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
        let args = to_manifest_value_and_unwrap!(&AccountCreateProofInput { resource_address });

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
        let args = to_manifest_value_and_unwrap!(&AccountCreateProofOfAmountInput {
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
        let args = to_manifest_value_and_unwrap!(&AccountCreateProofOfNonFungiblesInput {
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
            package_address: ACCESS_CONTROLLER_PACKAGE.into(),
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
