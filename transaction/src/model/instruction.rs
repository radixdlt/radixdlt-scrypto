use radix_engine_interface::api::types::*;
use radix_engine_interface::crypto::EcdsaSecp256k1PublicKey;
use radix_engine_interface::data::types::{ManifestBlobRef, ManifestBucket, ManifestProof};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum BasicInstruction {
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

    /// Drops all of the proofs in the transaction.
    DropAllProofs,

    /// Publish a package.
    PublishPackage {
        code: ManifestBlobRef,
        abi: ManifestBlobRef,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
        access_rules: AccessRules,
    },

    /// Publish a package with owner.
    PublishPackageWithOwner {
        code: ManifestBlobRef,
        abi: ManifestBlobRef,
        owner_badge: NonFungibleGlobalId,
    },

    BurnResource {
        bucket_id: ManifestBucket,
    },

    RecallResource {
        vault_id: VaultId,
        amount: Decimal,
    },

    SetMetadata {
        entity_address: GlobalAddress,
        key: String,
        value: String,
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
        index: u32,
        key: AccessRuleKey,
        rule: AccessRule,
    },

    MintFungible {
        resource_address: ResourceAddress,
        amount: Decimal,
    },

    MintNonFungible {
        resource_address: ResourceAddress,
        entries: BTreeMap<NonFungibleLocalId, (Vec<u8>, Vec<u8>)>,
    },

    MintUuidNonFungible {
        resource_address: ResourceAddress,
        entries: Vec<(Vec<u8>, Vec<u8>)>,
    },

    CreateFungibleResource {
        divisibility: u8,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        initial_supply: Option<Decimal>,
    },

    CreateFungibleResourceWithOwner {
        divisibility: u8,
        metadata: BTreeMap<String, String>,
        owner_badge: NonFungibleGlobalId,
        initial_supply: Option<Decimal>,
    },

    CreateNonFungibleResource {
        id_type: NonFungibleIdType,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        initial_supply: Option<BTreeMap<NonFungibleLocalId, (Vec<u8>, Vec<u8>)>>,
    },

    CreateNonFungibleResourceWithOwner {
        id_type: NonFungibleIdType,
        metadata: BTreeMap<String, String>,
        owner_badge: NonFungibleGlobalId,
        initial_supply: Option<BTreeMap<NonFungibleLocalId, (Vec<u8>, Vec<u8>)>>,
    },

    CreateValidator {
        key: EcdsaSecp256k1PublicKey,
        owner_access_rule: AccessRule,
    },

    CreateAccessController {
        controlled_asset: ManifestBucket,
        primary_role: AccessRule,
        recovery_role: AccessRule,
        confirmation_role: AccessRule,
        timed_recovery_delay_in_minutes: Option<u32>,
    },

    CreateIdentity {
        access_rule: AccessRule,
    },

    AssertAccessRule {
        access_rule: AccessRule,
    },

    ///
    /// Buckets and proofs in arguments moves from transaction context to the callee.
    CallFunction {
        package_address: PackageAddress,
        blueprint_name: String,
        function_name: String,
        args: Vec<u8>,
    },

    /// Calls a method.
    ///
    /// Buckets and proofs in arguments moves from transaction context to the callee.
    CallMethod {
        component_address: ComponentAddress,
        method_name: String,
        args: Vec<u8>,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum Instruction {
    Basic(BasicInstruction),
    System(NativeInvocation),
}

impl From<BasicInstruction> for Instruction {
    fn from(i: BasicInstruction) -> Self {
        Instruction::Basic(i)
    }
}

impl From<NativeInvocation> for Instruction {
    fn from(i: NativeInvocation) -> Self {
        Instruction::System(i)
    }
}
