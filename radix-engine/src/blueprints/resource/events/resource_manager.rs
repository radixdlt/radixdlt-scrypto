use radix_engine_interface::api::types::rust::collections::BTreeSet;
use radix_engine_interface::api::types::{NonFungibleLocalId, RENodeId};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::*;

#[derive(ScryptoSbor)]
pub struct VaultCreationEvent {
    pub vault_id: RENodeId,
}

#[derive(ScryptoSbor)]
pub enum MintResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}

#[derive(ScryptoSbor)]
pub enum BurnResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}
