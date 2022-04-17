use crate::core::ScryptoActor;
use crate::engine::types::{BucketId, ProofId, VaultId};
use crate::resource::ResourceAddress;
use sbor::*;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum SNodeRef {
    AuthZoneRef,
    WorktopRef,
    Scrypto(ScryptoActor),
    ResourceStatic,
    ResourceRef(ResourceAddress),
    Bucket(BucketId),
    BucketRef(BucketId),
    ProofRef(ProofId),
    Proof(ProofId),
    VaultRef(VaultId),
}
