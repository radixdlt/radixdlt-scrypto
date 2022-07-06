use bech32::{Error, Variant};
use sbor::rust::fmt;

/// Represents an error when decoding an address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseAddressError {
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
impl std::error::Error for ParseAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
