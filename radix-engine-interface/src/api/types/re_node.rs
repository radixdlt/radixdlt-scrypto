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
    Account,
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
)]
pub enum RENodeId {
    Bucket(BucketId),
    Proof(ProofId),
    AuthZoneStack,
    Worktop,
    Logger,
    TransactionRuntime,
    Global(Address),
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
    Account(AccountId),
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
            Self::Global(address) => f.debug_tuple("Global").field(address).finish(),
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
            Self::ResourceManager(id) => f
                .debug_tuple("ResourceManager")
                .field(&hex::encode(id))
                .finish(),
            Self::Package(id) => f.debug_tuple("Package").field(&hex::encode(id)).finish(),
            Self::EpochManager(id) => f
                .debug_tuple("EpochManager")
                .field(&hex::encode(id))
                .finish(),
            Self::Identity(id) => f.debug_tuple("Identity").field(&hex::encode(id)).finish(),
            Self::Clock(id) => f.debug_tuple("Clock").field(&hex::encode(id)).finish(),
            Self::Validator(id) => f.debug_tuple("Validator").field(&hex::encode(id)).finish(),
            Self::Account(id) => f.debug_tuple("Account").field(&hex::encode(id)).finish(),
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
            RENodeId::ResourceManager(id) => id,
            RENodeId::Package(id) => id,
            RENodeId::EpochManager(id) => id,
            RENodeId::Identity(id) => id,
            RENodeId::Validator(id) => id,
            RENodeId::Clock(id) => id,
            RENodeId::Account(id) => id,
            RENodeId::AccessController(id) => id,
            RENodeId::Proof(id) => id,
            RENodeId::Bucket(id) => id,
            RENodeId::Worktop => [3u8; 36], // TODO: Remove, this is here to preserve receiver in invocation for now
            RENodeId::Logger => [4u8; 36], // TODO: Remove, this is here to preserve receiver in invocation for now
            RENodeId::TransactionRuntime => [5u8; 36], // TODO: Remove, this is here to preserve receiver in invocation for now
            RENodeId::AuthZoneStack => [6u8; 36], // TODO: Remove, this is here to preserve receiver in invocation for now
            _ => panic!("Not a stored id"),
        }
    }
}

impl Into<ComponentAddress> for RENodeId {
    fn into(self) -> ComponentAddress {
        match self {
            RENodeId::Global(Address::Component(address)) => address,
            _ => panic!("Not a component address"),
        }
    }
}

impl Into<PackageAddress> for RENodeId {
    fn into(self) -> PackageAddress {
        match self {
            RENodeId::Global(Address::Package(package_address)) => package_address,
            _ => panic!("Not a package address"),
        }
    }
}

impl Into<ResourceAddress> for RENodeId {
    fn into(self) -> ResourceAddress {
        match self {
            RENodeId::Global(Address::Resource(resource_address)) => resource_address,
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
    PackageTypeInfo, // TODO: Unify with ComponentTypeInfo
    SELF,
    ComponentTypeInfo,
    Metadata,
    AccessRules,
    AccessRules1,
    ComponentRoyalty,
    PackageRoyalty,
    PackageAccessRules,
}

impl NodeModuleId {
    pub fn from_u32(i: u32) -> Option<NodeModuleId> {
        match i {
            0u32 => Some(NodeModuleId::PackageTypeInfo),
            1u32 => Some(NodeModuleId::ComponentTypeInfo),
            2u32 => Some(NodeModuleId::SELF),
            3u32 => Some(NodeModuleId::Metadata),
            4u32 => Some(NodeModuleId::AccessRules),
            5u32 => Some(NodeModuleId::AccessRules1),
            6u32 => Some(NodeModuleId::ComponentRoyalty),
            7u32 => Some(NodeModuleId::PackageRoyalty),
            8u32 => Some(NodeModuleId::PackageAccessRules),
            _ => None,
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            NodeModuleId::PackageTypeInfo => 0u32,
            NodeModuleId::ComponentTypeInfo => 1u32,
            NodeModuleId::SELF => 2u32,
            NodeModuleId::Metadata => 3u32,
            NodeModuleId::AccessRules => 4u32,
            NodeModuleId::AccessRules1 => 5u32,
            NodeModuleId::ComponentRoyalty => 6u32,
            NodeModuleId::PackageRoyalty => 7u32,
            NodeModuleId::PackageAccessRules => 8u32,
        }
    }
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AuthZoneStackOffset {
    AuthZoneStack,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AccessRulesChainOffset {
    AccessRulesChain,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ComponentTypeInfoOffset {
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
    NativeCode,
    WasmCode,
    Info,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GlobalOffset {
    Global,
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
    Proof,
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
    PackageTypeInfo, // TODO: Unify with ComponentTypeInfo
    Global(GlobalOffset),
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
    ComponentTypeInfo(ComponentTypeInfoOffset),
    AccessRulesChain(AccessRulesChainOffset),
    PackageAccessRules,
    Metadata(MetadataOffset),
    Royalty(RoyaltyOffset),
}

/// TODO: separate space addresses?
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
pub struct SubstateId(pub RENodeId, pub NodeModuleId, pub SubstateOffset);
