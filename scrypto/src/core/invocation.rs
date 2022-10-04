use sbor::rust::string::*;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::component::PackageAddress;
use crate::engine::types::RENodeId;

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum FnIdent {
    Function(FunctionIdent),
    Method(ReceiverMethodIdent),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, TypeId, Encode, Decode)]
pub enum FunctionIdent {
    Scrypto {
        package_address: PackageAddress,
        blueprint_name: String,
        ident: String,
    },
    Native(NativeFunction),
}

#[derive(Debug, Clone, Eq, PartialEq, Copy, TypeId, Encode, Decode)]
pub enum Receiver {
    Consumed(RENodeId),
    Ref(RENodeId),
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum MethodIdent {
    Scrypto(String),
    Native(NativeMethod),
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub struct ReceiverMethodIdent {
    pub receiver: Receiver,
    pub method_ident: MethodIdent,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum NativeMethod {
    Component(ComponentMethod),
    System(SystemMethod),
    AuthZone(AuthZoneMethod),
    ResourceManager(ResourceManagerMethod),
    Bucket(BucketMethod),
    Vault(VaultMethod),
    Proof(ProofMethod),
    Worktop(WorktopMethod),
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum NativeFunction {
    System(SystemFunction),
    ResourceManager(ResourceManagerFunction),
    Package(PackageFunction),
    TransactionProcessor(TransactionProcessorFunction),
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum ComponentMethod {
    AddAccessCheck,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum SystemFunction {
    Create,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum SystemMethod {
    GetTransactionHash,
    GetCurrentEpoch,
    SetEpoch,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum AuthZoneMethod {
    Pop,
    Push,
    CreateProof,
    CreateProofByAmount,
    CreateProofByIds,
    Clear,
    Drain,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum ResourceManagerFunction {
    Create,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum ResourceManagerMethod {
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
pub enum BucketMethod {
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
pub enum VaultMethod {
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
pub enum ProofMethod {
    Clone,
    GetAmount,
    GetNonFungibleIds,
    GetResourceAddress,
    Drop,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum WorktopMethod {
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
pub enum PackageFunction {
    Publish,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TypeId, Encode, Decode, Describe, PartialOrd, Ord,
)]
pub enum TransactionProcessorFunction {
    Run,
}

// TODO: Remove and replace with real HeapRENodes
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ScryptoRENode {
    Component(PackageAddress, String, Vec<u8>),
    KeyValueStore,
}
