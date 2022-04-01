use sbor::*;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::*;
use scrypto::engine::types::*;
use scrypto::rust::collections::BTreeSet;
use scrypto::rust::string::String;
use scrypto::rust::vec::Vec;

use crate::model::ValidatedData;

/// Represents a signed or signed transaction, parsed but not validated.
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct Transaction {
    pub instructions: Vec<Instruction>,
}

/// Represents an instruction
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
    ReturnToWorktop { bucket_id: BucketId },

    /// Asserts worktop contains resource.
    AssertWorktopContains { resource_def_id: ResourceDefId },

    /// Asserts worktop contains resource by at least the given amount.
    AssertWorktopContainsByAmount {
        amount: Decimal,
        resource_def_id: ResourceDefId,
    },

    /// Asserts worktop contains resource by at least the given non-fungible IDs.
    AssertWorktopContainsByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
    },

    /// Takes the last proof from the auth zone.
    TakeFromAuthZone,

    /// Adds a proof to the auth zone.
    MoveToAuthZone { proof_id: ProofId },

    /// Drops all proofs in the auth zone
    ClearAuthZone,

    // TODO: do we need `CreateProofFromWorktop`, to avoid taking resource out and then creating proof?
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

    /// Specifies transaction nonce
    Nonce {
        nonce: u64, // TODO: may be replaced with substate id for entropy
    },

    /// Marks the end of transaction with signatures.
    End { signatures: Vec<EcdsaSignature> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedTransaction {
    pub hash: Hash,
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
    ReturnToWorktop {
        bucket_id: BucketId,
    },
    AssertWorktopContains {
        resource_def_id: ResourceDefId,
    },
    AssertWorktopContainsByAmount {
        amount: Decimal,
        resource_def_id: ResourceDefId,
    },
    AssertWorktopContainsByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
    },
    TakeFromAuthZone,
    MoveToAuthZone {
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

impl Transaction {
    pub fn to_vec(&self) -> Vec<u8> {
        scrypto_encode(self)
    }

    pub fn is_signed(&self) -> bool {
        match self.instructions.last() {
            Some(Instruction::End { .. }) => true,
            _ => false,
        }
    }

    pub fn hash(&self) -> Hash {
        let instructions = if self.is_signed() {
            &self.instructions[..self.instructions.len() - 1]
        } else {
            &self.instructions
        };
        let bytes = scrypto_encode(instructions);
        sha256(bytes)
    }

    pub fn sign<T: AsRef<[EcdsaPrivateKey]>>(mut self, private_keys: T) -> Self {
        if self.is_signed() {
            panic!("Transaction already signed!");
        }

        let hash = self.hash();
        let signatures = private_keys
            .as_ref()
            .iter()
            .map(|sk| sk.sign(&hash))
            .collect();

        self.instructions.push(Instruction::End { signatures });
        self
    }
}
