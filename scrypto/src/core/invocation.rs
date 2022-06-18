use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::core::ScryptoActor;
use crate::engine::types::{BucketId, KeyValueStoreId, ProofId, VaultId};
use crate::prelude::ComponentAddress;
use crate::resource::ResourceAddress;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum SNodeRef {
    SystemStatic,
    PackageStatic,
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
    TransactionProcessor,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum DataAddress {
    KeyValueEntry(KeyValueStoreId, Vec<u8>),
    Component(ComponentAddress),
}
