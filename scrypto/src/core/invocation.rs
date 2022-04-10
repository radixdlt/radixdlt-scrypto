use crate::core::ScryptoActor;
use crate::engine::types::{BucketId, VaultId};
use crate::resource::ResourceAddress;
use sbor::*;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum SNodeRef {
    Scrypto(ScryptoActor),
    ResourceStatic,
    Resource(ResourceAddress),
    Bucket(BucketId),
    Vault(VaultId),
}
