use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::rust::borrow::ToOwned;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents a reference id.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rid(pub u32);

/// Represents an error when parsing Rid.
#[derive(Debug, Clone)]
pub enum ParseRidError {
    InvalidU32(String),
    InvalidLength(usize),
}

impl Rid {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

impl FromStr for Rid {
    type Err = ParseRidError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            u32::from_str(s).map_err(|_| ParseRidError::InvalidU32(s.to_owned()))?,
        ))
    }
}

impl TryFrom<&[u8]> for Rid {
    type Error = ParseRidError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 4 {
            Err(ParseRidError::InvalidLength(slice.len()))
        } else {
            Ok(Self(u32::from_le_bytes(copy_u8_array(slice))))
        }
    }
}

impl fmt::Debug for Rid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rid({})", self.0)
    }
}

impl fmt::Display for Rid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TypeId for Rid {
    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_RID
    }
}

impl Encode for Rid {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }
}

impl Decode for Rid {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(SCRYPTO_TYPE_RID))
    }
}

impl Describe for Rid {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_RID.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::string::ToString;

    #[test]
    fn test_from_to_string() {
        let s = "123";
        let a = Rid::from_str(s).unwrap();
        assert_eq!(a.to_string(), s);
    }
}
