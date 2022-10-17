use sbor::rust::vec::Vec;
use sbor::*;

use crate::component::{ComponentAddress, PackageAddress};
use crate::crypto::*;
use crate::resource::NonFungibleId;
use crate::resource::ResourceAddress;

pub type LockHandle = u32;
pub type AuthZoneId = u32;
pub type BucketId = u32;
pub type ProofId = u32;

pub type ComponentId = (Hash, u32);
pub type KeyValueStoreId = (Hash, u32);
pub type NonFungibleStoreId = (Hash, u32);
pub type VaultId = (Hash, u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, TypeId, Ord, PartialOrd)]
pub enum RENodeId {
    Bucket(BucketId),
    Proof(ProofId),
    AuthZone(AuthZoneId),
    Worktop,

    Global(GlobalAddress),
    KeyValueStore(KeyValueStoreId),
    NonFungibleStore(NonFungibleStoreId),
    Component(ComponentId),
    System(ComponentId),
    Vault(VaultId),
    ResourceManager(ResourceAddress), // TODO: Convert this into id
    Package(PackageAddress),          // TODO: Convert this into id
}

impl Into<(Hash, u32)> for RENodeId {
    fn into(self) -> KeyValueStoreId {
        match self {
            RENodeId::KeyValueStore(id) => id,
            RENodeId::NonFungibleStore(id) => id,
            RENodeId::Vault(id) => id,
            RENodeId::Component(id) => id,
            RENodeId::System(id) => id,
            _ => panic!("Not a stored id"),
        }
    }
}

impl Into<u32> for RENodeId {
    fn into(self) -> u32 {
        match self {
            RENodeId::Bucket(id) => id,
            RENodeId::Proof(id) => id,
            _ => panic!("Not a transient id"),
        }
    }
}

impl Into<PackageAddress> for RENodeId {
    fn into(self) -> PackageAddress {
        match self {
            RENodeId::Package(package_address) => package_address,
            _ => panic!("Not a package address"),
        }
    }
}

impl Into<ResourceAddress> for RENodeId {
    fn into(self) -> ResourceAddress {
        match self {
            RENodeId::ResourceManager(resource_address) => resource_address,
            _ => panic!("Not a resource address"),
        }
    }
}

#[derive(Debug, Clone, Copy, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AuthZoneOffset {
    AuthZone,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ComponentOffset {
    Info,
    State,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PackageOffset {
    Package,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GlobalOffset {
    Global,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceManagerOffset {
    ResourceManager,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeyValueStoreOffset {
    Space,
    Entry(Vec<u8>),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NonFungibleStoreOffset {
    Space,
    Entry(NonFungibleId),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum VaultOffset {
    Vault,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SystemOffset {
    System,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BucketOffset {
    Bucket,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProofOffset {
    Proof,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WorktopOffset {
    Worktop,
}

/// Specifies a specific Substate into a given RENode
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SubstateOffset {
    Global(GlobalOffset),
    AuthZone(AuthZoneOffset),
    Component(ComponentOffset),
    Package(PackageOffset),
    ResourceManager(ResourceManagerOffset),
    KeyValueStore(KeyValueStoreOffset),
    NonFungibleStore(NonFungibleStoreOffset),
    Vault(VaultOffset),
    System(SystemOffset),
    Bucket(BucketOffset),
    Proof(ProofOffset),
    Worktop(WorktopOffset),
}

/// TODO: separate space addresses?
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SubstateId(pub RENodeId, pub SubstateOffset);
