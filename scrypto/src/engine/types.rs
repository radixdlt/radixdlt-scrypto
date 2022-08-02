// Ideally, only the types listed below can be used by Radix Engine.
// We need a better strategy to enforce this.

pub use crate::component::ComponentAddress;
pub use crate::component::PackageAddress;
pub use crate::core::Level;
pub use crate::crypto::EcdsaPublicKey;
pub use crate::crypto::EcdsaSignature;
pub use crate::crypto::Hash;
pub use crate::math::Decimal;
pub use crate::resource::MintParams;
pub use crate::resource::NonFungibleAddress;
pub use crate::resource::NonFungibleId;
pub use crate::resource::ResourceAddress;
pub use crate::resource::ResourceType;
pub use crate::sbor::*;

pub type KeyValueStoreId = (Hash, u32);
pub type VaultId = (Hash, u32);
pub type BucketId = u32;
pub type ProofId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum RENodeId {
    Bucket(BucketId),
    Proof(ProofId),
    KeyValueStore(KeyValueStoreId),
    Worktop,
    Component(ComponentAddress),
    Vault(VaultId),
    Resource(ResourceAddress),
    Package(PackageAddress),
    System,
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
            RENodeId::Resource(resource_address) => resource_address,
            _ => panic!("Not a resource address"),
        }
    }
}

pub use crate::constants::*;
