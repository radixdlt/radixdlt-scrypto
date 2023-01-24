use crate::api::types::*;
use sbor::rust::fmt::Debug;

pub trait EngineActorApi<E: Debug> {
    fn fn_identifier(&mut self) -> Result<FnIdentifier, E>;
}
