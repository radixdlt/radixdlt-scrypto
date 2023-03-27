use crate::api::types::*;
use radix_engine_common::data::scrypto::model::Address;
use sbor::rust::fmt::Debug;

pub trait ClientActorApi<E: Debug> {
    fn get_global_address(&mut self) -> Result<Address, E>;
    fn get_fn_identifier(&mut self) -> Result<FnIdentifier, E>;
}
