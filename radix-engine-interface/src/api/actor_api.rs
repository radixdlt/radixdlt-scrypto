use crate::types::*;
use sbor::rust::fmt::Debug;

pub trait ClientActorApi<E: Debug> {
    fn get_global_address(&mut self) -> Result<GlobalAddress, E>;
    fn get_blueprint(&mut self) -> Result<Blueprint, E>;
}
