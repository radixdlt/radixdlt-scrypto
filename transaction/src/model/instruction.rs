use radix_engine_interface::api::types::*;
use radix_engine_interface::crypto::*;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::scrypto;
use sbor::rust::collections::BTreeMap;
use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
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

    /// Publish a package with owner.
    PublishPackageWithOwner {
        code: Blob,
        abi: Blob,
        owner_badge: NonFungibleAddress,
    },

    BurnResource {
        bucket_id: BucketId,
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
        entries: BTreeMap<NonFungibleId, (Vec<u8>, Vec<u8>)>,
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
        owner_badge: NonFungibleAddress,
        initial_supply: Option<Decimal>,
    },

    CreateNonFungibleResource {
        id_type: NonFungibleIdType,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        initial_supply: Option<BTreeMap<NonFungibleId, (Vec<u8>, Vec<u8>)>>,
    },

    CreateNonFungibleResourceWithOwner {
        id_type: NonFungibleIdType,
        metadata: BTreeMap<String, String>,
        owner_badge: NonFungibleAddress,
        initial_supply: Option<BTreeMap<NonFungibleId, (Vec<u8>, Vec<u8>)>>,
    },

    CreateValidator {
        key: EcdsaSecp256k1PublicKey,
    },

    RegisterValidator {
        validator_address: SystemAddress, // TODO: Replace this with ValidatorAddress
    },

    UnregisterValidator {
        validator_address: SystemAddress, // TODO: Replace this with ValidatorAddress
    },

    StakeValidator {
        validator_address: SystemAddress, // TODO: Replace this with ValidatorAddress
        stake: BucketId,
    },

    UnstakeValidator {
        validator_address: SystemAddress, // TODO: Replace this with ValidatorAddress
        lp_tokens: BucketId,
    },

    ClaimXrd {
        validator_address: SystemAddress, // TODO: Replace this with ValidatorAddress
        claim_bucket: BucketId,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
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
