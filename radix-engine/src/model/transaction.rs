use sbor::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;

use crate::model::ValidatedData;

/// Represents an unvalidated transaction.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Transaction {
    pub instructions: Vec<Instruction>,
}

/// Represents an unvalidated instruction in transaction
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum Instruction {
    /// Takes resource from worktop.
    TakeFromWorktop { resource_def_id: ResourceDefId },

    /// Takes resource from worktop by the given amount.
    TakeFromWorktopByAmount {
        amount: Decimal,
        resource_def_id: ResourceDefId,
    },

    /// Takes resource from worktop by the given non-fungible IDs.
    TakeFromWorktopByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
    },

    /// Returns a bucket of resource to worktop.
    AddToWorktop { bucket_id: BucketId },

    /// Asserts worktop contains resource.
    AssertWorktop { resource_def_id: ResourceDefId },

    /// Asserts worktop contains resource by at least the given amount.
    AssertWorktopByAmount {
        amount: Decimal,
        resource_def_id: ResourceDefId,
    },

    /// Asserts worktop contains resource by at least the given non-fungible IDs.
    AssertWorktopByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
    },

    /// Takes the last proof from the auth zone.
    TakeFromAuthZone,

    /// Adds a proof to the auth zone.
    AddToAuthZone { proof_id: ProofId },

    /// Drops all proofs in the auth zone
    ClearAuthZone,

    // TODO: do we need `CreateProofFromWorktop`, to avoid taking and creating proof?
    /// Creates a proof from the auth zone
    CreateProofFromAuthZone { resource_def_id: ResourceDefId },

    /// Creates a proof from the auth zone, by the given amount
    CreateProofFromAuthZoneByAmount {
        amount: Decimal,
        resource_def_id: ResourceDefId,
    },

    /// Creates a proof from the auth zone, by the given non-fungible IDs.
    CreateProofFromAuthZoneByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
    },

    /// Creates a proof from a bucket.
    CreateProofFromBucket { bucket_id: BucketId },

    /// Clones a proof.
    CloneProof { proof_id: ProofId },

    /// Drops a proof.
    DropProof { proof_id: ProofId },

    /// Calls a blueprint function.
    ///
    /// Buckets and proofs in arguments moves from transaction context to the callee.
    CallFunction {
        package_id: PackageId,
        blueprint_name: String,
        function: String,
        args: Vec<Vec<u8>>,
    },

    /// Calls a component method.
    ///
    /// Buckets and proofs in arguments moves from transaction context to the callee.
    CallMethod {
        component_id: ComponentId,
        method: String,
        args: Vec<Vec<u8>>,
    },

    /// Calls a component method with all resources owned by the transaction.
    CallMethodWithAllResources {
        component_id: ComponentId,
        method: String,
    },

    /// Publishes a package.
    PublishPackage { code: Vec<u8> },

    /// Marks the end of transaction from signatures.
    /// TODO: replace public key from signature.
    End { signatures: Vec<EcdsaPublicKey> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedTransaction {
    pub instructions: Vec<ValidatedInstruction>,
    pub signers: Vec<EcdsaPublicKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatedInstruction {
    TakeFromWorktop {
        resource_def_id: ResourceDefId,
    },
    TakeFromWorktopByAmount {
        amount: Decimal,
        resource_def_id: ResourceDefId,
    },
    TakeFromWorktopByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
    },
    AddToWorktop {
        bucket_id: BucketId,
    },
    AssertWorktop {
        resource_def_id: ResourceDefId,
    },
    AssertWorktopByAmount {
        amount: Decimal,
        resource_def_id: ResourceDefId,
    },
    AssertWorktopByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
    },
    TakeFromAuthZone,
    AddToAuthZone {
        proof_id: ProofId,
    },
    ClearAuthZone,
    CreateProofFromAuthZone {
        resource_def_id: ResourceDefId,
    },
    CreateProofFromAuthZoneByAmount {
        amount: Decimal,
        resource_def_id: ResourceDefId,
    },
    CreateProofFromAuthZoneByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
    },
    CreateProofFromBucket {
        bucket_id: BucketId,
    },
    CloneProof {
        proof_id: ProofId,
    },
    DropProof {
        proof_id: ProofId,
    },
    CallFunction {
        package_id: PackageId,
        blueprint_name: String,
        function: String,
        args: Vec<ValidatedData>,
    },
    CallMethod {
        component_id: ComponentId,
        method: String,
        args: Vec<ValidatedData>,
    },
    CallMethodWithAllResources {
        component_id: ComponentId,
        method: String,
    },
    PublishPackage {
        code: Vec<u8>,
    },
}
