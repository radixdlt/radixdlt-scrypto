use crate::data::manifest::ManifestCustomValueKind;
use crate::data::scrypto::*;
use crate::*;
use radix_engine_constants::NODE_ID_LENGTH;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Address(pub [u8; NODE_ID_LENGTH]);

impl Address {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = ParseAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            NODE_ID_LENGTH => Ok(Self(copy_u8_array(slice))),
            _ => Err(ParseAddressError::InvalidLength(slice.len())),
        }
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseAddressError {
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

well_known_scrypto_custom_type!(
    Address,
    ScryptoCustomValueKind::Address,
    Type::Address,
    NODE_ID_LENGTH,
    ADDRESS_ID
);

//==================
// binary (manifest)
//==================

manifest_type!(Address, ManifestCustomValueKind::Address, NODE_ID_LENGTH);

//======
// text
//======

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "GlobalRef({})", hex::encode(&self.0))
    }
}
