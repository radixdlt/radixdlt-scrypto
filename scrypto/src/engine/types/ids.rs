use super::*;

pub type LockHandle = u32;
pub type AuthZoneId = u32;
pub type BucketId = u32;
pub type ProofId = u32;

pub type ComponentId = (Hash, u32);
pub type KeyValueStoreId = (Hash, u32);
pub type NonFungibleStoreId = (Hash, u32);
pub type VaultId = (Hash, u32);
pub type ResourceManagerId = (Hash, u32);
pub type PackageId = (Hash, u32);