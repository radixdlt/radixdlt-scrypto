use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::rust::borrow::ToOwned;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::vec;
use crate::types::*;

/// Represents a 32-byte hash digest.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct H256(pub [u8; 32]);

/// Represents an error when parsing H256.
#[derive(Debug, Clone)]
pub enum ParseH256Error {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

impl H256 {
    /// Returns the lower 26 bytes.
    pub fn lower_26_bytes(&self) -> [u8; 26] {
        let mut result = [0u8; 26];
        result.copy_from_slice(&self.0[6..32]);
        result
    }

    /// Returns the lower 16 bytes.
    pub fn lower_16_bytes(&self) -> [u8; 16] {
        let mut result = [0u8; 16];
        result.copy_from_slice(&self.0[16..32]);
        result
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl FromStr for H256 {
    type Err = ParseH256Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseH256Error::InvalidHex)?;
        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&[u8]> for H256 {
    type Error = ParseH256Error;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 32 {
            Err(ParseH256Error::InvalidLength(slice.len()))
        } else {
            Ok(H256(copy_u8_array(slice)))
        }
    }
}

impl From<H256> for Vec<u8> {
    fn from(a: H256) -> Vec<u8> {
        a.0.to_vec()
    }
}

impl AsRef<[u8]> for H256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for H256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self))
    }
}

impl fmt::Display for H256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self))
    }
}

impl TypeId for H256 {
    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_H256
    }
}

impl Encode for H256 {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.as_ref();
        encoder.write_len(bytes.len());
        encoder.write_slice(bytes);
    }
}

impl Decode for H256 {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(SCRYPTO_TYPE_H256))
    }
}

impl Describe for H256 {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_H256.to_owned(),
            generics: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::string::ToString;

    #[test]
    fn test_from_to_string() {
        let s = "b177968c9c68877dc8d33e25759183c556379daa45a4d78a2b91c70133c873ca";
        let h = H256::from_str(s).unwrap();
        assert_eq!(h.to_string(), s);
    }
}
