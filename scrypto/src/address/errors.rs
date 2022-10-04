use bech32::{Error, Variant};
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;

/// Represents an error in addressing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    Bech32mEncodingError(Error),
    Bech32mDecodingError(Error),
    HexDecodingError,
    InvalidVariant(Variant),
    DataSectionTooShort,
    InvalidLength(usize),
    InvalidEntityTypeId(u8),
    InvalidHrp,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for AddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
