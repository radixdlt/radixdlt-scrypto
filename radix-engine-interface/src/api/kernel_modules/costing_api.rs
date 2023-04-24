use crate::blueprints::resource::LiquidFungibleResource;
use crate::types::*;
use radix_engine_common::types::*;

pub trait ClientCostingApi<E> {
    /// Adds cost to the client execution
    fn consume_cost_units(&mut self, units: u32, reason: ClientCostingReason) -> Result<(), E>;

    /// Adds cost to the client execution
    fn credit_cost_units(
        &mut self,
        vault_id: NodeId,
        locked_fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, E>;
}
