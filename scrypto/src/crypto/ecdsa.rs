use sbor::*;

use crate::crypto::{Hash, HASH_LENGTH};
use crate::misc::copy_u8_array;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::vec::Vec;
use crate::types::{custom_type, CustomType};

pub const ECDSA_PRIVATE_KEY_LENGTH: usize = 32;
pub const ECDSA_PUBLIC_KEY_LENGTH: usize = 33;
pub const ECDSA_SIGNATURE_LENGTH: usize = 65;

/// Represents an ECDSA private key.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EcdsaPrivateKey(pub [u8; ECDSA_PRIVATE_KEY_LENGTH]);

/// Represents an ECDSA public key.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EcdsaPublicKey(pub [u8; ECDSA_PUBLIC_KEY_LENGTH]);

/// Represents an ECDSA signature.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EcdsaSignature([u8; ECDSA_SIGNATURE_LENGTH]);

/// Represents an error ocurred when validating a signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValidationError {}

impl EcdsaPublicKey {}

impl EcdsaPrivateKey {
    pub fn public_key(&self) -> EcdsaPublicKey {
        // TODO replace with real implementation once signature algorithm is decided.
        let mut bytes = [0u8; ECDSA_PUBLIC_KEY_LENGTH];
        (&mut bytes[0..ECDSA_PRIVATE_KEY_LENGTH]).copy_from_slice(&self.0);
        EcdsaPublicKey(bytes)
    }

    pub fn sign(&self, hash: Hash) -> EcdsaSignature {
        // TODO replace with real implementation once signature algorithm is decided.
        let mut bytes = [0u8; ECDSA_SIGNATURE_LENGTH];
        (&mut bytes[0..HASH_LENGTH]).copy_from_slice(&hash.0);
        (&mut bytes[HASH_LENGTH..HASH_LENGTH + ECDSA_PRIVATE_KEY_LENGTH]).copy_from_slice(&self.0);
        EcdsaSignature::new(bytes).unwrap()
    }
}

impl EcdsaSignature {
    pub fn new(raw: [u8; ECDSA_SIGNATURE_LENGTH]) -> Result<Self, SignatureValidationError> {
        // TODO replace with real implementation once signature algorithm is decided.
        Ok(Self(raw))
    }
}

//======
// error
//======

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
        if slice.len() == ECDSA_PUBLIC_KEY_LENGTH {
            Ok(Self(copy_u8_array(slice)))
        } else {
            Err(ParseEcdsaPublicKeyError::InvalidLength(slice.len()))
        }
    }
}

impl EcdsaPublicKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

custom_type!(EcdsaPublicKey, CustomType::EcdsaPublicKey, Vec::new());

impl TryFrom<&[u8]> for EcdsaSignature {
    type Error = ParseEcdsaSignatureError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() == ECDSA_SIGNATURE_LENGTH {
            Ok(Self(copy_u8_array(slice)))
        } else {
            Err(ParseEcdsaSignatureError::InvalidLength(slice.len()))
        }
    }
}

impl EcdsaSignature {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

custom_type!(EcdsaSignature, CustomType::EcdsaSignature, Vec::new());

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
        write!(f, "{}", hex::encode(self.0))
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
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Debug for EcdsaSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
