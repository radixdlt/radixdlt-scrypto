use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::core::ScryptoActor;
use crate::engine::types::{BucketId, KeyValueStoreId, ProofId, ValueId, VaultId};
use crate::prelude::ComponentAddress;
use crate::resource::ResourceAddress;

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum SNodeRef {
    PackageStatic,
    AuthZoneRef,
    Scrypto(ScryptoActor),
    Component(ComponentAddress),
    ResourceStatic,
    ResourceRef(ResourceAddress),
    Consumed(ValueId),
    BucketRef(BucketId),
    ProofRef(ProofId),
    SystemRef,
    WorktopRef,
    VaultRef(VaultId),
    TransactionProcessor,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ComponentOffset {
    Info,
    State,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum DataAddress {
    KeyValueEntry(KeyValueStoreId, Vec<u8>),
    Component(ComponentAddress, ComponentOffset),
}
