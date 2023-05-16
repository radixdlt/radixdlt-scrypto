use radix_engine_common::data::scrypto::model::*;
use radix_engine_interface::data::manifest::{model::*, ManifestValue};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::*;
use radix_engine_interface::*;
use sbor::rust::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor)]
pub enum Instruction {
    //==============
    // Worktop
    //==============
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

    //==============
    // Auth zone
    //==============
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

    /// Drop all virtual proofs (can only be auth zone proofs).
    #[sbor(discriminator(INSTRUCTION_CLEAR_SIGNATURE_PROOFS_DISCRIMINATOR))]
    ClearSignatureProofs,

    //==============
    // Named bucket
    //==============
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

    #[sbor(discriminator(INSTRUCTION_BURN_RESOURCE_DISCRIMINATOR))]
    BurnResource { bucket_id: ManifestBucket },

    //==============
    // Named proof
    //==============
    /// Clones a proof.
    #[sbor(discriminator(INSTRUCTION_CLONE_PROOF_DISCRIMINATOR))]
    CloneProof { proof_id: ManifestProof },

    /// Drops a proof.
    #[sbor(discriminator(INSTRUCTION_DROP_PROOF_DISCRIMINATOR))]
    DropProof { proof_id: ManifestProof },

    //==============
    // Invocation
    //==============
    #[sbor(discriminator(INSTRUCTION_CALL_FUNCTION_DISCRIMINATOR))]
    CallFunction {
        package_address: PackageAddress,
        blueprint_name: String,
        function_name: String,
        args: ManifestValue,
    },

    #[sbor(discriminator(INSTRUCTION_CALL_METHOD_DISCRIMINATOR))]
    CallMethod {
        address: GlobalAddress,
        method_name: String,
        args: ManifestValue,
    },

    #[sbor(discriminator(INSTRUCTION_RECALL_RESOURCE_DISCRIMINATOR))]
    RecallResource {
        vault_id: InternalAddress,
        amount: Decimal,
    },

    //==============
    // Complex
    //==============
    /// Drops all proofs, both named proofs and auth zone proofs.
    #[sbor(discriminator(INSTRUCTION_DROP_ALL_PROOFS_DISCRIMINATOR))]
    DropAllProofs,
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

//==============
// Worktop
//==============
pub const INSTRUCTION_TAKE_FROM_WORKTOP_DISCRIMINATOR: u8 = 0x00;
pub const INSTRUCTION_TAKE_NON_FUNGIBLES_FROM_WORKTOP_DISCRIMINATOR: u8 = 0x01;
pub const INSTRUCTION_TAKE_ALL_FROM_WORKTOP_DISCRIMINATOR: u8 = 0x02;
pub const INSTRUCTION_RETURN_TO_WORKTOP_DISCRIMINATOR: u8 = 0x03;
pub const INSTRUCTION_ASSERT_WORKTOP_CONTAINS_DISCRIMINATOR: u8 = 0x04;
pub const INSTRUCTION_ASSERT_WORKTOP_CONTAINS_NON_FUNGIBLES_DISCRIMINATOR: u8 = 0x05;

//==============
// Auth zone
//==============
pub const INSTRUCTION_POP_FROM_AUTH_ZONE_DISCRIMINATOR: u8 = 0x10;
pub const INSTRUCTION_PUSH_TO_AUTH_ZONE_DISCRIMINATOR: u8 = 0x11;
pub const INSTRUCTION_CLEAR_AUTH_ZONE_DISCRIMINATOR: u8 = 0x12;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_DISCRIMINATOR: u8 = 0x13;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_AMOUNT_DISCRIMINATOR: u8 = 0x14;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_NON_FUNGIBLES_DISCRIMINATOR: u8 = 0x15;
pub const INSTRUCTION_CREATE_PROOF_FROM_AUTH_ZONE_OF_ALL_DISCRIMINATOR: u8 = 0x16;
pub const INSTRUCTION_CLEAR_SIGNATURE_PROOFS_DISCRIMINATOR: u8 = 0x17;

//==============
// Named bucket
//==============
pub const INSTRUCTION_CREATE_PROOF_FROM_BUCKET_DISCRIMINATOR: u8 = 0x20;
pub const INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_AMOUNT_DISCRIMINATOR: u8 = 0x21;
pub const INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_NON_FUNGIBLES_DISCRIMINATOR: u8 = 0x22;
pub const INSTRUCTION_CREATE_PROOF_FROM_BUCKET_OF_ALL_DISCRIMINATOR: u8 = 0x23;
pub const INSTRUCTION_BURN_RESOURCE_DISCRIMINATOR: u8 = 0x24;

//==============
// Named proof
//==============
pub const INSTRUCTION_CLONE_PROOF_DISCRIMINATOR: u8 = 0x30;
pub const INSTRUCTION_DROP_PROOF_DISCRIMINATOR: u8 = 0x31;

//==============
// Invocation
//==============
pub const INSTRUCTION_CALL_FUNCTION_DISCRIMINATOR: u8 = 0x40;
pub const INSTRUCTION_CALL_METHOD_DISCRIMINATOR: u8 = 0x41;
pub const INSTRUCTION_RECALL_RESOURCE_DISCRIMINATOR: u8 = 0x42;

//==============
// Complex
//==============
pub const INSTRUCTION_DROP_ALL_PROOFS_DISCRIMINATOR: u8 = 0x50;
