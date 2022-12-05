use radix_engine_interface::api::types::{BucketId, GlobalAddress, ProofId};
use radix_engine_interface::crypto::Blob;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use radix_engine_interface::scrypto;
use sbor::rust::collections::BTreeMap;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
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
        ids: BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    },

    /// Returns a bucket of resource to worktop.
    ReturnToWorktop {
        bucket_id: BucketId,
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
        ids: BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    },

    /// Takes the last proof from the auth zone.
    PopFromAuthZone,

    /// Adds a proof to the auth zone.
    PushToAuthZone {
        proof_id: ProofId,
    },

    /// Drops all proofs in the auth zone
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
        ids: BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    },

    /// Creates a proof from a bucket.
    CreateProofFromBucket {
        bucket_id: BucketId,
    },

    /// Clones a proof.
    CloneProof {
        proof_id: ProofId,
    },

    /// Drops a proof.
    DropProof {
        proof_id: ProofId,
    },

    /// Drops all of the proofs in the transaction.
    DropAllProofs,

    /// Calls a scrypto function.
    ///
    /// Buckets and proofs in arguments moves from transaction context to the callee.
    CallFunction {
        package_address: PackageAddress,
        blueprint_name: String,
        function_name: String,
        args: Vec<u8>,
    },

    /// Calls a scrypto method.
    ///
    /// Buckets and proofs in arguments moves from transaction context to the callee.
    CallMethod {
        component_address: ComponentAddress,
        method_name: String,
        args: Vec<u8>,
    },

    /// Publish a package.
    PublishPackage {
        code: Blob,
        abi: Blob,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
        access_rules: AccessRules,
    },

    /// Publishes a package and set up auth rules using the owner badge.
    PublishPackageWithOwner {
        code: Blob,
        abi: Blob,
        owner_badge: NonFungibleAddress,
    },

    CreateResource {
        resource_type: ResourceType,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, Mutability)>,
        mint_params: Option<MintParams>,
    },

    CreateResourceWithOwner {
        resource_type: ResourceType,
        metadata: BTreeMap<String, String>,
        owner_badge: NonFungibleAddress,
        mint_params: Option<MintParams>,
    },

    BurnResource {
        bucket_id: BucketId,
    },

    MintFungible {
        resource_address: ResourceAddress,
        amount: Decimal,
    },

    SetMetadata {
        entity_address: GlobalAddress,
        metadata: BTreeMap<String, String>,
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
    // TODO: add_access_rules & set_access_rules
}
