use crate::types::*;

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct VaultCreationEvent {
    pub vault_id: NodeId,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct MintFungibleResourceEvent {
    pub amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct BurnFungibleResourceEvent {
    pub amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct MintNonFungibleResourceEvent {
    pub ids: BTreeSet<NonFungibleLocalId>,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq)]
pub struct BurnNonFungibleResourceEvent {
    pub ids: BTreeSet<NonFungibleLocalId>,
}
