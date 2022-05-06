use crate::core::ScryptoActor;
use crate::engine::types::{BucketId, ProofId, VaultId};
use crate::resource::ResourceAddress;
use crate::rust::string::ToString;
use sbor::*;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum SNodeRef {
    PackageStatic,
    AuthZoneRef,
    CallerAuthZoneRef,
    AuthZoneManager,
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
