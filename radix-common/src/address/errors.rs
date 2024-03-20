use bech32;
use sbor::rust::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressBech32EncodeError {
    Bech32mEncodingError(bech32::Error),
    FormatError(fmt::Error),
    MissingEntityTypeByte,
    InvalidEntityTypeId(u8),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for AddressBech32EncodeError {}

impl fmt::Display for AddressBech32EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddressBech32DecodeError {
    MissingEntityTypeByte,
    Bech32mEncodingError(bech32::Error),
    Bech32mDecodingError(bech32::Error),
    InvalidVariant(bech32::Variant),
    InvalidEntityTypeId(u8),
    InvalidHrp,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for AddressBech32DecodeError {}

impl fmt::Display for AddressBech32DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
