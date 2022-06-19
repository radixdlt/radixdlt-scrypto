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

pub type KeyValueStoreId = (Hash, u32);
pub type VaultId = (Hash, u32);
pub type BucketId = u32;
pub type ProofId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransientValueId {
    Bucket(BucketId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StoredValueId {
    KeyValueStoreId(KeyValueStoreId),
    VaultId(VaultId),
}

impl Into<(Hash, u32)> for StoredValueId {
    fn into(self) -> KeyValueStoreId {
        match self {
            StoredValueId::KeyValueStoreId(id) => id,
            StoredValueId::VaultId(id) => id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueId {
    Stored(StoredValueId)
}

impl ValueId {
    pub fn kv_store_id(id: KeyValueStoreId) -> Self {
        ValueId::Stored(StoredValueId::KeyValueStoreId(id))
    }

    pub fn vault_id(id: VaultId) -> Self {
        ValueId::Stored(StoredValueId::VaultId(id))
    }
}

impl Into<(Hash, u32)> for ValueId {
    fn into(self) -> KeyValueStoreId {
        match self {
            ValueId::Stored(StoredValueId::KeyValueStoreId(id)) => id,
            ValueId::Stored(StoredValueId::VaultId(id)) => id,
        }
    }
}

pub use crate::constants::*;
