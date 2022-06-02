use sbor::rust::string::ToString;
use sbor::*;

use crate::core::ScryptoActor;
use crate::engine::types::{BucketId, ProofId, VaultId};
use crate::resource::ResourceAddress;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum SNodeRef {
    SystemStatic,
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
    TransactionProcessor,
}
