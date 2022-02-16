use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::vec::Vec;
use sbor::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Describe, Encode, Decode, TypeId)]
pub struct EcdsaPublicKey(pub [u8; 33]);

impl EcdsaPublicKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone().to_vec()
    }
}

#[derive(Debug, Clone)]
pub enum ParseEcdsaPublicKeyError {
    InvalidHex(hex::FromHexError),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseEcdsaPublicKeyError {}

impl fmt::Display for ParseEcdsaPublicKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for EcdsaPublicKey {
    type Err = ParseEcdsaPublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(ParseEcdsaPublicKeyError::InvalidHex)?;
        bytes
            .try_into()
            .map(|k| EcdsaPublicKey(k))
            .map_err(|k| ParseEcdsaPublicKeyError::InvalidLength(k.len()))
    }
}

impl fmt::Display for EcdsaPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}
