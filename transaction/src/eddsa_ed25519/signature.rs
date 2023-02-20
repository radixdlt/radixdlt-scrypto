use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

/// Represents an ED25519 signature.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub struct EddsaEd25519Signature(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

/// Represents an error ocurred when validating a signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValidationError {}

/// EddsaEd25519 signature verifier.
pub struct EddsaEd25519Verifier;

impl EddsaEd25519Signature {
    pub const LENGTH: usize = 64;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl TryFrom<&[u8]> for EddsaEd25519Signature {
    type Error = ParseEddsaEd25519SignatureError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != EddsaEd25519Signature::LENGTH {
            return Err(ParseEddsaEd25519SignatureError::InvalidLength(slice.len()));
        }

        Ok(EddsaEd25519Signature(copy_u8_array(slice)))
    }
}

//======
// error
//======

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseEddsaEd25519SignatureError {
    InvalidHex(String),
    InvalidLength(usize),
}

/// Represents an error when parsing ED25519 signature from hex.
#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseEddsaEd25519SignatureError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseEddsaEd25519SignatureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// text
//======

impl FromStr for EddsaEd25519Signature {
    type Err = ParseEddsaEd25519SignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)
            .map_err(|_| ParseEddsaEd25519SignatureError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for EddsaEd25519Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for EddsaEd25519Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
