use radix_engine_common::data::scrypto::model::Own;
use radix_engine_common::ScryptoSbor;

#[derive(Debug, PartialEq, Eq, ScryptoSbor, Clone)]
pub struct SingleResourcePoolSubstate {
    vault: Own,
}
