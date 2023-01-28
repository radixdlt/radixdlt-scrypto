use crate::api::types::*;
use crate::blueprints::resource::*;

pub trait ClientMeteringApi<E> {
    fn lock_fee(
        &mut self,
        vault_id: VaultId,
        fee: Resource,
        contingent: bool,
    ) -> Result<Resource, E>;

    fn consume_cost_units(&mut self, units: u32) -> Result<(), E>;
}
