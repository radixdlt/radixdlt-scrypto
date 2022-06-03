use p256::ecdsa::Signature;
use p256::ecdsa::{signature::Verifier, VerifyingKey};
use p256::elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};
use p256::{EncodedPoint, PublicKey};
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::abi::{scrypto_type, ScryptoType};

/// Represents an ECDSA public key.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EcdsaPublicKey(pub PublicKey);

/// Represents an ECDSA signature.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EcdsaSignature(pub Signature);

/// Represents an error ocurred when validating a signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValidationError {}

/// Ecdsa signature verifier.
pub struct EcdsaVerifier;

impl EcdsaPublicKey {
    // uncompressed
    pub const LENGTH: usize = 65;
}

impl EcdsaSignature {
    pub const LENGTH: usize = 64;
}

impl EcdsaVerifier {
    pub fn verify(msg: &[u8], pk: &EcdsaPublicKey, sig: &EcdsaSignature) -> bool {
        let verifier = VerifyingKey::from(pk.0);
        verifier.verify(msg, &sig.0).is_ok()
    }
}

//======
// error
//======

/// Represents an error when parsing ECDSA public key from hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseEcdsaPublicKeyError {
    InvalidHex(String),
    InvalidLength(usize),
    InvalidKey,
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
    InvalidSignature,
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

        let pk = PublicKey::from_encoded_point(
            &EncodedPoint::from_bytes(slice).map_err(|_| ParseEcdsaPublicKeyError::InvalidKey)?,
        );
        if pk.is_some().unwrap_u8() > 0 {
            Ok(EcdsaPublicKey(pk.unwrap()))
        } else {
            Err(ParseEcdsaPublicKeyError::InvalidKey)
        }
    }
}

impl EcdsaPublicKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_encoded_point(false).as_bytes().to_vec()
    }
}

scrypto_type!(EcdsaPublicKey, ScryptoType::EcdsaPublicKey, Vec::new());

impl TryFrom<&[u8]> for EcdsaSignature {
    type Error = ParseEcdsaSignatureError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != EcdsaSignature::LENGTH {
            return Err(ParseEcdsaSignatureError::InvalidLength(slice.len()));
        }

        let signature =
            Signature::try_from(slice).map_err(|_| ParseEcdsaSignatureError::InvalidSignature)?;
        Ok(EcdsaSignature(signature))
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
