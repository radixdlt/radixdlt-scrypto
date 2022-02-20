use sbor::*;

use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::vec::Vec;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Encode, Decode, TypeId, Describe)]
pub struct EcdsaPublicKey(pub [u8; 33]);

impl EcdsaPublicKey {}

//======
// error
//======

#[derive(Debug, Clone)]
pub enum ParseEcdsaPublicKeyError {
    InvalidHex(hex::FromHexError),
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

//======
// binary
//======

impl EcdsaPublicKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

//======
// text
//======

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
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Debug for EcdsaPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
