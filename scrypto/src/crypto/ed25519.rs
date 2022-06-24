use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::{scrypto_type, ScryptoType};
use crate::misc::copy_u8_array;

/// Represents an ED25519 public key.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ed25519PublicKey([u8; Self::LENGTH]);

/// Represents an ED25519 signature.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Ed25519Signature([u8; Self::LENGTH]);

/// Represents an error ocurred when validating a signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValidationError {}

/// Ed25519 signature verifier.
pub struct Ed25519Verifier;

impl Ed25519PublicKey {
    pub const LENGTH: usize = 32;
}

impl Ed25519Signature {
    pub const LENGTH: usize = 64;
}

//======
// error
//======

/// Represents an error when parsing ED25519 public key from hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseEd25519PublicKeyError {
    InvalidHex(String),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseEd25519PublicKeyError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseEd25519PublicKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
// binary
//======

impl TryFrom<&[u8]> for Ed25519PublicKey {
    type Error = ParseEd25519PublicKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != Ed25519PublicKey::LENGTH {
            return Err(ParseEd25519PublicKeyError::InvalidLength(slice.len()));
        }

        Ok(Ed25519PublicKey(copy_u8_array(slice)))
    }
}

impl Ed25519PublicKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(Ed25519PublicKey, ScryptoType::Ed25519PublicKey, Vec::new());

impl TryFrom<&[u8]> for Ed25519Signature {
    type Error = ParseEd25519SignatureError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != Ed25519Signature::LENGTH {
            return Err(ParseEd25519SignatureError::InvalidLength(slice.len()));
        }

        Ok(Ed25519Signature(copy_u8_array(slice)))
    }
}

impl Ed25519Signature {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(Ed25519Signature, ScryptoType::Ed25519Signature, Vec::new());

//======
// text
//======

impl FromStr for Ed25519PublicKey {
    type Err = ParseEd25519PublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseEd25519PublicKeyError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Ed25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Ed25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

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
