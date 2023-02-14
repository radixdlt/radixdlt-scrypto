use super::RoyaltyReceiver;
use crate::model::Resource;
use crate::types::*;
use radix_engine_interface::api::types::VaultId;
use sbor::rust::collections::*;

#[derive(Debug, Clone, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct FeeSummary {
    /// The cost unit price in XRD.
    pub cost_unit_price: Decimal,
    /// The tip percentage
    pub tip_percentage: u16,
    /// The specified max cost units can be consumed.
    pub cost_unit_limit: u32,
    /// The total number of cost units consumed.
    pub cost_unit_consumed: u32,
    /// The total amount of XRD burned.
    pub total_execution_cost_xrd: Decimal,
    /// The total royalty.
    pub total_royalty_cost_xrd: Decimal,
    /// The (non-negative) amount of bad debt due to transaction unable to repay loan.
    pub bad_debt_xrd: Decimal,
    /// The vaults locked for XRD payment
    pub vault_locks: Vec<(VaultId, Resource, bool)>,
    /// The resultant vault charges in XRD (only present on commit)
    pub vault_payments_xrd: Option<BTreeMap<VaultId, Decimal>>,
    /// The execution cost breakdown
    pub execution_cost_unit_breakdown: HashMap<String, u32>,
    /// The royalty cost breakdown.
    pub royalty_cost_unit_breakdown: HashMap<RoyaltyReceiver, u32>,
}

impl FeeSummary {
    pub fn loan_fully_repaid(&self) -> bool {
        self.bad_debt_xrd == 0.into()
    }
}
