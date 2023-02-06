use crate::api::types::*;
use crate::blueprints::resource::*;

// TODO: more thinking on whether should be part of the ClientApi.
pub trait ClientMeteringApi<E> {
    fn consume_cost_units(&mut self, units: u32) -> Result<(), E>;
}
