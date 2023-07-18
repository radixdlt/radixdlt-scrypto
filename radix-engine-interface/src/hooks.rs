use crate::ScryptoSbor;
use radix_engine_common::types::GlobalAddressReservation;
use radix_engine_common::types::NodeId;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct OnVirtualizeInput {
    pub variant_id: u8,
    pub rid: [u8; NodeId::RID_LENGTH],
    pub address_reservation: GlobalAddressReservation,
}

pub type OnVirtualizeOutput = ();

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct OnDropInput {}

pub type OnDropOutput = ();

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct OnMoveInput {}

pub type OnMoveOutput = ();

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct OnPersistInput {}

pub type OnPersistOutput = ();
