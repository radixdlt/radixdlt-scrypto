use scrypto::engine::types::*;

#[derive(Debug)]
pub struct FeeSummary {
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
}
