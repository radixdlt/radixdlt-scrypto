use super::{CostingReason, RoyaltyRecipient};
use crate::types::*;
use radix_engine_interface::blueprints::resource::LiquidFungibleResource;
use sbor::rust::collections::*;

#[derive(Debug, Clone, ScryptoSbor)]
pub struct FeeSummary {
    /// The cost unit price in XRD.
    pub cost_unit_price: Decimal,
    /// The tip percentage
    pub tip_percentage: u16,
    /// The specified max cost units can be consumed.
    pub cost_unit_limit: u32,
    /// The total amount of XRD burned.
    pub total_execution_cost_xrd: Decimal,
    /// The total royalty.
    pub total_royalty_cost_xrd: Decimal,
    /// The (non-negative) amount of bad debt due to transaction unable to repay loan.
    pub total_bad_debt_xrd: Decimal,
    /// The vaults locked for XRD payment
    pub locked_fees: Vec<(Option<NodeId>, LiquidFungibleResource, bool)>,
    /// The execution cost breakdown
    pub execution_cost_breakdown: BTreeMap<CostingReason, u32>,
    /// The total number of cost units consumed (excluding royalties).
    pub execution_cost_sum: u32,
    /// The royalty cost breakdown
    pub royalty_cost_breakdown: BTreeMap<RoyaltyRecipient, (NodeId, Decimal)>,
}

impl FeeSummary {
    pub fn loan_fully_repaid(&self) -> bool {
        self.total_bad_debt_xrd == 0.into()
    }

    pub fn fees_to_distribute(&self) -> Decimal {
        self.cost_unit_price * Decimal::from(self.execution_cost_sum)
    }

    pub fn tips_to_distribute(&self) -> Decimal {
        (self.cost_unit_price * Decimal::from(self.tip_percentage) / Decimal::from(100u32))
            * Decimal::from(self.execution_cost_sum)
    }

    // For testing only
    pub fn expected_reward_if_single_validator(&self) -> Decimal {
        self.expected_reward_as_proposer_if_single_validator()
            + self.expected_reward_as_active_validator_if_single_validator()
    }
    pub fn expected_reward_as_proposer_if_single_validator(&self) -> Decimal {
        self.tips_to_distribute() * (TIPS_PROPOSER_SHARE_PERCENTAGE) / dec!(100)
            + self.fees_to_distribute() * (FEES_PROPOSER_SHARE_PERCENTAGE) / dec!(100)
    }
    pub fn expected_reward_as_active_validator_if_single_validator(&self) -> Decimal {
        self.tips_to_distribute() * (TIPS_VALIDATOR_SET_SHARE_PERCENTAGE) / dec!(100)
            + self.fees_to_distribute() * (FEES_VALIDATOR_SET_SHARE_PERCENTAGE) / dec!(100)
    }
}
