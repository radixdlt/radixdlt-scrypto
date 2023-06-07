use crate::types::*;
use radix_engine_common::math::Decimal;
use radix_engine_common::{ScryptoEvent, ScryptoSbor};

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct ContributionEvent {
    pub amount_of_resources_contributed: Decimal,
    pub pool_units_minted: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct RedemptionEvent {
    pub pool_unit_tokens_redeemed: Decimal,
    pub redeemed_amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct WithdrawEvent {
    pub amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct DepositEvent {
    pub amount: Decimal,
}
