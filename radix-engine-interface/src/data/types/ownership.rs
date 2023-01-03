use crate::abi::*;
use crate::api::types::*;
use crate::data::ScryptoCustomTypeId;
use crate::scrypto;
use crate::scrypto_type;
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use utils::copy_u8_array;

// TODO: it's still up to debate whether this should be an enum OR dedicated types for each variant.
#[scrypto(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Own {
    Bucket(BucketId),
    Proof(ProofId),
    Vault(VaultId),
}

impl Own {
    pub fn vault_id(&self) -> VaultId {
        match self {
            Own::Vault(v) => v.clone(),
            _ => panic!("Not a vault ownership"),
        }
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseOwnError {
    InvalidLength(usize),
    UnknownVariant(u8),
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
        match slice
            .get(0)
            .ok_or(ParseOwnError::InvalidLength(slice.len()))?
        {
            0 => {
                if slice.len() == 5 {
                    Ok(Self::Bucket(u32::from_le_bytes(copy_u8_array(&slice[1..]))))
                } else {
                    Err(ParseOwnError::InvalidLength(slice.len()))
                }
            }
            1 => {
                if slice.len() == 5 {
                    Ok(Self::Proof(u32::from_le_bytes(copy_u8_array(&slice[1..]))))
                } else {
                    Err(ParseOwnError::InvalidLength(slice.len()))
                }
            }
            2 => {
                if slice.len() == 37 {
                    Ok(Self::Vault(copy_u8_array(&slice[1..])))
                } else {
                    Err(ParseOwnError::InvalidLength(slice.len()))
                }
            }
            id => Err(ParseOwnError::UnknownVariant(*id)),
        }
    }
}

impl Own {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            Own::Bucket(id) => {
                buf.push(0);
                buf.extend(id.to_le_bytes());
            }
            Own::Proof(id) => {
                buf.push(1);
                buf.extend(id.to_le_bytes());
            }
            Own::Vault(id) => {
                buf.push(2);
                buf.extend(id);
            }
        }
        buf
    }
}

scrypto_type!(Own, ScryptoCustomTypeId::Own, Type::Vault); // FIXME can be bucket, proof, or vault
