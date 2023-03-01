use crate::types::*;

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
