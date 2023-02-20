use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

/// Represents an ECDSA signature.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub struct EcdsaSecp256k1Signature(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

/// Represents an error ocurred when validating a signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValidationError {}

/// EcdsaSecp256k1 signature verifier.
pub struct EcdsaSecp256k1Verifier;

impl EcdsaSecp256k1Signature {
    pub const LENGTH: usize = 65; // recovery id + signature

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

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
// text
//======

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
