use sbor::rust::string::*;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::component::PackageAddress;
use crate::engine::types::RENodeId;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum FnIdent {
    Function(FunctionIdent),
    Method(MethodIdent),
}

impl FnIdent {
    pub fn fn_identifier(&self) -> &FunctionIdent {
        match self {
            FnIdent::Function(fn_identifier) => &fn_identifier,
            FnIdent::Method(MethodIdent { fn_ident: fn_identifier, .. }) => &fn_identifier,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy, TypeId, Encode, Decode)]
pub enum Receiver {
    Consumed(RENodeId),
    Ref(RENodeId),
    CurrentAuthZone,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, TypeId, Encode, Decode)]
pub enum FunctionIdent {
    Scrypto {
        package_address: PackageAddress,
        blueprint_name: String,
        ident: String,
    },
    Native(NativeFnIdentifier),
}

/*
#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum MethodFnIdent {
    Scrypto {
        package_address: PackageAddress,
        blueprint_name: String,
        ident: String,
    },
    Native(NativeFnIdentifier),
}
 */

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub struct MethodIdent {
    pub receiver: Receiver,
    pub fn_ident: FunctionIdent,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum NativeFnIdentifier {
    Component(ComponentFnIdentifier),
    System(SystemFnIdentifier),
    AuthZone(AuthZoneFnIdentifier),
    ResourceManager(ResourceManagerFnIdentifier),
    Bucket(BucketFnIdentifier),
    Vault(VaultFnIdentifier),
    Proof(ProofFnIdentifier),
    Worktop(WorktopFnIdentifier),
    Package(PackageFnIdentifier),
    TransactionProcessor(TransactionProcessorFnIdentifier),
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum ComponentFnIdentifier {
    AddAccessCheck,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum SystemFnIdentifier {
    Create,
    GetTransactionHash,
    GetCurrentEpoch,
    SetEpoch,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum AuthZoneFnIdentifier {
    Pop,
    Push,
    CreateProof,
    CreateProofByAmount,
    CreateProofByIds,
    Clear,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum ResourceManagerFnIdentifier {
    Create,
    UpdateAuth,
    LockAuth,
    Mint,
    UpdateNonFungibleData,
    GetNonFungible,
    GetMetadata,
    GetResourceType,
    GetTotalSupply,
    UpdateMetadata,
    NonFungibleExists,
    CreateBucket,
    CreateVault,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum BucketFnIdentifier {
    Burn,
    Take,
    TakeNonFungibles,
    Put,
    GetNonFungibleIds,
    GetAmount,
    GetResourceAddress,
    CreateProof,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum VaultFnIdentifier {
    Take,
    LockFee,
    LockContingentFee,
    Put,
    TakeNonFungibles,
    GetAmount,
    GetResourceAddress,
    GetNonFungibleIds,
    CreateProof,
    CreateProofByAmount,
    CreateProofByIds,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum ProofFnIdentifier {
    Clone,
    GetAmount,
    GetNonFungibleIds,
    GetResourceAddress,
    Drop,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum WorktopFnIdentifier {
    TakeAll,
    TakeAmount,
    TakeNonFungibles,
    Put,
    AssertContains,
    AssertContainsAmount,
    AssertContainsNonFungibles,
    Drain,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum PackageFnIdentifier {
    Publish,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum TransactionProcessorFnIdentifier {
    Run,
}

// TODO: Remove and replace with real HeapRENodes
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ScryptoRENode {
    Component(PackageAddress, String, Vec<u8>),
    KeyValueStore,
}
