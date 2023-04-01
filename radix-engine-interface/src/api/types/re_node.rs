use super::*;
use crate::data::scrypto::model::*;
use crate::*;
use radix_engine_common::address::AddressDisplayContext;
use sbor::rust::fmt;
use sbor::rust::prelude::*;
use utils::ContextualDisplay;

pub const INTERNAL_OBJECT_NORMAL_COMPONENT_ID: u8 = 0x0d;
pub const INTERNAL_OBJECT_VAULT_ID: u8 = 0x0e;
pub const INTERNAL_KV_STORE_ID: u8 = 0x0f;

// TODO: Remove when better type system implemented
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor)]
pub enum AllocateEntityType {
    GlobalAccount,
    GlobalComponent,
    GlobalFungibleResourceManager,
    GlobalNonFungibleResourceManager,
    GlobalPackage,
    GlobalEpochManager,
    GlobalValidator,
    GlobalAccessController,
    GlobalIdentity,
    KeyValueStore,
    Object,
    Vault,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, ScryptoSbor, ManifestSbor)]
pub enum RENodeId {
    GlobalObject(Address),
    KeyValueStore(KeyValueStoreId),
    // This is only used for owned objects (global objects have addresses)
    // TODO: Rename to OwnedObject when it won't cause so many merge conflicts!
    Object(ObjectId),
}

impl<'a> ContextualDisplay<AddressDisplayContext<'a>> for RENodeId {
    type Error = fmt::Error;

    fn contextual_format<F: fmt::Write>(
        &self,
        f: &mut F,
        context: &AddressDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        match self {
            Self::KeyValueStore(id) => write!(f, "KeyValueStore({})", hex::encode(id)),
            Self::Object(id) => write!(f, "Object({})", hex::encode(id)),
            Self::GlobalObject(address) => address
                .contextual_format(f, context)
                .map_err(|_| fmt::Error),
        }
    }
}

impl fmt::Debug for RENodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyValueStore(id) => f
                .debug_tuple("KeyValueStore")
                .field(&hex::encode(id))
                .finish(),
            Self::Object(id) => f.debug_tuple("Object").field(&hex::encode(id)).finish(),
            Self::GlobalObject(address) => f.debug_tuple("GlobalObject").field(&address).finish(),
        }
    }
}

impl From<RENodeId> for [u8; OBJECT_ID_LENGTH] {
    fn from(value: RENodeId) -> Self {
        match value {
            RENodeId::KeyValueStore(id) => id,
            RENodeId::Object(id) => id,
            _ => panic!("Not a stored id: {:?}", value),
        }
    }
}

impl From<RENodeId> for Vec<u8> {
    fn from(value: RENodeId) -> Self {
        // Note - these are all guaranteed to be distinct
        match value {
            RENodeId::KeyValueStore(id) => id.to_vec(),
            RENodeId::Object(id) => id.to_vec(),
            RENodeId::GlobalObject(address) => address.to_vec(),
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
    ComponentRoyalty,
}

impl NodeModuleId {
    pub fn from_u32(i: u32) -> Option<NodeModuleId> {
        match i {
            0u32 => Some(NodeModuleId::SELF),
            1u32 => Some(NodeModuleId::TypeInfo),
            2u32 => Some(NodeModuleId::Metadata),
            3u32 => Some(NodeModuleId::AccessRules),
            4u32 => Some(NodeModuleId::ComponentRoyalty),
            _ => None,
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            NodeModuleId::SELF => 0u32,
            NodeModuleId::TypeInfo => 1u32,
            NodeModuleId::Metadata => 2u32,
            NodeModuleId::AccessRules => 3u32,
            NodeModuleId::ComponentRoyalty => 4u32,
        }
    }
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
    FunctionAccessRules,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceManagerOffset {
    ResourceManager,
}

#[derive(Debug, Clone, ScryptoSbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeyValueStoreOffset {
    Entry(Vec<u8>),
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
    RegisteredValidators,
    RegisteredValidatorsByStake,
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
pub enum AccountOffset {
    Account,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AccessControllerOffset {
    AccessController,
}

#[derive(Debug, Clone, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AuthZoneOffset {
    AuthZone,
}

/// Specifies a specific Substate into a given RENode
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
pub enum SubstateOffset {
    Component(ComponentOffset),
    Package(PackageOffset),
    ResourceManager(ResourceManagerOffset),
    KeyValueStore(KeyValueStoreOffset),
    Vault(VaultOffset),
    EpochManager(EpochManagerOffset),
    Validator(ValidatorOffset),
    Bucket(BucketOffset),
    Proof(ProofOffset),
    Worktop(WorktopOffset),
    Clock(ClockOffset),
    Account(AccountOffset),
    AccessController(AccessControllerOffset),
    AuthZone(AuthZoneOffset),

    // Node modules
    // TODO: align with module ID allocation?
    TypeInfo(TypeInfoOffset),
    AccessRules(AccessRulesOffset),
    Royalty(RoyaltyOffset),
}

/// TODO: separate space addresses?
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor)]
pub struct SubstateId(pub RENodeId, pub NodeModuleId, pub SubstateOffset);
