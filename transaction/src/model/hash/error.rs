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
