use radix_engine_common::{data::scrypto::ScryptoSbor, types::PackageAddress};

/// Represents the data structure of a non-fungible.
pub trait NonFungibleData: ScryptoSbor {
    const MUTABLE_FIELDS: &'static [&'static str];
}

pub trait RegisteredType: ScryptoSbor {
    const PACKAGE_ADDRESS: Option<PackageAddress>;

    const BLUEPRINT_NAME: &'static str;

    const TYPE_NAME: &'static str;
}
