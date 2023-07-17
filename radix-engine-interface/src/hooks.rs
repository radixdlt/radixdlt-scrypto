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
pub struct OnMoveInput {
    /// True if the node moves from caller to callee, otherwise false.
    pub is_moving_down: bool,

    /// True if the destination actor is a barrier, otherwise false.
    ///
    /// TODO: expose generic information but fully-detailed actor?
    pub is_to_barrier: bool,

    /// True if the destination actor is auth zone, otherwise false.
    ///
    /// TODO: expose generic information but fully-detailed actor?
    pub is_to_auth_zone: bool,

    /// True if the destination actor is self blueprint.
    pub is_to_self_blueprint: bool,
}

pub type OnMoveOutput = ();
