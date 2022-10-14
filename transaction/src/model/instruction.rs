use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::core::{
    Blob, NativeFunctionIdent, NativeMethodIdent, ScryptoFunctionIdent, ScryptoMethodIdent,
};
use scrypto::engine::types::*;
use scrypto::math::*;
use scrypto::resource::{NonFungibleId, ResourceAddress};

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum Instruction {
    /// Takes resource from worktop.
    TakeFromWorktop { resource_address: ResourceAddress },

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
    ReturnToWorktop { bucket_id: BucketId },

    /// Asserts worktop contains resource.
    AssertWorktopContains { resource_address: ResourceAddress },

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
    PushToAuthZone { proof_id: ProofId },

    /// Drops all proofs in the auth zone
    ClearAuthZone,

    // TODO: do we need `CreateProofFromWorktop`, to avoid taking resource out and then creating proof?
    /// Creates a proof from the auth zone
    CreateProofFromAuthZone { resource_address: ResourceAddress },

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
    CreateProofFromBucket { bucket_id: BucketId },

    /// Clones a proof.
    CloneProof { proof_id: ProofId },

    /// Drops a proof.
    DropProof { proof_id: ProofId },

    /// Drops all of the proofs in the transaction.
    DropAllProofs,

    /// Calls a scrypto function.
    ///
    /// Buckets and proofs in arguments moves from transaction context to the callee.
    CallFunction {
        function_ident: ScryptoFunctionIdent,
        args: Vec<u8>,
    },

    /// Calls a scrypto method.
    ///
    /// Buckets and proofs in arguments moves from transaction context to the callee.
    CallMethod {
        method_ident: ScryptoMethodIdent,
        args: Vec<u8>,
    },

    /// Calls a native function.
    ///
    /// Buckets and proofs in arguments moves from transaction context to the callee.
    CallNativeFunction {
        function_ident: NativeFunctionIdent,
        args: Vec<u8>,
    },

    /// Calls a native method.
    ///
    /// Buckets and proofs in arguments moves from transaction context to the callee.
    CallNativeMethod {
        method_ident: NativeMethodIdent,
        args: Vec<u8>,
    },

    /// Publishes a package.
    PublishPackage { code: Blob, abi: Blob },
}
