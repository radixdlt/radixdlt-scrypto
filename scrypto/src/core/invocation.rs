use sbor::rust::string::*;
use sbor::rust::vec::Vec;
use sbor::*;
use strum::*;

use crate::component::ComponentAddress;
use crate::component::PackageAddress;
use crate::engine::types::{ComponentId, RENodeId};

#[derive(Debug, Clone, Eq, PartialEq, Hash, TypeId, Encode, Decode)]
pub struct ScryptoFunctionIdent {
    pub package_ident: ScryptoPackageIdent,
    pub blueprint_name: String,
    pub function_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, TypeId, Encode, Decode)]
pub struct NativeFunctionIdent {
    pub blueprint_name: String,
    pub function_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub struct ScryptoMethodIdent {
    pub receiver: ScryptoReceiver,
    pub method_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub struct NativeMethodIdent {
    pub receiver: Receiver,
    pub method_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, TypeId, Encode, Decode)]
pub enum ScryptoPackageIdent {
    Global(PackageAddress),
    // The following variant is disabled because packages are always globalized ATM.
    // Package(PackageId),
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum ScryptoReceiver {
    Global(ComponentAddress),
    Component(ComponentId),
}

#[derive(Debug, Clone, Eq, PartialEq, Copy, TypeId, Encode, Decode)]
pub enum Receiver {
    Consumed(RENodeId),
    Ref(RENodeId),
}

impl Receiver {
    pub fn node_id(&self) -> RENodeId {
        match self {
            Receiver::Consumed(node_id) | Receiver::Ref(node_id) => *node_id,
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    TypeId,
    Encode,
    Decode,
    Describe,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[strum(serialize_all = "snake_case")]
pub enum ComponentMethod {
    AddAccessCheck,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    TypeId,
    Encode,
    Decode,
    Describe,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[strum(serialize_all = "snake_case")]
pub enum SystemFunction {
    Create,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    TypeId,
    Encode,
    Decode,
    Describe,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[strum(serialize_all = "snake_case")]
pub enum SystemMethod {
    GetTransactionHash,
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
    TypeId,
    Encode,
    Decode,
    Describe,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
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
    TypeId,
    Encode,
    Decode,
    Describe,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[strum(serialize_all = "snake_case")]
pub enum ResourceManagerFunction {
    Create,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    TypeId,
    Encode,
    Decode,
    Describe,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
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
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    TypeId,
    Encode,
    Decode,
    Describe,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[strum(serialize_all = "snake_case")]
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
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    TypeId,
    Encode,
    Decode,
    Describe,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[strum(serialize_all = "snake_case")]
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
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    TypeId,
    Encode,
    Decode,
    Describe,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[strum(serialize_all = "snake_case")]
pub enum ProofMethod {
    Clone,
    GetAmount,
    GetNonFungibleIds,
    GetResourceAddress,
    Drop,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    TypeId,
    Encode,
    Decode,
    Describe,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
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
    TypeId,
    Encode,
    Decode,
    Describe,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
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
    TypeId,
    Encode,
    Decode,
    Describe,
    PartialOrd,
    Ord,
    EnumString,
    EnumVariantNames,
    IntoStaticStr,
    AsRefStr,
    Display,
)]
#[strum(serialize_all = "snake_case")]
pub enum TransactionProcessorFunction {
    Run,
}

// TODO: Remove and replace with real HeapRENodes
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ScryptoRENode {
    Component(PackageAddress, String, Vec<u8>),
    KeyValueStore,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::str::FromStr;

    #[test]
    fn from_into_string() {
        let method = WorktopMethod::TakeAll;
        let name: &str = method.into();
        assert_eq!(name, "take_all");
        let method2 = WorktopMethod::from_str("take_all").unwrap();
        assert_eq!(method2, method);
    }
}
