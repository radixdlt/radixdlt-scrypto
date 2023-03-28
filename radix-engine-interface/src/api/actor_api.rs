use crate::types::*;
use sbor::rust::fmt::Debug;

pub trait ClientActorApi<E: Debug> {
    fn get_fn_identifier(&mut self) -> Result<FnIdentifier, E>;
}
