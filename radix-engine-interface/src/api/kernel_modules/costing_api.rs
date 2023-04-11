use crate::blueprints::resource::LiquidFungibleResource;
use crate::types::*;
use radix_engine_common::types::*;

pub trait ClientCostingApi<E> {
    fn consume_cost_units(&mut self, units: u32, reason: ClientCostingReason) -> Result<(), E>;

    fn credit_cost_units(
        &mut self,
        vault_id: NodeId,
        locked_fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, E>;
}
