use crate::model::Resource;
use crate::types::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct FeeSummary {
    /// Whether the system loan is fully repaid
    pub loan_fully_repaid: bool,
    /// The specified max cost units can be consumed.
    pub cost_unit_limit: u32,
    /// The total number of cost units consumed.
    pub cost_unit_consumed: u32,
    /// The cost unit price in XRD.
    pub cost_unit_price: Decimal,
    /// The tip percentage
    pub tip_percentage: u32,
    /// The total amount of XRD burned.
    pub burned: Decimal,
    /// The total amount of XRD tipped to validators.
    pub tipped: Decimal,
    /// The fee payments
    pub payments: Vec<(VaultId, Resource, bool)>,
    /// The cost breakdown
    pub cost_breakdown: HashMap<String, u32>,
}
