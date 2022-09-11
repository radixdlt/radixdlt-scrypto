use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::{scrypto_type, ScryptoType};
use crate::misc::copy_u8_array;

/// Represents an ECDSA public key.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EcdsaPublicKey(pub [u8; Self::LENGTH]);

/// Represents an ECDSA signature.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EcdsaSignature(pub [u8; Self::LENGTH]);

/// Represents an error ocurred when validating a signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValidationError {}

/// Ecdsa signature verifier.
pub struct EcdsaVerifier;

impl EcdsaPublicKey {
    pub const LENGTH: usize = 33;
}

impl EcdsaSignature {
    pub const LENGTH: usize = 64;
}

//======
// error
//======

/// Represents an error when parsing ECDSA public key from hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseEcdsaPublicKeyError {
    InvalidHex(String),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseEcdsaPublicKeyError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseEcdsaPublicKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseEcdsaSignatureError {
    InvalidHex(String),
    InvalidLength(usize),
}

/// Represents an error when parsing ECDSA signature from hex.
#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseEcdsaSignatureError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseEcdsaSignatureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// binary
//======

impl TryFrom<&[u8]> for EcdsaPublicKey {
    type Error = ParseEcdsaPublicKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != EcdsaPublicKey::LENGTH {
            return Err(ParseEcdsaPublicKeyError::InvalidLength(slice.len()));
        }

        Ok(EcdsaPublicKey(copy_u8_array(slice)))
    }
}

impl EcdsaPublicKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(EcdsaPublicKey, ScryptoType::EcdsaPublicKey, Vec::new());

impl TryFrom<&[u8]> for EcdsaSignature {
    type Error = ParseEcdsaSignatureError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != EcdsaSignature::LENGTH {
            return Err(ParseEcdsaSignatureError::InvalidLength(slice.len()));
        }

        Ok(EcdsaSignature(copy_u8_array(slice)))
    }
}

impl EcdsaSignature {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(EcdsaSignature, ScryptoType::EcdsaSignature, Vec::new());

//======
// text
//======

impl FromStr for EcdsaPublicKey {
    type Err = ParseEcdsaPublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseEcdsaPublicKeyError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for EcdsaPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for EcdsaPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

impl FromStr for EcdsaSignature {
    type Err = ParseEcdsaSignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseEcdsaSignatureError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for EcdsaSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for EcdsaSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
