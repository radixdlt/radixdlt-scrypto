use radix_engine_common::data::scrypto::ScryptoSbor;
use sbor::rust::fmt::Debug;

/// Represents the data structure of a non-fungible.
pub trait NonFungibleData: ScryptoSbor {
    const MUTABLE_FIELDS: &'static [&'static str];
}