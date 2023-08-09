use super::RoyaltyRecipient;
use crate::types::*;
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use sbor::rust::collections::*;

#[derive(Default, Debug, Clone, ScryptoSbor)]
pub struct FeeSummary {
    /// The max execution cost units to consume.
    pub execution_cost_unit_limit: u32,
    /// The price of execution cost unit in XRD.
    pub execution_cost_unit_price: Decimal,
    /// The max finalization cost units to consume.
    pub finalization_cost_unit_limit: u32,
    /// The price of finalization cost unit in XRD.
    pub finalization_cost_unit_price: Decimal,
    /// The price of USD in XRD.
    pub usd_price_in_xrd: Decimal,
    /// The price of storage in XRD.
    pub storage_price_in_xrd: Decimal,
    /// The tip percentage
    pub tip_percentage: u16,
    /// The total cost for execution, excluding tips
    pub total_execution_cost_in_xrd: Decimal,
    /// The total cost for tipping
    pub total_tipping_cost_in_xrd: Decimal,
    /// The total cost for state expansion
    pub total_storage_cost_in_xrd: Decimal,
    /// The total cost for royalty
    pub total_royalty_cost_in_xrd: Decimal,
    /// The (non-negative) amount of bad debt due to transaction unable to repay loan.
    pub total_bad_debt_in_xrd: Decimal,
    /// The vaults locked for XRD payment
    pub locked_fees: Vec<(NodeId, LiquidFungibleResource, bool)>,
    /// The total number of cost units consumed (excluding royalties).
    pub total_execution_cost_units_consumed: u32,
    /// The royalty cost breakdown
    pub royalty_cost_breakdown: IndexMap<RoyaltyRecipient, Decimal>,
}

impl FeeSummary {
    pub fn loan_fully_repaid(&self) -> bool {
        self.total_bad_debt_in_xrd == 0.into()
    }

    fn tips(&self) -> Decimal {
        self.total_tipping_cost_in_xrd
    }

    fn fees(&self) -> Decimal {
        self.total_execution_cost_in_xrd + self.total_storage_cost_in_xrd
    }

    pub fn to_proposer_amount(&self) -> Decimal {
        self.tips() * TIPS_PROPOSER_SHARE_PERCENTAGE / dec!(100)
            + self.fees() * FEES_PROPOSER_SHARE_PERCENTAGE / dec!(100)
    }

    pub fn to_validator_set_amount(&self) -> Decimal {
        self.tips() * TIPS_VALIDATOR_SET_SHARE_PERCENTAGE / dec!(100)
            + self.fees() * FEES_VALIDATOR_SET_SHARE_PERCENTAGE / dec!(100)
    }

    pub fn to_burn_amount(&self) -> Decimal {
        self.tips() * TIPS_TO_BURN_PERCENTAGE / dec!(100)
            + self.fees() * FEES_TO_BURN_PERCENTAGE / dec!(100)
    }

    pub fn total_cost(&self) -> Decimal {
        self.total_execution_cost_in_xrd
            + self.total_tipping_cost_in_xrd
            + self.total_storage_cost_in_xrd
            + self.total_royalty_cost_in_xrd
    }

    pub fn used_free_credit(&self) -> Decimal {
        self.total_cost() - self.total_payments()
    }

    //===================
    // For testing only
    //===================

    pub fn expected_reward_if_single_validator(&self) -> Decimal {
        self.expected_reward_as_proposer_if_single_validator()
            + self.expected_reward_as_active_validator_if_single_validator()
    }

    pub fn expected_reward_as_proposer_if_single_validator(&self) -> Decimal {
        self.tips() * (TIPS_PROPOSER_SHARE_PERCENTAGE) / dec!(100)
            + self.fees() * (FEES_PROPOSER_SHARE_PERCENTAGE) / dec!(100)
    }

    pub fn expected_reward_as_active_validator_if_single_validator(&self) -> Decimal {
        self.tips() * (TIPS_VALIDATOR_SET_SHARE_PERCENTAGE) / dec!(100)
            + self.fees() * (FEES_VALIDATOR_SET_SHARE_PERCENTAGE) / dec!(100)
    }
}
