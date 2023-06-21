use sbor::prelude::fmt;

#[derive(Clone, Debug)]
pub enum TransactionHashBech32EncodeError {
    FormatError(fmt::Error),
    Bech32mEncodingError(bech32::Error),
}

impl From<fmt::Error> for TransactionHashBech32EncodeError {
    fn from(value: fmt::Error) -> Self {
        Self::FormatError(value)
    }
}

impl From<bech32::Error> for TransactionHashBech32EncodeError {
    fn from(value: bech32::Error) -> Self {
        Self::Bech32mEncodingError(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionHashBech32DecodeError {
    MissingEntityTypeByte,
    Bech32mDecodingError(bech32::Error),
    InvalidVariant(bech32::Variant),
    InvalidHrp,
    InvalidLength,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for TransactionHashBech32EncodeError {}

impl fmt::Display for TransactionHashBech32EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
