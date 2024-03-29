use crate::internal_prelude::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::blueprints::locker::*;

#[derive(ScryptoSbor, ScryptoEvent, Debug, Clone, PartialEq, Eq)]
pub struct StoreEvent {
    pub claimant: Global<AccountMarker>,
    pub resource_address: ResourceAddress,
    pub resources: ResourceSpecifier,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, Clone, PartialEq, Eq)]
pub struct RecoverEvent {
    pub claimant: Global<AccountMarker>,
    pub resource_address: ResourceAddress,
    pub resources: ResourceSpecifier,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, Clone, PartialEq, Eq)]
pub struct ClaimEvent {
    pub claimant: Global<AccountMarker>,
    pub resource_address: ResourceAddress,
    pub resources: ResourceSpecifier,
}
