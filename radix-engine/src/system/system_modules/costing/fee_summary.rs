use super::RoyaltyRecipient;
use crate::types::*;
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use sbor::rust::collections::*;

#[derive(Default, Debug, Clone, ScryptoSbor)]
pub struct FeeReserveFinalizationSummary {
    /// The total execution cost units consumed
    pub total_execution_cost_units_consumed: u32,
    /// The total finalization cost units consumed
    pub total_finalization_cost_units_consumed: u32,

    /// The total cost for execution
    pub total_execution_cost_in_xrd: Decimal,
    /// The total cost for finalization
    pub total_finalization_cost_in_xrd: Decimal,
    /// The total cost for tipping
    pub total_tipping_cost_in_xrd: Decimal,
    /// The total cost for storage
    pub total_storage_cost_in_xrd: Decimal,
    /// The total cost for royalty
    pub total_royalty_cost_in_xrd: Decimal,

    /// The (non-negative) amount of bad debt due to transaction unable to repay loan.
    pub total_bad_debt_in_xrd: Decimal,
    /// The vaults locked for XRD payment
    pub locked_fees: Vec<(NodeId, LiquidFungibleResource, bool)>,
    /// The royalty cost breakdown
    pub royalty_cost_breakdown: IndexMap<RoyaltyRecipient, Decimal>,
}

impl FeeReserveFinalizationSummary {
    pub fn loan_fully_repaid(&self) -> bool {
        self.total_bad_debt_in_xrd == 0.into()
    }

    pub fn total_cost(&self) -> Decimal {
        self.total_execution_cost_in_xrd
            .safe_add(self.total_finalization_cost_in_xrd)
            .unwrap()
            .safe_add(self.total_tipping_cost_in_xrd)
            .unwrap()
            .safe_add(self.total_storage_cost_in_xrd)
            .unwrap()
            .safe_add(self.total_royalty_cost_in_xrd)
            .unwrap()
    }

    pub fn network_fees(&self) -> Decimal {
        self.total_execution_cost_in_xrd
            .safe_add(self.total_finalization_cost_in_xrd)
            .unwrap()
            .safe_add(self.total_storage_cost_in_xrd)
            .unwrap()
    }

    pub fn to_proposer_amount(&self) -> Decimal {
        let dec_100 = dec!(100);

        self.total_tipping_cost_in_xrd
            .safe_mul(TIPS_PROPOSER_SHARE_PERCENTAGE.safe_div(dec_100).unwrap())
            .unwrap()
            .safe_add(
                self.network_fees()
                    .safe_mul(
                        NETWORK_FEES_PROPOSER_SHARE_PERCENTAGE
                            .safe_div(dec_100)
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .unwrap()
    }

    pub fn to_validator_set_amount(&self) -> Decimal {
        let dec_100 = dec!(100);

        self.total_tipping_cost_in_xrd
            .safe_mul(
                TIPS_VALIDATOR_SET_SHARE_PERCENTAGE
                    .safe_div(dec_100)
                    .unwrap(),
            )
            .unwrap()
            .safe_add(
                self.network_fees()
                    .safe_mul(
                        NETWORK_FEES_VALIDATOR_SET_SHARE_PERCENTAGE
                            .safe_div(dec_100)
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .unwrap()
    }

    pub fn to_burn_amount(&self) -> Decimal {
        let dec_100 = dec!(100);

        self.total_tipping_cost_in_xrd
            .safe_mul(TIPS_TO_BURN_PERCENTAGE.safe_div(dec_100).unwrap())
            .unwrap()
            .safe_add(
                self.network_fees()
                    .safe_mul(NETWORK_FEES_TO_BURN_PERCENTAGE.safe_div(dec_100).unwrap())
                    .unwrap(),
            )
            .unwrap()
    }
}
