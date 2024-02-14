use crate::internal_prelude::*;

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct VaultCreationEvent {
    pub vault_id: NodeId,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct MintFungibleResourceEvent {
    pub amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct BurnFungibleResourceEvent {
    pub amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct MintNonFungibleResourceEvent {
    pub ids: IndexSet<NonFungibleLocalId>,
}

#[derive(ScryptoSbor, ScryptoEvent, PartialEq, Eq, Debug)]
pub struct BurnNonFungibleResourceEvent {
    pub ids: IndexSet<NonFungibleLocalId>,
}
