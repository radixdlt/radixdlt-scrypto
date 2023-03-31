use crate::*;
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

/// Represents an ED25519 public key.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct EddsaEd25519PublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl EddsaEd25519PublicKey {
    pub const LENGTH: usize = 32;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl TryFrom<&[u8]> for EddsaEd25519PublicKey {
    type Error = ParseEddsaEd25519PublicKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != EddsaEd25519PublicKey::LENGTH {
            return Err(ParseEddsaEd25519PublicKeyError::InvalidLength(slice.len()));
        }

        Ok(EddsaEd25519PublicKey(copy_u8_array(slice)))
    }
}

//======
// error
//======

/// Represents an error when parsing ED25519 public key from hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseEddsaEd25519PublicKeyError {
    InvalidHex(String),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseEddsaEd25519PublicKeyError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseEddsaEd25519PublicKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// text
//======

impl FromStr for EddsaEd25519PublicKey {
    type Err = ParseEddsaEd25519PublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)
            .map_err(|_| ParseEddsaEd25519PublicKeyError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for EddsaEd25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for EddsaEd25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
