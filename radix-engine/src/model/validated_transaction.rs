use sbor::rust::collections::BTreeSet;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::crypto::*;
use scrypto::engine::types::*;
use scrypto::values::*;

/// Represents a validated transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedTransaction {
    pub raw_hash: Hash,
    pub instructions: Vec<ValidatedInstruction>,
    pub signers: Vec<EcdsaPublicKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatedInstruction {
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
    CallFunction {
        package_address: PackageAddress,
        blueprint_name: String,
        method_name: String,
        arg: ScryptoValue,
    },
    CallMethod {
        component_address: ComponentAddress,
        method_name: String,
        arg: ScryptoValue,
    },
    CallMethodWithAllResources {
        component_address: ComponentAddress,
        method: String,
    },
    PublishPackage {
        package: Vec<u8>,
    },
}
