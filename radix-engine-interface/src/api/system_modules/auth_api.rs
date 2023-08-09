use radix_engine_common::types::*;
use sbor::rust::fmt::Debug;

pub trait ClientAuthApi<E: Debug> {
    fn get_auth_zone(&mut self) -> Result<NodeId, E>;
}
