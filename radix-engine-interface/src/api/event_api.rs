use crate::api::types::*;
use crate::blueprints::resource::Resource;

// TODO: more thinking on whether should be part of the ClientApi.
pub trait ClientEventApi<E> {
    fn consume_cost_units(&mut self, units: u32) -> Result<(), E>;

    fn credit_cost_units(
        &mut self,
        vault_id: VaultId,
        locked_fee: Resource,
        contingent: bool,
    ) -> Result<Resource, E>;

    fn on_instantiate_wasm_code(&mut self, code: &[u8]) -> Result<(), E>;

    fn on_update_instruction_index(&mut self, new_index: usize) -> Result<(), E>;
}
