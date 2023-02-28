use super::*;
use crate::api::component::ComponentAddress;
use crate::api::package::PackageAddress;
use crate::blueprints::resource::NonFungibleLocalId;
use crate::blueprints::resource::ResourceAddress;
use crate::*;
use sbor::rust::fmt;
use transaction_data::*;

// TODO: Remove when better type system implemented
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor)]
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
    GlobalAccessController,
    GlobalIdentity,
    KeyValueStore,
    NonFungibleStore,
    Component,
    Vault,
    EpochManager,
    Validator,
    TransactionRuntime,
    Logger,
    AccessController,
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    ScryptoSbor,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
    LegacyDescribe,
)]
pub enum RENodeId {
    Bucket(BucketId),
    Proof(ProofId),
    AuthZoneStack,
    Worktop,
    Logger,
    TransactionRuntime,
    GlobalComponent(ComponentAddress),
    GlobalResourceManager(ResourceAddress),
    GlobalPackage(PackageAddress),
    KeyValueStore(KeyValueStoreId),
    NonFungibleStore(NonFungibleStoreId),
    Component(ComponentId),
    Vault(VaultId),
    EpochManager(EpochManagerId),
    Validator(ValidatorId),
    AccessController(AccessControllerId),
}

impl fmt::Debug for RENodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bucket(id) => f.debug_tuple("Bucket").field(&hex::encode(id)).finish(),
            Self::Proof(id) => f.debug_tuple("Proof").field(&hex::encode(id)).finish(),
            Self::AuthZoneStack => write!(f, "AuthZoneStack"),
            Self::Worktop => write!(f, "Worktop"),
            Self::Logger => write!(f, "Logger"),
            Self::TransactionRuntime => write!(f, "TransactionRuntime"),
            Self::GlobalComponent(address) => {
                f.debug_tuple("GlobalComponent").field(address).finish()
            }
            Self::KeyValueStore(id) => f
                .debug_tuple("KeyValueStore")
                .field(&hex::encode(id))
                .finish(),
            Self::NonFungibleStore(id) => f
                .debug_tuple("NonFungibleStore")
                .field(&hex::encode(id))
                .finish(),
            Self::Component(id) => f.debug_tuple("Component").field(&hex::encode(id)).finish(),
            Self::Vault(id) => f.debug_tuple("Vault").field(&hex::encode(id)).finish(),
            Self::GlobalResourceManager(address) => {
                f.debug_tuple("ResourceManager").field(&address).finish()
            }
            Self::GlobalPackage(address) => f.debug_tuple("GlobalPackage").field(&address).finish(),
            Self::EpochManager(id) => f
                .debug_tuple("EpochManager")
                .field(&hex::encode(id))
                .finish(),
            Self::Validator(id) => f.debug_tuple("Validator").field(&hex::encode(id)).finish(),
            Self::AccessController(id) => f
                .debug_tuple("AccessController")
                .field(&hex::encode(id))
                .finish(),
        }
    }
}

impl Into<[u8; 36]> for RENodeId {
    fn into(self) -> [u8; 36] {
        match self {
            RENodeId::KeyValueStore(id) => id,
            RENodeId::NonFungibleStore(id) => id,
            RENodeId::Vault(id) => id,
            RENodeId::Component(id) => id,
            RENodeId::EpochManager(id) => id,
            RENodeId::Validator(id) => id,
            RENodeId::AccessController(id) => id,
            RENodeId::Proof(id) => id,
            RENodeId::Bucket(id) => id,
            RENodeId::Worktop => [3u8; 36], // TODO: Remove, this is here to preserve receiver in invocation for now
            RENodeId::Logger => [4u8; 36], // TODO: Remove, this is here to preserve receiver in invocation for now
            RENodeId::TransactionRuntime => [5u8; 36], // TODO: Remove, this is here to preserve receiver in invocation for now
            RENodeId::AuthZoneStack => [6u8; 36], // TODO: Remove, this is here to preserve receiver in invocation for now
            _ => panic!("Not a stored id: {:?}", self),
        }
    }
}

impl From<RENodeId> for Address {
    fn from(node_id: RENodeId) -> Self {
        match node_id {
            RENodeId::GlobalComponent(component_address) => component_address.into(),
            RENodeId::GlobalResourceManager(resource_address) => resource_address.into(),
            RENodeId::GlobalPackage(package_address) => package_address.into(),
            _ => panic!("Not an address"),
        }
    }
}

impl From<Address> for RENodeId {
    fn from(address: Address) -> Self {
        match address {
            Address::Component(component_address) => RENodeId::GlobalComponent(component_address),
            Address::Resource(resource_address) => {
                RENodeId::GlobalResourceManager(resource_address)
            }
            Address::Package(package_address) => RENodeId::GlobalPackage(package_address),
        }
    }
}

impl Into<ComponentAddress> for RENodeId {
    fn into(self) -> ComponentAddress {
        match self {
            RENodeId::GlobalComponent(address) => address,
            _ => panic!("Not a component address: {:?}", self),
        }
    }
}

impl Into<PackageAddress> for RENodeId {
    fn into(self) -> PackageAddress {
        match self {
            RENodeId::GlobalPackage(package_address) => package_address,
            _ => panic!("Not a package address"),
        }
    }
}

impl Into<ResourceAddress> for RENodeId {
    fn into(self) -> ResourceAddress {
        match self {
            RENodeId::GlobalResourceManager(resource_address) => resource_address,
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
    ScryptoSbor,
    ManifestCategorize,
    ManifestEncode,
    ManifestDecode,
    LegacyDescribe,
)]
pub enum NodeModuleId {
    SELF,
    TypeInfo,
    Metadata,
    AccessRules,
    AccessRules1,
    ComponentRoyalty,
    PackageRoyalty,
    FunctionAccessRules,
}

impl NodeModuleId {
    pub fn from_u32(i: u32) -> Option<NodeModuleId> {
        match i {
            0u32 => Some(NodeModuleId::SELF),
            1u32 => Some(NodeModuleId::TypeInfo),
            2u32 => Some(NodeModuleId::Metadata),
            3u32 => Some(NodeModuleId::AccessRules),
            4u32 => Some(NodeModuleId::AccessRules1),
            5u32 => Some(NodeModuleId::ComponentRoyalty),
            6u32 => Some(NodeModuleId::PackageRoyalty),
            7u32 => Some(NodeModuleId::FunctionAccessRules),
            _ => None,
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            NodeModuleId::SELF => 0u32,
            NodeModuleId::TypeInfo => 1u32,
            NodeModuleId::Metadata => 2u32,
            NodeModuleId::AccessRules => 3u32,
            NodeModuleId::AccessRules1 => 4u32,
            NodeModuleId::ComponentRoyalty => 5u32,
            NodeModuleId::PackageRoyalty => 6u32,
            NodeModuleId::FunctionAccessRules => 7u32,
        }
    }
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AuthZoneStackOffset {
    AuthZoneStack,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AccessRulesOffset {
    AccessRules,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TypeInfoOffset {
    TypeInfo,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MetadataOffset {
    Metadata,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RoyaltyOffset {
    RoyaltyConfig,
    RoyaltyAccumulator,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ComponentOffset {
    /// Component application state at offset `0x00`.
    State0,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PackageOffset {
    Info,
    CodeType,
    Code,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceManagerOffset {
    ResourceManager,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeyValueStoreOffset {
    Entry(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
pub enum NonFungibleStoreOffset {
    Entry(NonFungibleLocalId),
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VaultOffset {
    Info,
    LiquidFungible,
    LiquidNonFungible,
    LockedFungible,
    LockedNonFungible,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EpochManagerOffset {
    EpochManager,
    CurrentValidatorSet,
    PreparingValidatorSet,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ValidatorOffset {
    Validator,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BucketOffset {
    Info,
    LiquidFungible,
    LiquidNonFungible,
    LockedFungible,
    LockedNonFungible,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProofOffset {
    Info,
    Fungible,
    NonFungible,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WorktopOffset {
    Worktop,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LoggerOffset {
    Logger,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ClockOffset {
    CurrentTimeRoundedToMinutes,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TransactionRuntimeOffset {
    TransactionRuntime,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AccountOffset {
    Account,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AccessControllerOffset {
    AccessController,
}

/// Specifies a specific Substate into a given RENode
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
pub enum SubstateOffset {
    AuthZoneStack(AuthZoneStackOffset),
    Component(ComponentOffset),
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
    Account(AccountOffset),
    AccessController(AccessControllerOffset),

    // Node modules
    // TODO: align with module ID allocation?
    TypeInfo(TypeInfoOffset),
    AccessRules(AccessRulesOffset),
    PackageAccessRules,
    Metadata(MetadataOffset),
    Royalty(RoyaltyOffset),
}

/// TODO: separate space addresses?
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
pub struct SubstateId(pub RENodeId, pub NodeModuleId, pub SubstateOffset);
