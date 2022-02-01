use core::fmt::{Display, Formatter};
use core::fmt;
use crate::rust::str::FromStr;
use crate::rust::borrow::ToOwned;
use crate::rust::vec;
use crate::rust::vec::Vec;
use sbor::{describe::Type, *};
use crate::buffer::{SCRYPTO_NAME_NFT_KEY, SCRYPTO_TYPE_NFT_KEY};

#[derive(Debug, Clone)]
pub enum ParseNftKeyError {
    InvalidHex(hex::FromHexError)
}

/// Represents a key for an NFT resource
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NftKey(Vec<u8>);

impl NftKey {
    pub fn new(v: Vec<u8>) -> Self {
        NftKey(v)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
    }
}

impl From<u128> for NftKey {
    fn from(u: u128) -> Self {
        NftKey(u.to_be_bytes().to_vec())
    }
}

impl TryFrom<&[u8]> for NftKey {
    type Error = ParseNftKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self(slice.to_vec()))
    }
}

impl TypeId for NftKey {
    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_NFT_KEY
    }
}

impl Display for NftKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:x?}", self.0)
    }
}

impl Encode for NftKey {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }
}

impl Decode for NftKey {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(SCRYPTO_TYPE_NFT_KEY))
    }
}

impl FromStr for NftKey {
    type Err = ParseNftKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseNftKeyError::InvalidHex)?;
        Self::try_from(bytes.as_slice())
    }
}

impl Describe for NftKey {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_NFT_KEY.to_owned(),
            generics: vec![],
        }
    }
}
