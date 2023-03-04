use crate::types::*;

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
