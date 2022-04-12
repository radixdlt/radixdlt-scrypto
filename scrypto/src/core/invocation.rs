use crate::core::ScryptoActor;
use crate::engine::types::{BucketId, VaultId};
use crate::resource::ResourceAddress;
use sbor::*;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum SNodeRef {
    Scrypto(ScryptoActor),
    Resource(ResourceAddress),
    Bucket(BucketId),
    Vault(VaultId),
}
