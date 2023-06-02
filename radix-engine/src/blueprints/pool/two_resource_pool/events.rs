use crate::types::*;
use radix_engine_common::math::Decimal;
use radix_engine_common::{ScryptoEvent, ScryptoSbor};
use sbor::rust::prelude::*;

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct ContributionEvent {
    pub contributed_resources: BTreeMap<ResourceAddress, Decimal>,
    pub pool_units_minted: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct RedemptionEvent {
    pub pool_unit_tokens_redeemed: Decimal,
    pub redeemed_resources: BTreeMap<ResourceAddress, Decimal>,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct WithdrawEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct DepositEvent {
    pub resource_address: ResourceAddress,
    pub amount: Decimal,
}
