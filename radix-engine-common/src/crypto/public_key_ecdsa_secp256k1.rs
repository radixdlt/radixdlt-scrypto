use crate::*;
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

/// Represents an ECDSA public key.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct EcdsaSecp256k1PublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl EcdsaSecp256k1PublicKey {
    pub const LENGTH: usize = 33;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

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

//======
// error
//======

/// Represents an error when parsing ED25519 public key from hex.
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
