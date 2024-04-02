use crate::internal_prelude::*;
use crate::types::BlueprintId;
use radix_common::types::GlobalAddressReservation;
use radix_common::types::NodeId;

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

// TODO: expose generic information, but fully-detailed actor? and then remove `is_to_barrier`.
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct OnMoveInput {
    /// True if the node moves from caller to callee, otherwise false.
    pub is_moving_down: bool,

    /// True if the destination actor is a barrier, otherwise false.
    pub is_to_barrier: bool,

    /// The destination blueprint id.
    pub destination_blueprint_id: Option<BlueprintId>,
}

pub type OnMoveOutput = ();
