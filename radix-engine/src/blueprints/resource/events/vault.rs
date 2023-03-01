use radix_engine_interface::api::types::rust::collections::BTreeSet;
use radix_engine_interface::api::types::NonFungibleLocalId;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::*;

#[derive(ScryptoSbor)]
pub struct LockFeeEvent {
    pub amount: Decimal,
}

#[derive(ScryptoSbor)]
pub enum WithdrawResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}

#[derive(ScryptoSbor)]
pub enum DepositResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}

#[derive(ScryptoSbor)]
pub enum RecallResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}
