use crate::types::*;

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct LockFeeEvent {
    pub amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub enum WithdrawResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub enum DepositResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub enum RecallResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}
