use radix_engine_common::data::scrypto::model::*;
use radix_engine_common::types::*;
use radix_engine_common::*;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, Clone)]
pub struct SingleResourcePoolSubstate {
    pub vault: Own,
    pub pool_unit_resource: ResourceAddress,
}
