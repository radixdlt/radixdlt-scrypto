use crate::blueprints::resource::LiquidFungibleResource;
use crate::types::*;
use radix_engine_common::math::Decimal;
use radix_engine_common::types::*;

pub trait ClientCostingApi<E> {
    fn consume_cost_units(&mut self, costing_entry: ClientCostingEntry) -> Result<(), E>;

    fn credit_cost_units(
        &mut self,
        vault_id: NodeId,
        locked_fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, E>;

    fn cost_unit_limit(&mut self) -> Result<u32, E>;

    fn cost_unit_price(&mut self) -> Result<Decimal, E>;

    fn tip_percentage(&mut self) -> Result<u32, E>;

    fn fee_balance(&mut self) -> Result<Decimal, E>;
}
