use radix_engine_common::data::scrypto::model::*;
use radix_engine_interface::api::node_modules::metadata::MetadataEntry;
use radix_engine_interface::blueprints::resource::{AccessRule, AccessRulesConfig, MethodKey};
use radix_engine_interface::data::manifest::{model::*, ManifestValue};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor)]
pub enum Instruction {
    /// Takes resource from worktop.
    TakeFromWorktop {
        resource_address: ResourceAddress,
    },

    /// Takes resource from worktop by the given amount.
    TakeFromWorktopByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },

    /// Takes resource from worktop by the given non-fungible IDs.
    TakeFromWorktopByIds {
        ids: BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    },

    /// Returns a bucket of resource to worktop.
    ReturnToWorktop {
        bucket_id: ManifestBucket,
    },

    /// Asserts worktop contains resource.
    AssertWorktopContains {
        resource_address: ResourceAddress,
    },

    /// Asserts worktop contains resource by at least the given amount.
    AssertWorktopContainsByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },

    /// Asserts worktop contains resource by at least the given non-fungible IDs.
    AssertWorktopContainsByIds {
        ids: BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    },

    /// Takes the last proof from the auth zone.
    PopFromAuthZone,

    /// Adds a proof to the auth zone.
    PushToAuthZone {
        proof_id: ManifestProof,
    },

    /// Clears the auth zone.
    ClearAuthZone,

    // TODO: do we need `CreateProofFromWorktop`, to avoid taking resource out and then creating proof?
    /// Creates a proof from the auth zone
    CreateProofFromAuthZone {
        resource_address: ResourceAddress,
    },

    /// Creates a proof from the auth zone, by the given amount
    CreateProofFromAuthZoneByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },

    /// Creates a proof from the auth zone, by the given non-fungible IDs.
    CreateProofFromAuthZoneByIds {
        ids: BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    },

    /// Creates a proof from a bucket.
    CreateProofFromBucket {
        bucket_id: ManifestBucket,
    },

    /// Clones a proof.
    CloneProof {
        proof_id: ManifestProof,
    },

    /// Drops a proof.
    DropProof {
        proof_id: ManifestProof,
    },

    /// Drops all proofs, both named proofs and auth zone proofs.
    DropAllProofs,

    /// Drop all virtual proofs (can only be auth zone proofs).
    ClearSignatureProofs,

    /// Publish a package.
    PublishPackage {
        code: ManifestBlobRef,
        schema: ManifestBlobRef,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
    },

    PublishPackageAdvanced {
        code: ManifestBlobRef,
        schema: ManifestBlobRef,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
        access_rules: AccessRulesConfig,
    },

    BurnResource {
        bucket_id: ManifestBucket,
    },

    RecallResource {
        vault_id: LocalAddress,
        amount: Decimal,
    },

    SetMetadata {
        entity_address: GlobalAddress,
        key: String,
        value: MetadataEntry,
    },

    RemoveMetadata {
        entity_address: GlobalAddress,
        key: String,
    },

    SetPackageRoyaltyConfig {
        package_address: PackageAddress,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
    },

    SetComponentRoyaltyConfig {
        component_address: ComponentAddress,
        royalty_config: RoyaltyConfig,
    },

    ClaimPackageRoyalty {
        package_address: PackageAddress,
    },

    ClaimComponentRoyalty {
        component_address: ComponentAddress,
    },

    SetMethodAccessRule {
        entity_address: GlobalAddress,
        key: MethodKey,
        rule: AccessRule,
    },

    MintFungible {
        resource_address: ResourceAddress,
        amount: Decimal,
    },

    MintNonFungible {
        resource_address: ResourceAddress,
        args: ManifestValue,
    },

    MintUuidNonFungible {
        resource_address: ResourceAddress,
        args: ManifestValue,
    },

    CallFunction {
        package_address: PackageAddress,
        blueprint_name: String,
        function_name: String,
        args: ManifestValue,
    },

    CallMethod {
        component_address: ComponentAddress,
        method_name: String,
        args: ManifestValue,
    },
}
