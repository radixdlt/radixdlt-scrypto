// Ideally, only the types listed below can be used by Radix Engine.
// We need a better strategy to enforce this.

pub use crate::component::ComponentAddress;
pub use crate::component::PackageAddress;
pub use crate::core::Level;
pub use crate::core::ScryptoActorInfo;
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
pub enum StoredValueId {
    KeyValueStoreId(KeyValueStoreId),
    Component(ComponentAddress),
    VaultId(VaultId),
}

impl Into<ComponentAddress> for StoredValueId {
    fn into(self) -> ComponentAddress {
        match self {
            StoredValueId::Component(component_address) => component_address,
            _ => panic!("Expected to be a component"),
        }
    }
}

impl Into<(Hash, u32)> for StoredValueId {
    fn into(self) -> KeyValueStoreId {
        match self {
            StoredValueId::KeyValueStoreId(id) => id,
            StoredValueId::VaultId(id) => id,
            StoredValueId::Component(..) => panic!("ComponentAddress not expected"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Encode, Decode)]
pub enum ValueId {
    Bucket(BucketId),
    Proof(ProofId),
    Stored(StoredValueId),
    Resource(ResourceAddress),
    NonFungibles(ResourceAddress),
    Package(PackageAddress),
}

impl ValueId {
    pub fn kv_store_id(id: KeyValueStoreId) -> Self {
        ValueId::Stored(StoredValueId::KeyValueStoreId(id))
    }

    pub fn vault_id(id: VaultId) -> Self {
        ValueId::Stored(StoredValueId::VaultId(id))
    }
}

impl Into<StoredValueId> for ValueId {
    fn into(self) -> StoredValueId {
        match self {
            ValueId::Stored(id) => id,
            _ => panic!("Not a stored id"),
        }
    }
}

impl Into<(Hash, u32)> for ValueId {
    fn into(self) -> KeyValueStoreId {
        match self {
            ValueId::Stored(StoredValueId::KeyValueStoreId(id)) => id,
            ValueId::Stored(StoredValueId::VaultId(id)) => id,
            _ => panic!("Not a stored id"),
        }
    }
}

impl Into<u32> for ValueId {
    fn into(self) -> u32 {
        match self {
            ValueId::Bucket(id) => id,
            ValueId::Proof(id) => id,
            _ => panic!("Not a transient id"),
        }
    }
}

impl Into<ComponentAddress> for ValueId {
    fn into(self) -> ComponentAddress {
        match self {
            ValueId::Stored(StoredValueId::Component(component_address)) => component_address,
            _ => panic!("Not a component address"),
        }
    }
}

impl Into<PackageAddress> for ValueId {
    fn into(self) -> PackageAddress {
        match self {
            ValueId::Package(package_address) => package_address,
            _ => panic!("Not a package address"),
        }
    }
}

impl Into<ResourceAddress> for ValueId {
    fn into(self) -> ResourceAddress {
        match self {
            ValueId::Resource(resource_address) => resource_address,
            _ => panic!("Not a resource address"),
        }
    }
}

pub use crate::constants::*;
