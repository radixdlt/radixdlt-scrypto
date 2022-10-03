use sbor::rust::vec::Vec;
use sbor::*;

use crate::component::{ComponentAddress, PackageAddress};
use crate::crypto::*;
use crate::resource::{NonFungibleId, ResourceAddress};

pub type AuthZoneId = u32;
pub type BucketId = u32;
pub type ProofId = u32;

pub type KeyValueStoreId = (Hash, u32);
pub type VaultId = (Hash, u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, TypeId, Ord, PartialOrd)]
pub enum RENodeId {
    AuthZone(AuthZoneId),
    Bucket(BucketId),
    Proof(ProofId),
    KeyValueStore(KeyValueStoreId),
    Worktop,
    Component(ComponentAddress),
    Vault(VaultId),
    ResourceManager(ResourceAddress),
    Package(PackageAddress),
    System(ComponentAddress),
    Global(GlobalAddress),
}

impl Into<(Hash, u32)> for RENodeId {
    fn into(self) -> KeyValueStoreId {
        match self {
            RENodeId::KeyValueStore(id) => id,
            RENodeId::Vault(id) => id,
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

impl Into<ComponentAddress> for RENodeId {
    fn into(self) -> ComponentAddress {
        match self {
            RENodeId::Component(component_address) => component_address,
            RENodeId::System(component_address) => component_address,
            _ => panic!("Not a component address"),
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
    AuthZone
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
    Global
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ResourceManagerOffset {
    ResourceManager,
    NonFungibleSpace,
    NonFungible(NonFungibleId),
}


/// TODO: separate space addresses?
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SubstateId {
    Global(GlobalAddress, GlobalOffset),

    AuthZone(AuthZoneId, AuthZoneOffset),

    Component(ComponentAddress, ComponentOffset),

    Package(PackageAddress, PackageOffset),

    ResourceManager(ResourceAddress, ResourceManagerOffset),

    KeyValueStoreSpace(KeyValueStoreId),
    KeyValueStoreEntry(KeyValueStoreId, Vec<u8>),

    Vault(VaultId),
    System(ComponentAddress),
    Bucket(BucketId),
    Proof(ProofId),
    Worktop,
}

impl Into<ComponentAddress> for SubstateId {
    fn into(self) -> ComponentAddress {
        match self {
            SubstateId::Component(component_address, ..) => component_address,
            _ => panic!("Address is not a component address"),
        }
    }
}

impl Into<ResourceAddress> for SubstateId {
    fn into(self) -> ResourceAddress {
        if let SubstateId::ResourceManager(resource_address, ..) = self {
            return resource_address;
        } else {
            panic!("Address is not a resource address");
        }
    }
}
