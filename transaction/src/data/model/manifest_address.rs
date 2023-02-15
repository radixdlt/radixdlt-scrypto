use radix_engine_interface::address::EntityType;
use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

use crate::data::*;
use crate::manifest_type;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestAddress(pub [u8; 27]);

//========
// error
//========

/// Represents an error when parsing ManifestAddress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestAddressError {
    InvalidLength,
    InvalidEntityTypeId,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestAddressError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestAddressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ManifestAddress {
    type Error = ParseManifestAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 27 {
            return Err(Self::Error::InvalidLength);
        }
        EntityType::try_from(slice[0]).map_err(|_| Self::Error::InvalidEntityTypeId)?;
        Ok(Self(copy_u8_array(slice)))
    }
}

impl ManifestAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

manifest_type!(ManifestAddress, ManifestCustomValueKind::Address, 27);
