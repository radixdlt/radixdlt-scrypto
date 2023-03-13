use super::*;
use crate::data::scrypto::model::*;
use crate::*;
use sbor::rust::fmt;
use sbor::rust::prelude::*;

pub const INTERNAL_OBJECT_NORMAL_COMPONENT_ID: u8 = 0x0d;
pub const INTERNAL_OBJECT_VAULT_ID: u8 = 0x0e;
pub const INTERNAL_KV_STORE_ID: u8 = 0x0f;

// TODO: Remove when better type system implemented
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor)]
pub enum RENodeType {
    AuthZoneStack,
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
    Object,
    Vault,
    TransactionRuntime,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum RENodeId {
    AuthZoneStack,
    TransactionRuntime,
    GlobalObject(Address),
    KeyValueStore(KeyValueStoreId),
    NonFungibleStore(NonFungibleStoreId),
    Object(ObjectId),
}

impl fmt::Debug for RENodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuthZoneStack => write!(f, "AuthZoneStack"),
            Self::TransactionRuntime => write!(f, "TransactionRuntime"),
            Self::KeyValueStore(id) => f
                .debug_tuple("KeyValueStore")
                .field(&hex::encode(id))
                .finish(),
            Self::NonFungibleStore(id) => f
                .debug_tuple("NonFungibleStore")
                .field(&hex::encode(id))
                .finish(),
            Self::Object(id) => f.debug_tuple("Object").field(&hex::encode(id)).finish(),
            Self::GlobalObject(address) => f.debug_tuple("Global").field(&address).finish(),
        }
    }
}

impl Into<[u8; OBJECT_ID_LENGTH]> for RENodeId {
    fn into(self) -> [u8; OBJECT_ID_LENGTH] {
        match self {
            RENodeId::KeyValueStore(id) => id,
            RENodeId::NonFungibleStore(id) => id,
            RENodeId::Object(id) => id,
            RENodeId::TransactionRuntime => [4u8; OBJECT_ID_LENGTH], // TODO: Remove, this is here to preserve receiver in invocation for now
            RENodeId::AuthZoneStack => [5u8; OBJECT_ID_LENGTH], // TODO: Remove, this is here to preserve receiver in invocation for now
            _ => panic!("Not a stored id: {:?}", self),
        }
    }
}

impl From<RENodeId> for Address {
    fn from(node_id: RENodeId) -> Self {
        match node_id {
            RENodeId::GlobalObject(address) => address,
            _ => panic!("Not an address"),
        }
    }
}

impl From<Address> for RENodeId {
    fn from(address: Address) -> Self {
        RENodeId::GlobalObject(address)
    }
}

impl Into<ComponentAddress> for RENodeId {
    fn into(self) -> ComponentAddress {
        match self {
            RENodeId::GlobalObject(address) => address.into(),
            _ => panic!("Not a component address: {:?}", self),
        }
    }
}

impl Into<PackageAddress> for RENodeId {
    fn into(self) -> PackageAddress {
        match self {
            RENodeId::GlobalObject(address) => address.into(),
            _ => panic!("Not a package address"),
        }
    }
}

impl Into<ResourceAddress> for RENodeId {
    fn into(self) -> ResourceAddress {
        match self {
            RENodeId::GlobalObject(address) => address.into(),
            _ => panic!("Not a resource address"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor, ManifestSbor)]
pub enum NodeModuleId {
    SELF,
    TypeInfo,
    Metadata,
    AccessRules,
    AccessRules1, // TODO: remove
    ComponentRoyalty,
    FunctionAccessRules, // TODO: remove
    PackageEventSchema,  // TODO: remove
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
            7u32 => Some(NodeModuleId::FunctionAccessRules),
            8u32 => Some(NodeModuleId::PackageEventSchema),
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
            NodeModuleId::FunctionAccessRules => 7u32,
            NodeModuleId::PackageEventSchema => 8u32,
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
pub enum RoyaltyOffset {
    RoyaltyConfig,
    RoyaltyAccumulator,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PackageEventSchemaOffset {
    PackageEventSchema,
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
    Royalty,
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
    LockedFungible,
    LiquidNonFungible,
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
    LockedFungible,
    LiquidNonFungible,
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
    PackageAccessRules,
    ResourceManager(ResourceManagerOffset),
    KeyValueStore(KeyValueStoreOffset),
    NonFungibleStore(NonFungibleStoreOffset),
    Vault(VaultOffset),
    EpochManager(EpochManagerOffset),
    Validator(ValidatorOffset),
    Bucket(BucketOffset),
    Proof(ProofOffset),
    Worktop(WorktopOffset),
    Clock(ClockOffset),
    TransactionRuntime(TransactionRuntimeOffset),
    Account(AccountOffset),
    AccessController(AccessControllerOffset),

    // Node modules
    // TODO: align with module ID allocation?
    TypeInfo(TypeInfoOffset),
    AccessRules(AccessRulesOffset),
    Royalty(RoyaltyOffset),
    PackageEventSchema(PackageEventSchemaOffset),
}

/// TODO: separate space addresses?
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
pub struct SubstateId(pub RENodeId, pub NodeModuleId, pub SubstateOffset);
