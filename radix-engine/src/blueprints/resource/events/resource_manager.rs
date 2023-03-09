use crate::types::*;

#[derive(ScryptoSbor, PartialEq, Eq)]
pub struct VaultCreationEvent {
    pub vault_id: RENodeId,
}

#[derive(ScryptoSbor, PartialEq, Eq)]
pub enum MintResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}

#[derive(ScryptoSbor, PartialEq, Eq)]
pub enum BurnResourceEvent {
    Amount(Decimal),
    Ids(BTreeSet<NonFungibleLocalId>),
}
