use sbor::rust::string::String;
use sbor::*;

use crate::api::types::*;
use crate::model::*;
use crate::scrypto;
use crate::Describe;

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum FnIdentifier {
    Scrypto(ScryptoFnIdentifier),
    Native(NativeFn),
}

impl FnIdentifier {
    pub fn is_scrypto_or_transaction(&self) -> bool {
        matches!(
            self,
            FnIdentifier::Scrypto(..)
                | FnIdentifier::Native(NativeFn::Function(NativeFunction::TransactionProcessor(
                    TransactionProcessorFunction::Run
                )))
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ScryptoFnIdentifier {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub ident: String,
}

impl ScryptoFnIdentifier {
    pub fn new(package_address: PackageAddress, blueprint_name: String, ident: String) -> Self {
        Self {
            package_address,
            blueprint_name,
            ident,
        }
    }

    pub fn package_address(&self) -> &PackageAddress {
        &self.package_address
    }

    pub fn blueprint_name(&self) -> &String {
        &self.blueprint_name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub enum NativeFn {
    Method(NativeMethod),
    Function(NativeFunction),
}

// Native function enum used by Kernel SystemAPI and WASM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub enum NativeMethod {
    AccessRulesChain(AccessRulesChainMethod),
    Component(ComponentMethod), // TODO: investigate whether to make royalty universal and take any "receiver".
    Package(PackageMethod),
    Metadata(MetadataMethod),
    EpochManager(EpochManagerMethod),
    AuthZoneStack(AuthZoneStackMethod),
    ResourceManager(ResourceManagerMethod),
    Bucket(BucketMethod),
    Vault(VaultMethod),
    Proof(ProofMethod),
    Worktop(WorktopMethod),
    Clock(ClockMethod),
    Logger(LoggerMethod),
    TransactionHash(TransactionHashMethod),
}

impl Into<FnIdentifier> for NativeMethod {
    fn into(self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Method(self))
    }
}

// Native method enum used by Kernel SystemAPI and WASM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[scrypto(TypeId, Encode, Decode, Describe)]
pub enum NativeFunction {
    Component(ComponentFunction),
    EpochManager(EpochManagerFunction),
    ResourceManager(ResourceManagerFunction),
    Package(PackageFunction),
    TransactionProcessor(TransactionProcessorFunction),
    Clock(ClockFunction),
}

impl Into<FnIdentifier> for NativeFunction {
    fn into(self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::Function(self))
    }
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
pub enum AccessRulesChainMethod {
    AddAccessCheck,
    SetMethodAccessRule,
    SetGroupAccessRule,
    SetMethodMutability,
    SetGroupMutability,
    GetLength,
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
pub enum MetadataMethod {
    Set,
    Get,
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
pub enum ComponentFunction {
    Globalize,
    GlobalizeWithOwner,
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
pub enum ComponentMethod {
    SetRoyaltyConfig,
    ClaimRoyalty,
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
pub enum PackageMethod {
    SetRoyaltyConfig,
    ClaimRoyalty,
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
    NextRound,
    SetEpoch,
    RegisterValidator,
    UnregisterValidator,
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
pub enum AuthZoneStackMethod {
    Pop,
    Push,
    CreateProof,
    CreateProofByAmount,
    CreateProofByIds,
    Clear,
    Drain,
    AssertAccessRule,
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
    Mint,
    Burn,
    UpdateVaultAuth,
    LockAuth,
    UpdateNonFungibleData,
    GetNonFungible,
    GetResourceType,
    GetTotalSupply,
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
    Recall,
    RecallNonFungibles,
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
pub enum ClockFunction {
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
pub enum ClockMethod {
    SetCurrentTime,
    GetCurrentTime,
    CompareCurrentTime,
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
pub enum LoggerMethod {
    Log,
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
pub enum TransactionHashMethod {
    Get,
    GenerateUuid,
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
