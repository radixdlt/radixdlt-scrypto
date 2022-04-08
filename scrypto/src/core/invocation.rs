use crate::core::ScryptoActor;
use crate::engine::types::{BucketId, VaultId};
use crate::resource::ResourceAddress;
use crate::rust::string::String;
use crate::rust::vec::Vec;

#[derive(Debug, Clone)]
pub enum SNodeRef {
    Scrypto(ScryptoActor),
    Resource(ResourceAddress),
    Bucket(BucketId),
    Vault(VaultId),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Invocation {
    snode_ref: SNodeRef,
    function: String,
    args: Vec<Vec<u8>>,
}