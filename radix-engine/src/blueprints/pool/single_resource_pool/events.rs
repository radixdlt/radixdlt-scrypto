use radix_engine_common::{math::Decimal, ScryptoSbor};

#[derive(ScryptoSbor)]
pub struct SingleResourcePoolContributionEvent {
    contribution_amount: Decimal,
    pool_unit_tokens_minted: Decimal,
}

#[derive(ScryptoSbor)]
pub struct SingleResourcePoolRedemptionEvent {
    pool_unit_tokens_redeemed: Decimal,
    redeemed_amount: Decimal,
}

#[derive(ScryptoSbor)]
pub struct SingleResourceProtectedWithdrawEvent {
    amount: Decimal,
}

#[derive(ScryptoSbor)]
pub struct SingleResourceProtectedDepositEvent {
    amount: Decimal,
}
