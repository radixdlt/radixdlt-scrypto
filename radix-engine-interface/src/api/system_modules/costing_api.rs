use crate::blueprints::resource::LiquidFungibleResource;
use crate::types::*;
use radix_engine_common::math::Decimal;

pub trait ClientCostingApi<E> {
    /// Check if costing is enabled.
    fn start_lock_fee(&mut self, amount: Decimal) -> Result<bool, E>;

    /// Add cost units to the reserve. This should never fail.
    fn lock_fee(&mut self, locked_fee: LiquidFungibleResource, contingent: bool);

    fn consume_cost_units(&mut self, costing_entry: ClientCostingEntry) -> Result<(), E>;

    fn execution_cost_unit_limit(&mut self) -> Result<u32, E>;

    fn execution_cost_unit_price(&mut self) -> Result<Decimal, E>;

    fn finalization_cost_unit_limit(&mut self) -> Result<u32, E>;

    fn finalization_cost_unit_price(&mut self) -> Result<Decimal, E>;

    fn usd_price(&mut self) -> Result<Decimal, E>;

    fn max_per_function_royalty_in_xrd(&mut self) -> Result<Decimal, E>;

    fn tip_percentage(&mut self) -> Result<u32, E>;

    fn fee_balance(&mut self) -> Result<Decimal, E>;
}
