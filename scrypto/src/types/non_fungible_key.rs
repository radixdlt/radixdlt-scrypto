use crate::buffer::{SCRYPTO_NAME_NON_FUNGIBLE_KEY, SCRYPTO_TYPE_NON_FUNGIBLE_KEY};
use crate::rust::borrow::ToOwned;
use crate::rust::str::FromStr;
use crate::rust::vec;
use crate::rust::vec::Vec;
use core::fmt;
use core::fmt::{Display, Formatter};
use sbor::{describe::Type, *};

#[derive(Debug, Clone)]
pub enum ParseNonFungibleKeyError {
    InvalidHex(hex::FromHexError),
}

/// Represents a key for a non-fungible resource
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonFungibleKey(Vec<u8>);

impl NonFungibleKey {
    pub fn new(v: Vec<u8>) -> Self {
        NonFungibleKey(v)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
    }
}

impl From<u128> for NonFungibleKey {
    fn from(u: u128) -> Self {
        NonFungibleKey(u.to_be_bytes().to_vec())
    }
}

impl TryFrom<&[u8]> for NonFungibleKey {
    type Error = ParseNonFungibleKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(slice.to_vec()))
    }
}

impl TypeId for NonFungibleKey {
    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_NON_FUNGIBLE_KEY
    }
}

impl Display for NonFungibleKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

impl Encode for NonFungibleKey {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }
}

impl Decode for NonFungibleKey {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice)
            .map_err(|_| DecodeError::InvalidCustomData(SCRYPTO_TYPE_NON_FUNGIBLE_KEY))
    }
}

impl FromStr for NonFungibleKey {
    type Err = ParseNonFungibleKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseNonFungibleKeyError::InvalidHex)?;
        Self::try_from(bytes.as_slice())
    }
}

impl Describe for NonFungibleKey {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_NON_FUNGIBLE_KEY.to_owned(),
            generics: vec![],
        }
    }
}
