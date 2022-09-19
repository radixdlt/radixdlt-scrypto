use sbor::rust::collections::BTreeSet;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::component::ComponentAddress;
use scrypto::core::{Blob, FnIdentifier};
use scrypto::crypto::*;
use scrypto::engine::types::*;
use scrypto::math::*;
use scrypto::resource::{NonFungibleId, ResourceAddress};

/// Represents an instruction that can be executed by Radix Engine.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum ExecutableInstruction {
    TakeFromWorktop {
        resource_address: ResourceAddress,
    },
    TakeFromWorktopByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },
    TakeFromWorktopByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    },
    ReturnToWorktop {
        bucket_id: BucketId,
    },
    AssertWorktopContains {
        resource_address: ResourceAddress,
    },
    AssertWorktopContainsByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },
    AssertWorktopContainsByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    },
    PopFromAuthZone,
    PushToAuthZone {
        proof_id: ProofId,
    },
    ClearAuthZone,
    CreateProofFromAuthZone {
        resource_address: ResourceAddress,
    },
    CreateProofFromAuthZoneByAmount {
        amount: Decimal,
        resource_address: ResourceAddress,
    },
    CreateProofFromAuthZoneByIds {
        ids: BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
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
    DropAllProofs,
    CallFunction {
        fn_identifier: FnIdentifier,
        args: Vec<u8>,
    },
    CallMethod {
        component_address: ComponentAddress,
        method_name: String,
        args: Vec<u8>,
    },
    PublishPackage {
        code: Blob,
        abi: Blob,
    },
}

/// A common trait for all transactions that can be executed by Radix Engine.
pub trait ExecutableTransaction {
    /// Returns the transaction hash, which must be globally unique.
    fn transaction_hash(&self) -> Hash;

    /// Returns the manifest size.
    fn manifest_instructions_size(&self) -> u32;

    /// Returns the limit of cost units consumable
    fn cost_unit_limit(&self) -> u32;

    /// Returns the tip percentage
    fn tip_percentage(&self) -> u32;

    /// Returns the instructions to execute.
    fn instructions(&self) -> &[ExecutableInstruction];

    /// Returns the public key of signers.
    fn signer_public_keys(&self) -> &[PublicKey];

    fn blobs(&self) -> &[Vec<u8>];
}
