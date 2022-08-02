use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::engine::types::{BucketId, KeyValueStoreId, PackageAddress, ProofId, RENodeId, VaultId};
use crate::prelude::{ComponentAddress, ResourceAddress};

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum TypeName {
    Package,
    ResourceManager,
    TransactionProcessor,
    Blueprint(PackageAddress, String),
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Receiver {
    Consumed(RENodeId),
    Component(ComponentAddress),
    ComponentMetaRef(ComponentAddress),
    ResourceManagerRef(ResourceAddress),
    BucketRef(BucketId),
    ProofRef(ProofId),
    VaultRef(VaultId),
    SystemRef,
    WorktopRef,
    AuthZoneRef,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, TypeId, Encode, Decode)]
pub enum DataAddress {
    KeyValueEntry(KeyValueStoreId, Vec<u8>),
    ComponentInfo(ComponentAddress, bool),
    ComponentState(ComponentAddress),
}
