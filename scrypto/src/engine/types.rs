/// Types (aliases to be accurate) used by radix engine.

pub type Hash = crate::crypto::Hash;
pub type Decimal = crate::math::Decimal;
pub type ResourceType = crate::resource::ResourceType;
pub type Supply = crate::resource::Supply;
pub type NonFungibleKey = crate::resource::NonFungibleKey;
pub type Level = crate::core::Level;
pub type Actor = crate::core::Actor;

pub type PackageId = [u8; 26];
pub type BlueprintId = (PackageId, String);
pub type ComponentId = [u8; 26];
pub type ResourceDefId = [u8; 26];
pub type LazyMapId = (Hash, u32);
pub type BucketId = u32;
pub type BucketRefId = u32;
pub type VaultId = (Hash, u32);
