use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

use crate::abi::*;
use crate::data::*;
use crate::scrypto_type;

/// Represents an ED25519 public key.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EddsaEd25519PublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

/// Represents an ED25519 signature.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EddsaEd25519Signature(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

/// Represents an error ocurred when validating a signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValidationError {}

/// EddsaEd25519 signature verifier.
pub struct EddsaEd25519Verifier;

impl EddsaEd25519PublicKey {
    pub const LENGTH: usize = 32;
}

impl EddsaEd25519Signature {
    pub const LENGTH: usize = 64;
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
// binary
//======

impl TryFrom<&[u8]> for EddsaEd25519PublicKey {
    type Error = ParseEddsaEd25519PublicKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != EddsaEd25519PublicKey::LENGTH {
            return Err(ParseEddsaEd25519PublicKeyError::InvalidLength(slice.len()));
        }

        Ok(EddsaEd25519PublicKey(copy_u8_array(slice)))
    }
}

impl From<EddsaEd25519PublicKey> for Vec<u8> {
    fn from(value: EddsaEd25519PublicKey) -> Self {
        value.to_vec()
    }
}

impl EddsaEd25519PublicKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(
    EddsaEd25519PublicKey,
    ScryptoCustomValueKind::EddsaEd25519PublicKey,
    Type::EddsaEd25519PublicKey,
    EddsaEd25519PublicKey::LENGTH
);

impl TryFrom<&[u8]> for EddsaEd25519Signature {
    type Error = ParseEddsaEd25519SignatureError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != EddsaEd25519Signature::LENGTH {
            return Err(ParseEddsaEd25519SignatureError::InvalidLength(slice.len()));
        }

        Ok(EddsaEd25519Signature(copy_u8_array(slice)))
    }
}

impl From<EddsaEd25519Signature> for Vec<u8> {
    fn from(value: EddsaEd25519Signature) -> Self {
        value.to_vec()
    }
}

impl EddsaEd25519Signature {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(
    EddsaEd25519Signature,
    ScryptoCustomValueKind::EddsaEd25519Signature,
    Type::EddsaEd25519Signature,
    EddsaEd25519Signature::LENGTH
);

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
