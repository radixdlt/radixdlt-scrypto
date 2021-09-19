use sbor::{describe::Type, *};

use crate::constants::*;
use crate::rust::borrow::ToOwned;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents a storage id.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SID(pub H256, pub u32);

/// Represents an error when parsing SID.
#[derive(Debug, Clone)]
pub enum ParseSIDError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

impl SID {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(36);
        vec.extend(self.0.as_ref());
        vec.extend(&self.1.to_le_bytes());
        vec
    }
}

impl FromStr for SID {
    type Err = ParseSIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseSIDError::InvalidHex)?;
        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&[u8]> for SID {
    type Error = ParseSIDError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 36 {
            Err(ParseSIDError::InvalidLength(slice.len()))
        } else {
            Ok(Self(
                H256(copy_u8_array(&slice[..32])),
                u32::from_le_bytes(copy_u8_array(&slice[32..])),
            ))
        }
    }
}

impl From<&str> for SID {
    fn from(s: &str) -> Self {
        Self::from_str(s).unwrap()
    }
}

impl From<String> for SID {
    fn from(s: String) -> Self {
        Self::from_str(&s).unwrap()
    }
}

impl fmt::Debug for SID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Display for SID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl TypeId for SID {
    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_SID
    }
}

impl Encode for SID {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }
}

impl Decode for SID {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(SCRYPTO_TYPE_SID))
    }
}

impl Describe for SID {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_SID.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::string::ToString;

    #[test]
    fn test_from_to_string() {
        let s = "f4cb57e4c4cd9d6564823eee427779d022d4f5f601791484a97837e6ffcf4cba01000000";
        let a = SID::from_str(s).unwrap();
        assert_eq!(a.to_string(), s);
    }
}
