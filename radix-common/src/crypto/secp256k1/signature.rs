use crate::ScryptoSbor;
use radix_rust::copy_u8_array;
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

/// Represents an ECDSA Secp256k1 signature.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub struct Secp256k1Signature(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl Secp256k1Signature {
    pub const LENGTH: usize = 65; // recovery id + signature

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl TryFrom<&[u8]> for Secp256k1Signature {
    type Error = ParseSecp256k1SignatureError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != Secp256k1Signature::LENGTH {
            return Err(ParseSecp256k1SignatureError::InvalidLength(slice.len()));
        }

        Ok(Secp256k1Signature(copy_u8_array(slice)))
    }
}

impl AsRef<Self> for Secp256k1Signature {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<[u8]> for Secp256k1Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

//======
// error
//======

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ParseSecp256k1SignatureError {
    InvalidHex(String),
    InvalidLength(usize),
}

/// Represents an error when parsing an ECDSA Secp256k1 signature from hex.
#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseSecp256k1SignatureError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseSecp256k1SignatureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// text
//======

impl FromStr for Secp256k1Signature {
    type Err = ParseSecp256k1SignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseSecp256k1SignatureError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Secp256k1Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Secp256k1Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
