use native_sdk::{LegacyDescribe, ScryptoSbor};
use radix_engine_interface::api::types::rust::collections::BTreeSet;
use radix_engine_interface::api::types::NonFungibleLocalId;
use radix_engine_interface::math::Decimal;

#[derive(ScryptoSbor, LegacyDescribe)]
pub struct LockFeeEvent {
    pub amount: Decimal,
}

#[derive(ScryptoSbor, LegacyDescribe)]
pub enum WithdrawResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}

#[derive(ScryptoSbor, LegacyDescribe)]
pub enum DepositResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}

#[derive(ScryptoSbor, LegacyDescribe)]
pub enum RecallResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}
