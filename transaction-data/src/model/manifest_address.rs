use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

use crate::manifest_type;
use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManifestAddress {
    Package([u8; 27]),
    Component([u8; 27]),
    ResourceManager([u8; 27]),
}

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
        // FIXME: move HRP constants to `radix-engine-constants`, and remove hard-coded range here
        if slice[0] == 0x00 {
            Ok(Self::ResourceManager(copy_u8_array(slice)))
        } else if slice[0] == 0x01 {
            Ok(Self::Package(copy_u8_array(slice)))
        } else if slice[0] <= 0x0c {
            Ok(Self::Component(copy_u8_array(slice)))
        } else {
            Err(Self::Error::InvalidEntityTypeId)
        }
    }
}

impl ManifestAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            ManifestAddress::Package(v) => v.to_vec(),
            ManifestAddress::Component(v) => v.to_vec(),
            ManifestAddress::ResourceManager(v) => v.to_vec(),
        }
    }
}

manifest_type!(ManifestAddress, ManifestCustomValueKind::Address, 27);
