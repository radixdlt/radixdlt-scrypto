use crate::types::*;

#[derive(ScryptoSbor, PartialEq, Eq)]
pub struct VaultCreationEvent {
    pub vault_id: RENodeId,
}

#[derive(ScryptoSbor, PartialEq, Eq)]
pub struct MintFungibleResourceEvent {
    pub amount: Decimal,
}

#[derive(ScryptoSbor, PartialEq, Eq)]
pub struct BurnFungibleResourceEvent {
    pub amount: Decimal,
}

#[derive(ScryptoSbor, PartialEq, Eq)]
pub struct MintNonFungibleResourceEvent {
    pub ids: BTreeSet<NonFungibleLocalId>,
}

#[derive(ScryptoSbor, PartialEq, Eq)]
pub struct BurnNonFungibleResourceEvent {
    pub ids: BTreeSet<NonFungibleLocalId>,
}
