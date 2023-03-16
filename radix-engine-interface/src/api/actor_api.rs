use crate::api::types::*;
use sbor::rust::fmt::Debug;

pub trait ClientActorApi<E: Debug> {
    fn get_fn_identifier(&mut self) -> Result<FnIdentifier, E>;

    fn get_current_auth_zone(&mut self) -> Result<ObjectId, E>;
}
