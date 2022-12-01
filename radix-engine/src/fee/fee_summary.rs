use super::RoyaltyReceiver;
use crate::model::Resource;
use crate::types::*;
use radix_engine_interface::api::types::VaultId;

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub struct FeeSummary {
    /// The cost unit price in XRD.
    pub cost_unit_price: Decimal,
    /// The tip percentage
    pub tip_percentage: u8,
    /// The specified max cost units can be consumed.
    pub cost_unit_limit: u64,
    /// The total number of cost units consumed.
    pub cost_unit_consumed: u64,
    /// The total amount of XRD burned.
    pub execution: Decimal,
    /// The total royalty.
    pub royalty: Decimal,
    /// The amount of bad debt due to transaction unable to repay loan.
    pub bad_debt: Decimal,
    /// The fee payments
    pub payments: Vec<(VaultId, Resource, bool)>,
    /// The execution cost breakdown
    pub execution_breakdown: HashMap<String, u64>,
    /// The royalty cost breakdown.
    pub royalty_breakdown: HashMap<RoyaltyReceiver, u64>,
}

impl FeeSummary {
    pub fn loan_fully_repaid(&self) -> bool {
        self.bad_debt <= 0.into()
    }
}
