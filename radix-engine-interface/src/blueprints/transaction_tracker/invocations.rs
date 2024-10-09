use crate::internal_prelude::*;

pub const TRANSACTION_TRACKER_CREATE_IDENT: &str = "create";

pub const TRANSACTION_TRACKER_CREATE_EXPORT_NAME: &str = "create";

#[derive(Debug, Clone, ScryptoSbor)]
pub struct TransactionTrackerCreateInput {
    pub address_reservation: GlobalAddressReservation,
}

#[derive(Debug, Clone, ManifestSbor)]
pub struct TransactionTrackerCreateManifestInput {
    pub address_reservation: ManifestAddressReservation,
}
