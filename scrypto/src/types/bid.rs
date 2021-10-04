use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::rust::borrow::ToOwned;
use crate::rust::convert::TryFrom;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents a bucket id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bid(pub u32);

/// Represents an error when parsing Bid.
#[derive(Debug, Clone)]
pub enum ParseBidError {
    InvalidU32(String),
    InvalidLength(usize),
}

impl Bid {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

impl TryFrom<&[u8]> for Bid {
    type Error = ParseBidError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 4 {
            Err(ParseBidError::InvalidLength(slice.len()))
        } else {
            Ok(Self(u32::from_le_bytes(copy_u8_array(slice))))
        }
    }
}

impl TypeId for Bid {
    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_BID
    }
}

impl Encode for Bid {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }
}

impl Decode for Bid {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(SCRYPTO_TYPE_BID))
    }
}

impl Describe for Bid {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_BID.to_owned(),
        }
    }
}
