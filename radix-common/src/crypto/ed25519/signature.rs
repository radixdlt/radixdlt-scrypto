use crate::ScryptoSbor;
use radix_rust::copy_u8_array;
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

/// Represents an ED25519 signature.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub struct Ed25519Signature(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl Ed25519Signature {
    pub const LENGTH: usize = 64;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl TryFrom<&[u8]> for Ed25519Signature {
    type Error = ParseEd25519SignatureError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != Ed25519Signature::LENGTH {
            return Err(ParseEd25519SignatureError::InvalidLength(slice.len()));
        }

        Ok(Ed25519Signature(copy_u8_array(slice)))
    }
}

impl AsRef<Self> for Ed25519Signature {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<[u8]> for Ed25519Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

//======
// error
//======

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ParseEd25519SignatureError {
    InvalidHex(String),
    InvalidLength(usize),
}

/// Represents an error when parsing ED25519 signature from hex.
#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseEd25519SignatureError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseEd25519SignatureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// text
//======

impl FromStr for Ed25519Signature {
    type Err = ParseEd25519SignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseEd25519SignatureError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Ed25519Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Ed25519Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
