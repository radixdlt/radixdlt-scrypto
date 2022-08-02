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
pub use crate::sbor::rust::vec::Vec;
pub use crate::constants::*;

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

/// TODO: separate space addresses?
///
/// FIXME: RESIM listing is broken ATM.
/// By using scrypto codec, we lose sorting capability of the address space.
/// Can also be resolved by A) using prefix search instead of range search or B) use special codec as before
#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SubstateId {
    // TODO: Remove this bool which represents globalization
    ComponentInfo(ComponentAddress, bool),
    Package(PackageAddress),
    ResourceManager(ResourceAddress),
    NonFungibleSpace(ResourceAddress),
    NonFungible(ResourceAddress, NonFungibleId),
    KeyValueStoreSpace(KeyValueStoreId),
    KeyValueStoreEntry(KeyValueStoreId, Vec<u8>),
    Vault(VaultId),
    ComponentState(ComponentAddress),
    System,
}

impl SubstateId {
    pub fn get_node_id(&self) -> RENodeId {
        match self {
            SubstateId::ComponentInfo(component_address, ..) => {
                RENodeId::Component(*component_address)
            }
            SubstateId::ComponentState(component_address) => {
                RENodeId::Component(*component_address)
            }
            SubstateId::NonFungibleSpace(resource_address) => RENodeId::Resource(*resource_address),
            SubstateId::NonFungible(resource_address, ..) => RENodeId::Resource(*resource_address),
            SubstateId::KeyValueStoreSpace(kv_store_id) => RENodeId::KeyValueStore(*kv_store_id),
            SubstateId::KeyValueStoreEntry(kv_store_id, ..) => {
                RENodeId::KeyValueStore(*kv_store_id)
            }
            SubstateId::Vault(vault_id) => RENodeId::Vault(*vault_id),
            SubstateId::Package(package_address) => RENodeId::Package(*package_address),
            SubstateId::ResourceManager(resource_address) => RENodeId::Resource(*resource_address),
            SubstateId::System => RENodeId::System,
        }
    }

    pub fn is_native(&self) -> bool {
        match self {
            SubstateId::KeyValueStoreEntry(..) => false,
            SubstateId::ComponentState(..) => false,
            SubstateId::NonFungible(..) => false,
            SubstateId::ComponentInfo(..) => true,
            SubstateId::NonFungibleSpace(..) => true,
            SubstateId::KeyValueStoreSpace(..) => true,
            SubstateId::Vault(..) => true,
            SubstateId::Package(..) => true,
            SubstateId::ResourceManager(..) => true,
            SubstateId::System => true,
        }
    }

    pub fn can_own_nodes(&self) -> bool {
        match self {
            SubstateId::KeyValueStoreEntry(..) => true,
            SubstateId::ComponentState(..) => true,
            SubstateId::ComponentInfo(..) => false,
            SubstateId::NonFungible(..) => false,
            SubstateId::NonFungibleSpace(..) => false,
            SubstateId::KeyValueStoreSpace(..) => false,
            SubstateId::Vault(..) => false,
            SubstateId::Package(..) => false,
            SubstateId::ResourceManager(..) => false,
            SubstateId::System => false,
        }
    }
}

impl Into<ComponentAddress> for SubstateId {
    fn into(self) -> ComponentAddress {
        match self {
            SubstateId::ComponentInfo(component_address, ..)
            | SubstateId::ComponentState(component_address) => component_address,
            _ => panic!("Address is not a component address"),
        }
    }
}

impl Into<ResourceAddress> for SubstateId {
    fn into(self) -> ResourceAddress {
        if let SubstateId::ResourceManager(resource_address) = self {
            return resource_address;
        } else {
            panic!("Address is not a resource address");
        }
    }
}