use crate::api::types::*;
use crate::blueprints::resource::LiquidFungibleResource;

pub trait ClientCostingApi<E> {
    fn consume_cost_units(&mut self, units: u32, reason: ClientCostingReason) -> Result<(), E>;

    fn credit_cost_units(
        &mut self,
        vault_id: ObjectId,
        locked_fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, E>;
}
