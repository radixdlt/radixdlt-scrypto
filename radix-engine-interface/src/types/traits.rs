use radix_engine_common::data::scrypto::ScryptoSbor;

/// Represents the data structure of a non-fungible.
pub trait NonFungibleData: ScryptoSbor {
    const MUTABLE_FIELDS: &'static [&'static str];
}

pub trait RegisteredType: ScryptoSbor {
    const BLUEPRINT_NAME: &'static str;
}
