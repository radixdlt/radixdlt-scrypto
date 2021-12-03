use sbor::{describe::*, *};

use crate::rust::vec::Vec;

pub trait NftData {
    /// Decodes `Self` from the serialized immutable and mutable parts.
    fn decode(immutable_data: &[u8], mutable_data: &[u8]) -> Result<Self, DecodeError>
    where
        Self: Sized;

    /// Returns the serialization of the immutable data part.
    fn immutable_data(&self) -> Vec<u8>;

    /// Returns the serialization of the mutable data part.
    fn mutable_data(&self) -> Vec<u8>;

    /// Returns the schema of the immutable data.
    fn immutable_data_schema(&self) -> Type;

    /// Returns the schema of the mutable data.
    fn mutable_data_schema(&self) -> Type;
}
