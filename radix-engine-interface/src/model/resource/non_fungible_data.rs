use crate::data::ScryptoValue;
use sbor::DecodeError;
use scrypto_abi::Type;

/// Represents the data structure of a non-fungible.
pub trait NonFungibleData {
    /// Decodes `Self` from the serialized immutable and mutable parts.
    fn decode(
        immutable_data: &ScryptoValue,
        mutable_data: &ScryptoValue,
    ) -> Result<Self, DecodeError>
    where
        Self: Sized;

    /// Returns the serialization of the immutable data part.
    fn immutable_data(&self) -> ScryptoValue;

    /// Returns the serialization of the mutable data part.
    fn mutable_data(&self) -> ScryptoValue;

    /// Returns the schema of the immutable data.
    fn immutable_data_schema() -> Type;

    /// Returns the schema of the mutable data.
    fn mutable_data_schema() -> Type;
}
