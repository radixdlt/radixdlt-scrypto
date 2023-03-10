use crate::types::*;

#[derive(ScryptoSbor, PartialEq, Eq)]
pub struct LockFeeEvent {
    pub amount: Decimal,
}

#[derive(ScryptoSbor, PartialEq, Eq)]
pub enum WithdrawResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}

#[derive(ScryptoSbor, PartialEq, Eq, Debug)]
pub enum DepositResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}

#[derive(ScryptoSbor, PartialEq, Eq)]
pub enum RecallResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}
