use super::*;
use crate::model::*;
use crate::*;

// TODO: Remove and replace with real HeapRENodes
#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ScryptoRENode {
    Component(PackageAddress, String, Vec<u8>),
    KeyValueStore,
}

// TODO: Remove when better type system implemented
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
pub enum RENodeType {
    Bucket,
    Proof,
    AuthZoneStack,
    Worktop,
    GlobalAccount,
    GlobalComponent,
    GlobalResourceManager,
    GlobalPackage,
    GlobalEpochManager,
    GlobalValidator,
    GlobalClock,
    GlobalAccessController,
    GlobalIdentity,
    KeyValueStore,
    NonFungibleStore,
    Component,
    Vault,
    ResourceManager,
    Package,
    EpochManager,
    Validator,
    Clock,
    Identity,
    TransactionRuntime,
    Logger,
    AccessController,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
pub enum RENodeId {
    Bucket(BucketId),
    Proof(ProofId),
    AuthZoneStack(AuthZoneStackId),
    FeeReserve(FeeReserveId),
    Worktop,
    Logger,
    Global(GlobalAddress),
    KeyValueStore(KeyValueStoreId),
    NonFungibleStore(NonFungibleStoreId),
    Component(ComponentId),
    Vault(VaultId),
    ResourceManager(ResourceManagerId),
    Package(PackageId),
    EpochManager(EpochManagerId),
    Identity(IdentityId),
    Clock(ClockId),
    Validator(ValidatorId),
    TransactionRuntime(TransactionRuntimeId),
    AccessController(AccessControllerId),
}

impl Into<[u8; 36]> for RENodeId {
    fn into(self) -> [u8; 36] {
        match self {
            RENodeId::KeyValueStore(id) => id,
            RENodeId::NonFungibleStore(id) => id,
            RENodeId::Vault(id) => id,
            RENodeId::Component(id) => id,
            RENodeId::ResourceManager(id) => id,
            RENodeId::Package(id) => id,
            RENodeId::EpochManager(id) => id,
            RENodeId::Identity(id) => id,
            RENodeId::Validator(id) => id,
            RENodeId::Clock(id) => id,
            RENodeId::AccessController(id) => id,
            _ => panic!("Not a stored id"),
        }
    }
}

impl Into<u32> for RENodeId {
    fn into(self) -> u32 {
        match self {
            RENodeId::Bucket(id) => id,
            RENodeId::Proof(id) => id,
            RENodeId::AuthZoneStack(id) => id,
            RENodeId::FeeReserve(id) => id,
            RENodeId::TransactionRuntime(id) => id,
            _ => panic!("Not a transient id"),
        }
    }
}

impl Into<ComponentAddress> for RENodeId {
    fn into(self) -> ComponentAddress {
        match self {
            RENodeId::Global(GlobalAddress::Component(address)) => address,
            _ => panic!("Not a component address"),
        }
    }
}

impl Into<PackageAddress> for RENodeId {
    fn into(self) -> PackageAddress {
        match self {
            RENodeId::Global(GlobalAddress::Package(package_address)) => package_address,
            _ => panic!("Not a package address"),
        }
    }
}

impl Into<ResourceAddress> for RENodeId {
    fn into(self) -> ResourceAddress {
        match self {
            RENodeId::Global(GlobalAddress::Resource(resource_address)) => resource_address,
            _ => panic!("Not a resource address"),
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
    PartialOrd,
    Ord,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
pub enum GlobalAddress {
    Component(ComponentAddress),
    Package(PackageAddress),
    Resource(ResourceAddress),
}

impl Into<ComponentAddress> for GlobalAddress {
    fn into(self) -> ComponentAddress {
        match self {
            GlobalAddress::Component(component_address) => component_address,
            _ => panic!("Not a component address"),
        }
    }
}

impl Into<PackageAddress> for GlobalAddress {
    fn into(self) -> PackageAddress {
        match self {
            GlobalAddress::Package(package_address) => package_address,
            _ => panic!("Not a package address"),
        }
    }
}

impl Into<ResourceAddress> for GlobalAddress {
    fn into(self) -> ResourceAddress {
        match self {
            GlobalAddress::Resource(resource_address) => resource_address,
            _ => panic!("Not a resource address"),
        }
    }
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AuthZoneStackOffset {
    AuthZoneStack,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AccessRulesChainOffset {
    AccessRulesChain,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MetadataOffset {
    Metadata,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ComponentOffset {
    Info,
    State,
    RoyaltyConfig,
    RoyaltyAccumulator,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PackageOffset {
    Info,
    RoyaltyConfig,
    RoyaltyAccumulator,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GlobalOffset {
    Global,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceManagerOffset {
    ResourceManager,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeyValueStoreOffset {
    Entry(Vec<u8>),
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
pub enum NonFungibleStoreOffset {
    Entry(NonFungibleLocalId),
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VaultOffset {
    Vault,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EpochManagerOffset {
    EpochManager,
    CurrentValidatorSet,
    PreparingValidatorSet,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ValidatorOffset {
    Validator,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FeeReserveOffset {
    FeeReserve,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BucketOffset {
    Bucket,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProofOffset {
    Proof,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WorktopOffset {
    Worktop,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LoggerOffset {
    Logger,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ClockOffset {
    CurrentTimeRoundedToMinutes,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TransactionRuntimeOffset {
    TransactionRuntime,
}

#[derive(Debug, Clone, Categorize, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AccessControllerOffset {
    AccessController,
}

/// Specifies a specific Substate into a given RENode
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
pub enum SubstateOffset {
    Global(GlobalOffset),
    AuthZoneStack(AuthZoneStackOffset),
    FeeReserve(FeeReserveOffset),
    Component(ComponentOffset),
    AccessRulesChain(AccessRulesChainOffset),
    VaultAccessRulesChain(AccessRulesChainOffset),
    Metadata(MetadataOffset),
    Package(PackageOffset),
    ResourceManager(ResourceManagerOffset),
    KeyValueStore(KeyValueStoreOffset),
    NonFungibleStore(NonFungibleStoreOffset),
    Vault(VaultOffset),
    EpochManager(EpochManagerOffset),
    Validator(ValidatorOffset),
    Bucket(BucketOffset),
    Proof(ProofOffset),
    Worktop(WorktopOffset),
    Logger(LoggerOffset),
    Clock(ClockOffset),
    TransactionRuntime(TransactionRuntimeOffset),
    AccessController(AccessControllerOffset),
}

/// TODO: separate space addresses?
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
)]
pub struct SubstateId(pub RENodeId, pub SubstateOffset);
