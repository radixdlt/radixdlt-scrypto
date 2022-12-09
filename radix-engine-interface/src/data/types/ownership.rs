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
pub enum Ownership {
    Vault(VaultId),
    // TODO: add more
}

impl Ownership {
    pub fn vault_id(&self) -> VaultId {
        match self {
            Ownership::Vault(v) => v.clone(),
        }
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseOwnershipError {
    InvalidHex(String),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseOwnershipError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseOwnershipError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for Ownership {
    type Error = ParseOwnershipError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            36 => Ok(Self::Vault(copy_u8_array(slice))),
            _ => Err(ParseOwnershipError::InvalidLength(slice.len())),
        }
    }
}

impl Ownership {
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            Self::Vault(v) => v.to_vec(),
        }
    }
}

scrypto_type!(Ownership, ScryptoCustomTypeId::Ownership, Type::Vault, 36);

//======
// text
//======

impl FromStr for Ownership {
    type Err = ParseOwnershipError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ParseOwnershipError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Ownership {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Ownership {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
