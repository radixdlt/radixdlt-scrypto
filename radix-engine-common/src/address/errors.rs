use bech32;
use sbor::rust::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeBech32AddressError {
    Bech32mEncodingError(bech32::Error),
    FormatError(fmt::Error),
    MissingEntityTypeByte,
    InvalidEntityTypeId(u8),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for EncodeBech32AddressError {}

impl fmt::Display for EncodeBech32AddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeBech32AddressError {
    MissingEntityTypeByte,
    Bech32mEncodingError(bech32::Error),
    Bech32mDecodingError(bech32::Error),
    InvalidVariant(bech32::Variant),
    InvalidEntityTypeId(u8),
    InvalidHrp,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for DecodeBech32AddressError {}

impl fmt::Display for DecodeBech32AddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
