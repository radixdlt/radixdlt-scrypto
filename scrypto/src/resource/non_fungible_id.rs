use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::*;
use crate::buffer::scrypto_encode;
use crate::values::ScryptoValue;

/// Represents a key for a non-fungible resource
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonFungibleId(pub Vec<u8>);

impl NonFungibleId {
    /// Creates a non-fungible ID from some uuid.
    pub fn random() -> Self {
        let bytes = crate::core::Runtime::generate_uuid().to_be_bytes().to_vec();
        Self::from_bytes(bytes)
    }

    /// Creates a non-fungible ID from an arbitrary byte array.
    pub fn from_bytes(v: Vec<u8>) -> Self {
        Self(scrypto_encode(&v))
    }

    /// Creates a non-fungible ID from a `u32` number.
    pub fn from_u32(u: u32) -> Self {
        Self(scrypto_encode(&u))
    }

    /// Creates a non-fungible ID from a `u64` number.
    pub fn from_u64(u: u64) -> Self {
        Self(scrypto_encode(&u))
    }
}

//========
// error
//========

/// Represents an error when decoding non-fungible id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseNonFungibleIdError {
    InvalidHex(String),
    InvalidValue,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseNonFungibleIdError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseNonFungibleIdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for NonFungibleId {
    type Error = ParseNonFungibleIdError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let value = ScryptoValue::from_slice(slice)
            .map_err(|_| ParseNonFungibleIdError::InvalidValue)?;
        Ok(Self(value.raw))
    }
}

impl NonFungibleId {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
    }
}

scrypto_type!(NonFungibleId, ScryptoType::NonFungibleId, Vec::new());

//======
// text
//======

impl FromStr for NonFungibleId {
    type Err = ParseNonFungibleIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseNonFungibleIdError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for NonFungibleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl fmt::Debug for NonFungibleId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::vec;

    #[test]
    fn test_non_fungible_id_string_rep() {
        assert_eq!(
            NonFungibleId::from_str("3007020000003575").unwrap(),
            NonFungibleId::from_bytes(vec![53u8, 117u8]),
        );
        assert_eq!(
            NonFungibleId::from_str("0905000000").unwrap(),
            NonFungibleId::from_u32(5)
        );
        assert_eq!(
            NonFungibleId::from_str("0a0500000000000000").unwrap(),
            NonFungibleId::from_u64(5)
        );
    }
}
