use bech32::{Error, Variant};
use sbor::rust::fmt;

/// Represents an error in addressing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    EncodingError(Error),
    DecodingError(Error),
    InvalidVariant(Variant),
    DataSectionTooShort,
    InvalidEntityTypeId(u8),
    InvalidHrp,
    TryFromError,
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for AddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
