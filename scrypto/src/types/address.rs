use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::rust::borrow::ToOwned;
use crate::rust::convert::TryFrom;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::vec::Vec;
use crate::types::*;

/// Represents an address.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Address {
    /// Radix System address
    System,

    /// Radix native token address.
    RadixToken,

    /// Represents a resource.
    Resource([u8; 26]),

    /// Represents a public key.
    PublicKey([u8; 33]),

    /// Represents a package.
    Package([u8; 26]),

    /// Represents a component.
    Component([u8; 26]),
}

/// Represents an error when parsing Address.
#[derive(Debug, Clone)]
pub enum ParseAddressError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

impl Address {
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Self::System => [0].to_vec(),
            Self::RadixToken => [1].to_vec(),
            Self::Resource(d) => combine(3, d),
            Self::PublicKey(d) => combine(4, d),
            Self::Package(d) => combine(5, d),
            Self::Component(d) => combine(6, d),
        }
    }
}

impl FromStr for Address {
    type Err = ParseAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseAddressError::InvalidHex)?;
        Self::try_from(bytes.as_slice())
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = ParseAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match (slice.get(0), slice.len()) {
            (Some(0), 1) => Ok(Self::System),
            (Some(1), 1) => Ok(Self::RadixToken),
            (Some(3), 27) => Ok(Self::Resource(copy_u8_array(&slice[1..]))),
            (Some(4), 34) => Ok(Self::PublicKey(copy_u8_array(&slice[1..]))),
            (Some(5), 27) => Ok(Self::Package(copy_u8_array(&slice[1..]))),
            (Some(6), 27) => Ok(Self::Component(copy_u8_array(&slice[1..]))),
            (_, len) => Err(ParseAddressError::InvalidLength(len)),
        }
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl TypeId for Address {
    #[inline]
    fn type_id() -> u8 {
        SCRYPTO_TYPE_ADDRESS
    }
}

impl Encode for Address {
    fn encode_value(&self, encoder: &mut Encoder) {
        let bytes = self.to_vec();
        encoder.write_len(bytes.len());
        encoder.write_slice(&bytes);
    }
}

impl Decode for Address {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        let len = decoder.read_len()?;
        let slice = decoder.read_bytes(len)?;
        Self::try_from(slice).map_err(|_| DecodeError::InvalidCustomData(SCRYPTO_TYPE_ADDRESS))
    }
}

impl Describe for Address {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_ADDRESS.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::string::ToString;

    #[test]
    fn test_from_to_string() {
        let s = "040377bac8066e51cd0d6b320c338d5abbcdbcca25572b6b3eee9443eafc92106bba";
        let a = Address::from_str(s).unwrap();
        assert_eq!(a.to_string(), s);
    }
}
