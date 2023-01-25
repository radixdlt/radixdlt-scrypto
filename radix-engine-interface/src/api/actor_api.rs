use crate::api::types::*;
use sbor::rust::fmt::Debug;

pub trait ClientActorApi<E: Debug> {
    fn fn_identifier(&mut self) -> Result<FnIdentifier, E>;
}
