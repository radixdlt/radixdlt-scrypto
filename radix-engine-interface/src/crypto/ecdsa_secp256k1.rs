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

/// Represents an ECDSA public key.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct EcdsaSecp256k1PublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

/// Represents an ECDSA signature.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EcdsaSecp256k1Signature(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

/// Represents an error ocurred when validating a signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValidationError {}

/// EcdsaSecp256k1 signature verifier.
pub struct EcdsaSecp256k1Verifier;

impl EcdsaSecp256k1PublicKey {
    pub const LENGTH: usize = 33;
}

impl EcdsaSecp256k1Signature {
    pub const LENGTH: usize = 65; // recovery id + signature
}

//======
// error
//======

/// Represents an error when parsing ECDSA public key from hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseEcdsaSecp256k1PublicKeyError {
    InvalidHex(String),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseEcdsaSecp256k1PublicKeyError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseEcdsaSecp256k1PublicKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseEcdsaSecp256k1SignatureError {
    InvalidHex(String),
    InvalidLength(usize),
}

/// Represents an error when parsing ECDSA signature from hex.
#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseEcdsaSecp256k1SignatureError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseEcdsaSecp256k1SignatureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// binary
//======

impl TryFrom<&[u8]> for EcdsaSecp256k1PublicKey {
    type Error = ParseEcdsaSecp256k1PublicKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != EcdsaSecp256k1PublicKey::LENGTH {
            return Err(ParseEcdsaSecp256k1PublicKeyError::InvalidLength(
                slice.len(),
            ));
        }

        Ok(EcdsaSecp256k1PublicKey(copy_u8_array(slice)))
    }
}

impl From<EcdsaSecp256k1PublicKey> for Vec<u8> {
    fn from(value: EcdsaSecp256k1PublicKey) -> Self {
        value.to_vec()
    }
}

impl EcdsaSecp256k1PublicKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(
    EcdsaSecp256k1PublicKey,
    ScryptoCustomValueKind::EcdsaSecp256k1PublicKey,
    Type::EcdsaSecp256k1PublicKey,
    EcdsaSecp256k1PublicKey::LENGTH
);

impl TryFrom<&[u8]> for EcdsaSecp256k1Signature {
    type Error = ParseEcdsaSecp256k1SignatureError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != EcdsaSecp256k1Signature::LENGTH {
            return Err(ParseEcdsaSecp256k1SignatureError::InvalidLength(
                slice.len(),
            ));
        }

        Ok(EcdsaSecp256k1Signature(copy_u8_array(slice)))
    }
}

impl From<EcdsaSecp256k1Signature> for Vec<u8> {
    fn from(value: EcdsaSecp256k1Signature) -> Self {
        value.to_vec()
    }
}

impl EcdsaSecp256k1Signature {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(
    EcdsaSecp256k1Signature,
    ScryptoCustomValueKind::EcdsaSecp256k1Signature,
    Type::EcdsaSecp256k1Signature,
    EcdsaSecp256k1Signature::LENGTH
);

//======
// text
//======

impl FromStr for EcdsaSecp256k1PublicKey {
    type Err = ParseEcdsaSecp256k1PublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)
            .map_err(|_| ParseEcdsaSecp256k1PublicKeyError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for EcdsaSecp256k1PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for EcdsaSecp256k1PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

impl FromStr for EcdsaSecp256k1Signature {
    type Err = ParseEcdsaSecp256k1SignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s)
            .map_err(|_| ParseEcdsaSecp256k1SignatureError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for EcdsaSecp256k1Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for EcdsaSecp256k1Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
