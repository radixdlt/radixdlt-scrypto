use crate::api::types::*;
use crate::blueprints::resource::LiquidFungibleResource;

#[derive(Clone, Copy, Debug)]
pub enum ClientCostingReason {
    RunWasm,
    RunNative,
}

/// Unsafe APIs for interacting with kernel modules.
///
/// TODO: more thinking on whether should be part of the ClientApi.
pub trait ClientUnsafeApi<E> {
    fn consume_cost_units(&mut self, units: u32, reason: ClientCostingReason) -> Result<(), E>;

    fn credit_cost_units(
        &mut self,
        vault_id: VaultId,
        locked_fee: LiquidFungibleResource,
        contingent: bool,
    ) -> Result<LiquidFungibleResource, E>;

    fn update_instruction_index(&mut self, new_index: usize) -> Result<(), E>;

    fn update_wasm_memory_usage(&mut self, size: usize) -> Result<(), E>;
}
