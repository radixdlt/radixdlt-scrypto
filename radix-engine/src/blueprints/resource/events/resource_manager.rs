use native_sdk::{LegacyDescribe, ScryptoSbor};
use radix_engine_interface::api::types::rust::collections::BTreeSet;
use radix_engine_interface::api::types::{NonFungibleLocalId, RENodeId};
use radix_engine_interface::math::Decimal;

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct VaultCreationEvent {
    pub vault_id: RENodeId,
}

#[derive(ScryptoSbor, LegacyDescribe)]
pub enum MintResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}

#[derive(ScryptoSbor, LegacyDescribe)]
pub enum BurnResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}
