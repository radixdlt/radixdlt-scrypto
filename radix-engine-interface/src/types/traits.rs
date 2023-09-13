use radix_engine_common::{data::scrypto::ScryptoSbor, types::BlueprintTypeIdentifier};

/// Represents the data structure of a non-fungible.
pub trait NonFungibleData: ScryptoSbor {
    const MUTABLE_FIELDS: &'static [&'static str];
}

/// A type that is registered under a blueprint.
///
///
/// # Implementation Notes
/// Rust doesn't allow implementing foreign trait for foreign types (expect for marker traits, which is unstable).
///
/// We've added the generic parameter `T` to allow Scrypto blueprint crates to implement this trait
/// on any types, given T is substituted with some type within that crate.
pub trait RegisteredType<T>: ScryptoSbor {
    fn blueprint_type_identifier() -> BlueprintTypeIdentifier;
}

impl NonFungibleData for () {
    const MUTABLE_FIELDS: &'static [&'static str] = &[];
}
