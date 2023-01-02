use crate::abi::*;
use crate::api::types::*;
use crate::data::ScryptoCustomTypeId;
use crate::scrypto;
use crate::scrypto_type;
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use utils::copy_u8_array;

// TODO: it's still up to debate whether this should be an enum OR dedicated types for each variant.
#[scrypto(Clone, PartialEq, Eq)]
pub enum Own {
    Vault(VaultId),
    // TODO: add more
}

impl Own {
    pub fn vault_id(&self) -> VaultId {
        match self {
            Own::Vault(v) => v.clone(),
        }
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseOwnError {
    InvalidHex(String),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseOwnError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseOwnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Own {
    type Error = ParseOwnError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            36 => Ok(Self::Vault(copy_u8_array(slice))),
            _ => Err(ParseOwnError::InvalidLength(slice.len())),
        }
    }
}

impl Own {
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Self::Vault(v) => v.to_vec(),
        }
    }
}

scrypto_type!(Own, ScryptoCustomTypeId::Own, Type::Vault, 36);

//======
// text
//======

impl FromStr for Own {
    type Err = ParseOwnError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ParseOwnError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Own {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Own {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
