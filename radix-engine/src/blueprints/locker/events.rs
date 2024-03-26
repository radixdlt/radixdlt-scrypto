use crate::internal_prelude::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::locker::*;

#[derive(ScryptoSbor, ScryptoEvent, Debug, Clone, PartialEq, Eq)]
pub struct StoreEvent {
    pub claimant: Global<AccountObjectTypeInfo>,
    pub resource_address: ResourceAddress,
    pub resources: ResourceSpecifier,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, Clone, PartialEq, Eq)]
pub struct BatchStoreEvent {
    pub claimants: IndexMap<Global<AccountObjectTypeInfo>, ResourceSpecifier>,
    pub resource_address: ResourceAddress,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, Clone, PartialEq, Eq)]
pub struct RecoveryEvent {
    pub claimant: Global<AccountObjectTypeInfo>,
    pub resource_address: ResourceAddress,
    pub resources: ResourceSpecifier,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, Clone, PartialEq, Eq)]
pub struct ClaimEvent {
    pub claimant: Global<AccountObjectTypeInfo>,
    pub resource_address: ResourceAddress,
    pub resources: ResourceSpecifier,
}
