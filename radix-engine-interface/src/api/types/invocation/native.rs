use crate::api::types::*;
use crate::scrypto;

// Native function identifier used by transaction model
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[scrypto(TypeId, Encode, Decode)]
pub struct NativeFunctionIdent {
    pub blueprint_name: String,
    pub function_name: String,
}

// Native method identifier used by transaction model
#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct NativeMethodIdent {
    pub receiver: RENodeId,
    pub method_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[scrypto(TypeId, Encode, Decode)]
pub enum NativeFn {
    Method(NativeMethod),
    Function(NativeFunction),
}

// Native function enum used by Kernel SystemAPI and WASM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[scrypto(TypeId, Encode, Decode)]
pub enum NativeMethod {
    AccessRules(AccessRulesMethod),
    EpochManager(EpochManagerMethod),
    AuthZone(AuthZoneMethod),
    ResourceManager(ResourceManagerMethod),
    Bucket(BucketMethod),
    Vault(VaultMethod),
    Proof(ProofMethod),
    Worktop(WorktopMethod),
}

// Native method enum used by Kernel SystemAPI and WASM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[scrypto(TypeId, Encode, Decode)]
pub enum NativeFunction {
    EpochManager(EpochManagerFunction),
    ResourceManager(ResourceManagerFunction),
    Package(PackageFunction),
    TransactionProcessor(TransactionProcessorFunction),
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[scrypto(TypeId, Encode, Decode, Describe)]
#[strum(serialize_all = "snake_case")]
pub enum AccessRulesMethod {
    AddAccessCheck,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[scrypto(TypeId, Encode, Decode, Describe)]
#[strum(serialize_all = "snake_case")]
pub enum EpochManagerFunction {
    Create,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[scrypto(TypeId, Encode, Decode, Describe)]
#[strum(serialize_all = "snake_case")]
pub enum EpochManagerMethod {
    GetCurrentEpoch,
    SetEpoch,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[scrypto(TypeId, Encode, Decode, Describe)]
#[strum(serialize_all = "snake_case")]
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
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[scrypto(TypeId, Encode, Decode, Describe)]
#[strum(serialize_all = "snake_case")]
pub enum ResourceManagerFunction {
    Create,
    BurnBucket,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[scrypto(TypeId, Encode, Decode, Describe)]
#[strum(serialize_all = "snake_case")]
pub enum ResourceManagerMethod {
    Burn,
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
    SetResourceAddress,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[scrypto(TypeId, Encode, Decode, Describe)]
#[strum(serialize_all = "snake_case")]
pub enum BucketMethod {
    Take,
    TakeNonFungibles,
    Put,
    GetNonFungibleIds,
    GetAmount,
    GetResourceAddress,
    CreateProof,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[scrypto(TypeId, Encode, Decode, Describe)]
#[strum(serialize_all = "snake_case")]
pub enum VaultMethod {
    Take,
    LockFee,
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
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[scrypto(TypeId, Encode, Decode, Describe)]
#[strum(serialize_all = "snake_case")]
pub enum ProofMethod {
    Clone,
    GetAmount,
    GetNonFungibleIds,
    GetResourceAddress,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[scrypto(TypeId, Encode, Decode, Describe)]
#[strum(serialize_all = "snake_case")]
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
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[scrypto(TypeId, Encode, Decode, Describe)]
#[strum(serialize_all = "snake_case")]
pub enum PackageFunction {
    Publish,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[scrypto(TypeId, Encode, Decode, Describe)]
#[strum(serialize_all = "snake_case")]
pub enum TransactionProcessorFunction {
    Run,
}
