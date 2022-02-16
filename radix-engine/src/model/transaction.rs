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
    /// Takes fixed amount resource from worktop.
    TakeFromWorktop {
        amount: Decimal,
        resource_def_ref: ResourceDefRef,
    },

    /// Takes all of a given resource from worktop.
    TakeAllFromWorktop { resource_def_ref: ResourceDefRef },

    /// Takes non-fungibles from worktop.
    TakeNonFungiblesFromWorktop {
        keys: BTreeSet<NonFungibleKey>,
        resource_def_ref: ResourceDefRef,
    },

    /// Returns resource to worktop.
    ReturnToWorktop { bucket_id: BucketId },

    /// Asserts worktop contains at least this amount.
    AssertWorktopContains {
        amount: Decimal,
        resource_def_ref: ResourceDefRef,
    },

    /// Creates a proof.
    CreateProof { bucket_id: BucketId },

    /// Clones a proof.
    CloneProof { proof_id: ProofId },

    /// Drops a proof.
    DropProof { proof_id: ProofId },

    /// Calls a blueprint function.
    ///
    /// Buckets and proofs in arguments moves from transaction context to the callee.
    CallFunction {
        package_ref: PackageRef,
        blueprint_name: String,
        function: String,
        args: Vec<Vec<u8>>,
    },

    /// Calls a component method.
    ///
    /// Buckets and proofs in arguments moves from transaction context to the callee.
    CallMethod {
        component_ref: ComponentRef,
        method: String,
        args: Vec<Vec<u8>>,
    },

    /// With method with all resources from transaction context.
    CallMethodWithAllResources {
        component_ref: ComponentRef,
        method: String,
    },

    /// Marks the end of transaction with signatures.
    /// TODO: replace public key with signature.
    End { signatures: Vec<EcdsaPublicKey> },
}

#[derive(Debug, Clone)]
pub struct ValidatedTransaction {
    pub instructions: Vec<ValidatedInstruction>,
    pub signers: Vec<EcdsaPublicKey>,
}

#[derive(Debug, Clone)]
pub enum ValidatedInstruction {
    TakeFromWorktop {
        amount: Decimal,
        resource_def_ref: ResourceDefRef,
    },
    TakeAllFromWorktop {
        resource_def_ref: ResourceDefRef,
    },
    TakeNonFungiblesFromWorktop {
        keys: BTreeSet<NonFungibleKey>,
        resource_def_ref: ResourceDefRef,
    },
    ReturnToWorktop {
        bucket_id: BucketId,
    },
    AssertWorktopContains {
        amount: Decimal,
        resource_def_ref: ResourceDefRef,
    },
    CreateProof {
        bucket_id: BucketId,
    },
    CloneProof {
        proof_id: ProofId,
    },
    DropProof {
        proof_id: ProofId,
    },
    CallFunction {
        package_ref: PackageRef,
        blueprint_name: String,
        function: String,
        args: Vec<ValidatedData>,
    },
    CallMethod {
        component_ref: ComponentRef,
        method: String,
        args: Vec<ValidatedData>,
    },
    CallMethodWithAllResources {
        component_ref: ComponentRef,
        method: String,
    },
}
