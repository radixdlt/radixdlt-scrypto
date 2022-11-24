use sbor::rust::vec::Vec;
use sbor::{DecodeError, EncodeError};
use scrypto_abi::Type;

/// Represents the data structure of a non-fungible.
pub trait NonFungibleData {
    /// Decodes `Self` from the serialized immutable and mutable parts.
    fn decode(immutable_data: &[u8], mutable_data: &[u8]) -> Result<Self, DecodeError>
    where
        Self: Sized;

    /// Returns the serialization of the immutable data part.
    fn immutable_data(&self) -> Result<Vec<u8>, EncodeError>;

    /// Returns the serialization of the mutable data part.
    fn mutable_data(&self) -> Result<Vec<u8>, EncodeError>;

    /// Returns the schema of the immutable data.
    fn immutable_data_schema() -> Type;

    /// Returns the schema of the mutable data.
    fn mutable_data_schema() -> Type;
}
