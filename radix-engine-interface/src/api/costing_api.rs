use crate::blueprints::resource::LiquidFungibleResource;
use crate::types::*;
use radix_common::math::Decimal;

pub trait SystemCostingApi<E> {
    /// Check if costing is enabled.
    fn start_lock_fee(&mut self, amount: Decimal, contingent: bool) -> Result<bool, E>;

    /// Add cost units to the reserve. This should never fail.
    fn lock_fee(&mut self, locked_fee: LiquidFungibleResource, contingent: bool);

    /// Consume an amount of cost units.
    fn consume_cost_units(&mut self, costing_entry: ClientCostingEntry) -> Result<(), E>;

    /// Retrieve the cost unit limit for the transaction
    fn execution_cost_unit_limit(&mut self) -> Result<u32, E>;

    /// Retrieve the cost unit price in XRD
    fn execution_cost_unit_price(&mut self) -> Result<Decimal, E>;

    /// Retrieve the finalization cost unit limit
    fn finalization_cost_unit_limit(&mut self) -> Result<u32, E>;

    /// Retrieve the finalization cost unit price in XRD
    fn finalization_cost_unit_price(&mut self) -> Result<Decimal, E>;

    /// Retrieve the usd price of XRD
    fn usd_price(&mut self) -> Result<Decimal, E>;

    /// Retrieve the maximum allowable royalty per function
    fn max_per_function_royalty_in_xrd(&mut self) -> Result<Decimal, E>;

    /// Retrieve the tip percentage of the transaction
    fn tip_percentage_truncated(&mut self) -> Result<u32, E>;

    /// Retrieve the current fee balance in XRD
    fn fee_balance(&mut self) -> Result<Decimal, E>;
}
