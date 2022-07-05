use bech32::{Error, Variant};

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
    InvalidLength(usize)
}