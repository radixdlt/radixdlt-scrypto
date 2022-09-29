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
    Native(NativeFunctionFnIdent),
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum MethodFnIdent {
    Scrypto(String),
    Native(NativeMethodFnIdent),
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub struct MethodIdent {
    pub receiver: Receiver,
    pub fn_ident: MethodFnIdent,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum NativeMethodFnIdent {
    Component(ComponentMethodFnIdent),
    System(SystemMethodFnIdent),
    AuthZone(AuthZoneMethodFnIdent),
    ResourceManager(ResourceManagerMethodFnIdent),
    Bucket(BucketMethodFnIdent),
    Vault(VaultMethodFnIdent),
    Proof(ProofMethodFnIdent),
    Worktop(WorktopMethodFnIdent),
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum NativeFunctionFnIdent {
    System(SystemFunctionFnIdent),
    ResourceManager(ResourceManagerFunctionFnIdent),
    Package(PackageFunctionFnIdent),
    TransactionProcessor(TransactionProcessorFunctionFnIdent),
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum ComponentMethodFnIdent {
    AddAccessCheck,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum SystemFunctionFnIdent {
    Create,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum SystemMethodFnIdent {
    GetTransactionHash,
    GetCurrentEpoch,
    SetEpoch,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum AuthZoneMethodFnIdent {
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
pub enum ResourceManagerFunctionFnIdent {
    Create,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum ResourceManagerMethodFnIdent {
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
pub enum BucketMethodFnIdent {
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
pub enum VaultMethodFnIdent {
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
pub enum ProofMethodFnIdent {
    Clone,
    GetAmount,
    GetNonFungibleIds,
    GetResourceAddress,
    Drop,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum WorktopMethodFnIdent {
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
pub enum PackageFunctionFnIdent {
    Publish,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum TransactionProcessorFunctionFnIdent {
    Run,
}

// TODO: Remove and replace with real HeapRENodes
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ScryptoRENode {
    Component(PackageAddress, String, Vec<u8>),
    KeyValueStore,
}
