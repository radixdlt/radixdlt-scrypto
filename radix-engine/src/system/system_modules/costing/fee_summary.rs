use super::RoyaltyRecipient;
use crate::internal_prelude::*;
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

    // NOTE: Decimal arithmetic operation safe unwrap.
    // No chance to overflow considering current costing parameters

    pub fn total_cost(&self) -> Decimal {
        self.total_execution_cost_in_xrd
            .checked_add(self.total_finalization_cost_in_xrd)
            .unwrap()
            .checked_add(self.total_tipping_cost_in_xrd)
            .unwrap()
            .checked_add(self.total_storage_cost_in_xrd)
            .unwrap()
            .checked_add(self.total_royalty_cost_in_xrd)
            .unwrap()
    }

    pub fn network_fees(&self) -> Decimal {
        self.total_execution_cost_in_xrd
            .checked_add(self.total_finalization_cost_in_xrd)
            .unwrap()
            .checked_add(self.total_storage_cost_in_xrd)
            .unwrap()
    }

    pub fn to_proposer_amount(&self) -> Decimal {
        let one_percent = Decimal::ONE_HUNDREDTH;

        self.total_tipping_cost_in_xrd
            .checked_mul(
                one_percent
                    .checked_mul(TIPS_PROPOSER_SHARE_PERCENTAGE)
                    .unwrap(),
            )
            .unwrap()
            .checked_add(
                self.network_fees()
                    .checked_mul(
                        one_percent
                            .checked_mul(NETWORK_FEES_PROPOSER_SHARE_PERCENTAGE)
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .unwrap()
    }

    pub fn to_validator_set_amount(&self) -> Decimal {
        let one_percent = Decimal::ONE_HUNDREDTH;

        self.total_tipping_cost_in_xrd
            .checked_mul(
                one_percent
                    .checked_mul(TIPS_VALIDATOR_SET_SHARE_PERCENTAGE)
                    .unwrap(),
            )
            .unwrap()
            .checked_add(
                self.network_fees()
                    .checked_mul(
                        one_percent
                            .checked_mul(NETWORK_FEES_VALIDATOR_SET_SHARE_PERCENTAGE)
                            .unwrap(),
                    )
                    .unwrap(),
            )
            .unwrap()
    }

    pub fn to_burn_amount(&self) -> Decimal {
        self.total_tipping_cost_in_xrd
            .checked_add(self.network_fees())
            .unwrap()
            .checked_sub(self.to_proposer_amount())
            .unwrap()
            .checked_sub(self.to_validator_set_amount())
            .unwrap()
    }
}
