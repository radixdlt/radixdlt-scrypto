use radix_engine_common::data::scrypto::model::*;
use radix_engine_interface::api::node_modules::metadata::MetadataEntry;
use radix_engine_interface::blueprints::resource::{AccessRule, AuthorityKey, ObjectKey, Roles};
use radix_engine_interface::data::manifest::{model::*, ManifestValue};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::schema::PackageSchema;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor)]
pub enum Instruction {
    /// Takes resource from worktop.
    #[sbor(discriminator(INSTRUCTION_TAKE_ALL_FROM_WORKTOP_DISCRIMINATOR))]
    TakeAllFromWorktop { resource_address: ResourceAddress },

    /// Takes resource from worktop by the given amount.
    #[sbor(discriminator(INSTRUCTION_TAKE_FROM_WORKTOP_DISCRIMINATOR))]
    TakeFromWorktop {
        resource_address: ResourceAddress,
        amount: Decimal,
    },

    /// Takes resource from worktop by the given non-fungible IDs.
    #[sbor(discriminator(INSTRUCTION_TAKE_NON_FUNGIBLES_FROM_WORKTOP_DISCRIMINATOR))]
    TakeNonFungiblesFromWorktop {
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
    },

    /// Returns a bucket of resource to worktop.
    #[sbor(discriminator(INSTRUCTION_RETURN_TO_WORKTOP_DISCRIMINATOR))]
    ReturnToWorktop { bucket_id: ManifestBucket },

    /// Asserts worktop contains resource by at least the given amount.
    #[sbor(discriminator(INSTRUCTION_ASSERT_WORKTOP_CONTAINS_DISCRIMINATOR))]
    AssertWorktopContains {
        resource_address: ResourceAddress,
        amount: Decimal,
    },

    /// Asserts worktop contains resource by at least the given non-fungible IDs.
    #[sbor(discriminator(INSTRUCTION_ASSERT_WORKTOP_CONTAINS_NON_FUNGIBLES_DISCRIMINATOR))]
    AssertWorktopContainsNonFungibles {
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
    },

    /// Takes the last proof from the auth zone.
    #[sbor(discriminator(INSTRUCTION_POP_FROM_AUTH_ZONE_DISCRIMINATOR))]
    PopFromAuthZone,

    /// Adds a proof to the auth zone.
    #[sbor(discriminator(INSTRUCTION_PUSH_TO_AUTH_ZONE_DISCRIMINATOR))]
    PushToAuthZone { proof_id: ManifestProof },

    /// Clears the auth zone.
    #[sbor(discriminator(INSTRUCTION_CLEAR_AUTH_ZONE_DISCRIMINATOR))]
    ClearAuthZone,

    // TODO: do we need `CreateProofFromWorktop`, to avoid taking resource out and then creating proof?
    /// Creates a proof from the auth zone
    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_DISCRIMINATOR))]
    CreateProofFromAuthZone { resource_address: ResourceAddress },

    /// Creates a proof from the auth zone, by the given amount
    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_AMOUNT_DISCRIMINATOR))]
    CreateProofFromAuthZoneOfAmount {
        resource_address: ResourceAddress,
        amount: Decimal,
    },

    /// Creates a proof from the auth zone, by the given non-fungible IDs.
    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES_DISCRIMINATOR))]
    CreateProofFromAuthZoneOfNonFungibles {
        resource_address: ResourceAddress,
        ids: BTreeSet<NonFungibleLocalId>,
    },

    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_ALL_DISCRIMINATOR))]
    CreateProofFromAuthZoneOfAll { resource_address: ResourceAddress },

    /// Creates a proof from a bucket.
    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_BUCKET_DISCRIMINATOR))]
    CreateProofFromBucket { bucket_id: ManifestBucket },

    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_AMOUNT_DISCRIMINATOR))]
    CreateProofFromBucketOfAmount {
        bucket_id: ManifestBucket,
        amount: Decimal,
    },

    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_NON_FUNGIBLES_DISCRIMINATOR))]
    CreateProofFromBucketOfNonFungibles {
        bucket_id: ManifestBucket,
        ids: BTreeSet<NonFungibleLocalId>,
    },

    #[sbor(discriminator(INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_ALL_DISCRIMINATOR))]
    CreateProofFromBucketOfAll { bucket_id: ManifestBucket },

    /// Clones a proof.
    #[sbor(discriminator(INSTRUCTION_CLONE_PROOF_DISCRIMINATOR))]
    CloneProof { proof_id: ManifestProof },

    /// Drops a proof.
    #[sbor(discriminator(INSTRUCTION_DROP_PROOF_DISCRIMINATOR))]
    DropProof { proof_id: ManifestProof },

    /// Drops all proofs, both named proofs and auth zone proofs.
    #[sbor(discriminator(INSTRUCTION_DROP_ALL_PROOFS_DISCRIMINATOR))]
    DropAllProofs,

    /// Drop all virtual proofs (can only be auth zone proofs).
    #[sbor(discriminator(INSTRUCTION_CLEAR_SIGNATURE_PROOFS_DISCRIMINATOR))]
    ClearSignatureProofs,

    /// Publish a package.
    #[sbor(discriminator(INSTRUCTION_PUBLISH_PACKAGE_DISCRIMINATOR))]
    PublishPackage {
        code: ManifestBlobRef,
        schema: PackageSchema,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
    },

    #[sbor(discriminator(INSTRUCTION_PUBLISH_PACKAGE_ADVANCED_DISCRIMINATOR))]
    PublishPackageAdvanced {
        code: ManifestBlobRef,
        schema: PackageSchema,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
        authority_rules: Roles,
    },

    #[sbor(discriminator(INSTRUCTION_BURN_RESOURCE_DISCRIMINATOR))]
    BurnResource { bucket_id: ManifestBucket },

    #[sbor(discriminator(INSTRUCTION_RECALL_RESOURCE_DISCRIMINATOR))]
    RecallResource {
        vault_id: InternalAddress,
        amount: Decimal,
    },

    #[sbor(discriminator(INSTRUCTION_SET_METADATA_DISCRIMINATOR))]
    SetMetadata {
        entity_address: GlobalAddress,
        key: String,
        value: MetadataEntry,
    },

    #[sbor(discriminator(INSTRUCTION_REMOVE_METADATA_DISCRIMINATOR))]
    RemoveMetadata {
        entity_address: GlobalAddress,
        key: String,
    },

    #[sbor(discriminator(INSTRUCTION_SET_PACKAGE_ROYALTY_DISCRIMINATOR))]
    SetPackageRoyaltyConfig {
        package_address: PackageAddress,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
    },

    #[sbor(discriminator(INSTRUCTION_SET_COMPONENT_ROYALTY_DISCRIMINATOR))]
    SetComponentRoyaltyConfig {
        component_address: ComponentAddress,
        royalty_config: RoyaltyConfig,
    },

    #[sbor(discriminator(INSTRUCTION_CLAIM_PACKAGE_ROYALTY_DISCRIMINATOR))]
    ClaimPackageRoyalty { package_address: PackageAddress },

    #[sbor(discriminator(INSTRUCTION_CLAIM_COMPONENT_ROYALTY_DISCRIMINATOR))]
    ClaimComponentRoyalty { component_address: ComponentAddress },

    #[sbor(discriminator(INSTRUCTION_MINT_FUNGIBLE_DISCRIMINATOR))]
    MintFungible {
        resource_address: ResourceAddress,
        amount: Decimal,
    },

    #[sbor(discriminator(INSTRUCTION_MINT_NON_FUNGIBLE_DISCRIMINATOR))]
    MintNonFungible {
        resource_address: ResourceAddress,
        args: ManifestValue,
    },

    #[sbor(discriminator(INSTRUCTION_MINT_UUID_NON_FUNGIBLE_DISCRIMINATOR))]
    MintUuidNonFungible {
        resource_address: ResourceAddress,
        args: ManifestValue,
    },

    #[sbor(discriminator(INSTRUCTION_CALL_FUNCTION_DISCRIMINATOR))]
    CallFunction {
        package_address: PackageAddress,
        blueprint_name: String,
        function_name: String,
        args: ManifestValue,
    },

    #[sbor(discriminator(INSTRUCTION_CALL_METHOD_DISCRIMINATOR))]
    CallMethod {
        component_address: ComponentAddress,
        method_name: String,
        args: ManifestValue,
    },

    #[sbor(discriminator(INSTRUCTION_SET_AUTHORITY_ACCESS_RULE_DISCRIMINATOR))]
    SetAuthorityAccessRule {
        entity_address: GlobalAddress,
        object_key: ObjectKey,
        authority_key: AuthorityKey,
        rule: AccessRule,
    },
    #[sbor(discriminator(INSTRUCTION_SET_AUTHORITY_MUTABILITY_DISCRIMINATOR))]
    SetAuthorityMutability {
        entity_address: GlobalAddress,
        object_key: ObjectKey,
        authority_key: AuthorityKey,
        mutability: AccessRule,
    },
}

//===============================================================
// INSTRUCTION DISCRIMINATORS:
//
// These are separately saved in the ledger app. To avoid too much
// churn there:
//
// - Try to keep these constant when adding/removing instructions:
//   > For a new instruction, allocate a new number from the end
//   > If removing an instruction, leave a gap
// - Feel free to move the enum around to make logical groupings
//   though
//===============================================================

// Note: instruction discriminator is not finalized yet!!

pub const INSTRUCTION_TAKE_ALL_FROM_WORKTOP_DISCRIMINATOR: u8 = 0;
pub const INSTRUCTION_TAKE_FROM_WORKTOP_DISCRIMINATOR: u8 = 1;
pub const INSTRUCTION_TAKE_NON_FUNGIBLES_FROM_WORKTOP_DISCRIMINATOR: u8 = 2;
pub const INSTRUCTION_RETURN_TO_WORKTOP_DISCRIMINATOR: u8 = 3;
pub const INSTRUCTION_ASSERT_WORKTOP_CONTAINS_DISCRIMINATOR: u8 = 4;
pub const INSTRUCTION_ASSERT_WORKTOP_CONTAINS_NON_FUNGIBLES_DISCRIMINATOR: u8 = 6;
pub const INSTRUCTION_POP_FROM_AUTH_ZONE_DISCRIMINATOR: u8 = 7;
pub const INSTRUCTION_PUSH_TO_AUTH_ZONE_DISCRIMINATOR: u8 = 8;
pub const INSTRUCTION_CLEAR_AUTH_ZONE_DISCRIMINATOR: u8 = 9;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_DISCRIMINATOR: u8 = 10;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_AMOUNT_DISCRIMINATOR: u8 = 11;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES_DISCRIMINATOR: u8 = 12;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_ALL_DISCRIMINATOR: u8 = 13;
pub const INSTRUCTION_CREATE_PROOF_FROM_BUCKET_DISCRIMINATOR: u8 = 14;
pub const INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_AMOUNT_DISCRIMINATOR: u8 = 15;
pub const INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_NON_FUNGIBLES_DISCRIMINATOR: u8 = 16;
pub const INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_ALL_DISCRIMINATOR: u8 = 17;
pub const INSTRUCTION_CLONE_PROOF_DISCRIMINATOR: u8 = 18;
pub const INSTRUCTION_DROP_PROOF_DISCRIMINATOR: u8 = 19;
pub const INSTRUCTION_DROP_ALL_PROOFS_DISCRIMINATOR: u8 = 20;
pub const INSTRUCTION_CLEAR_SIGNATURE_PROOFS_DISCRIMINATOR: u8 = 21;
pub const INSTRUCTION_PUBLISH_PACKAGE_DISCRIMINATOR: u8 = 22;
pub const INSTRUCTION_PUBLISH_PACKAGE_ADVANCED_DISCRIMINATOR: u8 = 23;
pub const INSTRUCTION_BURN_RESOURCE_DISCRIMINATOR: u8 = 24;
pub const INSTRUCTION_RECALL_RESOURCE_DISCRIMINATOR: u8 = 25;
pub const INSTRUCTION_SET_METADATA_DISCRIMINATOR: u8 = 26;
pub const INSTRUCTION_REMOVE_METADATA_DISCRIMINATOR: u8 = 27;
pub const INSTRUCTION_SET_PACKAGE_ROYALTY_DISCRIMINATOR: u8 = 28;
pub const INSTRUCTION_SET_COMPONENT_ROYALTY_DISCRIMINATOR: u8 = 29;
pub const INSTRUCTION_CLAIM_PACKAGE_ROYALTY_DISCRIMINATOR: u8 = 30;
pub const INSTRUCTION_CLAIM_COMPONENT_ROYALTY_DISCRIMINATOR: u8 = 31;
pub const INSTRUCTION_SET_METHOD_ACCESS_RULE_DISCRIMINATOR: u8 = 32;
pub const INSTRUCTION_MINT_FUNGIBLE_DISCRIMINATOR: u8 = 33;
pub const INSTRUCTION_MINT_NON_FUNGIBLE_DISCRIMINATOR: u8 = 34;
pub const INSTRUCTION_MINT_UUID_NON_FUNGIBLE_DISCRIMINATOR: u8 = 35;
pub const INSTRUCTION_CALL_FUNCTION_DISCRIMINATOR: u8 = 36;
pub const INSTRUCTION_CALL_METHOD_DISCRIMINATOR: u8 = 37;
pub const INSTRUCTION_SET_AUTHORITY_ACCESS_RULE_DISCRIMINATOR: u8 = 38;
pub const INSTRUCTION_SET_AUTHORITY_MUTABILITY_DISCRIMINATOR: u8 = 39;
