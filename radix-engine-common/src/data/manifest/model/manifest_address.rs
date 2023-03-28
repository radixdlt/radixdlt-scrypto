use crate::data::manifest::ManifestCustomValueKind;
use crate::types::NodeId;
use crate::*;
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

/// Any address supported by manifest, both global and local.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ManifestAddress(pub NodeId);

impl ManifestAddress {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl TryFrom<&[u8]> for ManifestAddress {
    type Error = ParseManifestAddressError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        match slice.len() {
            NodeId::LENGTH => Ok(Self(NodeId(copy_u8_array(slice)))),
            _ => Err(ParseManifestAddressError::InvalidLength(slice.len())),
        }
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestAddressError {
    InvalidLength(usize),
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

manifest_type!(
    ManifestAddress,
    ManifestCustomValueKind::Address,
    NodeId::LENGTH
);

//======
// text
//======

impl fmt::Debug for ManifestAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Address({})", hex::encode(&self.0))
    }
}
