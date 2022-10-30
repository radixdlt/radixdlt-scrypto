use bech32;
use sbor::rust::fmt;

/// Represents an error in addressing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressError {
    Bech32mEncodingError(bech32::Error),
    Bech32mDecodingError(bech32::Error),
    FormatError(fmt::Error),
    HexDecodingError,
    InvalidVariant(bech32::Variant),
    DataSectionTooShort,
    InvalidLength(usize),
    InvalidEntityTypeId(u8),
    InvalidHrp,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for AddressError {}

impl fmt::Display for AddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
